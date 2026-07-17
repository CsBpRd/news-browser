pub mod ai_config;
pub mod agent;
mod config;
mod prompt_template;
mod search_config;
mod search_tools;
mod settings;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportInfo {
    pub filename: String,
    pub path: String,
    pub date: String, // YYYY-MM-DD
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub size: u64,
    pub size_display: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportsData {
    pub reports: Vec<ReportInfo>,
    pub total_count: usize,
    pub total_size: u64,
    pub total_size_display: String,
    pub years: Vec<i32>,
    pub months: Vec<(String, u32)>,
    pub earliest_date: String,
    pub latest_date: String,
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn build_regex(pattern: &str, app_name: &str) -> Result<Regex, String> {
    // Replace placeholders with regex capture groups
    let mut rx_str = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            let mut placeholder = String::new();
            i += 1;
            while i < chars.len() && chars[i] != '}' {
                placeholder.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip '}'
            }
            match placeholder.as_str() {
                "name" => rx_str.push_str(&regex::escape(app_name)),
                "YYYY" => rx_str.push_str(r"(\d{4})"),
                "YY" => rx_str.push_str(r"(\d{2})"),
                "MM" => rx_str.push_str(r"(\d{2})"),
                "DD" => rx_str.push_str(r"(\d{2})"),
                _ => {
                    rx_str.push('{');
                    rx_str.push_str(&placeholder);
                    rx_str.push('}');
                }
            }
        } else {
            // Escape special regex chars for literal parts
            let c = chars[i];
            if ".+*?^$()[]{}|\\".contains(c) {
                rx_str.push('\\');
            }
            rx_str.push(c);
            i += 1;
        }
    }
    rx_str.push('$');
    rx_str.insert(0, '^');

    Regex::new(&rx_str).map_err(|e| format!("Invalid file pattern: {}", e))
}

#[tauri::command]
fn scan_reports(work_dir: String, file_pattern: String, app_name: String) -> Result<ReportsData, String> {
    let path = Path::new(&work_dir);
    if !path.exists() {
        return Err(format!("目录不存在: {}", work_dir));
    }
    if !path.is_dir() {
        return Err(format!("路径不是目录: {}", work_dir));
    }

    let re = build_regex(&file_pattern, &app_name)?;

    let mut reports: Vec<ReportInfo> = Vec::new();

    let entries = fs::read_dir(path).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let filename = entry.file_name().to_string_lossy().to_string();
        let file_path = entry.path().to_string_lossy().to_string();
        let metadata = entry.metadata().map_err(|e| format!("读取元数据失败: {}", e))?;

        if !metadata.is_file() {
            continue;
        }

        if let Some(caps) = re.captures(&filename) {
            // The capture groups in order: name (if present), YYYY, (YY optional), MM, DD
            // We need to figure out which groups captured what based on the pattern
            // For simplicity, find year (4-digit), month (2-digit), day (2-digit) from captures
            // Group 1 is the first numeric capture
            let mut year: i32 = 0;
            let mut month: u32 = 0;
            let mut day: u32 = 0;

            // Go through captures to find YYYY, MM, DD
            let mut cap_idx = 1;
            {
                // Parse placeholders to map capture groups
                let actual: Vec<&str> = file_pattern.split(&['{', '}'][..]).collect();
                // Index 1,3,5,7... are the placeholder names
                for idx in (1..actual.len()).step_by(2) {
                    let placeholder = actual[idx];
                    if cap_idx <= caps.len() {
                        match placeholder {
                            "YYYY" => {
                                year = caps.get(cap_idx)
                                    .and_then(|m| m.as_str().parse().ok())
                                    .unwrap_or(0);
                                cap_idx += 1;
                            }
                            "YY" => {
                                let yy: i32 = caps.get(cap_idx)
                                    .and_then(|m| m.as_str().parse().ok())
                                    .unwrap_or(0);
                                year = 2000 + yy;
                                cap_idx += 1;
                            }
                            "MM" => {
                                month = caps.get(cap_idx)
                                    .and_then(|m| m.as_str().parse().ok())
                                    .unwrap_or(0);
                                cap_idx += 1;
                            }
                            "DD" => {
                                day = caps.get(cap_idx)
                                    .and_then(|m| m.as_str().parse().ok())
                                    .unwrap_or(0);
                                cap_idx += 1;
                            }
                            "name" => {
                                // name is not a capture, skip
                            }
                            _ => {
                                cap_idx += 1;
                            }
                        }
                    }
                }
            }

            if year == 0 || month == 0 || day == 0 {
                continue; // skip files that don't have valid date captures
            }

            let size = metadata.len();

            reports.push(ReportInfo {
                filename: filename.clone(),
                path: file_path.clone(),
                date: format!("{}-{:02}-{:02}", year, month, day),
                year,
                month,
                day,
                size,
                size_display: format_file_size(size),
            });
        }
    }

    // Sort by date descending
    reports.sort_by(|a, b| b.date.cmp(&a.date));

    let mut years: Vec<i32> = reports.iter().map(|r| r.year).collect();
    years.sort();
    years.dedup();

    // Chinese month names
    let month_names = [
        "一月", "二月", "三月", "四月", "五月", "六月",
        "七月", "八月", "九月", "十月", "十一月", "十二月",
    ];

    let mut months: Vec<(String, u32)> = Vec::new();
    for &year in &years {
        let mut year_months: Vec<u32> = reports
            .iter()
            .filter(|r| r.year == year)
            .map(|r| r.month)
            .collect();
        year_months.sort();
        year_months.dedup();
        for m in year_months {
            let label = format!("{}年 {}", year, month_names[(m - 1) as usize]);
            months.push((label, m));
        }
    }

    let total_size: u64 = reports.iter().map(|r| r.size).sum();
    let earliest_date = reports.last().map(|r| r.date.clone()).unwrap_or_default();
    let latest_date = reports.first().map(|r| r.date.clone()).unwrap_or_default();

    Ok(ReportsData {
        total_count: reports.len(),
        total_size,
        total_size_display: format_file_size(total_size),
        years,
        months,
        earliest_date,
        latest_date,
        reports,
    })
}

#[tauri::command]
fn read_report(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取文件失败 {}: {}", path, e))
}

#[tauri::command]
fn detect_pattern(work_dir: String) -> Result<Option<String>, String> {
    let path = Path::new(&work_dir);
    if !path.exists() || !path.is_dir() {
        return Ok(None);
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };

    // Collect visible filenames (skip hidden files)
    let mut filenames: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if entry.path().is_file() {
            filenames.push(name);
        }
    }

    if filenames.is_empty() {
        return Ok(None);
    }

    // Try various date patterns, from most specific to least
    let patterns: Vec<(&str, &str)> = vec![
        // prefix_YYYYMMDD.ext
        (r"^(.+)_(\d{4})(\d{2})(\d{2})\.(.+)$", "{prefix}_{YYYY}{MM}{DD}.{ext}"),
        // prefix-YYYY-MM-DD.ext
        (r"^(.+)-(\d{4})-(\d{2})-(\d{2})\.(.+)$", "{prefix}-{YYYY}-{MM}-{DD}.{ext}"),
        // prefix_YYYY-MM-DD.ext
        (r"^(.+)_(\d{4})-(\d{2})-(\d{2})\.(.+)$", "{prefix}_{YYYY}-{MM}-{DD}.{ext}"),
        // prefixYYYYMMDD.ext (no separator)
        (r"^(.+)(\d{4})(\d{2})(\d{2})\.(.+)$", "{prefix}{YYYY}{MM}{DD}.{ext}"),
        // YYYYMMDD_prefix.ext (date at start)
        (r"^(\d{4})(\d{2})(\d{2})_(.+)\.(.+)$", "{YYYY}{MM}{DD}_{prefix}.{ext}"),
        // prefix_YYYY_MM_DD.ext
        (r"^(.+)_(\d{4})_(\d{2})_(\d{2})\.(.+)$", "{prefix}_{YYYY}_{MM}_{DD}.{ext}"),
    ];

    let mut best_pattern: Option<String> = None;
    let mut best_count = 0;

    for (regex_str, template) in &patterns {
        let re = match Regex::new(regex_str) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut match_count = 0;
        let mut prefix = String::new();
        let mut ext = String::new();

        for name in &filenames {
            if let Some(caps) = re.captures(name) {
                match_count += 1;
                if prefix.is_empty() {
                    // For patterns starting with prefix
                    if template.contains("{prefix}") {
                        prefix = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    }
                    if template.contains("{ext}") {
                        let ext_idx = caps.len() - 1;
                        ext = caps.get(ext_idx).map(|m| m.as_str().to_string()).unwrap_or_default();
                    }
                }
            }
        }

        if match_count > best_count {
            best_count = match_count;
            let mut pattern = template.replace("{prefix}", &prefix);
            pattern = pattern.replace("{ext}", &ext);
            best_pattern = Some(pattern);
        }
    }

    // If we found a pattern matching multiple files, return it
    if best_count >= 2 {
        Ok(best_pattern)
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn pick_work_dir(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .blocking_pick_folder();
    Ok(path.map(|p| p.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动时自动升级历史配置：将过小的 max_tokens 提升到 16000
    // 修复早期版本默认 4000 导致 HTML 报告被截断的问题
    upgrade_legacy_ai_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            scan_reports,
            read_report,
            detect_pattern,
            pick_work_dir,
            settings::get_settings,
            settings::save_settings_command,
            ai_config::get_ai_config,
            ai_config::save_ai_config,
            prompt_template::get_prompt_template,
            prompt_template::save_prompt_template,
            agent::generate_newspaper,
            search_config::get_search_config,
            search_config::save_search_config,
            search_tools::search_web_command,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 启动时升级历史 ai-config：将 max_tokens < 16000 提升到 16000
/// 静默失败，不影响 app 启动
fn upgrade_legacy_ai_config() {
    let mut config = ai_config::get_ai_config();
    if config.max_tokens > 0 && config.max_tokens < 16000 {
        config.max_tokens = 16000;
        if let Err(e) = ai_config::save_ai_config(config) {
            log::warn!("升级 ai-config max_tokens 失败: {}", e);
        } else {
            log::info!("已自动升级 ai-config max_tokens 到 16000");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_build_regex_basic() {
        let re = build_regex("{name}_{YYYY}{MM}{DD}.html", "通览").unwrap();
        assert!(re.is_match("通览_20260623.html"));
        assert!(!re.is_match("科技新闻周报_20260623.html"));
    }

    #[test]
    fn test_build_regex_literal_name() {
        let re = build_regex("科技新闻周报_{YYYY}{MM}{DD}.html", "通览").unwrap();
        assert!(re.is_match("科技新闻周报_20260623.html"));
        assert!(!re.is_match("通览_20260623.html"));
    }

    #[test]
    fn test_build_regex_md() {
        let re = build_regex("{name}_{YYYY}{MM}{DD}.md", "news").unwrap();
        assert!(re.is_match("news_20260623.md"));
        assert!(!re.is_match("news_20260623.html"));
    }

    #[test]
    fn test_scan_reports() {
        let tmp = std::env::temp_dir().join("tonglan_test_scan");
        fs::create_dir_all(&tmp).unwrap();

        let content = "<html><body>Test</body></html>";
        let files = [
            "科技新闻周报_20260623.html",
            "科技新闻周报_20260620.html",
            "科技新闻周报_20260515.html",
        ];
        for f in &files {
            let mut file = File::create(tmp.join(f)).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let result = scan_reports(
            tmp.to_string_lossy().to_string(),
            "科技新闻周报_{YYYY}{MM}{DD}.html".to_string(),
            "通览".to_string(),
        )
        .unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.reports.len(), 3);
        // Should be sorted by date descending
        assert_eq!(result.reports[0].filename, "科技新闻周报_20260623.html");
        assert_eq!(result.reports[0].year, 2026);
        assert_eq!(result.reports[0].month, 6);
        assert_eq!(result.reports[0].day, 23);
        assert_eq!(result.reports[2].filename, "科技新闻周报_20260515.html");
        assert_eq!(result.years, vec![2026]);
        assert_eq!(result.latest_date, "2026-06-23");
        assert_eq!(result.earliest_date, "2026-05-15");

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_scan_reports_with_name_placeholder() {
        let tmp = std::env::temp_dir().join("tonglan_test_name");
        fs::create_dir_all(&tmp).unwrap();

        let mut file = File::create(tmp.join("通览_20260627.html")).unwrap();
        file.write_all(b"<html>Test</html>").unwrap();

        let result = scan_reports(
            tmp.to_string_lossy().to_string(),
            "{name}_{YYYY}{MM}{DD}.html".to_string(),
            "通览".to_string(),
        )
        .unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.reports[0].filename, "通览_20260627.html");

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_scan_reports_empty_dir() {
        let tmp = std::env::temp_dir().join("tonglan_test_empty");
        fs::create_dir_all(&tmp).unwrap();

        let result = scan_reports(
            tmp.to_string_lossy().to_string(),
            "{name}_{YYYY}{MM}{DD}.html".to_string(),
            "通览".to_string(),
        )
        .unwrap();

        assert_eq!(result.total_count, 0);

        fs::remove_dir_all(&tmp).unwrap();
    }
}
