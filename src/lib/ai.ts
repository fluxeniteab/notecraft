import { invoke } from "@tauri-apps/api/core";

export interface AiProviderPreset {
  id: string;
  name: string;
  kind: "openai" | "gemini";
  base_url: string;
  model: string;
  needs_key: boolean;
  local: boolean;
  docs: string;
  note: string;
}

export interface AiConfig {
  provider: string;
  base_url: string;
  api_key: string;
  model: string;
  temperature: number;
}

export interface AiMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export const ai = {
  providers: () => invoke<AiProviderPreset[]>("ai_providers"),
  loadConfig: () => invoke<AiConfig>("ai_load_config"),
  saveConfig: (config: AiConfig) => invoke<void>("ai_save_config", { config }),
  chat: (config: AiConfig, system: string, messages: AiMessage[]) =>
    invoke<string>("ai_chat", { config, system, messages }),
};

export function isConfigured(config: AiConfig | null, provider: AiProviderPreset | undefined): boolean {
  if (!config || !config.model.trim() || !config.base_url.trim()) return false;
  if (provider && provider.needs_key && !config.api_key.trim()) return false;
  return true;
}
