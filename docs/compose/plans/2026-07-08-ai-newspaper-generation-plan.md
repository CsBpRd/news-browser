# AI报刊生成功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为news-browser项目添加AI生成报刊功能，允许用户自定义AI配置和提示词模板，生成HTML格式的报刊并保存到工作目录。

**Architecture:** 前端使用React组件实现配置界面和生成触发，后端使用Tauri命令实现AI API调用和文件生成。配置存储在独立的JSON文件中。

**Tech Stack:** React, TypeScript, Tauri, Rust, JSON配置文件

## Global Constraints

- 所有配置存储在独立的JSON文件中，不与现有设置混合
- AI API调用通过Tauri后端实现，确保API密钥安全
- 生成的HTML文件必须保存到用户配置的工作目录
- 提示词模板支持占位符替换，默认模板必须可用
- 生成过程必须显示详细进度信息
- 错误情况下必须显示具体的错误信息

---

## File Structure

### 前端文件
- `src/components/AIConfigTab.tsx` - AI配置标签页组件
- `src/components/PromptTemplateTab.tsx` - 提示词模板标签页组件
- `src/components/GenerateConfirmDialog.tsx` - 生成确认对话框组件
- `src/components/GenerateStatus.tsx` - 生成状态显示组件
- `src/ai-config.ts` - AI配置管理逻辑
- `src/prompt-template.ts` - 提示词模板管理逻辑

### 后端文件
- `src-tauri/src/ai_config.rs` - AI配置读写命令
- `src-tauri/src/prompt_template.rs` - 提示词模板读写命令
- `src-tauri/src/ai_generator.rs` - AI生成命令和API调用逻辑

### 配置文件
- `ai-config.json` - AI配置（存储在应用配置目录）
- `prompt-template.json` - 提示词模板配置（存储在应用配置目录）

---

### Task 1: 后端AI配置管理

**Covers:** S1

**Files:**
- Create: `src-tauri/src/ai_config.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `get_ai_config() -> Result<AIConfig, String>`, `save_ai_config(config: AIConfig) -> Result<(), String>`

- [ ] **Step 1: 创建AI配置结构体和命令**

```rust
// src-tauri/src/ai_config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            max_tokens: 4000,
        }
    }
}

fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;
    Ok(config_dir.join("ai-config.json"))
}

#[tauri::command]
pub fn get_ai_config(app: AppHandle) -> Result<AIConfig, String> {
    let path = get_config_path(&app)?;
    if !path.exists() {
        return Ok(AIConfig::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取AI配置失败: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("解析AI配置失败: {}", e))
}

#[tauri::command]
pub fn save_ai_config(app: AppHandle, config: AIConfig) -> Result<(), String> {
    let path = get_config_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化AI配置失败: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("保存AI配置失败: {}", e))
}
```

- [ ] **Step 2: 注册AI配置命令**

```rust
// src-tauri/src/lib.rs 添加到invoke_handler
.invoke_handler(tauri::generate_handler![
    scan_reports,
    read_report,
    detect_pattern,
    pick_work_dir,
    settings::get_settings,
    settings::save_settings_command,
    ai_config::get_ai_config,
    ai_config::save_ai_config,
])
```

- [ ] **Step 3: 测试AI配置命令**

运行Tauri应用，测试AI配置的读取和保存功能。

- [ ] **Step 4: 提交代码**

```bash
git add src-tauri/src/ai_config.rs src-tauri/src/lib.rs
git commit -m "feat: add AI config management backend"
```

---

### Task 2: 后端提示词模板管理

**Covers:** S2

**Files:**
- Create: `src-tauri/src/prompt_template.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `get_prompt_template() -> Result<String, String>`, `save_prompt_template(template: String) -> Result<(), String>`

- [ ] **Step 1: 创建提示词模板管理命令**

```rust
// src-tauri/src/prompt_template.rs
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

const DEFAULT_TEMPLATE: &str = "关注当<周期>的重要动态,筛选 20-30 条有价值的信息，生成<报告名称>的<周期>报特别详细说明事件内容及值得关注的原因，生成的新闻html格式+浅色模式，放到<工作目录>文件夹中，文件名格式：<文件名格式>";

fn get_template_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;
    Ok(config_dir.join("prompt-template.json"))
}

#[tauri::command]
pub fn get_prompt_template(app: AppHandle) -> Result<String, String> {
    let path = get_template_path(&app)?;
    if !path.exists() {
        return Ok(DEFAULT_TEMPLATE.to_string());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取提示词模板失败: {}", e))?;
    let template: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析提示词模板失败: {}", e))?;
    Ok(template["template"].as_str().unwrap_or(DEFAULT_TEMPLATE).to_string())
}

#[tauri::command]
pub fn save_prompt_template(app: AppHandle, template: String) -> Result<(), String> {
    let path = get_template_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let content = serde_json::json!({
        "template": template
    });
    let content_str = serde_json::to_string_pretty(&content)
        .map_err(|e| format!("序列化提示词模板失败: {}", e))?;
    fs::write(&path, content_str)
        .map_err(|e| format!("保存提示词模板失败: {}", e))
}
```

- [ ] **Step 2: 注册提示词模板命令**

```rust
// src-tauri/src/lib.rs 添加到invoke_handler
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
])
```

- [ ] **Step 3: 测试提示词模板命令**

运行Tauri应用，测试提示词模板的读取和保存功能。

- [ ] **Step 4: 提交代码**

```bash
git add src-tauri/src/prompt_template.rs src-tauri/src/lib.rs
git commit -m "feat: add prompt template management backend"
```

---

### Task 3: 后端AI生成功能

**Covers:** S3, S4

**Files:**
- Create: `src-tauri/src/ai_generator.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `get_ai_config()`, `get_prompt_template()`, `get_settings()`
- Produces: `generate_newspaper() -> Result<String, String>`

- [ ] **Step 1: 创建AI生成命令**

```rust
// src-tauri/src/ai_generator.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use chrono::Local;

use crate::ai_config::AIConfig;
use crate::settings::AppSettings;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

fn build_prompt(template: &str, settings: &AppSettings) -> String {
    let period_label = match settings.period.as_str() {
        "daily" => "日",
        "weekly" => "周",
        "monthly" => "月",
        _ => "日",
    };
    
    let today = Local::now();
    let date_str = today.format("%Y%m%d").to_string();
    let filename = settings.file_pattern
        .replace("{YYYY}", &today.format("%Y").to_string())
        .replace("{YY}", &today.format("%y").to_string())
        .replace("{MM}", &today.format("%m").to_string())
        .replace("{DD}", &today.format("%d").to_string())
        .replace("{name}", &settings.app_name);
    
    template
        .replace("<周期>", period_label)
        .replace("<报告名称>", &settings.app_name)
        .replace("<工作目录>", &settings.work_dir)
        .replace("<文件名格式>", &filename)
}

#[tauri::command]
pub async fn generate_newspaper(app: AppHandle) -> Result<String, String> {
    // 读取配置
    let ai_config = crate::ai_config::get_ai_config(app.clone())?;
    let prompt_template = crate::prompt_template::get_prompt_template(app.clone())?;
    let settings = crate::settings::get_settings(app.clone())?;
    
    // 验证配置
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
    
    // 构建提示词
    let prompt = build_prompt(&prompt_template, &settings);
    
    // 调用AI API
    let client = Client::new();
    let request = ChatRequest {
        model: ai_config.model_name.clone(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        temperature: ai_config.temperature,
        max_tokens: ai_config.max_tokens,
    };
    
    let response = client
        .post(&ai_config.api_endpoint)
        .header("Authorization", format!("Bearer {}", ai_config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("调用AI API失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("AI API返回错误 ({}): {}", status, body));
    }
    
    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析AI响应失败: {}", e))?;
    
    let html_content = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    
    if html_content.is_empty() {
        return Err("AI返回的内容为空".to_string());
    }
    
    // 生成文件名
    let today = Local::now();
    let filename = settings.file_pattern
        .replace("{YYYY}", &today.format("%Y").to_string())
        .replace("{YY}", &today.format("%y").to_string())
        .replace("{MM}", &today.format("%m").to_string())
        .replace("{DD}", &today.format("%d").to_string())
        .replace("{name}", &settings.app_name);
    
    let file_path = Path::new(&settings.work_dir).join(&filename);
    
    // 保存文件
    fs::write(&file_path, &html_content)
        .map_err(|e| format!("保存文件失败: {}", e))?;
    
    Ok(format!("生成完成！文件已保存到: {}", file_path.display()))
}
```

- [ ] **Step 2: 添加依赖到Cargo.toml**

```toml
# src-tauri/Cargo.toml [dependencies]
reqwest = { version = "0.12", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 3: 注册AI生成命令**

```rust
// src-tauri/src/lib.rs 添加到invoke_handler
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
    ai_generator::generate_newspaper,
])
```

- [ ] **Step 4: 测试AI生成功能**

运行Tauri应用，配置AI参数后测试生成功能。

- [ ] **Step 5: 提交代码**

```bash
git add src-tauri/src/ai_generator.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add AI newspaper generation backend"
```

---

### Task 4: 前端AI配置管理

**Covers:** S1

**Files:**
- Create: `src/ai-config.ts`
- Create: `src/components/AIConfigTab.tsx`

**Interfaces:**
- Produces: `getAIConfig()`, `saveAIConfig()`, `AIConfigTab` component

- [ ] **Step 1: 创建AI配置管理模块**

```typescript
// src/ai-config.ts
import { invoke } from "@tauri-apps/api/core";

export interface AIConfig {
  api_endpoint: string;
  api_key: string;
  model_name: string;
  temperature: number;
  max_tokens: number;
}

export const DEFAULT_AI_CONFIG: AIConfig = {
  api_endpoint: "",
  api_key: "",
  model_name: "",
  temperature: 0.7,
  max_tokens: 4000,
};

export async function getAIConfig(): Promise<AIConfig> {
  try {
    return await invoke<AIConfig>("get_ai_config");
  } catch (err) {
    console.warn("Failed to load AI config, using defaults:", err);
    return DEFAULT_AI_CONFIG;
  }
}

export async function saveAIConfig(config: AIConfig): Promise<void> {
  await invoke("save_ai_config", { config });
}
```

- [ ] **Step 2: 创建AI配置标签页组件**

```typescript
// src/components/AIConfigTab.tsx
import { useState, useEffect } from "react";
import { getAIConfig, saveAIConfig, type AIConfig } from "../ai-config";

interface AIConfigTabProps {
  onConfigSaved?: () => void;
}

function AIConfigTab({ onConfigSaved }: AIConfigTabProps) {
  const [config, setConfig] = useState<AIConfig>({
    api_endpoint: "",
    api_key: "",
    model_name: "",
    temperature: 0.7,
    max_tokens: 4000,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const loaded = await getAIConfig();
        setConfig(loaded);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    };
    loadConfig();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await saveAIConfig(config);
      onConfigSaved?.();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="loading">加载中...</div>;
  }

  return (
    <div className="ai-config-tab">
      <div className="setting-group">
        <label className="setting-label">API 端点</label>
        <input
          type="text"
          className="setting-input"
          value={config.api_endpoint}
          onChange={(e) => setConfig({ ...config, api_endpoint: e.target.value })}
          placeholder="https://api.openai.com/v1/chat/completions"
        />
        <span className="setting-hint">
          请输入完整的API端点URL，包括路径（如 /v1/chat/completions）
        </span>
      </div>

      <div className="setting-group">
        <label className="setting-label">API 密钥</label>
        <input
          type="password"
          className="setting-input"
          value={config.api_key}
          onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
          placeholder="sk-..."
        />
      </div>

      <div className="setting-group">
        <label className="setting-label">模型名称</label>
        <input
          type="text"
          className="setting-input"
          value={config.model_name}
          onChange={(e) => setConfig({ ...config, model_name: e.target.value })}
          placeholder="gpt-4"
        />
      </div>

      <div className="setting-group">
        <label className="setting-label">温度 (0-2)</label>
        <input
          type="number"
          className="setting-input"
          value={config.temperature}
          onChange={(e) => setConfig({ ...config, temperature: parseFloat(e.target.value) || 0.7 })}
          min="0"
          max="2"
          step="0.1"
        />
      </div>

      <div className="setting-group">
        <label className="setting-label">最大 Token 数</label>
        <input
          type="number"
          className="setting-input"
          value={config.max_tokens}
          onChange={(e) => setConfig({ ...config, max_tokens: parseInt(e.target.value) || 4000 })}
          min="100"
          max="128000"
        />
      </div>

      {error && <div className="error-message">{error}</div>}

      <button
        className="modal-btn modal-btn-save"
        onClick={handleSave}
        disabled={saving}
      >
        {saving ? "保存中..." : "保存AI配置"}
      </button>
    </div>
  );
}

export default AIConfigTab;
```

- [ ] **Step 3: 测试AI配置组件**

运行开发服务器，测试AI配置的加载和保存功能。

- [ ] **Step 4: 提交代码**

```bash
git add src/ai-config.ts src/components/AIConfigTab.tsx
git commit -m "feat: add AI config management frontend"
```

---

### Task 5: 前端提示词模板管理

**Covers:** S2

**Files:**
- Create: `src/prompt-template.ts`
- Create: `src/components/PromptTemplateTab.tsx`

**Interfaces:**
- Produces: `getPromptTemplate()`, `savePromptTemplate()`, `PromptTemplateTab` component

- [ ] **Step 1: 创建提示词模板管理模块**

```typescript
// src/prompt-template.ts
import { invoke } from "@tauri-apps/api/core";

export const DEFAULT_TEMPLATE = "关注当<周期>的重要动态,筛选 20-30 条有价值的信息，生成<报告名称>的<周期>报特别详细说明事件内容及值得关注的原因，生成的新闻html格式+浅色模式，放到<工作目录>文件夹中，文件名格式：<文件名格式>";

export async function getPromptTemplate(): Promise<string> {
  try {
    return await invoke<string>("get_prompt_template");
  } catch (err) {
    console.warn("Failed to load prompt template, using default:", err);
    return DEFAULT_TEMPLATE;
  }
}

export async function savePromptTemplate(template: string): Promise<void> {
  await invoke("save_prompt_template", { template });
}
```

- [ ] **Step 2: 创建提示词模板标签页组件**

```typescript
// src/components/PromptTemplateTab.tsx
import { useState, useEffect } from "react";
import { getPromptTemplate, savePromptTemplate, DEFAULT_TEMPLATE } from "../prompt-template";

interface PromptTemplateTabProps {
  onTemplateSaved?: () => void;
}

function PromptTemplateTab({ onTemplateSaved }: PromptTemplateTabProps) {
  const [template, setTemplate] = useState(DEFAULT_TEMPLATE);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadTemplate = async () => {
      try {
        const loaded = await getPromptTemplate();
        setTemplate(loaded);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    };
    loadTemplate();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await savePromptTemplate(template);
      onTemplateSaved?.();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setTemplate(DEFAULT_TEMPLATE);
  };

  if (loading) {
    return <div className="loading">加载中...</div>;
  }

  return (
    <div className="prompt-template-tab">
      <div className="setting-group">
        <label className="setting-label">提示词模板</label>
        <textarea
          className="setting-input setting-textarea"
          value={template}
          onChange={(e) => setTemplate(e.target.value)}
          rows={10}
          placeholder="输入提示词模板..."
        />
        <span className="setting-hint">
          可用占位符: {"<周期>"}, {"<报告名称>"}, {"<工作目录>"}, {"<文件名格式>"}
        </span>
      </div>

      {error && <div className="error-message">{error}</div>}

      <div className="button-group">
        <button
          className="modal-btn modal-btn-reset"
          onClick={handleReset}
        >
          恢复默认
        </button>
        <button
          className="modal-btn modal-btn-save"
          onClick={handleSave}
          disabled={saving}
        >
          {saving ? "保存中..." : "保存模板"}
        </button>
      </div>
    </div>
  );
}

export default PromptTemplateTab;
```

- [ ] **Step 3: 测试提示词模板组件**

运行开发服务器，测试提示词模板的加载和保存功能。

- [ ] **Step 4: 提交代码**

```bash
git add src/prompt-template.ts src/components/PromptTemplateTab.tsx
git commit -m "feat: add prompt template management frontend"
```

---

### Task 6: 前端设置弹窗集成

**Covers:** S1, S2

**Files:**
- Modify: `src/components/SettingsModal.tsx`
- Modify: `src/components/SettingsModal.css`

**Interfaces:**
- Consumes: `AIConfigTab`, `PromptTemplateTab`

- [ ] **Step 1: 修改设置弹窗支持标签页**

```typescript
// src/components/SettingsModal.tsx 添加标签页支持
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, Period } from "../settings";
import AIConfigTab from "./AIConfigTab";
import PromptTemplateTab from "./PromptTemplateTab";
import "./SettingsModal.css";

interface SettingsModalProps {
  isOpen: boolean;
  settings: AppSettings;
  onSave: (settings: Partial<AppSettings>) => Promise<void>;
  onClose: () => void;
}

type TabKey = "basic" | "ai" | "prompt";

function SettingsModal({ isOpen, settings, onSave, onClose }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState<TabKey>("basic");
  const [appName, setAppName] = useState(settings.app_name);
  const [period, setPeriod] = useState<Period>(settings.period);
  const [workDir, setWorkDir] = useState(settings.work_dir);
  const [filePattern, setFilePattern] = useState(settings.file_pattern);
  const [theme, setTheme] = useState<"dark" | "light">(settings.theme);
  const [saving, setSaving] = useState(false);

  // Reset form when modal opens
  useEffect(() => {
    if (isOpen) {
      setAppName(settings.app_name);
      setPeriod(settings.period);
      setWorkDir(settings.work_dir);
      setFilePattern(settings.file_pattern);
      setTheme(settings.theme);
      setActiveTab("basic");
    }
  }, [isOpen, settings]);

  if (!isOpen) return null;

  const handlePickDir = async () => {
    try {
      const result = await invoke<string | null>("pick_work_dir");
      if (result) {
        setWorkDir(result);
        // Auto-detect file naming pattern
        try {
          const detected = await invoke<string | null>("detect_pattern", {
            workDir: result,
          });
          if (detected) {
            setFilePattern(detected);
          }
        } catch (_) {
          // Detection failed, keep current pattern
        }
      }
    } catch (err) {
      console.warn("Failed to pick directory:", err);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await onSave({
        app_name: appName,
        period,
        work_dir: workDir,
        file_pattern: filePattern,
        theme,
      });
    } finally {
      setSaving(false);
    }
  };

  const periods: { key: Period; label: string }[] = [
    { key: "daily", label: "日报" },
    { key: "weekly", label: "周报" },
    { key: "monthly", label: "月报" },
  ];

  const tabs: { key: TabKey; label: string }[] = [
    { key: "basic", label: "基本设置" },
    { key: "ai", label: "AI配置" },
    { key: "prompt", label: "提示词模板" },
  ];

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-container" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>设置</h2>
          <button className="modal-close-btn" onClick={onClose}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div className="modal-tabs">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              className={`modal-tab ${activeTab === tab.key ? "active" : ""}`}
              onClick={() => setActiveTab(tab.key)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="modal-body">
          {activeTab === "basic" && (
            <>
              {/* App Name */}
              <div className="setting-group">
                <label className="setting-label">App 名称</label>
                <input
                  type="text"
                  className="setting-input"
                  value={appName}
                  onChange={(e) => setAppName(e.target.value)}
                  placeholder="通览"
                />
              </div>

              {/* Period */}
              <div className="setting-group">
                <label className="setting-label">报告周期</label>
                <div className="period-selector">
                  {periods.map((p) => (
                    <button
                      key={p.key}
                      className={`period-btn ${period === p.key ? "active" : ""}`}
                      onClick={() => setPeriod(p.key)}
                    >
                      {p.label}
                    </button>
                  ))}
                </div>
              </div>

              {/* Work Directory */}
              <div className="setting-group">
                <label className="setting-label">工作目录</label>
                <div className="dir-picker">
                  <input
                    type="text"
                    className="setting-input dir-input"
                    value={workDir}
                    onChange={(e) => setWorkDir(e.target.value)}
                    placeholder="选择包含报告的文件夹..."
                    readOnly
                  />
                  <button className="browse-btn" onClick={handlePickDir}>
                    浏览...
                  </button>
                </div>
                {workDir && (
                  <span className="setting-hint">{workDir}</span>
                )}
              </div>

              {/* File Pattern */}
              <div className="setting-group">
                <label className="setting-label">文件命名格式</label>
                <input
                  type="text"
                  className="setting-input"
                  value={filePattern}
                  onChange={(e) => setFilePattern(e.target.value)}
                  placeholder="{name}_{YYYY}{MM}{DD}.html"
                />
                <span className="setting-hint">
                  可用占位符: {"{name}"}, {"{YYYY}"}, {"{YY}"}, {"{MM}"}, {"{DD}"}
                </span>
              </div>

              {/* Theme */}
              <div className="setting-group">
                <label className="setting-label">界面主题</label>
                <div className="theme-switch-row">
                  <button
                    className={`theme-option ${theme === "dark" ? "active" : ""}`}
                    onClick={() => setTheme("dark")}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="theme-option-icon">
                      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                    </svg>
                    <span>暗色</span>
                  </button>
                  <button
                    className={`theme-option ${theme === "light" ? "active" : ""}`}
                    onClick={() => setTheme("light")}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="theme-option-icon">
                      <circle cx="12" cy="12" r="5" />
                      <line x1="12" y1="1" x2="12" y2="3" />
                      <line x1="12" y1="21" x2="12" y2="23" />
                      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
                      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
                      <line x1="1" y1="12" x2="3" y2="12" />
                      <line x1="21" y1="12" x2="23" y2="12" />
                      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
                      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
                    </svg>
                    <span>亮色</span>
                  </button>
                </div>
              </div>
            </>
          )}

          {activeTab === "ai" && (
            <AIConfigTab />
          )}

          {activeTab === "prompt" && (
            <PromptTemplateTab />
          )}
        </div>

        {activeTab === "basic" && (
          <div className="modal-footer">
            <button className="modal-btn modal-btn-cancel" onClick={onClose}>
              取消
            </button>
            <button
              className="modal-btn modal-btn-save"
              onClick={handleSave}
              disabled={saving}
            >
              {saving ? "保存中..." : "保存设置"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default SettingsModal;
```

- [ ] **Step 2: 添加标签页样式**

```css
/* src/components/SettingsModal.css 添加标签页样式 */
.modal-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  padding: 0 20px;
}

.modal-tab {
  padding: 10px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 14px;
  transition: all 0.2s;
}

.modal-tab:hover {
  color: var(--text-primary);
}

.modal-tab.active {
  color: var(--primary-color);
  border-bottom-color: var(--primary-color);
}

.ai-config-tab,
.prompt-template-tab {
  padding: 20px 0;
}

.setting-textarea {
  min-height: 120px;
  resize: vertical;
  font-family: inherit;
}

.button-group {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 20px;
}

.modal-btn-reset {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.modal-btn-reset:hover {
  background: var(--bg-hover);
}

.error-message {
  color: var(--error-color);
  background: var(--error-bg);
  padding: 10px;
  border-radius: 6px;
  margin: 10px 0;
  font-size: 14px;
}
```

- [ ] **Step 3: 测试设置弹窗**

运行开发服务器，测试设置弹窗的标签页切换和配置保存功能。

- [ ] **Step 4: 提交代码**

```bash
git add src/components/SettingsModal.tsx src/components/SettingsModal.css
git commit -m "feat: integrate AI config and prompt template into settings modal"
```

---

### Task 7: 前端生成功能集成

**Covers:** S3, S4

**Files:**
- Create: `src/components/GenerateConfirmDialog.tsx`
- Create: `src/components/GenerateStatus.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**Interfaces:**
- Consumes: `getPromptTemplate()`, `generate_newspaper()`
- Produces: `GenerateConfirmDialog`, `GenerateStatus` components

- [ ] **Step 1: 创建生成确认对话框组件**

```typescript
// src/components/GenerateConfirmDialog.tsx
import { useState, useEffect } from "react";
import { getPromptTemplate } from "../prompt-template";
import { useSettings } from "../settings";

interface GenerateConfirmDialogProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

function GenerateConfirmDialog({ isOpen, onConfirm, onCancel }: GenerateConfirmDialogProps) {
  const { settings } = useSettings();
  const [template, setTemplate] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (isOpen) {
      const loadTemplate = async () => {
        try {
          const loaded = await getPromptTemplate();
          // 替换占位符预览
          const periodLabel = settings.period === "daily" ? "日" : 
                            settings.period === "weekly" ? "周" : "月";
          const preview = loaded
            .replace("<周期>", periodLabel)
            .replace("<报告名称>", settings.app_name)
            .replace("<工作目录>", settings.work_dir || "（未配置）")
            .replace("<文件名格式>", settings.file_pattern);
          setTemplate(preview);
        } catch (err) {
          setTemplate("加载模板失败");
        } finally {
          setLoading(false);
        }
      };
      loadTemplate();
    }
  }, [isOpen, settings]);

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal-container generate-confirm-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>确认生成报刊</h2>
          <button className="modal-close-btn" onClick={onCancel}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div className="modal-body">
          <div className="prompt-preview">
            <label className="setting-label">提示词预览</label>
            {loading ? (
              <div className="loading">加载中...</div>
            ) : (
              <pre className="prompt-content">{template}</pre>
            )}
          </div>
          
          <div className="generate-info">
            <p>点击"开始生成"将调用AI接口生成报刊内容。</p>
            <p>生成的文件将保存到: <strong>{settings.work_dir || "（未配置工作目录）"}</strong></p>
          </div>
        </div>

        <div className="modal-footer">
          <button className="modal-btn modal-btn-cancel" onClick={onCancel}>
            取消
          </button>
          <button 
            className="modal-btn modal-btn-save" 
            onClick={onConfirm}
            disabled={!settings.work_dir}
          >
            开始生成
          </button>
        </div>
      </div>
    </div>
  );
}

export default GenerateConfirmDialog;
```

- [ ] **Step 2: 创建生成状态组件**

```typescript
// src/components/GenerateStatus.tsx
import { useState, useEffect } from "react";

interface GenerateStatusProps {
  status: "idle" | "generating" | "success" | "error";
  message: string;
  onClose: () => void;
}

function GenerateStatus({ status, message, onClose }: GenerateStatusProps) {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    if (status === "success" || status === "error") {
      const timer = setTimeout(() => {
        setVisible(false);
        onClose();
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [status, onClose]);

  if (!visible || status === "idle") return null;

  const getStatusIcon = () => {
    switch (status) {
      case "generating":
        return <div className="loading-spinner small" />;
      case "success":
        return <span className="status-icon success">✓</span>;
      case "error":
        return <span className="status-icon error">✕</span>;
      default:
        return null;
    }
  };

  const getStatusClass = () => {
    switch (status) {
      case "generating":
        return "status-generating";
      case "success":
        return "status-success";
      case "error":
        return "status-error";
      default:
        return "";
    }
  };

  return (
    <div className={`generate-status ${getStatusClass()}`}>
      <div className="status-content">
        {getStatusIcon()}
        <span className="status-message">{message}</span>
      </div>
      {(status === "success" || status === "error") && (
        <button className="status-close" onClick={() => {
          setVisible(false);
          onClose();
        }}>
          ✕
        </button>
      )}
    </div>
  );
}

export default GenerateStatus;
```

- [ ] **Step 3: 修改App.tsx添加生成功能**

```typescript
// src/App.tsx 添加生成功能
import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SettingsProvider, useSettings, periodLabel, type AppSettings } from "./settings";
import Sidebar from "./components/Sidebar";
import StatsBar from "./components/StatsBar";
import ReportCard from "./components/ReportCard";
import ReportViewer from "./components/ReportViewer";
import ThemeToggle from "./components/ThemeToggle";
import SettingsModal from "./components/SettingsModal";
import GenerateConfirmDialog from "./components/GenerateConfirmDialog";
import GenerateStatus from "./components/GenerateStatus";
import "./App.css";

export interface ReportInfo {
  filename: string;
  path: string;
  date: string;
  year: number;
  month: number;
  day: number;
  size: number;
  size_display: string;
}

export interface ReportsData {
  reports: ReportInfo[];
  total_count: number;
  total_size: number;
  total_size_display: string;
  years: number[];
  months: [string, number][];
  earliest_date: string;
  latest_date: string;
}

function AppContent() {
  const { settings, updateSettings, loading: settingsLoading } = useSettings();
  const [data, setData] = useState<ReportsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedYear, setSelectedYear] = useState<number | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<[string, number] | null>(null);
  const [viewingReport, setViewingReport] = useState<ReportInfo | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showGenerateConfirm, setShowGenerateConfirm] = useState(false);
  const [generateStatus, setGenerateStatus] = useState<"idle" | "generating" | "success" | "error">("idle");
  const [generateMessage, setGenerateMessage] = useState("");

  const plabel = periodLabel(settings.period);

  const loadData = useCallback(async () => {
    if (!settings.work_dir) {
      setLoading(false);
      setError("请先设置工作目录");
      return;
    }
    try {
      setLoading(true);
      const result = await invoke<ReportsData>("scan_reports", {
        workDir: settings.work_dir,
        filePattern: settings.file_pattern,
        appName: settings.app_name,
      });
      setData(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [settings.work_dir, settings.file_pattern, settings.app_name]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Update window title (HTML + Tauri native)
  useEffect(() => {
    document.title = settings.app_name;
    try {
      getCurrentWindow().setTitle(settings.app_name);
    } catch (_) {
      // Not running in Tauri (dev in browser)
    }
  }, [settings.app_name]);

  const filteredReports = useMemo(() => {
    if (!data) return [];
    let reports = data.reports;

    if (selectedYear) {
      reports = reports.filter((r) => r.year === selectedYear);
    }
    if (selectedMonth) {
      reports = reports.filter(
        (r) => r.year === selectedYear && r.month === selectedMonth[1]
      );
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      reports = reports.filter(
        (r) =>
          r.filename.toLowerCase().includes(q) ||
          r.date.includes(q)
      );
    }

    return reports;
  }, [data, selectedYear, selectedMonth, searchQuery]);

  const handleGenerate = async () => {
    setShowGenerateConfirm(false);
    setGenerateStatus("generating");
    setGenerateMessage("正在准备生成...");

    try {
      // 模拟进度更新
      setTimeout(() => setGenerateMessage("正在调用AI API..."), 1000);
      setTimeout(() => setGenerateMessage("正在生成HTML内容..."), 3000);
      setTimeout(() => setGenerateMessage("正在保存文件..."), 5000);

      const result = await invoke<string>("generate_newspaper");
      setGenerateStatus("success");
      setGenerateMessage(result);
      
      // 刷新报告列表
      await loadData();
    } catch (err) {
      setGenerateStatus("error");
      setGenerateMessage(String(err));
    }
  };

  const handleGenerateClose = () => {
    setGenerateStatus("idle");
    setGenerateMessage("");
  };

  // First time setup — no work dir configured
  if (!settingsLoading && !settings.work_dir) {
    return (
      <div className="onboarding-screen">
        <div className="onboarding-card">
          <div className="onboarding-icon">📰</div>
          <h1>欢迎使用通览</h1>
          <p>请先设置工作目录和偏好，开始浏览您的报告</p>
          <button
            className="onboarding-btn"
            onClick={() => setShowSettings(true)}
          >
            开始设置
          </button>
          <SettingsModal
            isOpen={showSettings}
            settings={settings}
            onSave={async (s) => {
              await updateSettings(s);
              setShowSettings(false);
            }}
            onClose={() => setShowSettings(false)}
          />
        </div>
      </div>
    );
  }

  if (settingsLoading || loading) {
    return (
      <div className="loading-screen">
        <div className="loading-spinner" />
        <p>正在加载{plabel}数据...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="error-screen">
        <div className="error-icon">⚠</div>
        <h2>加载失败</h2>
        <p>{error}</p>
        <button onClick={loadData} className="retry-btn">
          重试
        </button>
        <button
          onClick={() => setShowSettings(true)}
          className="retry-btn"
          style={{ marginLeft: 8 }}
        >
          修改设置
        </button>
        <SettingsModal
          isOpen={showSettings}
          settings={settings}
          onSave={async (s) => {
            await updateSettings(s);
            setShowSettings(false);
          }}
          onClose={() => setShowSettings(false)}
        />
      </div>
    );
  }

  return (
    <div className="app">
      <Sidebar
        data={data}
        selectedYear={selectedYear}
        selectedMonth={selectedMonth}
        onSelectYear={setSelectedYear}
        onSelectMonth={setSelectedMonth}
        period={settings.period}
      />
      <main className="main-content">
        <header className="main-header">
          <div className="header-top">
            <h1 className="app-title">{settings.app_name}</h1>
            <div className="header-actions">
              <button
                className="generate-btn"
                onClick={() => setShowGenerateConfirm(true)}
                disabled={generateStatus === "generating"}
              >
                {generateStatus === "generating" ? (
                  <>
                    <div className="loading-spinner small" />
                    生成中...
                  </>
                ) : (
                  <>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="generate-icon">
                      <path d="M12 2L2 7l10 5 10-5-10-5z" />
                      <path d="M2 17l10 5 10-5" />
                      <path d="M2 12l10 5 10-5" />
                    </svg>
                    生成报刊
                  </>
                )}
              </button>
              <ThemeToggle
                theme={settings.theme}
                onToggle={(theme) => updateSettings({ theme })}
              />
              <button
                className="settings-btn"
                onClick={() => setShowSettings(true)}
                title="设置"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="settings-icon">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
                </svg>
              </button>
              <div className="search-box">
                <svg className="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="11" cy="11" r="8" />
                  <path d="M21 21l-4.35-4.35" />
                </svg>
                <input
                  type="text"
                  placeholder={`搜索${plabel}...`}
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="search-input"
                />
                {searchQuery && (
                  <button
                    className="search-clear"
                    onClick={() => setSearchQuery("")}
                  >
                    ✕
                  </button>
                )}
              </div>
            </div>
          </div>
          {data && <StatsBar data={data} period={settings.period} />}
        </header>
        <section className="reports-section">
          <div className="section-header">
            <h2>
              {selectedMonth
                ? `${selectedMonth[0]}`
                : selectedYear
                ? `${selectedYear}年`
                : `全部${plabel}`}
            </h2>
            <span className="section-count">
              {filteredReports.length} 篇
            </span>
          </div>
          {filteredReports.length === 0 ? (
            <div className="empty-state">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="empty-icon">
                <path d="M13.5 3H12H8C6.34315 3 5 4.34315 5 6V18C5 19.6569 6.34315 21 8 21H11M13.5 3L19 8.625M13.5 3V7.625C13.5 8.17728 13.9477 8.625 14.5 8.625H19M19 8.625V11" />
                <path d="M15 18C15 18 16 17 18 19C18 19 22 15 22 15" />
              </svg>
              {searchQuery ? (
                <p>没有找到匹配 "{searchQuery}" 的{plabel}</p>
              ) : (
                <>
                  <p>该分类下还没有{plabel}</p>
                  <div className="empty-debug">
                    <span>📁 目录：{settings.work_dir}</span>
                    <span>📄 格式：{settings.file_pattern}</span>
                    <span>🏷 App名：{settings.app_name}</span>
                  </div>
                </>
              )}
            </div>
          ) : (
            <div className="reports-grid">
              {filteredReports.map((report) => (
                <ReportCard
                  key={report.path}
                  report={report}
                  onClick={() => setViewingReport(report)}
                  appName={settings.app_name}
                />
              ))}
            </div>
          )}
        </section>
      </main>
      {viewingReport && (
        <ReportViewer
          report={viewingReport}
          onClose={() => setViewingReport(null)}
          appName={settings.app_name}
        />
      )}
      <SettingsModal
        isOpen={showSettings}
        settings={settings}
        onSave={async (s) => {
          await updateSettings(s);
          setShowSettings(false);
        }}
        onClose={() => setShowSettings(false)}
      />
      <GenerateConfirmDialog
        isOpen={showGenerateConfirm}
        onConfirm={handleGenerate}
        onCancel={() => setShowGenerateConfirm(false)}
      />
      <GenerateStatus
        status={generateStatus}
        message={generateMessage}
        onClose={handleGenerateClose}
      />
    </div>
  );
}

function App() {
  return (
    <SettingsProvider>
      <AppContent />
    </SettingsProvider>
  );
}

export default App;
```

- [ ] **Step 4: 添加生成按钮和状态样式**

```css
/* src/App.css 添加生成按钮和状态样式 */
.generate-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.generate-btn:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: translateY(-1px);
}

.generate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.generate-icon {
  width: 18px;
  height: 18px;
}

.generate-status {
  position: fixed;
  bottom: 20px;
  right: 20px;
  padding: 12px 20px;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 300px;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.status-generating {
  border-left: 4px solid var(--primary-color);
}

.status-success {
  border-left: 4px solid var(--success-color);
}

.status-error {
  border-left: 4px solid var(--error-color);
}

.status-content {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
}

.status-icon {
  font-size: 18px;
  font-weight: bold;
}

.status-icon.success {
  color: var(--success-color);
}

.status-icon.error {
  color: var(--error-color);
}

.status-message {
  font-size: 14px;
  color: var(--text-primary);
}

.status-close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 16px;
  padding: 4px;
}

.status-close:hover {
  color: var(--text-primary);
}

.generate-confirm-dialog {
  max-width: 500px;
}

.prompt-preview {
  margin-bottom: 20px;
}

.prompt-content {
  background: var(--bg-secondary);
  padding: 15px;
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 200px;
  overflow-y: auto;
}

.generate-info {
  background: var(--bg-secondary);
  padding: 15px;
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.6;
}

.generate-info p {
  margin: 5px 0;
}

.generate-info strong {
  color: var(--primary-color);
}
```

- [ ] **Step 5: 测试生成功能**

运行开发服务器，测试生成确认对话框和状态显示功能。

- [ ] **Step 6: 提交代码**

```bash
git add src/components/GenerateConfirmDialog.tsx src/components/GenerateStatus.tsx src/App.tsx src/App.css
git commit -m "feat: add AI newspaper generation frontend"
```

---

### Task 8: 集成测试和验证

**Covers:** S1, S2, S3, S4

**Files:**
- Test: 所有前端和后端文件

**Interfaces:**
- 验证所有功能正常工作

- [ ] **Step 1: 运行后端测试**

```bash
cd src-tauri
cargo test
```

- [ ] **Step 2: 运行前端开发服务器**

```bash
npm run dev
```

- [ ] **Step 3: 测试完整流程**

1. 打开设置弹窗，配置AI参数
2. 配置提示词模板
3. 点击"生成报刊"按钮
4. 确认生成
5. 查看生成状态
6. 验证文件是否生成成功
7. 验证报告列表是否自动刷新

- [ ] **Step 4: 测试错误场景**

1. 不配置AI参数，尝试生成
2. 配置错误的API端点
3. 配置错误的API密钥
4. 工作目录不存在

- [ ] **Step 5: 提交最终代码**

```bash
git add .
git commit -m "feat: complete AI newspaper generation feature"
```

---

## 执行建议

这个计划包含8个任务，建议使用compose:subagent执行，每个任务一个子代理。任务之间有依赖关系，需要按顺序执行。

**执行顺序：**
1. Task 1: 后端AI配置管理
2. Task 2: 后端提示词模板管理
3. Task 3: 后端AI生成功能
4. Task 4: 前端AI配置管理
5. Task 5: 前端提示词模板管理
6. Task 6: 前端设置弹窗集成
7. Task 7: 前端生成功能集成
8. Task 8: 集成测试和验证

**关键依赖：**
- Task 1-3 是后端任务，可以并行执行
- Task 4-5 是前端任务，可以并行执行
- Task 6 依赖 Task 4 和 Task 5
- Task 7 依赖 Task 6
- Task 8 依赖所有其他任务

**风险点：**
- AI API调用可能需要处理各种错误情况
- 文件生成可能需要处理权限问题
- 前端状态管理可能比较复杂

**建议：**
- 每个任务完成后进行测试
- 及时处理错误情况
- 保持代码简洁和可维护性
