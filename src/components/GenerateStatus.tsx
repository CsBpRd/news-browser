import { useState, useEffect, useRef, useMemo } from "react";

export interface LogEntry {
  type: "thinking" | "tool_call" | "tool_result" | "search" | "fetch" | "output" | "error";
  content: string;
  timestamp: number;
}

interface GenerateStatusProps {
  status: "idle" | "generating" | "success" | "error";
  logs: LogEntry[];
  onClose: () => void;
}

function GenerateStatus({ status, logs, onClose }: GenerateStatusProps) {
  const [visible, setVisible] = useState(true);
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  useEffect(() => {
    if (status === "idle") {
      setVisible(true);
    }
  }, [status]);

  // 合并连续的 output 日志
  const mergedLogs = useMemo(() => {
    const result: { type: string; content: string }[] = [];
    let outputBuffer = "";
    
    for (const log of logs) {
      if (log.type === "output") {
        outputBuffer += log.content;
      } else {
        if (outputBuffer) {
          result.push({ type: "output", content: outputBuffer });
          outputBuffer = "";
        }
        result.push({ type: log.type, content: log.content });
      }
    }
    
    if (outputBuffer) {
      result.push({ type: "output", content: outputBuffer });
    }
    
    return result;
  }, [logs]);

  if (!visible || status === "idle") return null;

  const getLogIcon = (type: string) => {
    switch (type) {
      case "thinking": return "💭";
      case "tool_call": return "🔧";
      case "tool_result": return "📋";
      case "search": return "🔍";
      case "fetch": return "🌐";
      case "output": return "📄";
      case "error": return "❌";
      default: return "ℹ️";
    }
  };

  const getLogLabel = (type: string) => {
    switch (type) {
      case "thinking": return "思考";
      case "tool_call": return "工具";
      case "tool_result": return "结果";
      case "search": return "搜索";
      case "fetch": return "抓取";
      case "output": return "输出";
      case "error": return "错误";
      default: return "信息";
    }
  };

  return (
    <div className="generate-status-panel">
      <div className="status-panel-header">
        <div className="status-panel-title">
          {status === "generating" && <div className="loading-spinner small" />}
          <span>{status === "generating" ? "AI 正在生成中..." : status === "success" ? "生成完成" : "生成失败"}</span>
        </div>
        <button className="status-close" onClick={() => {
          setVisible(false);
          onClose();
        }}>
          &#10007;
        </button>
      </div>
      <div className="status-log-container">
        {mergedLogs.map((log, index) => (
          <div key={index} className={`log-entry log-${log.type}`}>
            <span className="log-icon">{getLogIcon(log.type)}</span>
            <span className="log-label">{getLogLabel(log.type)}</span>
            <span className="log-content">
              {log.type === "output" ? (
                <pre className="output-content">{log.content}</pre>
              ) : (
                log.content
              )}
            </span>
          </div>
        ))}
        {status === "generating" && (
          <div className="log-entry log-thinking">
            <span className="log-icon">💭</span>
            <span className="log-label">思考</span>
            <span className="log-content typing">等待 AI 响应...</span>
          </div>
        )}
        <div ref={logEndRef} />
      </div>
    </div>
  );
}

export default GenerateStatus;
