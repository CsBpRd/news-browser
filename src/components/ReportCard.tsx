import type { ReportInfo } from "../App";
import "./ReportCard.css";

interface ReportCardProps {
  report: ReportInfo;
  onClick: () => void;
  appName: string;
}

function ReportCard({ report, onClick, appName }: ReportCardProps) {
  const weekdayNames = ["日", "一", "二", "三", "四", "五", "六"];

  const dateObj = new Date(report.year, report.month - 1, report.day);
  const weekday = weekdayNames[dateObj.getDay()];

  const monthNames = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
  ];

  // Detect file type for badge
  const ext = report.filename.split(".").pop()?.toLowerCase() || "";
  const typeLabel =
    ext === "md" ? "MD" :
    ext === "pdf" ? "PDF" :
    ext === "txt" ? "TXT" : "HTML";

  return (
    <div className="report-card" onClick={onClick}>
      <div className="card-date-badge">
        <span className="date-day">{report.day}</span>
        <span className="date-month">{monthNames[report.month - 1]}</span>
      </div>
      <div className="card-content">
        <h3 className="card-title">{appName}</h3>
        <div className="card-meta">
          <span className="card-date">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="meta-icon">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
              <line x1="16" y1="2" x2="16" y2="6" />
              <line x1="8" y1="2" x2="8" y2="6" />
              <line x1="3" y1="10" x2="21" y2="10" />
            </svg>
            {report.date} · 周{weekday}
          </span>
          <span className="card-size">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="meta-icon">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            {report.size_display}
          </span>
          <span className={`card-type-badge type-${ext}`}>
            {typeLabel}
          </span>
        </div>
      </div>
      <div className="card-arrow">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M9 18l6-6-6-6" />
        </svg>
      </div>
    </div>
  );
}

export default ReportCard;
