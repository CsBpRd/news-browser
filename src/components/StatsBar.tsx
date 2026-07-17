import type { ReportsData } from "../App";
import { periodLabel, type Period } from "../settings";
import "./StatsBar.css";

interface StatsBarProps {
  data: ReportsData;
  period: Period;
}

function StatsBar({ data, period }: StatsBarProps) {
  const plabel = periodLabel(period);

  const yearRange =
    data.earliest_date && data.latest_date
      ? `${data.earliest_date.slice(0, 4)} - ${data.latest_date.slice(0, 4)}`
      : "-";

  const stats = [
    {
      label: `${plabel}总数`,
      value: String(data.total_count),
      icon: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
          <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
        </svg>
      ),
    },
    {
      label: "年份跨度",
      value: yearRange,
      icon: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="10" />
          <path d="M12 6v6l4 2" />
        </svg>
      ),
    },
    {
      label: "总大小",
      value: data.total_size_display,
      icon: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
        </svg>
      ),
    },
    {
      label: "最新一期",
      value: data.latest_date || "-",
      icon: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
          <line x1="16" y1="2" x2="16" y2="6" />
          <line x1="8" y1="2" x2="8" y2="6" />
          <line x1="3" y1="10" x2="21" y2="10" />
        </svg>
      ),
    },
  ];

  return (
    <div className="stats-bar">
      {stats.map((stat) => (
        <div key={stat.label} className="stat-card">
          <div className="stat-icon-wrapper">
            <div className="stat-icon">{stat.icon}</div>
          </div>
          <div className="stat-info">
            <span className="stat-label">{stat.label}</span>
            <span className="stat-value">{stat.value}</span>
          </div>
        </div>
      ))}
    </div>
  );
}

export default StatsBar;
