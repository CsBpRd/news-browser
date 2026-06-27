use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn app_data_dir() -> PathBuf {
    // Use Tauri's app data directory resolution
    // For macOS: ~/Library/Application Support/com.cbr.tonglan/
    // We fall back to a platform-appropriate default
    if let Some(data_dir) = dirs_next() {
        return data_dir;
    }
    // Fallback
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default = "default_app_name")]
    pub app_name: String,
    #[serde(default = "default_period")]
    pub period: String, // "daily", "weekly", "monthly"
    #[serde(default = "default_work_dir")]
    pub work_dir: String,
    #[serde(default = "default_theme")]
    pub theme: String, // "dark", "light"
    #[serde(default = "default_file_pattern")]
    pub file_pattern: String,
}

fn default_app_name() -> String {
    "通览".to_string()
}
fn default_period() -> String {
    "weekly".to_string()
}
fn default_work_dir() -> String {
    String::new()
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_file_pattern() -> String {
    "report_{YYYY}{MM}{DD}.html".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_name: default_app_name(),
            period: default_period(),
            work_dir: default_work_dir(),
            theme: default_theme(),
            file_pattern: default_file_pattern(),
        }
    }
}

fn settings_path() -> PathBuf {
    let dir = app_data_dir();
    dir.join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<AppSettings>(&content) {
                    Ok(settings) => return settings,
                    Err(e) => {
                        log::warn!("Failed to parse settings.json, using defaults: {}", e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read settings.json, using defaults: {}", e);
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let dir = app_data_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

    let path = settings_path();
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_settings() -> AppSettings {
    load_settings()
}

#[tauri::command]
pub fn save_settings_command(settings: AppSettings) -> Result<(), String> {
    save_settings(&settings)
}
