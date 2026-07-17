use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::app_data_dir;

// SECURITY WARNING: API key is stored in plaintext JSON file.
// For production use, consider integrating with system keychain
// (e.g., keyring crate on Linux, security-framework on macOS,
// or Windows Credential Manager) for secure credential storage.

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AIConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            api_endpoint: String::new(),
            api_key: String::new(),
            model_name: String::new(),
            temperature: 0.7,
            max_tokens: 16000,
        }
    }
}

fn config_path() -> PathBuf {
    app_data_dir().join("ai-config.json")
}

#[tauri::command]
pub fn get_ai_config() -> AIConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<AIConfig>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    log::warn!("Failed to parse ai-config.json, using defaults: {}", e);
                }
            },
            Err(e) => {
                log::warn!("Failed to read ai-config.json, using defaults: {}", e);
            }
        }
    }
    AIConfig::default()
}

#[tauri::command]
pub fn save_ai_config(config: AIConfig) -> Result<(), String> {
    if config.temperature < 0.0 || config.temperature > 2.0 {
        return Err("temperature must be between 0 and 2".to_string());
    }
    if config.max_tokens == 0 || config.max_tokens > 128000 {
        return Err("max_tokens must be between 1 and 128000".to_string());
    }

    let dir = app_data_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    let path = config_path();
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化AI配置失败: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("保存AI配置失败: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AIConfig::default();
        assert_eq!(config.api_endpoint, "");
        assert_eq!(config.api_key, "");
        assert_eq!(config.model_name, "");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 16000);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = AIConfig {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: "test-key".to_string(),
            model_name: "gpt-4".to_string(),
            temperature: 0.5,
            max_tokens: 8000,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AIConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.api_endpoint, config.api_endpoint);
        assert_eq!(deserialized.temperature, config.temperature);
        assert_eq!(deserialized.max_tokens, config.max_tokens);
    }

    #[test]
    fn test_deserialize_missing_fields_uses_defaults() {
        // Simulate an old config file missing new fields
        let json = r#"{"api_endpoint":"https://api.example.com"}"#;
        let config: AIConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_endpoint, "https://api.example.com");
        assert_eq!(config.api_key, ""); // default
        assert_eq!(config.temperature, 0.7); // default
        assert_eq!(config.max_tokens, 16000); // default
    }

    #[test]
    fn test_deserialize_empty_json_uses_defaults() {
        let json = "{}";
        let config: AIConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 16000);
    }

    #[test]
    fn test_validate_temperature_ok() {
        let config = AIConfig { temperature: 0.0, ..Default::default() };
        assert!(save_ai_config(config).is_ok());

        let config = AIConfig { temperature: 2.0, ..Default::default() };
        assert!(save_ai_config(config).is_ok());
    }

    #[test]
    fn test_validate_temperature_fail() {
        let config = AIConfig { temperature: -0.1, ..Default::default() };
        assert!(save_ai_config(config).is_err());

        let config = AIConfig { temperature: 2.1, ..Default::default() };
        assert!(save_ai_config(config).is_err());
    }

    #[test]
    fn test_validate_max_tokens_ok() {
        let config = AIConfig { max_tokens: 1, ..Default::default() };
        assert!(save_ai_config(config).is_ok());

        let config = AIConfig { max_tokens: 128000, ..Default::default() };
        assert!(save_ai_config(config).is_ok());
    }

    #[test]
    fn test_validate_max_tokens_fail() {
        let config = AIConfig { max_tokens: 0, ..Default::default() };
        assert!(save_ai_config(config).is_err());

        let config = AIConfig { max_tokens: 128001, ..Default::default() };
        assert!(save_ai_config(config).is_err());
    }
}
