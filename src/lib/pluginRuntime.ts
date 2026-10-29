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
  author?: string;
  description: string;
  enabled: boolean;
  permissions: string[];
}

export const pluginEngine = {
  importZip: (path: string) => invoke<InstalledPlugin>("import_plugin_zip", { path }),
  package: (id: string) => invoke<PluginPackage>("plugin_package", { id }),
};

/**
 * Permissions a plugin may request. Mirrors `KNOWN_PERMISSIONS` in
 * `src-tauri/src/plugins.rs`; unknown permissions are rejected at install time.
 */
export const KNOWN_PERMISSIONS = [
  "notes:read",
  "notes:write",
  "notes:create",
  "notes:delete",
  "links:read",
  "ui:notify",
  "editor:open",
  "settings:read",
] as const;

export type PluginPermission = (typeof KNOWN_PERMISSIONS)[number];

/** Permissions that can overwrite or destroy user data. */
export const SENSITIVE_PERMISSIONS: PluginPermission[] = ["notes:write", "notes:delete"];

/**
 * The plugin API surface, grouped by the permission that unlocks it.
 *
 * Notably absent: profile/password commands, AI provider configuration (which
 * holds API keys), settings writes, plugin management, backup/restore and sync.
 * A plugin cannot reach those through this object no matter what it declares.
 */
const PERMISSION_API = {
  "notes:read": [
    "readFile",
    "listDir",
    "listAllFiles",
    "parseNote",
    "search",
    "listTasks",
    "listSnapshots",
    "getSnapshot",
  ],
  "notes:write": ["writeFile", "writeFileIfUnchanged", "renamePath", "movePath", "snapshotNow"],
  "notes:create": ["createFile", "createDocument", "createDir"],
  "notes:delete": ["deletePath"],
  "links:read": ["backlinks", "outgoingLinks", "unresolvedLinks", "graphData"],
} as const satisfies Record<string, ReadonlyArray<keyof typeof api>>;

export type PluginApi = Partial<typeof api>;

/** Build the API object a plugin is allowed to touch. */
export function scopedPluginApi(permissions: string[]): PluginApi {
  const granted = new Set(permissions.map((perm) => perm.toLowerCase()));
  const scoped: Record<string, unknown> = {};
  for (const [permission, names] of Object.entries(PERMISSION_API)) {
    if (!granted.has(permission)) continue;
    for (const name of names) {
      const fn = (api as Record<string, unknown>)[name];
      if (typeof fn === "function") scoped[name] = fn;
    }
  }
  return scoped as PluginApi;
}

/**
 * Read-only view of the app store. `setState` is deliberately omitted: a plugin
 * must not be able to replace application state wholesale.
 */
function readOnlyStore() {
  return {
    getState: useAppStore.getState,
    subscribe: useAppStore.subscribe,
  };
}

export interface PluginContext {
  pluginId: string;
  version: string;
  permissions: string[];
  api: PluginApi;
  notify: (message: string) => void;
  store: ReturnType<typeof readOnlyStore>;
}

const loaded = new Set<string>();

/**
 * Run an installed plugin: inject its stylesheet and load its ES module entry.
 *
 * Plugins are trusted code rather than a sandbox: a JavaScript module shares the
 * webview realm with the app, so a determined plugin can bypass this facade.
 * Installing does not execute anything — a plugin only runs after the user
 * enables it in Settings, which is also where its permissions are listed.
 */
export async function loadPlugin(plugin: InstalledPlugin): Promise<void> {
  if (loaded.has(plugin.id)) return;
  const pkg = await pluginEngine.package(plugin.id);
  if (pkg.style) {
    document.querySelectorAll(`style[data-plugin="${plugin.id}"]`).forEach((node) => node.remove());
    const style = document.createElement("style");
    style.dataset.plugin = plugin.id;
    style.textContent = pkg.style;
    document.head.appendChild(style);
  }
  if (pkg.entry.toLowerCase().endsWith(".ts")) {
    throw new Error("This plugin ships TypeScript; rebuild it to JavaScript before installing.");
  }

  const permissions = plugin.permissions ?? [];
  const api = scopedPluginApi(permissions);
  const context: PluginContext = {
    pluginId: plugin.id,
    version: plugin.version,
    permissions,
    api,
    notify: (message: string) => {
      if (!permissions.includes("ui:notify")) {
        throw new Error(`"${plugin.id}" needs the ui:notify permission to show notifications.`);
      }
      toast(message);
    },
    store: readOnlyStore(),
  };

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
    await activate(context);
    loaded.add(plugin.id);
  } finally {
    URL.revokeObjectURL(url);
  }
}

export function unloadPluginStyles(id: string) {
  document.querySelectorAll(`style[data-plugin="${id}"]`).forEach((node) => node.remove());
  loaded.delete(id);
}
