import { useEffect, useState } from "react";
import { Panel as ResizablePanel, PanelGroup as ResizablePanelGroup, PanelResizeHandle as ResizableHandle } from "react-resizable-panels";
import GraphView from "./components/GraphView";
import CanvasView from "./components/CanvasView";
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
    applyTheme(theme, settings?.accent);
  }, [theme, settings?.accent]);

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
    <div className="h-full flex flex-col bg-bg text-fg">
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-bg-elev text-sm">
        <span className="font-semibold">NoteCraft</span>
        <span className="text-fg-muted text-xs">local-first knowledge base</span>
        <div className="flex-1" />
        <button className="text-xs px-2 py-1 rounded hover:bg-bg-muted" onClick={() => setSettingsOpen(true)}>Settings</button>
      </div>
      <div className="flex-1 min-h-0">
        <ResizablePanelGroup direction="horizontal" className="h-full">
          {panels.leftSidebar && (
            <>
              <ResizablePanel defaultSize={20} minSize={12} maxSize={40}>
                <Sidebar />
              </ResizablePanel>
              <ResizableHandle />
            </>
          )}
          <ResizablePanel defaultSize={panels.rightSidebar ? 55 : 80} minSize={30}>
            <div className="h-full flex flex-col min-h-0">
              <TabBar />
              <div className="flex-1 min-h-0">
                <EditorArea />
              </div>
              <StatusBar />
            </div>
          </ResizablePanel>
          {panels.rightSidebar && (
            <>
              <ResizableHandle />
              <ResizablePanel defaultSize={25} minSize={15} maxSize={45}>
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
