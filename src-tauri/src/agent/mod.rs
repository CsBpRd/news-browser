pub mod nodes;
pub mod prompts;
pub mod state;

use std::fs;
use std::path::Path;

use chrono::Local;

use crate::ai_config::AIConfig;
use crate::search_config::SearchConfig;

use nodes::*;

/// 日志辅助：有 AppHandle 则 emit 到前端，否则 print 到 stdout
pub(crate) fn log_msg(handle: Option<&tauri::AppHandle>, log_type: &str, content: String) {
    match handle {
        Some(h) => nodes::emit_log(h, log_type, content),
        None => println!("[{}] {}", log_type, content),
    }
}

/// 配置常量
const NUM_REFLECTIONS: u32 = 1;
const NUM_RESULTS_PER_SEARCH: u32 = 3;
const CAP_SEARCH_LENGTH: usize = 20000;

/// 构建用户提示（替换模板中的占位符）
fn build_research_topic(template: &str, app_name: &str) -> String {
    let today = Local::now();
    template
        .replace("{YYYY}", &today.format("%Y").to_string())
        .replace("{YY}", &today.format("%y").to_string())
        .replace("{MM}", &today.format("%m").to_string())
        .replace("{DD}", &today.format("%d").to_string())
        .replace("{name}", app_name)
        .replace("<周期>", "周")
        .replace("<报告名称>", app_name)
        .replace("<工作目录>", "")
        .replace("<文件名格式>", "")
}

/// 解析文件名（用于保存HTML文件）
fn resolve_filename(app_name: &str, file_pattern: &str) -> String {
    let today = Local::now();
    file_pattern
        .replace("{YYYY}", &today.format("%Y").to_string())
        .replace("{YY}", &today.format("%y").to_string())
        .replace("{MM}", &today.format("%m").to_string())
        .replace("{DD}", &today.format("%d").to_string())
        .replace("{name}", app_name)
}

/// 裁剪搜索结果内容长度（安全处理 UTF-8 边界）
fn cap_results(results: &mut [state::SearchResult]) {
    for r in results.iter_mut() {
        if r.content.len() > CAP_SEARCH_LENGTH {
            // 在 CAP_SEARCH_LENGTH 边界处向后找到最近的字符边界
            let idx = r.content.char_indices()
                .take_while(|(i, _)| *i < CAP_SEARCH_LENGTH)
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(0);
            r.content = r.content[..idx].to_string();
        }
    }
}

/// Deep Search Agent 总入口（带 Tauri 日志）
pub async fn deep_search(
    ai_config: &AIConfig,
    search_config: &SearchConfig,
    template: &str,
    app_name: &str,
    format: &str,
    reference: Option<&str>,
    app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    deep_search_impl(ai_config, search_config, template, app_name, format, reference, Some(app_handle)).await
}

/// Deep Search Agent 入口（无头模式，日志输出到 stdout）
pub async fn deep_search_headless(
    ai_config: &AIConfig,
    search_config: &SearchConfig,
    template: &str,
    app_name: &str,
    format: &str,
    reference: Option<&str>,
) -> Result<String, String> {
    deep_search_impl(ai_config, search_config, template, app_name, format, reference, None::<&tauri::AppHandle>).await
}

/// 内部实现：可选 Tauri AppHandle 日志
async fn deep_search_impl(
    ai_config: &AIConfig,
    search_config: &SearchConfig,
    template: &str,
    app_name: &str,
    format: &str,
    reference: Option<&str>,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<String, String> {
    let topic = build_research_topic(template, app_name);
    log_msg(app_handle, "thinking", format!("研究主题: {}", topic));

    // Step 1: 生成报告大纲
    let mut state = run_report_structure(ai_config, &topic, app_handle).await?;

    // Step 2: 遍历每个段落，执行搜索和反思
    for j in 0..state.paragraphs.len() {
        let title = state.paragraphs[j].title.clone();
        log_msg(app_handle, "thinking", format!("======= 段落 {}/{}: {} =======", j + 1, state.paragraphs.len(), title));

        let (search_query, reasoning) =
            run_first_search(ai_config, &state.paragraphs[j]).await?;
        log_msg(app_handle, "search", format!("[初始搜索] 查询: {} | 推理: {}", search_query, reasoning));

        let mut search_results = search_tavily(&search_query, search_config, NUM_RESULTS_PER_SEARCH).await?;
        cap_results(&mut search_results);
        log_msg(app_handle, "tool_result", format!("[初始搜索] 获取到 {} 条结果", search_results.len()));

        for r in &search_results {
            state.paragraphs[j].research.search_history.push(r.clone());
        }

        let summary = run_first_summary(ai_config, &state.paragraphs[j], &search_query, &search_results).await?;
        state.paragraphs[j].research.latest_summary = summary;
        log_msg(app_handle, "output", format!("[初始总结] 长度: {} 字", state.paragraphs[j].research.latest_summary.len()));

        for i in 0..NUM_REFLECTIONS {
            log_msg(app_handle, "thinking", format!("[反思 {}/{}] 开始...", i + 1, NUM_REFLECTIONS));

            let (reflect_query, reflect_reasoning) =
                run_reflection(ai_config, &state.paragraphs[j]).await?;
            log_msg(app_handle, "search", format!("[反思 {}/{}] 查询: {} | 推理: {}", i + 1, NUM_REFLECTIONS, reflect_query, reflect_reasoning));

            let mut reflect_results =
                search_tavily(&reflect_query, search_config, NUM_RESULTS_PER_SEARCH).await?;
            cap_results(&mut reflect_results);
            log_msg(app_handle, "tool_result", format!("[反思 {}/{}] 获取到 {} 条结果", i + 1, NUM_REFLECTIONS, reflect_results.len()));

            for r in &reflect_results {
                state.paragraphs[j].research.search_history.push(r.clone());
            }

            let updated_summary = run_reflection_summary(
                ai_config,
                &state.paragraphs[j],
                &reflect_query,
                &reflect_results,
            ).await?;
            state.paragraphs[j].research.latest_summary = updated_summary;
            state.paragraphs[j].research.reflection_iteration = i + 1;

            log_msg(app_handle, "output", format!("[反思 {}/{}] 更新后长度: {} 字", i + 1, NUM_REFLECTIONS, state.paragraphs[j].research.latest_summary.len()));
        }
    }

    // Step 3: 根据配置的格式生成最终报告
    let report = match format {
        "md" => {
            log_msg(app_handle, "thinking", "正在生成Markdown报告...".to_string());
            let md = run_report_formatting_md(ai_config, &state.paragraphs).await?;
            log_msg(app_handle, "output", format!("Markdown报告生成完成 ({} 字符)", md.len()));
            md
        }
        _ => {
            log_msg(app_handle, "thinking", "正在参考历史风格生成HTML报告...".to_string());
            let html = run_report_formatting_html(ai_config, &state.paragraphs, reference).await?;
            log_msg(app_handle, "output", format!("HTML报告生成完成 ({} 字符)", html.len()));
            html
        }
    };

    if report.is_empty() {
        return Err("报告内容为空，请重试".to_string());
    }

    Ok(report)
}

/// 扫描工作目录，找到最新的符合命名格式的报告作为风格参考
fn find_reference_report(work_dir: &str, file_pattern: &str, app_name: &str) -> Option<String> {
    let dir = Path::new(work_dir);
    if !dir.exists() {
        return None;
    }

    // 用 resolve_filename 生成当前文件名，排除它
    let current_filename = resolve_filename(app_name, file_pattern);

    let entries = fs::read_dir(dir).ok()?;
    let mut candidates: Vec<(std::time::SystemTime, String)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // 只取 .html 文件，排除当前要生成的文件
            if name.ends_with(".html") && name != current_filename {
                if let Ok(meta) = e.metadata() {
                    if let Ok(modified) = meta.modified() {
                        return Some((modified, name));
                    }
                }
            }
            None
        })
        .collect();

    // 按修改时间排序，取最新的
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    let latest = candidates.first()?;
    let path = dir.join(&latest.1);
    fs::read_to_string(path).ok()
}

/// Tauri 命令：执行深度搜索并生成报刊
#[tauri::command]
pub async fn generate_newspaper(app_handle: tauri::AppHandle) -> Result<String, String> {
    let ai_config = crate::ai_config::get_ai_config();
    let search_config = crate::search_config::get_search_config();
    let prompt_template = crate::prompt_template::get_prompt_template()?;
    let settings = crate::settings::get_settings();

    // 校验配置
    if ai_config.api_endpoint.is_empty() {
        return Err("请先配置AI API端点".to_string());
    }
    if ai_config.api_key.is_empty() {
        return Err("请先配置AI API密钥".to_string());
    }
    if ai_config.model_name.is_empty() {
        return Err("请先配置AI模型名称".to_string());
    }
    if settings.work_dir.is_empty() {
        return Err("请先配置工作目录".to_string());
    }
    if search_config.api_key.is_empty() && search_config.provider != crate::search_config::SearchProvider::Custom {
        return Err("请先配置搜索API密钥".to_string());
    }

    // 根据文件命名格式判断输出类型
    let ext = settings.file_pattern
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_default();
    let format = if ext == "md" { "md" } else { "html" };

    // 查找最新的已有报告作为风格参考（仅HTML模式）
    let reference = if format == "html" {
        find_reference_report(&settings.work_dir, &settings.file_pattern, &settings.app_name)
    } else {
        None
    };
    if reference.is_some() {
        log_msg(Some(&app_handle), "thinking", format!("找到参考报告，将沿用其风格"));
    }

    // 执行深度搜索（只生成匹配格式的产物）
    let report = deep_search(
        &ai_config,
        &search_config,
        &prompt_template,
        &settings.app_name,
        format,
        reference.as_deref(),
        &app_handle,
    )
    .await?;

    // 保存文件
    let filename = resolve_filename(&settings.app_name, &settings.file_pattern);
    let file_path = Path::new(&settings.work_dir).join(&filename);

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    std::fs::write(&file_path, &report)
        .map_err(|e| format!("保存文件失败: {}", e))?;

    log_msg(Some(&app_handle), "output", format!("报告已保存至: {}", file_path.display()));

    Ok(format!(
        "✅ 深度研究完成！\n📄 {}: {} (共 {} 字符)",
        if format == "md" { "Markdown" } else { "HTML" },
        file_path.display(),
        report.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deep_search_headless() {
        // 读取真实的配置文件
        let ai_config = crate::ai_config::get_ai_config();
        let search_config = crate::search_config::get_search_config();
        let template = "科技新闻 2025年7月 人工智能与科技发展动态 生成一份新闻周报";
        let app_name = "科技新闻周报";

        println!("\n========== Deep Search Agent 开始运行 ==========");
        println!("AI: {} | 模型: {}", ai_config.api_endpoint, ai_config.model_name);
        println!("搜索: {:?} | Key: {}...", search_config.provider, &search_config.api_key[..8]);
        println!("主题: {}", template);
        println!("App: {}", app_name);
        println!("================================================\n");

        let result = deep_search_headless(&ai_config, &search_config, &template, &app_name, "html", None).await;

        match result {
            Ok(report) => {
                // 保存报告到工作目录
                let settings = crate::settings::get_settings();
                let filename = resolve_filename(&app_name, &settings.file_pattern);
                let html_path = Path::new(&settings.work_dir).join(&filename);

                std::fs::write(&html_path, &report).expect("保存报告失败");

                println!("\n\n========== 报告前 500 字符 ==========");
                let preview: String = report.chars().take(500).collect();
                println!("{}", preview);
                println!("\n... (完整报告 {} 字符)", report.len());

                println!("\n\n✅ 报告已保存至: {}", html_path.display());

                assert!(!report.is_empty(), "报告不应为空");
            }
            Err(e) => {
                panic!("Deep Search Agent 执行失败: {}", e);
            }
        }
    }

    /// 端到端验证：跳过搜索步骤，直接测试 LLM 大纲生成 → HTML 格式化 → 保存到工作目录
    /// 用于在无搜索 API key 时验证 LLM 调用链（含重试机制）和 HTML 产物完整性
    #[tokio::test]
    async fn test_llm_to_html_e2e() {
        let mut ai_config = crate::ai_config::get_ai_config();
        // 测试中强制提升 max_tokens，避免 HTML 输出被截断
        ai_config.max_tokens = 16000;
        let settings = crate::settings::get_settings();
        let app_name = "科技新闻";

        println!("\n========== E2E 验证: LLM → HTML → 文件保存 ==========");
        println!("AI: {} | 模型: {}", ai_config.api_endpoint, ai_config.model_name);
        println!("工作目录: {}", settings.work_dir);
        println!("================================================\n");

        // 校验配置
        assert!(!ai_config.api_endpoint.is_empty(), "AI API 端点未配置");
        assert!(!ai_config.api_key.is_empty(), "AI API key 未配置");
        assert!(!ai_config.model_name.is_empty(), "AI 模型未配置");
        assert!(!settings.work_dir.is_empty(), "工作目录未配置");

        // Step 1: 生成大纲（验证 call_llm 含重试机制）
        println!("[Step 1] 生成报告大纲...");
        let topic = "科技新闻周报：人工智能与科技发展动态";
        let mut state = run_report_structure(&ai_config, topic, None)
            .await
            .expect("大纲生成失败");
        println!("[Step 1] ✅ 大纲生成完成，共 {} 个段落", state.paragraphs.len());
        assert!(!state.paragraphs.is_empty(), "大纲段落数不应为 0");

        // Step 2: 为每个段落填充模拟 summary（跳过搜索）
        let total = state.paragraphs.len();
        for (i, p) in state.paragraphs.iter_mut().enumerate() {
            p.research.latest_summary = format!(
                "这是「{}」段落的模拟研究内容。本段关注近期 {} 领域的重要进展，\
                 包括模型能力提升、开源生态演进及产业落地案例。",
                p.title, app_name
            );
            println!("[Step 2] 段落 {}/{} 已填充模拟内容", i + 1, total);
        }

        // Step 3: 生成 HTML 报告（验证 HTML 清洗组件）
        println!("[Step 3] 生成 HTML 报告...");
        let report = run_report_formatting_html(&ai_config, &state.paragraphs, None)
            .await
            .expect("HTML 报告生成失败");
        println!("[Step 3] ✅ HTML 生成完成，共 {} 字符", report.len());

        // 输出完整内容用于诊断
        println!("\n========== 完整 HTML 内容 ==========");
        println!("{}", &report);
        println!("========== HTML 内容结束 ==========\n");

        assert!(!report.is_empty(), "HTML 报告不应为空");

        // Step 4: 保存到临时目录（cargo test 沙箱无法访问工作目录）
        println!("[Step 4] 保存到临时目录...");
        let test_filename = format!("e2e_test_{}.html", Local::now().format("%Y%m%d%H%M%S"));
        let file_path = std::env::temp_dir().join(&test_filename);
        std::fs::write(&file_path, &report).expect("保存文件失败");
        println!("[Step 4] ✅ 文件已保存: {}", file_path.display());

        // 同步尝试保存到工作目录（非致命，失败仅警告）
        if !settings.work_dir.is_empty() {
            let work_path = Path::new(&settings.work_dir).join(&test_filename);
            match std::fs::create_dir_all(&settings.work_dir)
                .and_then(|_| std::fs::write(&work_path, &report))
            {
                Ok(_) => println!("[Step 4] ✅ 工作目录副本已保存: {}", work_path.display()),
                Err(e) => println!("[Step 4] ⚠️ 工作目录保存失败（沙箱限制）: {}", e),
            }
        }

        // 输出预览
        println!("\n========== HTML 前 500 字符 ==========");
        let preview: String = report.chars().take(500).collect();
        println!("{}", preview);
        println!("\n========== 验证通过 ✅ ==========");
        println!("文件位置: {}", file_path.display());
    }
}
