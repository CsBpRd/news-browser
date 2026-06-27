import "./ThemeToggle.css";

interface ThemeToggleProps {
  theme: "dark" | "light";
  onToggle: (theme: "dark" | "light") => void;
}

function ThemeToggle({ theme, onToggle }: ThemeToggleProps) {
  const isDark = theme === "dark";

  const handleToggle = () => {
    onToggle(isDark ? "light" : "dark");
  };

  return (
    <button
      className="theme-toggle"
      onClick={handleToggle}
      title={isDark ? "切换到亮色模式" : "切换到暗色模式"}
    >
      <div className={`theme-toggle-track ${isDark ? "dark" : "light"}`}>
        <div className="theme-toggle-thumb">
          {isDark ? (
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="toggle-icon">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
            </svg>
          ) : (
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="toggle-icon">
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
          )}
        </div>
      </div>
    </button>
  );
}

export default ThemeToggle;
