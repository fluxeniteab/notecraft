import { useEffect, useState } from "react";
import { Panel as ResizablePanel, PanelGroup as ResizablePanelGroup, PanelResizeHandle as ResizableHandle } from "react-resizable-panels";
import { useAppStore } from "./store/appStore";
import { api } from "./lib/tauri";
import { applyTheme } from "./lib/theme";
import Sidebar from "./components/Sidebar";
import TabBar from "./components/TabBar";
import EditorArea from "./components/EditorArea";
import RightSidebar from "./components/RightSidebar";
import StatusBar from "./components/StatusBar";
import CommandPalette from "./components/CommandPalette";
import QuickSwitcher from "./components/QuickSwitcher";
import SettingsDialog from "./components/SettingsDialog";
import GraphView from "./components/GraphView";
import CanvasView from "./components/CanvasView";

export default function App() {
  const { panels, setVaultRoot, setRecentVaults, setSettings, theme, settings } = useAppStore();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [graphOpen, setGraphOpen] = useState(false);
  const [canvasOpen, setCanvasOpen] = useState(false);

  useEffect(() => {
    (window as any).__nc_open_graph = () => setGraphOpen(true);
    (window as any).__nc_open_canvas = () => setCanvasOpen(true);
    return () => { delete (window as any).__nc_open_graph; delete (window as any).__nc_open_canvas; };
  }, []);

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      if (s.theme) useAppStore.getState().setTheme(s.theme as "system" | "light" | "dark");
    }).catch(() => {});
    api.listRecentVaults().then(setRecentVaults).catch(() => {});
    api.getVaultRoot().then((r) => setVaultRoot(r)).catch(() => {});
  }, []);

  useEffect(() => {
    applyTheme(theme, settings?.accent, settings?.reduced_motion, settings?.high_contrast);
  }, [theme, settings?.accent, settings?.reduced_motion, settings?.high_contrast]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "p") {
        e.preventDefault();
        useAppStore.getState().setCommandPaletteOpen(true);
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
        e.preventDefault();
        useAppStore.getState().setQuickSwitcherOpen(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="h-full flex flex-col bg-bg text-fg overflow-hidden">
      {/* Title bar */}
      <div className="flex items-center gap-3 px-4 py-2 border-b border-border bg-bg-elev/80 backdrop-blur-sm select-none" style={{ WebkitAppRegion: "drag" } as any}>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-accent shadow-[0_0_12px_var(--nc-accent)]" />
          <span className="font-semibold text-sm tracking-tight">NoteCraft</span>
        </div>
        <span className="text-fg-muted text-xs hidden md:inline">local-first knowledge base</span>
        <div className="flex-1" />
        <button className="text-xs px-3 py-1 rounded-md hover:bg-bg-muted transition-colors nc-btn" style={{ WebkitAppRegion: "no-drag" } as any} onClick={() => setSettingsOpen(true)}>Settings</button>
      </div>
      {/* Main workspace */}
      <div className="flex-1 min-h-0">
        <ResizablePanelGroup direction="horizontal" className="h-full">
          {panels.leftSidebar && (
            <>
              <ResizablePanel defaultSize={20} minSize={12} maxSize={40} className="transition-all duration-200">
                <Sidebar />
              </ResizablePanel>
              <ResizableHandle className="w-1 bg-border hover:bg-accent transition-colors" />
            </>
          )}
          <ResizablePanel defaultSize={panels.rightSidebar ? 55 : 80} minSize={30}>
            <div className="h-full flex flex-col min-h-0 bg-bg">
              <TabBar />
              <div className="flex-1 min-h-0 relative">
                <EditorArea />
              </div>
              <StatusBar />
            </div>
          </ResizablePanel>
          {panels.rightSidebar && (
            <>
              <ResizableHandle className="w-1 bg-border hover:bg-accent transition-colors" />
              <ResizablePanel defaultSize={25} minSize={15} maxSize={45} className="transition-all duration-200">
                <RightSidebar />
              </ResizablePanel>
            </>
          )}
        </ResizablePanelGroup>
      </div>
      <CommandPalette open={useAppStore((s) => s.commandPaletteOpen)} onClose={() => useAppStore.getState().setCommandPaletteOpen(false)} onOpenSettings={() => setSettingsOpen(true)} />
      <QuickSwitcher />
      <SettingsDialog open={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <GraphView open={graphOpen} onClose={() => setGraphOpen(false)} />
      <CanvasView open={canvasOpen} onClose={() => setCanvasOpen(false)} />
    </div>
  );
}
