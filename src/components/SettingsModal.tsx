import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, Period } from "../settings";
import "./SettingsModal.css";

interface SettingsModalProps {
  isOpen: boolean;
  settings: AppSettings;
  onSave: (settings: Partial<AppSettings>) => Promise<void>;
  onClose: () => void;
}

function SettingsModal({ isOpen, settings, onSave, onClose }: SettingsModalProps) {
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

        <div className="modal-body">
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
        </div>

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
      </div>
    </div>
  );
}

export default SettingsModal;
