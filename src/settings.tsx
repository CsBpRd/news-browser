import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
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
  theme: "dark",
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
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  loading: boolean;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);

  // Load settings from Rust backend on mount
  useEffect(() => {
    const load = async () => {
      try {
        const saved = await invoke<AppSettings>("get_settings");
        if (saved) {
          setSettings(saved);
        }
        // Apply theme
        document.documentElement.dataset.theme = saved?.theme || "dark";
      } catch (err) {
        console.warn("Failed to load settings, using defaults:", err);
        document.documentElement.dataset.theme = "dark";
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const updateSettings = useCallback(async (partial: Partial<AppSettings>) => {
    const merged = { ...settings, ...partial };
    setSettings(merged);

    // Apply theme immediately
    if (partial.theme) {
      document.documentElement.dataset.theme = partial.theme;
    }

    // Persist to Rust backend
    try {
      await invoke("save_settings_command", { settings: merged });
    } catch (err) {
      console.warn("Failed to save settings:", err);
    }
  }, [settings]);

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
