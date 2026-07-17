import { createContext, useContext, useState, useEffect, useCallback, useRef, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";

export type Period = "daily" | "weekly" | "monthly";

export interface AppSettings {
  app_name: string;
  period: Period;
  work_dir: string;
  theme: "dark" | "light";
  file_pattern: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  app_name: "通览",
  period: "weekly",
  work_dir: "",
  theme: "light",
  file_pattern: "report_{YYYY}{MM}{DD}.html",
};

export function periodLabel(period: Period): string {
  switch (period) {
    case "daily":
      return "日报";
    case "weekly":
      return "周报";
    case "monthly":
      return "月报";
  }
}

interface SettingsContextValue {
  settings: AppSettings;
  updateSettings: (partial: Partial<AppSettings>) => Promise<boolean>;
  loading: boolean;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const settingsRef = useRef(settings);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  // Load settings from Rust backend on mount
  useEffect(() => {
    const load = async () => {
      try {
        const saved = await invoke<AppSettings>("get_settings");
        if (saved) {
          setSettings(saved);
          // 第一次打开默认浅色：如果有保存的主题用保存的，否则默认 light
          document.documentElement.dataset.theme = saved.theme || "light";
        } else {
          // 无保存配置：默认浅色
          document.documentElement.dataset.theme = "light";
        }
      } catch (err) {
        console.warn("Failed to load settings, using defaults:", err);
        document.documentElement.dataset.theme = "light";
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  // 跟随系统主题：系统深浅色变化时自动切换
  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      const newTheme: "dark" | "light" = e.matches ? "dark" : "light";
      document.documentElement.dataset.theme = newTheme;
      const merged = { ...settingsRef.current, theme: newTheme };
      setSettings(merged);
      // 持久化
      invoke("save_settings_command", { settings: merged }).catch((err) => {
        console.warn("Failed to persist theme on system change:", err);
      });
    };
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  const updateSettings = useCallback(async (partial: Partial<AppSettings>): Promise<boolean> => {
    const merged = { ...settingsRef.current, ...partial };
    setSettings(merged);

    // Apply theme immediately
    if (partial.theme) {
      document.documentElement.dataset.theme = partial.theme;
    }

    // Persist to Rust backend
    try {
      await invoke("save_settings_command", { settings: merged });
      return true;
    } catch (err) {
      console.warn("Failed to save settings:", err);
      return false;
    }
  }, []);

  return (
    <SettingsContext.Provider value={{ settings, updateSettings, loading }}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsContext);
  if (!ctx) {
    throw new Error("useSettings must be used within SettingsProvider");
  }
  return ctx;
}
