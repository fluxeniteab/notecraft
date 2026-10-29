import { useCallback, useEffect, useRef, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  Panel as ResizablePanel,
  PanelGroup as ResizablePanelGroup,
  PanelResizeHandle as ResizableHandle,
} from "react-resizable-panels";
import { toast, Toaster } from "sonner";
import { Minimize2 } from "lucide-react";
import { useAppStore } from "./store/appStore";
import { api } from "./lib/tauri";
import { applyTheme, isTauri, normalizeTheme } from "./lib/theme";
import { loadPlugin } from "./lib/pluginRuntime";
import { flushAll, flushTab } from "./lib/saveRegistry";
import { pickVault, refreshVault } from "./lib/vaultActions";
import { createNote } from "./lib/noteActions";
import { setThemeEverywhere } from "./lib/settingsActions";
import TitleBar from "./components/TitleBar";
import Sidebar from "./components/Sidebar";
import TabBar from "./components/TabBar";
import EditorArea from "./components/EditorArea";
import RightSidebar from "./components/RightSidebar";
import StatusBar from "./components/StatusBar";
import CommandPalette from "./components/CommandPalette";
import QuickSwitcher from "./components/QuickSwitcher";
import QuickCapture from "./components/QuickCapture";
import SettingsDialog from "./components/SettingsDialog";
import GraphView from "./components/GraphView";
import CanvasView from "./components/CanvasView";
import LockScreen from "./components/LockScreen";
import { TooltipProvider } from "./components/ui";
import type { ProfileSession } from "./lib/types";
import { checkForUpdates } from "./lib/updater";

export default function App() {
  const theme = useAppStore((s) => s.theme);
  const settings = useAppStore((s) => s.settings);
  const panels = useAppStore((s) => s.panels);
  const focusMode = useAppStore((s) => s.focusMode);
  const commandPaletteOpen = useAppStore((s) => s.commandPaletteOpen);
  const profile = useAppStore((s) => s.profile);
  const [booting, setBooting] = useState(true);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [captureOpen, setCaptureOpen] = useState(false);
  const [graphOpen, setGraphOpen] = useState(false);
  const [canvasOpen, setCanvasOpen] = useState(false);
  const layoutTimer = useRef<number | null>(null);
  const fsRefreshTimer = useRef<number | null>(null);
  const reloadExternalRef = useRef(new Map<string, () => void>());
  const registerExternalReload = useCallback((path: string, reload: (() => void) | null) => {
    if (reload) reloadExternalRef.current.set(path, reload);
    else reloadExternalRef.current.delete(path);
  }, []);

  // Apply theme and editor metrics whenever they change.
  useEffect(() => {
    applyTheme(theme, {
      accent: settings?.accent,
      reducedMotion: settings?.reduced_motion,
      editorFontSize: settings?.editor_font_size,
      editorLineHeight: settings?.editor_line_height,
      readingWidth: settings?.reading_width,
    });
  }, [theme, settings]);

  // Load settings, recents and the vault for the current session.
  const bootstrapSession = useCallback(async () => {
    try {
      const loaded = await api.getSettings();
      const normalized = normalizeTheme(loaded.theme);
      useAppStore.getState().setSettings({ ...loaded, theme: normalized });
      useAppStore.getState().setTheme(normalized);
      if (loaded.theme !== normalized) {
        void api.saveSettings({ ...loaded, theme: normalized }).catch(() => {});
      }
    } catch {
      /* Browser preview or first run: keep local defaults. */
    }
    try {
      const vaults = await api.listRecentVaults();
      useAppStore.getState().setRecentVaults(vaults);
    } catch {
      /* backend unavailable */
    }
    try {
      const root = await api.getVaultRoot();
      if (root) {
        useAppStore.getState().setVaultRoot(root);
        await refreshVault();
      }
    } catch {
      /* backend unavailable */
    }
  }, []);

  useEffect(() => {
    let alive = true;
    void (async () => {
      await bootstrapSession();
      if (!alive) return;
      try {
        const session = await api.currentProfile();
        useAppStore.getState().setProfile(session);
      } catch {
        /* browser preview: the lock screen still renders */
      }
      if (alive) setBooting(false);
    })();
    return () => {
      alive = false;
    };
  }, [bootstrapSession]);

  const handleUnlocked = useCallback(
    async (session: ProfileSession) => {
      useAppStore.getState().setProfile(session);
      await bootstrapSession();
      void refreshVault();
      void checkForUpdates({ silent: true });
    },
    [bootstrapSession]
  );

  // The webview's native context menu is never used: every menu in this app is
  // custom, so the default one only ever gets in the way.
  useEffect(() => {
    const onContextMenu = (event: MouseEvent) => event.preventDefault();
    window.addEventListener("contextmenu", onContextMenu);
    return () => window.removeEventListener("contextmenu", onContextMenu);
  }, []);

  // Start the plugins the user enabled, once, when the app boots.
  useEffect(() => {
    if (!isTauri()) return;
    void api
      .listPlugins()
      .then((list) => {
        list
          .filter((plugin) => plugin.enabled)
          .forEach((plugin) => void loadPlugin(plugin.id).catch(() => {}));
      })
      .catch(() => {});
  }, []);

  // Keep the file explorer and open-note conflict state in sync with external edits.
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlistenChange: UnlistenFn | undefined;
    let unlistenWrite: UnlistenFn | undefined;
    void Promise.all([
      api.onVaultFsChange((paths) => {
        if (fsRefreshTimer.current !== null) window.clearTimeout(fsRefreshTimer.current);
        fsRefreshTimer.current = window.setTimeout(() => {
          fsRefreshTimer.current = null;
          void refreshVault();
          paths.forEach((path) => reloadExternalRef.current.get(path)?.());
        }, 180);
      }),
      api.onVaultFileWritten((path) => {
        if (fsRefreshTimer.current !== null) window.clearTimeout(fsRefreshTimer.current);
        fsRefreshTimer.current = window.setTimeout(() => {
          fsRefreshTimer.current = null;
          void refreshVault();
          reloadExternalRef.current.get(path)?.();
        }, 180);
      }),
    ]).then(([change, write]) => {
      if (disposed) {
        change();
        write();
      } else {
        unlistenChange = change;
        unlistenWrite = write;
      }
    }).catch((error) => console.error("Could not watch vault changes", error));
    return () => {
      disposed = true;
      if (fsRefreshTimer.current !== null) window.clearTimeout(fsRefreshTimer.current);
      unlistenChange?.();
      unlistenWrite?.();
    };
  }, []);

  // Flush unsaved buffers when the native window is closed.
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        unlisten = await win.onCloseRequested(async (event) => {
          event.preventDefault();
          await flushAll();
          const { saveStatus } = useAppStore.getState();
          if (Object.values(saveStatus).some((status) => status === "error" || status === "dirty" || status === "saving")) {
            toast.error("Some notes are still unsaved, so the window stayed open.", {
              description: "Check the save indicator, copy anything important, then close again.",
            });
            return;
          }
          try {
            await win.destroy();
          } catch {
            await win.close();
          }
        });
        if (disposed) unlisten();
      } catch {
        /* not running in Tauri */
      }
    })();
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const handleLayout = useCallback((sizes: number[]) => {
    if (layoutTimer.current) window.clearTimeout(layoutTimer.current);
    layoutTimer.current = window.setTimeout(() => {
      const store = useAppStore.getState();
      const { leftSidebar, rightSidebar } = store.panels;
      if (leftSidebar) store.setPanelWidth("leftWidth", sizes[0]);
      if (rightSidebar) store.setPanelWidth("rightWidth", sizes[sizes.length - 1]);
    }, 160);
  }, []);

  // Global shortcuts. Modifier shortcuts work while writing; bare keys do not.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const store = useAppStore.getState();
      const mod = event.ctrlKey || event.metaKey;
      const key = event.key.toLowerCase();

      if (!mod) {
        if (event.key === "Escape" && store.focusMode) {
          store.setFocusMode(false);
        }
        return;
      }

      const inPalette = store.commandPaletteOpen || store.quickSwitcherOpen;

      if (key === "k") {
        event.preventDefault();
        store.setQuickSwitcherOpen(false);
        store.setCommandPaletteOpen(!store.commandPaletteOpen);
        return;
      }
      if (key === "p") {
        event.preventDefault();
        if (event.shiftKey) {
          store.setCommandPaletteOpen(true);
        } else {
          store.setCommandPaletteOpen(false);
          store.setQuickSwitcherOpen(!store.quickSwitcherOpen);
        }
        return;
      }
      if (inPalette) return;

      if (key === "s") {
        event.preventDefault();
        const activeId = store.activeTabId;
        if (activeId) void flushTab(activeId);
        return;
      }
      if (key === "n" && event.shiftKey) {
        event.preventDefault();
        if (store.vaultRoot) setCaptureOpen(true);
        return;
      }
      if (key === "n" && event.altKey) {
        event.preventDefault();
        if (store.vaultRoot) {
          void api.createDailyNote().then((path) => useAppStore.getState().openTab(path)).catch((error) => {
            toast.error("Could not open daily note", { description: String(error) });
          });
        }
        return;
      }
      if (key === "n") {
        event.preventDefault();
        if (store.vaultRoot) void createNote();
        return;
      }
      if (key === "w") {
        event.preventDefault();
        const activeId = store.activeTabId;
        if (activeId) {
          void flushTab(activeId).then(() => {
            if (useAppStore.getState().saveStatus[activeId] === "error") {
              toast.error("This note could not be saved, so it was kept open");
              return;
            }
            useAppStore.getState().closeTab(activeId);
          });
        }
        return;
      }
      if (key === "b") {
        event.preventDefault();
        store.togglePanel("leftSidebar");
        return;
      }
      if (key === "\\") {
        event.preventDefault();
        store.togglePanel("rightSidebar");
        return;
      }
      if (key === ",") {
        event.preventDefault();
        setSettingsOpen(true);
        return;
      }
      if (key === "o") {
        event.preventDefault();
        void pickVault("open");
        return;
      }
      if (key === "f" && event.shiftKey) {
        event.preventDefault();
        store.toggleFocusMode();
        return;
      }
      if (key === "d" && event.shiftKey) {
        event.preventDefault();
        void setThemeEverywhere(store.theme === "dark" ? "light" : "dark");
        return;
      }
      if (key === "l" && event.shiftKey) {
        // Lock the profile: flush every buffer, then drop the encryption key.
        event.preventDefault();
        void flushAll().then(async () => {
          await api.lockProfile().catch(() => {});
          useAppStore.getState().setProfile(null);
          useAppStore.getState().setVaultRoot(null);
          useAppStore.getState().setFiles([]);
          useAppStore.getState().setAllFiles([]);
        });
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  if (booting) {
    return (
      <div className="nc-boot">
        <div className="nc-boot-mark">N</div>
        <span>Opening NoteCraft…</span>
      </div>
    );
  }

  if (!profile) {
    return <LockScreen onUnlocked={handleUnlocked} />;
  }

  const main = (
    <div className="nc-main">
      {!focusMode && <TabBar />}
      <EditorArea onExternalReload={registerExternalReload} />
    </div>
  );

  return (
    <TooltipProvider delayDuration={380} skipDelayDuration={120}>
      <div className="nc-app">
        {!focusMode && <TitleBar onOpenSettings={() => setSettingsOpen(true)} />}

        {focusMode ? (
          <div className="relative min-h-0 flex-1">{main}</div>
        ) : (
          <ResizablePanelGroup direction="horizontal" className="min-h-0 flex-1" onLayout={handleLayout}>
            {panels.leftSidebar && (
              <>
                <ResizablePanel
                  id="left"
                  order={1}
                  defaultSize={panels.leftWidth}
                  minSize={12}
                  maxSize={40}
                >
                  <Sidebar />
                </ResizablePanel>
                <ResizableHandle className="nc-resize-handle" />
              </>
            )}
            <ResizablePanel id="main" order={2} minSize={30}>
              {main}
            </ResizablePanel>
            {panels.rightSidebar && (
              <>
                <ResizableHandle className="nc-resize-handle" />
                <ResizablePanel
                  id="right"
                  order={3}
                  defaultSize={panels.rightWidth}
                  minSize={14}
                  maxSize={42}
                >
                  <RightSidebar />
                </ResizablePanel>
              </>
            )}
          </ResizablePanelGroup>
        )}

        {!focusMode && <StatusBar />}

        {focusMode && (
          <button
            type="button"
            className="nc-focus-exit"
            onClick={() => useAppStore.getState().setFocusMode(false)}
          >
            <Minimize2 size={13} /> Exit focus mode <kbd className="nc-kbd">Esc</kbd>
          </button>
        )}

        <CommandPalette
          open={commandPaletteOpen}
          onClose={() => useAppStore.getState().setCommandPaletteOpen(false)}
          onOpenSettings={() => setSettingsOpen(true)}
          onOpenGraph={() => setGraphOpen(true)}
          onOpenCanvas={() => setCanvasOpen(true)}
          onOpenCapture={() => setCaptureOpen(true)}
        />
        <QuickSwitcher />
        <QuickCapture open={captureOpen} onClose={() => setCaptureOpen(false)} />
        <SettingsDialog open={settingsOpen} onClose={() => setSettingsOpen(false)} />
        <GraphView open={graphOpen} onClose={() => setGraphOpen(false)} />
        <CanvasView open={canvasOpen} onClose={() => setCanvasOpen(false)} />

        <Toaster
          theme={theme}
          position="bottom-right"
          offset={16}
          gap={8}
          toastOptions={{
            style: {
              background: "var(--nc-elev)",
              color: "var(--nc-fg)",
              border: "1px solid var(--nc-border)",
              borderRadius: "var(--nc-r-md)",
              boxShadow: "var(--nc-shadow-lg)",
              fontSize: "12px",
            },
          }}
        />
      </div>
    </TooltipProvider>
  );
}
