import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { FileEntry, Settings } from "../lib/types";

export interface Tab {
  id: string;
  path: string;
  title: string;
  dirty: boolean;
  pinned: boolean;
  cursor?: { line: number; col: number };
  scroll?: number;
  mode: "source" | "live" | "reading" | "split";
}

export interface PanelState {
  leftSidebar: boolean;
  rightSidebar: boolean;
  bottomPanel: boolean;
  leftWidth: number;
  rightWidth: number;
}

interface AppState {
  vaultRoot: string | null;
  recentVaults: string[];
  files: FileEntry[];
  selectedFile: string | null;
  tabs: Tab[];
  activeTabId: string | null;
  panels: PanelState;
  settings: Settings | null;
  theme: "system" | "light" | "dark";
  commandPaletteOpen: boolean;
  quickSwitcherOpen: boolean;
  searchQuery: string;
  // actions
  setVaultRoot: (p: string | null) => void;
  setRecentVaults: (v: string[]) => void;
  setFiles: (f: FileEntry[]) => void;
  setSelectedFile: (p: string | null) => void;
  openTab: (path: string, title?: string) => void;
  closeTab: (id: string) => void;
  setActiveTab: (id: string | null) => void;
  setTabDirty: (id: string, dirty: boolean) => void;
  setTabMode: (id: string, mode: Tab["mode"]) => void;
  reorderTabs: (from: number, to: number) => void;
  togglePanel: (k: keyof PanelState) => void;
  setPanelWidth: (k: "leftWidth" | "rightWidth", w: number) => void;
  setSettings: (s: Settings) => void;
  setTheme: (t: AppState["theme"]) => void;
  setCommandPaletteOpen: (b: boolean) => void;
  setQuickSwitcherOpen: (b: boolean) => void;
  setSearchQuery: (q: string) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      vaultRoot: null,
      recentVaults: [],
      files: [],
      selectedFile: null,
      tabs: [],
      activeTabId: null,
      panels: { leftSidebar: true, rightSidebar: true, bottomPanel: false, leftWidth: 260, rightWidth: 300 },
      settings: null,
      theme: "system",
      commandPaletteOpen: false,
      quickSwitcherOpen: false,
      searchQuery: "",
      setVaultRoot: (p) => set({ vaultRoot: p }),
      setRecentVaults: (v) => set({ recentVaults: v }),
      setFiles: (f) => set({ files: f }),
      setSelectedFile: (p) => set({ selectedFile: p }),
      openTab: (path, title) => {
        const existing = get().tabs.find((t) => t.path === path);
        if (existing) {
          set({ activeTabId: existing.id });
          return;
        }
        const id = crypto.randomUUID();
        const tab: Tab = { id, path, title: title ?? path.split("/").pop() ?? path, dirty: false, pinned: false, mode: "live" };
        set({ tabs: [...get().tabs, tab], activeTabId: id });
      },
      closeTab: (id) => {
        const tabs = get().tabs.filter((t) => t.id !== id);
        const active = get().activeTabId === id ? (tabs[tabs.length - 1]?.id ?? null) : get().activeTabId;
        set({ tabs, activeTabId: active });
      },
      setActiveTab: (id) => set({ activeTabId: id }),
      setTabDirty: (id, dirty) => set({ tabs: get().tabs.map((t) => (t.id === id ? { ...t, dirty } : t)) }),
      setTabMode: (id, mode) => set({ tabs: get().tabs.map((t) => (t.id === id ? { ...t, mode } : t)) }),
      reorderTabs: (from, to) => {
        const tabs = [...get().tabs];
        const [m] = tabs.splice(from, 1);
        tabs.splice(to, 0, m);
        set({ tabs });
      },
      togglePanel: (k) => set({ panels: { ...get().panels, [k]: !get().panels[k] } }),
      setPanelWidth: (k, w) => set({ panels: { ...get().panels, [k]: w } }),
      setSettings: (s) => set({ settings: s }),
      setTheme: (t) => set({ theme: t }),
      setCommandPaletteOpen: (b) => set({ commandPaletteOpen: b }),
      setQuickSwitcherOpen: (b) => set({ quickSwitcherOpen: b }),
      setSearchQuery: (q) => set({ searchQuery: q }),
    }),
    { name: "notecraft-ui", partialize: (s) => ({ panels: s.panels, theme: s.theme }) }
  )
);
