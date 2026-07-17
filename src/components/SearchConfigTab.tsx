import { useState, useEffect } from "react";
import {
  getSearchConfig,
  saveSearchConfig,
  type SearchConfig,
  type SearchProvider,
} from "../search-config";

interface SearchConfigTabProps {
  onConfigSaved?: () => void;
}

function SearchConfigTab({ onConfigSaved }: SearchConfigTabProps) {
  const [config, setConfig] = useState<SearchConfig>({
    provider: "tavily",
    api_key: "",
    api_endpoint: "",
    max_results: 10,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testQuery, setTestQuery] = useState("");
  const [testResult, setTestResult] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const loaded = await getSearchConfig();
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
      await saveSearchConfig(config);
      onConfigSaved?.();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    if (!testQuery.trim()) return;
    setTesting(true);
    setTestResult(null);
    try {
      const { searchWeb } = await import("../search-config");
      const results = await searchWeb(testQuery);
      setTestResult(`找到 ${results.length} 条结果:\n${results.map((r, i) => `${i + 1}. ${r.title}\n   ${r.url}\n   ${r.content.substring(0, 100)}...`).join("\n\n")}`);
    } catch (err) {
      setTestResult(`测试失败: ${err}`);
    } finally {
      setTesting(false);
    }
  };

  if (loading) {
    return <div className="loading">加载中...</div>;
  }

  const providers: { value: SearchProvider; label: string }[] = [
    { value: "tavily", label: "Tavily (推荐)" },
    { value: "custom", label: "自定义 API" },
  ];

  return (
    <div className="search-config-tab">
      <div className="setting-group">
        <label className="setting-label">搜索提供商</label>
        <select
          className="setting-input"
          value={config.provider}
          onChange={(e) =>
            setConfig({ ...config, provider: e.target.value as SearchProvider })
          }
        >
          {providers.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
        <span className="setting-hint">
          {config.provider === "tavily" && "Tavily 是专为 AI 设计的搜索 API，推荐使用"}
          {config.provider === "custom" && "自定义搜索 API，需要提供 API 地址和密钥"}
        </span>
      </div>

      <div className="setting-group">
        <label className="setting-label">API 密钥</label>
        <input
          type="password"
          className="setting-input"
          value={config.api_key}
          onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
          placeholder={config.provider === "tavily" ? "tvly-xxx" : "输入 API 密钥"}
        />
        <span className="setting-hint">
          {config.provider === "tavily" && (
            <>访问 <a href="https://tavily.com" target="_blank" rel="noopener noreferrer">tavily.com</a> 获取免费 API 密钥</>
          )}
        </span>
      </div>

      {config.provider === "custom" && (
        <div className="setting-group">
          <label className="setting-label">API 地址</label>
          <input
            type="text"
            className="setting-input"
            value={config.api_endpoint}
            onChange={(e) => setConfig({ ...config, api_endpoint: e.target.value })}
            placeholder="https://api.example.com/search"
          />
          <span className="setting-hint">
            自定义搜索 API 的地址，支持 GET 请求，返回 JSON 格式结果
          </span>
        </div>
      )}

      <div className="setting-group">
        <label className="setting-label">最大结果数</label>
        <input
          type="number"
          className="setting-input"
          value={config.max_results}
          onChange={(e) =>
            setConfig({
              ...config,
              max_results: parseInt(e.target.value) || 10,
            })
          }
          min="1"
          max="20"
        />
      </div>

      <div className="setting-group">
        <label className="setting-label">测试搜索</label>
        <div style={{ display: "flex", gap: "8px" }}>
          <input
            type="text"
            className="setting-input"
            value={testQuery}
            onChange={(e) => setTestQuery(e.target.value)}
            placeholder="输入测试查询..."
            style={{ flex: 1 }}
          />
          <button
            className="modal-btn modal-btn-save"
            onClick={handleTest}
            disabled={testing || !testQuery.trim()}
          >
            {testing ? "测试中..." : "测试"}
          </button>
        </div>
        {testResult && (
          <pre className="test-result" style={{ 
            marginTop: "8px", 
            padding: "8px", 
            background: "var(--bg-secondary)", 
            borderRadius: "4px",
            fontSize: "12px",
            whiteSpace: "pre-wrap",
            maxHeight: "200px",
            overflowY: "auto"
          }}>
            {testResult}
          </pre>
        )}
      </div>

      {error && <div className="error-message">{error}</div>}

      <button
        className="modal-btn modal-btn-save"
        onClick={handleSave}
        disabled={saving}
        style={{ marginTop: "16px" }}
      >
        {saving ? "保存中..." : "保存搜索配置"}
      </button>
    </div>
  );
}

export default SearchConfigTab;
