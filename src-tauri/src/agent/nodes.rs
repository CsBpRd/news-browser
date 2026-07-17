use std::sync::OnceLock;
use tauri::Emitter;

use crate::ai_config::AIConfig;
use crate::search_config::SearchConfig;

use super::prompts;
use super::state::{Paragraph, Research, SearchResult, State};

/// ----------------------------------------
/// HTTP 客户端
/// ----------------------------------------
fn get_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    })
}

/// ----------------------------------------
/// LLM 调用
/// ----------------------------------------
const LLM_MAX_RETRIES: usize = 3;
const LLM_BASE_DELAY_MS: u64 = 1000;

/// 计算第 n 次重试的等待时间（指数退避 + 抖动）
fn retry_delay_ms(attempt: usize) -> u64 {
    let base = LLM_BASE_DELAY_MS * 2u64.saturating_pow(attempt as u32);
    // 简单抖动：使用当前时间戳尾数作为偏移
    let jitter = (chrono::Utc::now().timestamp_millis() as u64) % 200;
    base + jitter
}

async fn call_llm(
    config: &AIConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    let client = get_http_client();
    let today = chrono::Local::now().format("%Y年%m月%d日").to_string();
    let dated_system = format!("当前日期：{}。\n\n{}", today, system_prompt);

    let mut messages = vec![
        serde_json::json!({"role": "system", "content": dated_system}),
    ];
    if !user_prompt.is_empty() {
        messages.push(serde_json::json!({"role": "user", "content": user_prompt}));
    }

    let request = serde_json::json!({
        "model": config.model_name,
        "messages": messages,
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "stream": false
    });

    let mut last_err: Option<String> = None;

    for attempt in 0..=LLM_MAX_RETRIES {
        if attempt > 0 {
            let delay = retry_delay_ms(attempt - 1);
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        let response = client
            .post(&config.api_endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    let data: serde_json::Value = resp
                        .json()
                        .await
                        .map_err(|e| format!("解析响应失败: {}", e))?;

                    return extract_llm_content(&data);
                }

                let status_code = status.as_u16();
                let body = resp.text().await.unwrap_or_default();
                let err_msg = format!("LLM返回错误 ({}): {}", status, body);

                // 429、5xx 可重试；其他 4xx 直接返回
                let retryable = status_code == 429
                    || (500..600).contains(&status_code);
                if !retryable {
                    return Err(err_msg);
                }
                last_err = Some(err_msg);
            }
            Err(e) => {
                // 网络错误一律可重试
                last_err = Some(format!("LLM调用失败: {}", e));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "LLM调用失败：超过最大重试次数".to_string()))
}

/// 从 OpenAI 兼容响应中提取文本内容
/// 兼容多种字段：content / reasoning_content / text，以及错误响应
fn extract_llm_content(data: &serde_json::Value) -> Result<String, String> {
    // 1. 检查是否是错误响应
    if let Some(err) = data.get("error") {
        let msg = err["message"].as_str()
            .or_else(|| err.get("message").and_then(|m| m.as_str()))
            .unwrap_or("未知错误");
        let code = err["code"].as_str().or_else(|| err["type"].as_str()).unwrap_or("unknown");
        return Err(format!("LLM API 返回错误 [{}]: {}", code, msg));
    }

    // 2. 优先标准 OpenAI 字段 choices[0].message.content
    let msg = &data["choices"][0]["message"];
    if !msg.is_null() {
        // content 字段（标准）
        if let Some(s) = msg["content"].as_str() {
            if !s.is_empty() {
                return Ok(s.to_string());
            }
        }
        // content 是数组（部分 provider 会用数组结构）
        if let Some(arr) = msg["content"].as_array() {
            let joined: Vec<String> = arr.iter()
                .filter_map(|v| {
                    v.get("text").and_then(|t| t.as_str())
                        .or_else(|| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect();
            if !joined.is_empty() {
                return Ok(joined.join("\n"));
            }
        }
        // reasoning_content 字段（DeepSeek 等推理模型）
        if let Some(s) = msg["reasoning_content"].as_str() {
            if !s.is_empty() {
                return Ok(s.to_string());
            }
        }
        // thinking 字段（部分模型）
        if let Some(s) = msg["thinking"].as_str() {
            if !s.is_empty() {
                return Ok(s.to_string());
            }
        }
        // text 字段（部分 provider）
        if let Some(s) = msg["text"].as_str() {
            if !s.is_empty() {
                return Ok(s.to_string());
            }
        }
    }

    // 3. 直接取 choices[0].text（旧版 completions 接口）
    if let Some(s) = data["choices"][0]["text"].as_str() {
        if !s.is_empty() {
            return Ok(s.to_string());
        }
    }

    // 4. 直接取 content 字段（个别 provider 顶层返回）
    if let Some(s) = data["content"].as_str() {
        if !s.is_empty() {
            return Ok(s.to_string());
        }
    }

    // 5. 所有字段都为空，输出响应结构以便诊断
    let preview = serde_json::to_string_pretty(data)
        .unwrap_or_else(|_| format!("{:?}", data))
        .chars()
        .take(500)
        .collect::<String>();
    Err(format!(
        "LLM返回内容为空。响应结构预览：\n{}",
        preview
    ))
}

/// ----------------------------------------
/// XML 解析工具
/// ----------------------------------------

/// 从文本中提取第一个 XML 标签的内容
fn extract_xml(text: &str, tag: &str) -> Option<String> {
    let text = text.trim();
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    // 先找开放标签，支持自闭合 <tag/>
    if let Some(start) = text.find(&open) {
        let after_open = &text[start + open.len()..];
        if let Some(end) = after_open.find(&close) {
            let content = after_open[..end].trim();
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    None
}

/// 提取 XML 标签内容，失败时返回默认值
fn extract_xml_or<'a>(text: &'a str, tag: &str, default: &'a str) -> String {
    extract_xml(text, tag).unwrap_or_else(|| default.to_string())
}

/// 提取所有匹配的 XML 标签内容
fn extract_xml_all(text: &str, tag: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut remaining = text.trim();
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    loop {
        match remaining.find(&open) {
            Some(start) => {
                let after_open = &remaining[start + open.len()..];
                match after_open.find(&close) {
                    Some(end) => {
                        let content = after_open[..end].trim().to_string();
                        if !content.is_empty() {
                            results.push(content);
                        }
                        remaining = &after_open[end + close.len()..];
                    }
                    None => break,
                }
            }
            None => break,
        }
    }
    results
}

/// 从 LLM 响应中提取代码块内容（处理 ```html ```markdown 等包裹）
fn extract_code_block(text: &str, lang: &str) -> Option<String> {
    let text = text.trim();
    // 尝试 ```lang ... ```
    let open_fence = format!("```{}", lang);
    if let Some(start) = text.find(&open_fence) {
        let after_start = &text[start + open_fence.len()..];
        let content_start = after_start.find('\n').map(|i| i + 1).unwrap_or(0);
        let after_newline = &after_start[content_start..];
        if let Some(end) = after_newline.rfind("```") {
            let content = after_newline[..end].trim().to_string();
            if !content.is_empty() {
                return Some(content);
            }
        }
    }
    None
}

/// ----------------------------------------
/// HTML 清洗组件
/// ----------------------------------------

/// 清洗 LLM 输出的 HTML，移除思考过程等非内容文本，返回干净的 HTML
fn clean_html_output(raw: &str) -> Result<String, String> {
    let raw = raw.trim();

    if raw.is_empty() {
        return Err("LLM返回内容为空".to_string());
    }

    // 1. 优先尝试提取 ```html 代码块
    if let Some(html) = extract_code_block(raw, "html") {
        if has_html_structure(&html) {
            return Ok(html);
        }
    }

    // 2. 尝试查找 <!DOCTYPE 或 <html 标签
    if let Some(start) = raw.find("<!DOCTYPE").or_else(|| raw.find("<html")) {
        // 从 DOCTYPE/html 开始，到 </html> 结束
        let content = if let Some(end) = raw.rfind("</html>") {
            &raw[start..end + 7]
        } else {
            &raw[start..]
        };
        let cleaned = content.trim();
        if !cleaned.is_empty() && has_html_structure(cleaned) {
            return Ok(cleaned.to_string());
        }
    }

    // 3. 找第一个 ``` 代码块（任何语言）
    if let Some(html) = extract_code_block(raw, "") {
        if has_html_structure(&html) {
            return Ok(html);
        }
        // 即使没有完整HTML结构，有内容就返回
        if !html.is_empty() {
            return Ok(html);
        }
    }

    // 4. 尝试移除 think/reasoning 标签（DeepSeek等模型的思考过程）
    let cleaned = strip_thinking_tags(raw);
    if has_html_structure(&cleaned) {
        return Ok(cleaned);
    }

    // 5. 如果 raw 本身就包含 HTML 标签，直接返回
    if raw.contains('<') && raw.contains('>') && raw.len() > 100 {
        return Ok(raw.to_string());
    }

    Err(format!("无法提取有效HTML内容。原始响应前200字符: {}", &raw.chars().take(200).collect::<String>()))
}

/// 检查字符串是否有基本的HTML结构
fn has_html_structure(s: &str) -> bool {
    s.contains('<') && s.contains('>') && s.len() > 50
}

/// 剥离 think/reasoning XML 标签及其内容
fn strip_thinking_tags(text: &str) -> String {
    let mut result = text.to_string();
    // 反复移除直到没有更多匹配
    loop {
        let before = result.clone();
        // 移除 <think>...</think>
        if let Some(start) = result.find("<think>") {
            if let Some(end) = result[start..].find("</think>") {
                result.replace_range(start..start + end + 8, "");
                continue;
            }
        }
        // 移除 <reasoning>...</reasoning>
        if let Some(start) = result.find("<reasoning>") {
            if let Some(end) = result[start..].find("</reasoning>") {
                result.replace_range(start..start + end + 12, "");
                continue;
            }
        }
        if result == before {
            break;
        }
    }
    result.trim().to_string()
}

/// ----------------------------------------
/// 1. ReportStructureNode — 生成报告大纲
/// ----------------------------------------
pub async fn run_report_structure(
    config: &AIConfig,
    query: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<State, String> {
    if let Some(h) = app_handle {
        emit_log(h, "thinking", "正在生成报告大纲...".to_string());
    }

    let (system, _) = prompts::prompt_report_structure();
    let response = call_llm(config, &system, query).await?;

    let titles = extract_xml_all(&response, "title");
    let contents = extract_xml_all(&response, "content");

    let mut state = State::new(query.to_string());

    let count = titles.len().max(contents.len()).min(5);
    for i in 0..count {
        let title = titles.get(i).cloned().unwrap_or_else(|| format!("段落{}", i + 1));
        let content = contents.get(i).cloned().unwrap_or_default();
        state.paragraphs.push(Paragraph {
            title,
            content,
            research: Research::default(),
        });
    }

    // 如果XML解析失败，尝试用原始响应作为唯一段落
    if state.paragraphs.is_empty() && !response.trim().is_empty() {
        state.paragraphs.push(Paragraph {
            title: format!("{}深度分析", query.chars().take(20).collect::<String>()),
            content: response.trim().to_string(),
            research: Research::default(),
        });
    }

    let msg = format!("大纲生成完成，共 {} 个段落", state.paragraphs.len());
    if let Some(h) = app_handle {
        emit_log(h, "output", msg);
    }
    Ok(state)
}

/// ----------------------------------------
/// 2. FirstSearchNode — 生成首次搜索查询
/// ----------------------------------------
pub async fn run_first_search(
    config: &AIConfig,
    paragraph: &Paragraph,
) -> Result<(String, String), String> {
    let (system, user) = prompts::prompt_first_search(&paragraph.title, &paragraph.content);
    let response = call_llm(config, &system, &user).await?;

    let search_query = extract_xml_or(&response, "search_query", &paragraph.title);
    let reasoning = extract_xml_or(&response, "reasoning", "");

    Ok((search_query, reasoning))
}

/// ----------------------------------------
/// 3. FirstSummaryNode — 从搜索结果生成初稿
/// ----------------------------------------
pub async fn run_first_summary(
    config: &AIConfig,
    paragraph: &Paragraph,
    search_query: &str,
    search_results: &[SearchResult],
) -> Result<String, String> {
    let (system, user) = prompts::prompt_first_summary(
        &paragraph.title,
        &paragraph.content,
        search_query,
        search_results,
    );
    let response = call_llm(config, &system, &user).await?;

    let summary = extract_xml_or(&response, "paragraph_latest_state", &response);
    Ok(summary)
}

/// ----------------------------------------
/// 4. ReflectionNode — 反思并生成新的搜索查询
/// ----------------------------------------
pub async fn run_reflection(
    config: &AIConfig,
    paragraph: &Paragraph,
) -> Result<(String, String), String> {
    let (system, user) = prompts::prompt_reflection(
        &paragraph.title,
        &paragraph.content,
        &paragraph.research.latest_summary,
    );
    let response = call_llm(config, &system, &user).await?;

    let search_query = extract_xml_or(&response, "search_query", &paragraph.title);
    let reasoning = extract_xml_or(&response, "reasoning", "");

    Ok((search_query, reasoning))
}

/// ----------------------------------------
/// 5. ReflectionSummaryNode — 根据反思搜索结果更新段落
/// ----------------------------------------
pub async fn run_reflection_summary(
    config: &AIConfig,
    paragraph: &Paragraph,
    search_query: &str,
    search_results: &[SearchResult],
) -> Result<String, String> {
    let (system, user) = prompts::prompt_reflection_summary(
        &paragraph.title,
        &paragraph.content,
        search_query,
        search_results,
        &paragraph.research.latest_summary,
    );
    let response = call_llm(config, &system, &user).await?;

    let updated = extract_xml_or(&response, "updated_paragraph_latest_state", &paragraph.research.latest_summary);
    Ok(updated)
}

/// ----------------------------------------
/// 6. ReportFormattingNode — 格式化最终报告
/// ----------------------------------------
pub async fn run_report_formatting_html(
    config: &AIConfig,
    paragraphs: &[Paragraph],
    reference: Option<&str>,
) -> Result<String, String> {
    let report_data: Vec<serde_json::Value> = paragraphs
        .iter()
        .map(|p| {
            serde_json::json!({
                "title": p.title,
                "paragraph_latest_state": p.research.latest_summary
            })
        })
        .collect();

    let message = serde_json::to_string_pretty(&report_data)
        .map_err(|e| format!("序列化失败: {}", e))?;

    let (system, _) = prompts::prompt_format_html(reference);
    let response = call_llm(config, &system, &message).await?;

    // 使用清洗组件处理 LLM 输出
    clean_html_output(&response)
}

pub async fn run_report_formatting_md(
    config: &AIConfig,
    paragraphs: &[Paragraph],
) -> Result<String, String> {
    let report_data: Vec<serde_json::Value> = paragraphs
        .iter()
        .map(|p| {
            serde_json::json!({
                "title": p.title,
                "paragraph_latest_state": p.research.latest_summary
            })
        })
        .collect();

    let message = serde_json::to_string_pretty(&report_data)
        .map_err(|e| format!("序列化失败: {}", e))?;

    let (system, _) = prompts::prompt_format_md();
    let response = call_llm(config, &system, &message).await?;

    // 提取 ```markdown 或 ```md 代码块
    if let Some(md) = extract_code_block(&response, "markdown")
        .or_else(|| extract_code_block(&response, "md"))
    {
        return Ok(md);
    }
    // 如果没有代码块，返回原始响应去除 think 标签
    Ok(strip_thinking_tags(&response))
}

/// ----------------------------------------
/// 搜索工具
/// ----------------------------------------
pub async fn search_tavily(
    query: &str,
    config: &SearchConfig,
    max_results: u32,
) -> Result<Vec<SearchResult>, String> {
    let client = get_http_client();

    let request = serde_json::json!({
        "api_key": config.api_key,
        "query": query,
        "max_results": max_results,
        "search_depth": "basic"
    });

    let response = client
        .post("https://api.tavily.com/search")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Tavily搜索失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Tavily API错误 ({}): {}", status, body));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析Tavily结果失败: {}", e))?;

    let results = data["results"]
        .as_array()
        .ok_or("Tavily返回格式错误")?;

    let search_results: Vec<SearchResult> = results
        .iter()
        .map(|r| SearchResult {
            url: r["url"].as_str().unwrap_or("").to_string(),
            content: r["content"].as_str().unwrap_or("").to_string(),
            title: r["title"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(search_results)
}

/// 日志发射辅助函数
pub fn emit_log(app_handle: &tauri::AppHandle, log_type: &str, content: String) {
    let _ = app_handle.emit(
        "ai-log",
        serde_json::json!({
            "type": log_type,
            "content": content,
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
    );
}
