import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { FileEntry, ParsedNote, Settings, SearchResult, GraphData, Snapshot, Workspace, LinkRef, TaskItem, CollectionConfig, SyncFileStatus, ProfileSession, ProfileSummary } from "./types";

export const api = {
  // local profiles
  listProfiles: () => invoke<ProfileSummary[]>("list_profiles"),
  currentProfile: () => invoke<ProfileSession | null>("current_profile"),
  createProfile: (username: string, password: string) =>
    invoke<ProfileSummary>("create_profile", { username, password }),
  unlockProfile: (username: string, password: string) =>
    invoke<ProfileSummary>("unlock_profile", { username, password }),
  lockProfile: () => invoke<void>("lock_profile"),
  changeProfilePassword: (username: string, current: string, next: string) =>
    invoke<void>("change_profile_password", { username, current, next }),
  deleteProfile: (username: string, password: string) =>
    invoke<void>("delete_profile", { username, password }),
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
  writeFileIfUnchanged: (rel: string, content: string, expected: string) =>
    invoke<void>("write_file_if_unchanged", { rel, content, expected }),
  createFile: (rel: string, content: string) => invoke<void>("create_file", { rel, content }),
  createDir: (rel: string) => invoke<void>("create_dir", { rel }),
  deletePath: (rel: string, trash: boolean) => invoke<void>("delete_path", { rel, trash }),
  renamePath: (from: string, to: string) => invoke<void>("rename_path", { from, to }),
  movePath: (from: string, toDir: string) => invoke<string>("move_path", { from, toDir }),
  // markdown
  parseNote: (content: string) => invoke<ParsedNote>("parse_note", { content }),
  renderHtml: (content: string) => invoke<string>("render_html", { content }),
  renderDocument: (content: string, path: string) =>
    invoke<string>("render_document", { content, path }),
  // search
  search: (query: string, filters?: Record<string, unknown>) => invoke<SearchResult[]>("search", { query, filters: filters ?? {} }),
  saveSearch: (id: string, name: string, query: string, filters: Record<string, unknown>) =>
    invoke<void>("save_search", { id, name, query, filters }),
  listSavedSearches: () => invoke<Array<{ id: string; name: string; query: string; filters: Record<string, unknown> }>>("list_saved_searches"),
  deleteSavedSearch: (id: string) => invoke<void>("delete_saved_search", { id }),
  searchHistory: () => invoke<string[]>("search_history"),
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
  listTasks: (rel?: string) => invoke<TaskItem[]>("list_tasks", { rel }),
  setTaskDone: (rel: string, line: number, done: boolean) => invoke<void>("set_task_done", { rel, line, done }),
  // templates
  listTemplates: () => invoke<{ path: string; name: string }[]>("list_templates"),
  renderTemplate: (path: string, vars: Record<string, string>) => invoke<string>("render_template", { path, vars }),
  createDailyNote: (date?: string) => invoke<string>("create_daily_note", { date }),
  appendCapture: (text: string) => invoke<string>("append_capture", { text }),
  // collections (dynamic views)
  listCollections: () => invoke<CollectionConfig[]>("list_collections"),
  saveCollection: (id: string, name: string, config: CollectionConfig["config"]) =>
    invoke<void>("save_collection", { id, name, config }),
  deleteCollection: (id: string) => invoke<void>("delete_collection", { id }),
  collectionNotes: (id: string) => invoke<Array<{ path: string; title: string; properties: Record<string, string>; tags: string[]; modified: number; due?: string | null }>>("collection_notes", { id }),
  // links & tags
  unlinkedMentions: (target: string) => invoke<Array<{ path: string; line: number; text: string }>>("unlinked_mentions", { target }),
  propagateLinkRename: (from: string, to: string, apply: boolean) =>
    invoke<{ path: string; replacements: number }[]>("propagate_link_rename", { from, to, apply }),
  renameTag: (from: string, to: string) => invoke<number>("rename_tag", { from, to }),
  // sync (folder provider)
  syncPush: () => invoke<SyncFileStatus[]>("sync_push"),
  syncPull: () => invoke<SyncFileStatus[]>("sync_pull"),
  syncStatusFolder: () => invoke<SyncFileStatus[]>("sync_status_folder"),
  // plugins
  pluginCatalog: () => invoke<Array<{ id: string; name: string; version: string; description: string; installed: boolean }>>("plugin_catalog"),
  installCatalogPlugin: (id: string) => invoke<void>("install_catalog_plugin", { id }),
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
  listPlugins: () =>
    invoke<
      Array<{
        id: string;
        name: string;
        version: string;
        author: string;
        description: string;
        enabled: boolean;
        permissions: string[];
      }>
    >("list_plugins"),
  enablePlugin: (id: string, enabled: boolean) => invoke<void>("enable_plugin", { id, enabled }),
  uninstallPlugin: (id: string) => invoke<void>("uninstall_plugin", { id }),
  // sync
  syncStatus: () => invoke<unknown>("sync_status"),
  // security (profile passwords are managed through changeProfilePassword)
  // import/export
  importVault: (src: string, dest: string) => invoke<{ imported: number; brokenLinks: string[] }>("import_vault", { src, dest }),
  exportHtml: (rel: string) => invoke<string>("export_html", { rel }),
  exportNote: (rel: string, dest: string, format: string) =>
    invoke<string>("export_note", { rel, dest, format }),
  exportPrintHtml: (rel: string) => invoke<string>("export_print_html", { rel }),
  saveTextFile: (dest: string, content: string) =>
    invoke<void>("save_text_file", { dest, content }),
  backupVault: (dest: string) => invoke<void>("backup_vault", { dest }),
  restoreBackup: (src: string, dest: string) => invoke<void>("restore_backup", { src, dest }),
  // fs watch events
  onVaultFsChange: (callback: (paths: string[]) => void) =>
    listen<string[]>("vault-fs-change", (event) => callback(event.payload)),
  onVaultFileWritten: (callback: (path: string) => void) =>
    listen<string>("vault-file-written", (event) => callback(event.payload)),
};
