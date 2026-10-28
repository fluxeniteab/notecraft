import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, ParsedNote, Settings, SearchResult, GraphData, Snapshot, Workspace, LinkRef } from "./types";

export const api = {
  // vault
  openVault: (path: string) => invoke<string>("open_vault", { path }),
  createVault: (path: string) => invoke<string>("create_vault", { path }),
  closeVault: () => invoke<void>("close_vault"),
  getVaultRoot: () => invoke<string | null>("get_vault_root"),
  listRecentVaults: () => invoke<string[]>("list_recent_vaults"),
  listDir: (rel: string) => invoke<FileEntry[]>("list_dir", { rel }),
  listAllFiles: () => invoke<FileEntry[]>("list_all_files"),
  readFile: (rel: string) => invoke<string>("read_file", { rel }),
  writeFile: (rel: string, content: string) => invoke<void>("write_file", { rel, content }),
  createFile: (rel: string, content: string) => invoke<void>("create_file", { rel, content }),
  createDir: (rel: string) => invoke<void>("create_dir", { rel }),
  deletePath: (rel: string, trash: boolean) => invoke<void>("delete_path", { rel, trash }),
  renamePath: (from: string, to: string) => invoke<void>("rename_path", { from, to }),
  movePath: (from: string, toDir: string) => invoke<string>("move_path", { from, toDir }),
  // markdown
  parseNote: (content: string) => invoke<ParsedNote>("parse_note", { content }),
  renderHtml: (content: string) => invoke<string>("render_html", { content }),
  // search
  search: (query: string, filters?: Record<string, unknown>) => invoke<SearchResult[]>("search", { query, filters: filters ?? {} }),
  rebuildIndex: () => invoke<void>("rebuild_index"),
  indexStatus: () => invoke<{ total: number; indexed: number; status: string }>("index_status"),
  // links
  rebuildLinks: (rel: string, content: string) => invoke<void>("rebuild_links", { rel, content }),
  backlinks: (target: string) => invoke<string[]>("backlinks", { target }),
  outgoingLinks: (source: string) => invoke<LinkRef[]>("outgoing_links", { source }),
  unresolvedLinks: () => invoke<string[]>("unresolved_links"),
  // graph
  graphData: (scope?: string, filters?: Record<string, unknown>) => invoke<GraphData>("graph_data", { scope: scope ?? "global", filters: filters ?? {} }),
  // history
  snapshotNow: (rel: string, label?: string) => invoke<Snapshot>("snapshot_now", { rel, label }),
  listSnapshots: (rel: string) => invoke<Snapshot[]>("list_snapshots", { rel }),
  getSnapshot: (id: string) => invoke<string>("get_snapshot", { id }),
  restoreSnapshot: (id: string) => invoke<void>("restore_snapshot", { id }),
  diffSnapshots: (a: string, b: string) => invoke<string>("diff_snapshots", { a, b }),
  // tasks
  listTasks: (rel?: string) => invoke<unknown[]>("list_tasks", { rel }),
  setTaskDone: (rel: string, line: number, done: boolean) => invoke<void>("set_task_done", { rel, line, done }),
  // templates
  listTemplates: () => invoke<{ path: string; name: string }[]>("list_templates"),
  renderTemplate: (path: string, vars: Record<string, string>) => invoke<string>("render_template", { path, vars }),
  // settings
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (s: Settings) => invoke<void>("save_settings", { s }),
  // workspaces
  listWorkspaces: () => invoke<Workspace[]>("list_workspaces"),
  saveWorkspace: (id: string, name: string, data: string) => invoke<void>("save_workspace", { id, name, data }),
  deleteWorkspace: (id: string) => invoke<void>("delete_workspace", { id }),
  // automation
  listAutomations: () => invoke<unknown[]>("list_automations"),
  saveAutomation: (a: unknown) => invoke<void>("save_automation", { a }),
  deleteAutomation: (id: string) => invoke<void>("delete_automation", { id }),
  runAutomationDryRun: (id: string) => invoke<unknown>("run_automation_dry_run", { id }),
  runAutomation: (id: string, note?: string) => invoke<unknown>("run_automation", { id, note }),
  listAutomationRuns: (id?: string) => invoke<unknown[]>("list_automation_runs", { id }),
  // plugins
  listPlugins: () => invoke<unknown[]>("list_plugins"),
  enablePlugin: (id: string, enabled: boolean) => invoke<void>("enable_plugin", { id, enabled }),
  uninstallPlugin: (id: string) => invoke<void>("uninstall_plugin", { id }),
  // sync
  syncStatus: () => invoke<unknown>("sync_status"),
  // security
  setVaultPassword: (password: string) => invoke<void>("set_vault_password", { password }),
  // import/export
  importVault: (src: string, dest: string) => invoke<{ imported: number; brokenLinks: string[] }>("import_vault", { src, dest }),
  exportHtml: (rel: string) => invoke<string>("export_html", { rel }),
  backupVault: (dest: string) => invoke<void>("backup_vault", { dest }),
  restoreBackup: (src: string, dest: string) => invoke<void>("restore_backup", { src, dest }),
  // fs watch events handled via listen
};
