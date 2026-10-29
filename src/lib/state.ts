import { derived, get, writable } from "svelte/store";
import type { FileEntry, JsonTreeNode, ProfileSession, Settings, Theme } from "./types";
import { api } from "./ipc";
import { forgetEditorState } from "./editorCache";
import { capabilitiesFor } from "./documentTypes";
import { baseName, normalizeRel, stem } from "./files";

export type TabMode = "source" | "live" | "reading" | "split";
export type SaveStatus = "idle" | "dirty" | "saving" | "saved" | "error";
export type Validation = { valid: boolean; line?: number; column?: number; message?: string } | null;

export interface TabStats {
  words: number;
  chars: number;
  lines: number;
  validation: Validation;
}

export interface Tab {
  id: string;
  path: string;
  title: string;
  content: string;
  base: string;
  dirty: boolean;
  loading: boolean;
  loadError: string | null;
  status: SaveStatus;
  error: string | null;
  mode: TabMode;
  cursor: { line: number; col: number };
  stats: TabStats | null;
  json: JsonTreeNode | null;
  jsonError: { message: string; line: number; column: number } | null;
  jsonStats: { keys: number; nodes: number; depth: number; truncated: boolean } | null;
}

export type ToastKind = "success" | "error" | "info";
export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  description?: string;
}

export interface ModalRequest {
  kind: "prompt" | "confirm";
  title: string;
  description?: string;
  label?: string;
  value?: string;
  placeholder?: string;
  confirmLabel?: string;
  danger?: boolean;
  resolve: (value: string | boolean | null) => void;
}

export const DEFAULT_SETTINGS: Settings = {
  theme: "dark",
  accent: "#5b8cff",
  editor_font_size: 15,
  editor_line_height: 1.72,
  editor_font_family: "Inter",
  reading_width: 720,
  autosave_ms: 900,
  snapshot_interval_min: 10,
  snapshot_retention: 50,
  excluded_folders: [],
  hidden_files: false,
  reduced_motion: false,
  telemetry_opt_in: false,
  semantic_search_enabled: true,
  daily_note_folder: "Daily",
  daily_note_template: null,
  capture_note: "Inbox.md",
  explorer_sort: "name",
  explorer_group_dirs: true,
  shortcuts: {},
  writing_goal_words: 0,
  sync_folder: null,
  data_dir: null,
};

const SESSION_KEY = "notecraft:session:v2";

/* --------------------------------------------------------------- stores --- */

export const settings = writable<Settings>(DEFAULT_SETTINGS);
export const theme = derived(settings, ($s) => $s.theme as Theme);
export const profile = writable<ProfileSession | null>(null);
export const booting = writable(true);

export const vaultRoot = writable<string | null>(null);
export const recentVaults = writable<string[]>([]);
export const allFiles = writable<FileEntry[]>([]);

export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);
export const activeTab = derived([tabs, activeTabId], ([$tabs, $id]) => $tabs.find((tab) => tab.id === $id) ?? null);

export const selectedPath = writable<string | null>(null);
export const expanded = writable<Set<string>>(new Set());

export const sidebarView = writable<"files" | "search" | "tags">("files");
export const rightTab = writable<"outline" | "backlinks" | "tags" | "tasks" | "history" | "assistant">(
  "outline"
);
export const leftVisible = writable(true);
export const rightVisible = writable(true);
export const leftWidth = writable(268);
export const rightWidth = writable(300);
export const focusMode = writable(false);

export const paletteOpen = writable(false);
export const switcherOpen = writable(false);
export const settingsOpen = writable(false);
export const graphOpen = writable(false);
export const collectionsOpen = writable(false);
export const automationsOpen = writable(false);
export const canvasOpen = writable(false);
export const captureOpen = writable(false);
export const aboutOpen = writable(false);
export const shortcutsOpen = writable(false);
export const settingsSection = writable<"appearance" | "editor" | "workspace" | "plugins">("appearance");

export const searchQuery = writable("");
export const searchResults = writable<Array<{ path: string; title: string; snippet: string; score: number }>>([]);
export const searching = writable(false);

export const toasts = writable<Toast[]>([]);
export const modal = writable<ModalRequest | null>(null);

export const favoritePaths = writable<string[]>(readFavorites());
export const recentNotes = writable<string[]>([]);

/* ------------------------------------------------------------- helpers --- */

let toastSeq = 0;

export function pushToast(kind: ToastKind, title: string, description?: string): void {
  const id = ++toastSeq;
  toasts.update((list) => [...list, { id, kind, title, description }].slice(-4));
  window.setTimeout(() => dismissToast(id), kind === "error" ? 7000 : 4000);
}

export function dismissToast(id: number): void {
  toasts.update((list) => list.filter((toast) => toast.id !== id));
}

export function askPrompt(options: {
  title: string;
  description?: string;
  label?: string;
  value?: string;
  placeholder?: string;
  confirmLabel?: string;
}): Promise<string | null> {
  return new Promise((resolve) => {
    modal.set({ kind: "prompt", ...options, resolve: resolve as (value: string | boolean | null) => void });
  });
}

export function askConfirm(options: {
  title: string;
  description?: string;
  confirmLabel?: string;
  danger?: boolean;
}): Promise<boolean> {
  return new Promise((resolve) => {
    modal.set({ kind: "confirm", ...options, resolve: resolve as (value: string | boolean | null) => void });
  });
}

export function closeModal(value: string | boolean | null): void {
  const current = get(modal);
  modal.set(null);
  current?.resolve(value);
}

function readFavorites(): string[] {
  try {
    const raw = localStorage.getItem("notecraft:favorites:v2");
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter((v): v is string => typeof v === "string") : [];
  } catch {
    return [];
  }
}

export function toggleFavorite(path: string): void {
  favoritePaths.update((list) => {
    const next = list.includes(path) ? list.filter((p) => p !== path) : [path, ...list];
    localStorage.setItem("notecraft:favorites:v2", JSON.stringify(next.slice(0, 40)));
    return next;
  });
}

export function newId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `tab-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export function countWords(text: string): number {
  const trimmed = text.trim();
  return trimmed ? trimmed.split(/\s+/).length : 0;
}

export function statsFor(content: string, path: string): TabStats {
  const caps = capabilitiesFor(path);
  return {
    words: caps.stats === "prose" ? countWords(content) : 0,
    chars: content.length,
    lines: content ? content.split("\n").length : 0,
    validation: null,
  };
}

export function updateTab(id: string, patch: Partial<Tab>): void {
  tabs.update((list) => list.map((tab) => (tab.id === id ? { ...tab, ...patch } : tab)));
}

/* ------------------------------------------------------------ tab logic --- */

const previewTimer = new Map<string, number>();
const validationTimer = new Map<string, number>();
const saveTimers = new Map<string, number>();
const saveJobs = new Map<string, Promise<void>>();

async function renderPreview(tab: Tab): Promise<void> {
  window.clearTimeout(previewTimer.get(tab.id));
  previewTimer.set(
    tab.id,
    window.setTimeout(async () => {
      const current = get(tabs).find((item) => item.id === tab.id);
      if (!current || capabilitiesFor(current.path).preview === "none") return;
      try {
        if (capabilitiesFor(current.path).kind === "json") {
          await refreshJson(current);
        }
      } catch {
        /* preview errors are non-fatal */
      }
    }, 260)
  );
}

export async function refreshJson(tab: Tab): Promise<void> {
  try {
    const analysis = await api.jsonAnalyze(tab.content);
    updateTab(tab.id, {
      json: analysis.root,
      jsonError: analysis.error ? { message: analysis.error.message, line: analysis.error.line, column: analysis.error.column } : null,
      jsonStats: { keys: analysis.keys, nodes: analysis.nodes, depth: analysis.depth, truncated: analysis.truncated },
    });
  } catch (error) {
    updateTab(tab.id, { jsonError: { message: String(error), line: 0, column: 0 } });
  }
}

function scheduleValidation(id: string): void {
  window.clearTimeout(validationTimer.get(id));
  validationTimer.set(
    id,
    window.setTimeout(async () => {
      const tab = get(tabs).find((item) => item.id === id);
      if (!tab) return;
      const caps = capabilitiesFor(tab.path);
      if (caps.kind === "json") {
        try {
          const analysis = await api.jsonAnalyze(tab.content);
          const validation: Validation = analysis.valid
            ? { valid: true }
            : {
                valid: false,
                line: analysis.error?.line ?? undefined,
                column: analysis.error?.column ?? undefined,
                message: analysis.error?.message ?? "Invalid JSON",
              };
          tabs.update((list) =>
            list.map((t) => (t.id === id && t.stats ? { ...t, stats: { ...t.stats, validation } } : t))
          );
        } catch {
          /* parser errors are reported by the JSON view */
        }
      }
    }, 380)
  );
}

export function updateContent(id: string, content: string): void {
  const tab = get(tabs).find((item) => item.id === id);
  if (!tab || tab.loading) return;
  const unchanged = content === tab.base;
  updateTab(id, {
    content,
    dirty: !unchanged,
    status: unchanged ? "saved" : "dirty",
    error: null,
    stats: statsFor(content, tab.path),
  });
  scheduleSave(id);
  scheduleValidation(id);
  if (capabilitiesFor(tab.path).kind === "json") void refreshJson({ ...tab, content });
}

export function scheduleSave(id: string): void {
  const tab = get(tabs).find((item) => item.id === id);
  if (!tab || !tab.dirty || tab.status === "error") return;
  window.clearTimeout(saveTimers.get(id));
  saveTimers.set(
    id,
    window.setTimeout(() => {
      saveTimers.delete(id);
      void saveTab(id);
    }, Math.max(350, get(settings).autosave_ms))
  );
}

export async function saveTab(id: string, force = false): Promise<void> {
  window.clearTimeout(saveTimers.get(id));
  saveTimers.delete(id);

  const running = saveJobs.get(id);
  if (running) {
    await running;
    const after = get(tabs).find((item) => item.id === id);
    if (!after || !after.dirty) return;
  }

  const tab = get(tabs).find((item) => item.id === id);
  if (!tab || tab.loading || tab.loadError) return;
  if (!tab.dirty) {
    updateTab(id, { status: "saved" });
    return;
  }

  const snapshot = tab.content;
  const expected = tab.base;
  updateTab(id, { status: "saving" });

  const job = (async () => {
    try {
      if (force) await api.writeFile(tab.path, snapshot);
      else await api.writeFileIfUnchanged(tab.path, snapshot, expected);
      const latest = get(tabs).find((item) => item.id === id);
      const stillDirty = !!latest && latest.content !== snapshot;
      updateTab(id, {
        base: snapshot,
        dirty: stillDirty,
        status: stillDirty ? "dirty" : "saved",
        error: null,
      });
      if (stillDirty) scheduleSave(id);
    } catch (error) {
      updateTab(id, { status: "error", error: String(error) });
      pushToast("error", "Could not save this file", String(error));
    } finally {
      saveJobs.delete(id);
    }
  })();

  saveJobs.set(id, job);
  await job;
}

export async function flushTab(id: string): Promise<void> {
  await saveTab(id);
}

export async function flushAllTabs(): Promise<void> {
  await Promise.allSettled(get(tabs).map((tab) => saveTab(tab.id)));
}

export async function openTab(rawPath: string): Promise<void> {
  const path = normalizeRel(rawPath);
  if (!path) return;
  const existing = get(tabs).find((tab) => tab.path === path);
  if (existing) {
    activeTabId.set(existing.id);
    selectedPath.set(path);
    recentNotes.update((list) => [path, ...list.filter((p) => p !== path)].slice(0, 12));
    saveSession();
    return;
  }

  const id = newId();
  const tab: Tab = {
    id,
    path,
    title: stem(path) || baseName(path),
    content: "",
    base: "",
    dirty: false,
    loading: true,
    loadError: null,
    status: "idle",
    error: null,
    mode: "source",
    cursor: { line: 1, col: 1 },
    stats: null,
    json: null,
    jsonError: null,
    jsonStats: null,
  };
  tabs.update((list) => [...list, tab]);
  activeTabId.set(id);
  selectedPath.set(path);
  recentNotes.update((list) => [path, ...list.filter((p) => p !== path)].slice(0, 12));
  saveSession();

  try {
    const content = await api.readFile(path);
    updateTab(id, {
      content,
      base: content,
      loading: false,
      status: "idle",
      stats: statsFor(content, path),
      mode:
        capabilitiesFor(path).kind === "json"
          ? "split"
          : capabilitiesFor(path).kind === "markdown"
            ? "live"
            : "source",
    });
    const loaded = get(tabs).find((item) => item.id === id);
    if (loaded) {
      if (capabilitiesFor(path).kind === "json") {
        void refreshJson(loaded);
        scheduleValidation(id);
      }
      void renderPreview(loaded);
    }
  } catch (error) {
    updateTab(id, { loading: false, loadError: String(error) });
  }
}

export async function closeTab(id: string): Promise<boolean> {
  const tab = get(tabs).find((item) => item.id === id);
  if (!tab) return true;
  if (tab.dirty) await saveTab(id);
  const latest = get(tabs).find((item) => item.id === id);
  if (latest?.status === "error") {
    pushToast("error", "This file could not be saved, so it stayed open");
    return false;
  }
  tabs.update((list) => {
    const index = list.findIndex((item) => item.id === id);
    const next = list.filter((item) => item.id !== id);
    if (get(activeTabId) === id) {
      activeTabId.set(next[Math.min(index, next.length - 1)]?.id ?? null);
    }
    return next;
  });
  forgetEditorState(id);
  saveSession();
  return true;
}

export function cycleTab(direction: 1 | -1): void {
  const list = get(tabs);
  if (list.length < 2) return;
  const index = list.findIndex((tab) => tab.id === get(activeTabId));
  const next = list[(index + direction + list.length) % list.length];
  if (next) activeTabId.set(next.id);
}

/* -------------------------------------------------------------- vaulting --- */

export async function refreshFiles(): Promise<void> {
  if (!get(vaultRoot)) {
    allFiles.set([]);
    return;
  }
  try {
    allFiles.set(await api.listAllFiles());
  } catch (error) {
    pushToast("error", "Could not read the vault", String(error));
  }
}

export function resetWorkspace(): void {
  for (const tab of get(tabs)) forgetEditorState(tab.id);
  tabs.set([]);
  activeTabId.set(null);
  selectedPath.set(null);
  recentNotes.set([]);
}

/* --------------------------------------------------------------- session --- */

let sessionTimer = 0;

export function saveSession(): void {
  window.clearTimeout(sessionTimer);
  sessionTimer = window.setTimeout(() => {
    try {
      localStorage.setItem(
        SESSION_KEY,
        JSON.stringify({
          tabs: get(tabs).map((tab) => tab.path),
          active: get(tabs).find((tab) => tab.id === get(activeTabId))?.path ?? null,
          left: get(leftVisible),
          right: get(rightVisible),
          leftWidth: get(leftWidth),
          rightWidth: get(rightWidth),
          focus: get(focusMode),
          recents: get(recentNotes),
        })
      );
    } catch {
      /* storage can be unavailable */
    }
  }, 220);
}

export async function restoreSession(): Promise<void> {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as {
      tabs?: string[];
      active?: string | null;
      left?: boolean;
      right?: boolean;
      leftWidth?: number;
      rightWidth?: number;
      recents?: string[];
    };
    if (typeof parsed.left === "boolean") leftVisible.set(parsed.left);
    if (typeof parsed.right === "boolean") rightVisible.set(parsed.right);
    if (typeof parsed.leftWidth === "number") leftWidth.set(Math.min(420, Math.max(220, parsed.leftWidth)));
    if (typeof parsed.rightWidth === "number") rightWidth.set(Math.min(440, Math.max(240, parsed.rightWidth)));
    if (Array.isArray(parsed.recents)) {
      recentNotes.set(parsed.recents.filter((value): value is string => typeof value === "string").slice(0, 12));
    }
    const paths = (parsed.tabs ?? []).slice(0, 12);
    for (const path of paths) {
      const known = get(allFiles).some((file) => file.path === normalizeRel(path) && !file.is_dir);
      if (known) await openTab(path);
    }
    if (typeof parsed.active === "string" && parsed.active) {
      const activePath = normalizeRel(parsed.active);
      const tab = get(tabs).find((item) => item.path === activePath);
      if (tab) activeTabId.set(tab.id);
    }
  } catch {
    /* a malformed session must never block startup */
  }
}

/* ---------------------------------------------------------------- doc ops --- */
