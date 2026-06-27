import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { marked } from "marked";
import type { ReportInfo } from "../App";
import "./ReportViewer.css";

interface ReportViewerProps {
  report: ReportInfo;
  onClose: () => void;
  appName: string;
}

function getFileType(filename: string): "html" | "md" | "txt" | "other" {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  if (ext === "html" || ext === "htm") return "html";
  if (ext === "md" || ext === "markdown") return "md";
  if (ext === "txt") return "txt";
  return "other";
}

function ReportViewer({ report, onClose, appName }: ReportViewerProps) {
  const [content, setContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [fileType, setFileType] = useState<ReturnType<typeof getFileType>>("html");

  useEffect(() => {
    const loadReport = async () => {
      try {
        setLoading(true);
        const type = getFileType(report.filename);
        setFileType(type);

        const text = await invoke<string>("read_report", {
          path: report.path,
        });
        if (type === "md") {
          const html = await marked(text);
          setContent(html);
        } else {
          setContent(text);
        }
        setError(null);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    };
    loadReport();
  }, [report.path, report.filename]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  return (
    <div className="viewer-overlay" onClick={onClose}>
      <div className="viewer-container" onClick={(e) => e.stopPropagation()}>
        <div className="viewer-header">
          <div className="viewer-title-group">
            <h2>{appName}</h2>
            <span className="viewer-date">{report.date}</span>
            <span className="viewer-type-badge">{report.filename.split(".").pop()?.toUpperCase()}</span>
          </div>
          <div className="viewer-actions">
            <button className="viewer-btn viewer-btn-close" onClick={onClose}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>
        <div className="viewer-body">
          {loading && (
            <div className="viewer-loading">
              <div className="loading-spinner" />
              <p>加载中...</p>
            </div>
          )}
          {error && (
            <div className="viewer-error">
              <p>加载失败: {error}</p>
            </div>
          )}
          {content && !loading && !error && (
            <>
              {fileType === "html" && (
                <iframe
                  srcDoc={content}
                  className="viewer-iframe"
                  sandbox="allow-same-origin"
                  title={report.filename}
                />
              )}
              {fileType === "md" && (
                <iframe
                  srcDoc={content}
                  className="viewer-iframe"
                  sandbox="allow-same-origin"
                  title={report.filename}
                />
              )}
              {fileType === "txt" && (
                <pre className="viewer-txt">
                  <code>{content}</code>
                </pre>
              )}
              {fileType === "other" && (
                <div className="viewer-unsupported">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="unsupported-icon">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                    <polyline points="14 2 14 8 20 8" />
                  </svg>
                  <p>不支持预览此文件格式</p>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}

export default ReportViewer;
