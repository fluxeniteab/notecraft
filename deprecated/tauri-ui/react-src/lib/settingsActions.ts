import { toast } from "sonner";
import { api } from "./tauri";
import { useAppStore } from "../store/appStore";
import type { Settings, Theme } from "./types";

export async function persistSettings(next: Settings) {
  const store = useAppStore.getState();
  store.setSettings(next);
  try {
    await api.saveSettings(next);
  } catch (error) {
    console.error(error);
    toast.error("Settings were applied but could not be saved", { description: String(error) });
  }
}

export async function setThemeEverywhere(theme: Theme) {
  const store = useAppStore.getState();
  store.setTheme(theme);
  if (store.settings) {
    await persistSettings({ ...store.settings, theme });
  }
}
