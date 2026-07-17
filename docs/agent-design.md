# Deep Search Agent — 架构设计

## 架构概览

基于 **Deep Search** 框架（参考 vanilla-research-agent 设计），构建一个深度研究型报刊生成 Agent。

核心流程：**生成大纲 → 逐段深度研究 → 反思迭代 → 格式化输出**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Deep Search Agent                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  用户输入: "生成今日科技新闻报刊"                                  │
│         ↓                                                        │
│  ┌───────────────────────────────────────────────────┐           │
│  │  1. ReportStructureNode                           │           │
│  │     → LLM 生成报告大纲（段落标题+内容描述）         │           │
│  └───────────────────────────────────────────────────┘           │
│         ↓                                                        │
│  ┌───────────────────────────────────────────────────┐           │
│  │  2. 遍历每个段落                                    │           │
│  │                                                     │           │
│  │  ┌──────────────────────┐                          │           │
│  │  │ FirstSearchNode      │ → 生成搜索查询            │           │
│  │  └──────────┬───────────┘                          │           │
│  │             ↓                                      │           │
│  │  ┌──────────────────────┐                          │           │
│  │  │ Tavily Search        │ → 执行网络搜索            │           │
│  │  └──────────┬───────────┘                          │           │
│  │             ↓                                      │           │
│  │  ┌──────────────────────┐                          │           │
│  │  │ FirstSummaryNode     │ → 生成段落初稿            │           │
│  │  └──────────┬───────────┘                          │           │
│  │             ↓                                      │           │
│  │  ┌──────────────────────────────────────────┐      │           │
│  │  │  Reflection Loop (×2)                    │      │           │
│  │  │  ┌────────────┐  ┌────────────┐          │      │           │
│  │  │  │ Reflect    │→ │ Search     │          │      │           │
│  │  │  └────────────┘  └─────┬──────┘          │      │           │
│  │  │                        ↓                 │      │           │
│  │  │  ┌──────────────────────┐                │      │           │
│  │  │  │ ReflectionSummary    │ → 更新段落      │      │           │
│  │  │  └──────────────────────┘                │      │           │
│  │  └──────────────────────────────────────────┘      │           │
│  └───────────────────────────────────────────────────┘           │
│         ↓                                                        │
│  ┌───────────────────────────────────────────────────┐           │
│  │  3. ReportFormattingNode                          │           │
│  │     → LLM 格式化输出 (HTML + Markdown)             │           │
│  └───────────────────────────────────────────────────┘           │
│         ↓                                                        │
│  输出: HTML 报刊 + Markdown 报告，保存到工作目录                    │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. State（状态管理）
```
State
├── report_title: String
└── paragraphs: Vec<Paragraph>
      ├── title: String              # 段落标题
      ├── content: String            # 段落预期内容描述
      └── research: Research
            ├── search_history: Vec<SearchResult>  # 搜索记录
            ├── latest_summary: String             # 当前最新总结
            └── reflection_iteration: u32          # 反思次数
```

### 2. Nodes（处理节点）

| Node | 输入 | 输出 | 功能 |
|------|------|------|------|
| ReportStructureNode | 用户查询 | State（大纲） | 生成报告段落结构 |
| FirstSearchNode | 段落信息 | search_query + reasoning | 生成首次搜索关键词 |
| FirstSummaryNode | 搜索结果 | paragraph_latest_state | 从搜索结果写初稿 |
| ReflectionNode | 当前段落 | search_query + reasoning | 发现遗漏，生成新查询 |
| ReflectionSummaryNode | 新搜索结果 | updated_paragraph_latest_state | 丰富段落内容 |
| ReportFormattingNode | 所有段落 | HTML + Markdown | 格式化最终输出 |

### 3. Prompt 模板

每个节点都有专用的 System Prompt，包含 JSON Schema 定义输入输出格式：

- **SYSTEM_PROMPT_REPORT_STRUCTURE** — 大纲生成
- **SYSTEM_PROMPT_FIRST_SEARCH** — 首次搜索
- **SYSTEM_PROMPT_FIRST_SUMMARY** — 首次总结
- **SYSTEM_PROMPT_REFLECTION** — 反思搜索
- **SYSTEM_PROMPT_REFLECTION_SUMMARY** — 反思总结
- **SYSTEM_PROMPT_REPORT_FORMATTING_HTML** — HTML 格式化
- **SYSTEM_PROMPT_REPORT_FORMATTING_MD** — Markdown 格式化

### 4. Tools（工具）

- **Tavily Search** — 网络搜索 API
- **Custom Search** — 自定义搜索 API（兼容格式）

## 模块结构（Rust）

```
src-tauri/src/agent/
├── mod.rs          # 总入口 + Tauri command + 编排逻辑
├── state.rs        # 状态数据结构
├── prompts.rs      # System Prompt 常量
└── nodes.rs        # 节点实现（LLM调用、搜索、JSON解析）
```

## 配置项

| 配置 | 说明 | 存储位置 |
|------|------|----------|
| AI API | endpoint / key / model / temperature / max_tokens | ai_config.rs |
| 搜索API | provider（Tavily/Custom）/ key / endpoint | search_config.rs |
| 提示词模板 | 用户自定义模板，含 <周期> 等占位符 | prompt_template.rs |
| 应用设置 | 工作目录、文件名格式、App名称等 | settings.rs |

## 与旧架构的区别

| 维度 | 旧架构（ReAct） | 新架构（Deep Search） |
|------|----------------|----------------------|
| 流程 | 生成关键词 → 搜索 → 生成报告 | 生成大纲 → 逐段研究 × 反思迭代 → 格式化 |
| 搜索轮次 | 1 次批量搜索 | 每段 1 次初始搜索 + 2 次反思搜索 |
| 状态管理 | 无显式状态 | State → Paragraph → Research 逐级管理 |
| 输出格式 | 仅 HTML | HTML + Markdown |
| 迭代优化 | 无 | Reflection 循环自动补缺 |
| 实现语言 | Rust (ai_generator.rs) | Rust (agent/ 模块) |
