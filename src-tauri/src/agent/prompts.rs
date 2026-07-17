use serde_json::json;

/// 生成报告结构（大纲）
pub fn prompt_report_structure() -> (String, String) {
    let system = r#"你是一位深度研究助手。给定一个查询，你需要规划一个报告的结构和其中包含的段落。最多五个段落。
确保段落的排序合理有序。

请输出XML格式：
<paragraphs>
  <paragraph>
    <title>段落标题</title>
    <content>段落的预期内容描述</content>
  </paragraph>
</paragraphs>

只返回XML，不要有解释或额外文本。"#;
    (system.to_string(), String::new())
}

/// 每个段落第一次搜索
pub fn prompt_first_search(paragraph_title: &str, paragraph_content: &str) -> (String, String) {
    let system = r#"你是一位深度研究助手。你将获得报告中的一个段落，请思考这个主题并提供最佳的网络搜索查询。

请输出XML格式：
<search_query>搜索关键词</search_query>
<reasoning>选择这个搜索词的原因</reasoning>

只返回XML，不要有解释或额外文本。"#;
    let user = json!({"title": paragraph_title, "content": paragraph_content}).to_string();
    (system.to_string(), user)
}

/// 每个段落第一次总结
pub fn prompt_first_summary(
    title: &str,
    content: &str,
    search_query: &str,
    search_results: &[super::state::SearchResult],
) -> (String, String) {
    let system = r#"你是一位深度研究助手。使用搜索结果撰写与段落主题一致的内容，并适当地组织结构以便纳入报告中。

请输出XML格式：
<paragraph_latest_state>段落的完整内容</paragraph_latest_state>

只返回XML，不要有解释或额外文本。"#;

    let results_text: Vec<String> = search_results
        .iter()
        .map(|r| format!("标题: {}\n链接: {}\n内容: {}", r.title_or_fallback(), r.url, r.content))
        .collect();

    let user = json!({
        "title": title,
        "content": content,
        "search_query": search_query,
        "search_results": results_text
    })
    .to_string();

    (system.to_string(), user)
}

/// 反思
pub fn prompt_reflection(title: &str, content: &str, latest_summary: &str) -> (String, String) {
    let system = r#"你是一位深度研究助手。反思段落文本的当前状态，思考是否遗漏了主题的某些关键方面，并提供最佳的网络搜索查询来丰富最新状态。

请输出XML格式：
<search_query>搜索关键词</search_query>
<reasoning>选择这个搜索词的原因</reasoning>

只返回XML，不要有解释或额外文本。"#;

    let user = json!({
        "title": title,
        "content": content,
        "paragraph_latest_state": latest_summary
    })
    .to_string();

    (system.to_string(), user)
}

/// 总结反思
pub fn prompt_reflection_summary(
    title: &str,
    content: &str,
    search_query: &str,
    search_results: &[super::state::SearchResult],
    latest_summary: &str,
) -> (String, String) {
    let system = r#"你是一位深度研究助手。根据搜索结果和预期内容丰富段落的当前最新状态。
不要删除最新状态中的关键信息，尽量丰富它，只添加缺失的信息。

请输出XML格式：
<updated_paragraph_latest_state>更新后的段落内容</updated_paragraph_latest_state>

只返回XML，不要有解释或额外文本。"#;

    let results_text: Vec<String> = search_results
        .iter()
        .map(|r| format!("标题: {}\n链接: {}\n内容: {}", r.title_or_fallback(), r.url, r.content))
        .collect();

    let user = json!({
        "title": title,
        "content": content,
        "search_query": search_query,
        "search_results": results_text,
        "paragraph_latest_state": latest_summary
    })
    .to_string();

    (system.to_string(), user)
}

/// 最终HTML格式化
pub fn prompt_format_html(reference: Option<&str>) -> (String, String) {
    let base = r#"你是一位专业科技新闻编辑。将研究数据格式化为一份科技新闻周报HTML。
参考下方提供的参考HTML格式（如果有的话），生成完全同样风格的HTML。

参考HTML格式要点：
- 顶部报头：kicker + 标题 + 日期 + 元信息（新闻条数、分类数等）
- 分类导航（cat-nav）：每个类别一个锚点链接
- 统计条（stats-bar）：几个数据卡片
- 每个分类一个section，带emoji图标
- 每条新闻是一个news-card，包含编号圆圈、标题、元信息、正文、"值得关注"高亮框、引用角标
- 底部参考来源列表（有序列表，含标题和URL）
- 使用CSS变量定义颜色，内联CSS
- 浅色主题，适合阅读

使用段落标题来创建分类，从研究内容提取关键新闻条目。
如果没有结论段落，请在末尾添加一个结论章节。

只返回完整HTML代码，不要有任何解释、思考过程或额外文本。直接用 ```html 包裹。"#;

    let mut system = base.to_string();
    if let Some(ref_html) = reference {
        let excerpt: String = ref_html.chars().take(3000).collect();
        system.push_str("\n\n===== 参考HTML格式（前3000字符）=====\n");
        system.push_str(&excerpt);
    }

    (system, String::new())
}

/// 最终Markdown格式化
pub fn prompt_format_md() -> (String, String) {
    let system = r#"你是一位深度研究助手。将研究数据格式化为美观的Markdown格式。
如果没有结论段落，请根据其他段落的内容在报告末尾添加一个结论。
使用段落标题来创建报告的标题。

只返回Markdown内容，不要有解释或额外文本。用 ```markdown 包裹内容。"#;
    (system.to_string(), String::new())
}
