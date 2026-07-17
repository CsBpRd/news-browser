use serde::{Deserialize, Serialize};

/// 单次搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

impl SearchResult {
    pub fn title_or_fallback(&self) -> &str {
        if self.title.is_empty() {
            "(无标题)"
        } else {
            &self.title
        }
    }
}

/// 单个段落的研究进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Research {
    pub search_history: Vec<SearchResult>,
    pub latest_summary: String,
    pub reflection_iteration: u32,
}

impl Default for Research {
    fn default() -> Self {
        Self {
            search_history: Vec::new(),
            latest_summary: String::new(),
            reflection_iteration: 0,
        }
    }
}

/// 报告中的一个段落
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub title: String,
    pub content: String,
    pub research: Research,
}

/// 整个报告的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub report_title: String,
    pub paragraphs: Vec<Paragraph>,
}

impl State {
    pub fn new(report_title: String) -> Self {
        Self {
            report_title,
            paragraphs: Vec::new(),
        }
    }
}
