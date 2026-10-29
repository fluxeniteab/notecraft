import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { api } from "./tauri";
import { useAppStore } from "../store/appStore";

export interface PluginPackage {
  id: string;
  entry: string;
  code: string;
  style: string | null;
  icon: string | null;
}

export interface InstalledPlugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
  permissions: string[];
}

export const pluginEngine = {
  importZip: (path: string) => invoke<InstalledPlugin>("import_plugin_zip", { path }),
  package: (id: string) => invoke<PluginPackage>("plugin_package", { id }),
};

const loaded = new Set<string>();

export interface PluginContext {
  pluginId: string;
  version: string;
  api: typeof api;
  toast: typeof toast;
  store: typeof useAppStore;
}

/**
 * Run an installed plugin: inject its stylesheet and load its ES module entry.
 * Plugins run with the app's privileges, so the settings page shows a warning
 * and only loads plugins the user enabled.
 */
export async function loadPlugin(id: string): Promise<void> {
  if (loaded.has(id)) return;
  const pkg = await pluginEngine.package(id);
  if (pkg.style) {
    document.querySelectorAll(`style[data-plugin="${id}"]`).forEach((node) => node.remove());
    const style = document.createElement("style");
    style.dataset.plugin = id;
    style.textContent = pkg.style;
    document.head.appendChild(style);
  }
  if (pkg.entry.toLowerCase().endsWith(".ts")) {
    throw new Error("This plugin ships TypeScript; rebuild it to JavaScript before installing.");
  }
  const url = URL.createObjectURL(new Blob([pkg.code], { type: "text/javascript" }));
  try {
    const module = (await import(/* @vite-ignore */ url)) as {
      activate?: (context: PluginContext) => unknown;
      default?: (context: PluginContext) => unknown;
    };
    const activate = module.activate ?? module.default;
    if (typeof activate !== "function") {
      throw new Error("The plugin entry does not export activate()");
    }
    await activate({ pluginId: id, version: "0.2.1", api, toast, store: useAppStore });
    loaded.add(id);
  } finally {
    URL.revokeObjectURL(url);
  }
}

export function unloadPluginStyles(id: string) {
  document.querySelectorAll(`style[data-plugin="${id}"]`).forEach((node) => node.remove());
  loaded.delete(id);
}
