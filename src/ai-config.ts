import { invoke } from "@tauri-apps/api/core";

export interface AIConfig {
  api_endpoint: string;
  api_key: string;
  model_name: string;
  temperature: number;
  max_tokens: number;
}

export const DEFAULT_AI_CONFIG: AIConfig = {
  api_endpoint: "",
  api_key: "",
  model_name: "",
  temperature: 0.7,
  max_tokens: 16000,
};

export async function getAIConfig(): Promise<AIConfig> {
  try {
    return await invoke<AIConfig>("get_ai_config");
  } catch (err) {
    console.warn("Failed to load AI config, using defaults:", err);
    return DEFAULT_AI_CONFIG;
  }
}

export async function saveAIConfig(config: AIConfig): Promise<void> {
  await invoke("save_ai_config", { config });
}
