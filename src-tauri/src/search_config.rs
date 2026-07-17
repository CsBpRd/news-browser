use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    pub provider: SearchProvider,
    pub api_key: String,
    pub api_endpoint: String,
    pub max_results: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchProvider {
    #[serde(rename = "tavily")]
    Tavily,
    #[serde(rename = "custom")]
    Custom,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            provider: SearchProvider::Tavily,
            api_key: String::new(),
            api_endpoint: String::new(),
            max_results: 10,
        }
    }
}

fn app_data_dir() -> PathBuf {
    if let Some(data_dir) = dirs_next() {
        return data_dir;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".tonglan")
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join("Library/Application Support/com.cbr.tonglan"))
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".local/share/tonglan"))
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").ok()?;
        Some(PathBuf::from(appdata).join("tonglan"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".tonglan"))
    }
}

fn config_path() -> PathBuf {
    app_data_dir().join("search-config.json")
}

#[tauri::command]
pub fn get_search_config() -> SearchConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<SearchConfig>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    log::warn!("Failed to parse search-config.json, using defaults: {}", e);
                }
            },
            Err(e) => {
                log::warn!("Failed to read search-config.json, using defaults: {}", e);
            }
        }
    }
    SearchConfig::default()
}

#[tauri::command]
pub fn save_search_config(config: SearchConfig) -> Result<(), String> {
    // Tavily 和自定义 API 都需要 API Key
    if config.api_key.is_empty() {
        return Err("搜索API密钥不能为空".to_string());
    }
    
    // 自定义 API 需要 API 地址
    if config.provider == SearchProvider::Custom && config.api_endpoint.is_empty() {
        return Err("自定义API地址不能为空".to_string());
    }
    
    let dir = app_data_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    
    let path = config_path();
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化搜索配置失败: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("保存搜索配置失败: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_config() {
        let config = SearchConfig::default();
        assert_eq!(config.provider, SearchProvider::Tavily);
        assert_eq!(config.max_results, 10);
    }
}
