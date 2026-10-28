import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { FileEntry, Settings, Theme } from "../lib/types";
import { baseName, normalizeRel, stem } from "../lib/files";

export type TabMode = "source" | "live" | "reading" | "split";
export type SaveStatus = "idle" | "dirty" | "saving" | "saved" | "error";

export interface Tab {
  id: string;
  path: string;
  title: string;
  dirty: boolean;
  pinned: boolean;
  cursor?: { line: number; col: number };
  scroll?: number;
  mode: TabMode;
}

export interface PanelState {
  leftSidebar: boolean;
  rightSidebar: boolean;
  leftWidth: number;
  rightWidth: number;
}

export interface TabStats {
  words: number;
  chars: number;
}

export interface GotoRequest {
  path: string;
  line: number;
  nonce: number;
}

interface AppState {
  vaultRoot: string | null;
  recentVaults: string[];
  files: FileEntry[];
  allFiles: FileEntry[];
  selectedFile: string | null;
  tabs: Tab[];
  activeTabId: string | null;
  panels: PanelState;
  settings: Settings | null;
  theme: Theme;
  focusMode: boolean;
  commandPaletteOpen: boolean;
  quickSwitcherOpen: boolean;
  searchQuery: string;
  tabStats: Record<string, TabStats>;
  saveStatus: Record<string, SaveStatus>;
  cursors: Record<string, { line: number; col: number }>;
  recentNotes: string[];
  gotoRequest: GotoRequest | null;
  // actions
  setVaultRoot: (p: string | null) => void;
  setRecentVaults: (v: string[]) => void;
  setFiles: (f: FileEntry[]) => void;
  setAllFiles: (f: FileEntry[]) => void;
  setSelectedFile: (p: string | null) => void;
  openTab: (path: string, title?: string) => void;
  closeTab: (id: string) => void;
  closeTabsUnder: (prefix: string) => void;
  setActiveTab: (id: string | null) => void;
  setTabDirty: (id: string, dirty: boolean) => void;
  setTabMode: (id: string, mode: TabMode) => void;
  setTabTitle: (path: string, title: string) => void;
  remapPaths: (from: string, to: string) => void;
  reorderTabs: (from: number, to: number) => void;
  togglePanel: (k: "leftSidebar" | "rightSidebar") => void;
  setPanelWidth: (k: "leftWidth" | "rightWidth", w: number) => void;
  setSettings: (s: Settings) => void;
  setTheme: (t: Theme) => void;
  setFocusMode: (b: boolean) => void;
  toggleFocusMode: () => void;
  setCommandPaletteOpen: (b: boolean) => void;
  setQuickSwitcherOpen: (b: boolean) => void;
  setSearchQuery: (q: string) => void;
  setTabStats: (id: string, stats: TabStats) => void;
  setSaveStatus: (id: string, status: SaveStatus) => void;
  setCursor: (id: string, cursor: { line: number; col: number }) => void;
  gotoLine: (path: string, line: number) => void;
}

function tabTitle(path: string): string {
  return stem(path) || baseName(path);
}

function newId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `tab-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      vaultRoot: null,
      recentVaults: [],
      files: [],
      allFiles: [],
      selectedFile: null,
      tabs: [],
      activeTabId: null,
      panels: { leftSidebar: true, rightSidebar: true, leftWidth: 19, rightWidth: 24 },
      settings: null,
      theme: "dark",
      focusMode: false,
      commandPaletteOpen: false,
      quickSwitcherOpen: false,
      searchQuery: "",
      tabStats: {},
      saveStatus: {},
      cursors: {},
      recentNotes: [],
      gotoRequest: null,
      setVaultRoot: (p) => set({ vaultRoot: p }),
      setRecentVaults: (v) => set({ recentVaults: v }),
      setFiles: (f) => set({ files: f }),
      setAllFiles: (f) => set({ allFiles: f }),
      setSelectedFile: (p) => set({ selectedFile: p }),
      openTab: (rawPath, title) => {
        const path = normalizeRel(rawPath);
        if (!path) return;
        const existing = get().tabs.find((t) => t.path === path);
        if (existing) {
          set({ activeTabId: existing.id, selectedFile: path });
          return;
        }
        let id = newId();
        while (get().tabs.some((t) => t.id === id)) id = newId();
        const tab: Tab = {
          id,
          path,
          title: title ?? tabTitle(path),
          dirty: false,
          pinned: false,
          mode: "live",
        };
        const recentNotes = [path, ...get().recentNotes.filter((p) => p !== path)].slice(0, 12);
        set({
          tabs: [...get().tabs, tab],
          activeTabId: id,
          selectedFile: path,
          recentNotes,
        });
      },
      closeTab: (id) => {
        const state = get();
        const index = state.tabs.findIndex((t) => t.id === id);
        const tabs = state.tabs.filter((t) => t.id !== id);
        let active = state.activeTabId;
        if (active === id) {
          active = tabs[Math.min(index, tabs.length - 1)]?.id ?? null;
        }
        const tabStats = { ...state.tabStats };
        const saveStatus = { ...state.saveStatus };
        const cursors = { ...state.cursors };
        delete tabStats[id];
        delete saveStatus[id];
        delete cursors[id];
        set({ tabs, activeTabId: active, tabStats, saveStatus, cursors });
      },
      closeTabsUnder: (prefix) => {
        const p = normalizeRel(prefix);
        const state = get();
        const removed = state.tabs.filter((t) => t.path === p || t.path.startsWith(`${p}/`));
        const ids = new Set(removed.map((t) => t.id));
        if (!ids.size) return;
        const tabs = state.tabs.filter((t) => !ids.has(t.id));
        const active = ids.has(state.activeTabId ?? "") ? (tabs[tabs.length - 1]?.id ?? null) : state.activeTabId;
        const tabStats = { ...state.tabStats };
        const saveStatus = { ...state.saveStatus };
        const cursors = { ...state.cursors };
        ids.forEach((id) => {
          delete tabStats[id];
          delete saveStatus[id];
          delete cursors[id];
        });
        set({ tabs, activeTabId: active, tabStats, saveStatus, cursors });
      },
      setActiveTab: (id) => {
        const tab = get().tabs.find((t) => t.id === id);
        set({ activeTabId: id, selectedFile: tab?.path ?? null });
      },
      setTabDirty: (id, dirty) =>
        set({ tabs: get().tabs.map((t) => (t.id === id ? { ...t, dirty } : t)) }),
      setTabMode: (id, mode) =>
        set({ tabs: get().tabs.map((t) => (t.id === id ? { ...t, mode } : t)) }),
      setTabTitle: (path, title) =>
        set({
          tabs: get().tabs.map((t) => (t.path === path ? { ...t, title } : t)),
        }),
      remapPaths: (from, to) => {
        const a = normalizeRel(from);
        const b = normalizeRel(to);
        const affected = (path: string) => path === a || path.startsWith(`${a}/`);
        const remap = (path: string) => `${b}${path.slice(a.length)}`;
        set({
          tabs: get().tabs.map((t) => {
            if (affected(t.path)) {
              const next = remap(t.path);
              return { ...t, path: next, title: tabTitle(next) };
            }
            return t;
          }),
          selectedFile:
            get().selectedFile && affected(get().selectedFile!)
              ? remap(get().selectedFile!)
              : get().selectedFile,
        });
      },
      reorderTabs: (from, to) => {
        const tabs = [...get().tabs];
        if (from < 0 || from >= tabs.length || to < 0 || to >= tabs.length) return;
        const [m] = tabs.splice(from, 1);
        tabs.splice(to, 0, m);
        set({ tabs });
      },
      togglePanel: (k) => set({ panels: { ...get().panels, [k]: !get().panels[k] } }),
      setPanelWidth: (k, w) => {
        const clamped = Math.max(8, Math.min(48, w));
        set({ panels: { ...get().panels, [k]: clamped } });
      },
      setSettings: (s) => set({ settings: s }),
      setTheme: (t) => set({ theme: t }),
      setFocusMode: (b) => set({ focusMode: b }),
      toggleFocusMode: () => set({ focusMode: !get().focusMode }),
      setCommandPaletteOpen: (b) => set({ commandPaletteOpen: b }),
      setQuickSwitcherOpen: (b) => set({ quickSwitcherOpen: b }),
      setSearchQuery: (q) => set({ searchQuery: q }),
      setTabStats: (id, stats) => set({ tabStats: { ...get().tabStats, [id]: stats } }),
      setSaveStatus: (id, status) => set({ saveStatus: { ...get().saveStatus, [id]: status } }),
      setCursor: (id, cursor) => set({ cursors: { ...get().cursors, [id]: cursor } }),
      gotoLine: (path, line) =>
        set({
          gotoRequest: { path: normalizeRel(path), line, nonce: Date.now() },
        }),
    }),
    {
      name: "notecraft-ui",
      version: 3,
      migrate: (persisted, version) => {
        const state = (persisted ?? {}) as Partial<AppState> & { theme?: string; panels?: Partial<PanelState> };
        if (version < 3) {
          const left = state.panels?.leftWidth ?? 19;
          const right = state.panels?.rightWidth ?? 24;
          return {
            ...state,
            theme: state.theme === "light" ? "light" : "dark",
            panels: {
              leftSidebar: state.panels?.leftSidebar ?? true,
              rightSidebar: state.panels?.rightSidebar ?? true,
              leftWidth: left > 60 ? 19 : left,
              rightWidth: right > 60 ? 24 : right,
            },
          } as AppState;
        }
        return state as AppState;
      },
      partialize: (s) => ({
        panels: s.panels,
        theme: s.theme,
        recentNotes: s.recentNotes,
      }),
    }
  )
);
