use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

use crate::search_config::{SearchConfig, SearchProvider};

fn get_search_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create search HTTP client")
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct TavilyRequest {
    api_key: String,
    query: String,
    max_results: u32,
    search_depth: String,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

#[tauri::command]
pub async fn search_web_command(query: String) -> Result<Vec<SearchResult>, String> {
    let config = crate::search_config::get_search_config();
    search_web(&query, &config).await
}

pub async fn search_web(query: &str, config: &SearchConfig) -> Result<Vec<SearchResult>, String> {
    let client = get_search_client();

    match config.provider {
        SearchProvider::Tavily => search_tavily(client, query, config).await,
        SearchProvider::Custom => search_custom(client, query, config).await,
    }
}

async fn search_tavily(
    client: &Client,
    query: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>, String> {
    let request = TavilyRequest {
        api_key: config.api_key.clone(),
        query: query.to_string(),
        max_results: config.max_results,
        search_depth: "basic".to_string(),
    };

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

    let tavily_response: TavilyResponse = response
        .json()
        .await
        .map_err(|e| format!("解析Tavily响应失败: {}", e))?;

    Ok(tavily_response
        .results
        .into_iter()
        .map(|r| SearchResult {
            title: r.title,
            url: r.url,
            content: r.content,
        })
        .collect())
}

async fn search_custom(
    client: &Client,
    query: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>, String> {
    if config.api_endpoint.is_empty() {
        return Err("自定义API地址未配置".to_string());
    }

    let response = client
        .get(&config.api_endpoint)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .query(&[("q", query), ("num", &config.max_results.to_string())])
        .send()
        .await
        .map_err(|e| format!("自定义搜索失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("自定义API错误 ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析自定义API响应失败: {}", e))?;

    // 尝试解析通用格式：{ results: [{ title, url, content/snippet }] }
    let results = json["results"]
        .as_array()
        .or_else(|| json["data"].as_array())
        .or_else(|| json["items"].as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| {
                    Some(SearchResult {
                        title: r["title"].as_str()?.to_string(),
                        url: r["url"].as_str().or(r["link"].as_str())?.to_string(),
                        content: r["content"]
                            .as_str()
                            .or(r["snippet"].as_str())
                            .or(r["description"].as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}

// 抓取网页内容
pub async fn fetch_web_content(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("抓取网页失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("网页返回错误: {}", response.status()));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("读取网页内容失败: {}", e))?;

    // 简单提取文本内容（去掉 HTML 标签）
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re.replace_all(&html, " ");
    let text = text.split_whitespace().collect::<Vec<&str>>().join(" ");

    // 限制内容长度
    if text.len() > 3000 {
        Ok(text[..3000].to_string())
    } else {
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_structure() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            content: "Test content".to_string(),
        };
        assert_eq!(result.title, "Test");
    }
}
