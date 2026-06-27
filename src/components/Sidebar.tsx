import type { ReportsData } from "../App";
import { periodLabel, type Period } from "../settings";
import "./Sidebar.css";

interface SidebarProps {
  data: ReportsData | null;
  selectedYear: number | null;
  selectedMonth: [string, number] | null;
  onSelectYear: (year: number | null) => void;
  onSelectMonth: (month: [string, number] | null) => void;
  period: Period;
}

const monthNames = [
  "一月", "二月", "三月", "四月", "五月", "六月",
  "七月", "八月", "九月", "十月", "十一月", "十二月",
];

function Sidebar({ data, selectedYear, selectedMonth, onSelectYear, onSelectMonth, period }: SidebarProps) {
  if (!data) return null;

  const plabel = periodLabel(period);

  const handleYearClick = (year: number) => {
    if (selectedYear === year && !selectedMonth) {
      onSelectYear(null);
      onSelectMonth(null);
    } else {
      onSelectYear(year);
      onSelectMonth(null);
    }
  };

  const handleMonthClick = (month: [string, number]) => {
    if (selectedMonth && selectedMonth[0] === month[0]) {
      onSelectMonth(null);
    } else {
      if (!selectedYear) {
        for (const y of data.years) {
          const hasMonth = data.reports.some(
            (r) => r.year === y && r.month === month[1]
          );
          if (hasMonth) {
            onSelectYear(y);
            break;
          }
        }
      }
      onSelectMonth(month);
    }
  };

  const isYearSelected = (year: number) => selectedYear === year && !selectedMonth;

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div className="sidebar-logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="logo-icon">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
          </svg>
          <span>{plabel}导航</span>
        </div>
      </div>

      <nav className="sidebar-nav">
        <button
          className={`nav-item all-btn ${!selectedYear && !selectedMonth ? "active" : ""}`}
          onClick={() => { onSelectYear(null); onSelectMonth(null); }}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="nav-icon">
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
          </svg>
          <span>全部{plabel}</span>
          <span className="nav-badge">{data.total_count}</span>
        </button>

        <div className="nav-divider">
          <span>按年份</span>
        </div>

        {data.years.map((year) => (
          <div key={year} className="year-group">
            <button
              className={`nav-item year-btn ${isYearSelected(year) ? "active" : ""}`}
              onClick={() => handleYearClick(year)}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="nav-icon">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 6v6l4 2" />
              </svg>
              <span>{year}年</span>
              <span className="nav-badge">
                {data.reports.filter((r) => r.year === year).length}
              </span>
            </button>

            {(selectedYear === year || !selectedYear) &&
              data.months
                .filter(([, m]) => {
                  return data.reports.some(
                    (r) => r.year === year && r.month === m
                  );
                })
                .filter(([, _m]) => {
                  if (selectedYear && selectedYear !== year) return false;
                  return true;
                })
                .slice(0, selectedYear ? 12 : 4)
                .map(([label, m]) => (
                  <button
                    key={`${year}-${m}`}
                    className={`nav-item month-btn ${selectedMonth && selectedMonth[0] === label ? "active" : ""}`}
                    onClick={() => handleMonthClick([label, m] as [string, number])}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="nav-icon-sm">
                      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                    </svg>
                    <span>{monthNames[(m - 1) as number]}</span>
                  </button>
                ))}
          </div>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="footer-stat">
          <span className="footer-label">共 {data.total_count} 篇</span>
          <span className="footer-value">{data.total_size_display}</span>
        </div>
      </div>
    </aside>
  );
}

export default Sidebar;
