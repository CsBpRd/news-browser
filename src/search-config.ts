import { invoke } from "@tauri-apps/api/core";

export type SearchProvider = "tavily" | "custom";

export interface SearchConfig {
  provider: SearchProvider;
  api_key: string;
  api_endpoint: string;
  max_results: number;
}

export const DEFAULT_SEARCH_CONFIG: SearchConfig = {
  provider: "tavily",
  api_key: "",
  api_endpoint: "",
  max_results: 10,
};

export async function getSearchConfig(): Promise<SearchConfig> {
  try {
    return await invoke<SearchConfig>("get_search_config");
  } catch (err) {
    console.warn("Failed to load search config, using defaults:", err);
    return DEFAULT_SEARCH_CONFIG;
  }
}

export async function saveSearchConfig(config: SearchConfig): Promise<void> {
  await invoke("save_search_config", { config });
}

export interface SearchResult {
  title: string;
  url: string;
  content: string;
}

export async function searchWeb(query: string): Promise<SearchResult[]> {
  return await invoke<SearchResult[]>("search_web_command", { query });
}
