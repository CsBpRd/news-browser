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
