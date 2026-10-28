import { useEffect, useState } from "react";
import {
  ChevronDown,
  FolderOpen,
  FolderPlus,
  LogOut,
  Minus,
  Moon,
  NotebookPen,
  PanelLeft,
  PanelRight,
  Search,
  Settings,
  Square,
  Sun,
  X,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { baseName } from "../lib/files";
import { isTauri } from "../lib/theme";
import { openVaultAt, pickVault, closeCurrentVault } from "../lib/vaultActions";
import { setThemeEverywhere } from "../lib/settingsActions";
import { ContextMenu, IconButton, Kbd, type MenuEntry } from "./ui";

export default function TitleBar({ onOpenSettings }: { onOpenSettings: () => void }) {
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const recentVaults = useAppStore((s) => s.recentVaults);
  const panels = useAppStore((s) => s.panels);
  const theme = useAppStore((s) => s.theme);
  const togglePanel = useAppStore((s) => s.togglePanel);
  const setCommandPaletteOpen = useAppStore((s) => s.setCommandPaletteOpen);
  const [maximized, setMaximized] = useState(false);
  const [vaultMenu, setVaultMenu] = useState<{ x: number; y: number } | null>(null);
  const tauri = isTauri();

  useEffect(() => {
    if (!tauri) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        const state = await win.isMaximized();
        if (!disposed) setMaximized(state);
        unlisten = await win.onResized(async () => {
          try {
            const next = await win.isMaximized();
            if (!disposed) setMaximized(next);
          } catch {
            /* ignore */
          }
        });
      } catch {
        /* running outside Tauri */
      }
    })();
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [tauri]);

  const windowAction = async (action: "minimize" | "maximize" | "close") => {
    if (!tauri) return;
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      if (action === "minimize") await win.minimize();
      else if (action === "maximize") await win.toggleMaximize();
      else await win.close();
    } catch (error) {
      console.error(error);
    }
  };

  const vaultItems: MenuEntry[] = [
    ...(recentVaults.length
      ? [
          { header: "Recent vaults" } as const,
          ...recentVaults.slice(0, 6).map((path) => ({
            label: baseName(path),
            icon: <FolderOpen size={13} />,
            onSelect: () => void openVaultAt(path),
          })),
          { separator: true } as const,
        ]
      : []),
    { label: "Open vault…", icon: <FolderOpen size={13} />, onSelect: () => void pickVault("open") },
    { label: "Create vault…", icon: <FolderPlus size={13} />, onSelect: () => void pickVault("create") },
    ...(vaultRoot
      ? [
          { separator: true } as const,
          {
            label: "Close vault",
            icon: <LogOut size={13} />,
            danger: true,
            onSelect: () => void closeCurrentVault(),
          },
        ]
      : []),
  ];

  return (
    <header className="nc-titlebar" data-tauri-drag-region>
      <div className="nc-brand" data-tauri-drag-region>
        <div className="nc-brand-mark">
          <NotebookPen size={13} strokeWidth={2.2} />
        </div>
        <span className="nc-brand-name">NoteCraft</span>
      </div>

      <button
        type="button"
        className="nc-vault-chip"
        onClick={(event) => {
          const rect = event.currentTarget.getBoundingClientRect();
          setVaultMenu({ x: rect.left, y: rect.bottom + 6 });
        }}
        title={vaultRoot ?? "Open a vault"}
      >
        <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-accent" />
        <span className="truncate">{vaultRoot ? baseName(vaultRoot) : "Open a vault"}</span>
        <ChevronDown size={11} className="shrink-0 opacity-60" />
      </button>

      <div className="flex-1" data-tauri-drag-region />

      <button
        type="button"
        className="nc-search-pill"
        onClick={() => setCommandPaletteOpen(true)}
        aria-label="Search notes and commands"
      >
        <Search size={13} />
        <span>Search notes and commands…</span>
        <Kbd>Ctrl K</Kbd>
      </button>

      <div className="flex-1" data-tauri-drag-region />

      <div className="flex items-center gap-0.5">
        <IconButton
          label="Toggle file explorer"
          shortcut="Ctrl B"
          active={panels.leftSidebar}
          onClick={() => togglePanel("leftSidebar")}
        >
          <PanelLeft size={15} />
        </IconButton>
        <IconButton
          label="Toggle context panel"
          shortcut="Ctrl \\"
          active={panels.rightSidebar}
          onClick={() => togglePanel("rightSidebar")}
        >
          <PanelRight size={15} />
        </IconButton>
        <IconButton
          label={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
          onClick={() => void setThemeEverywhere(theme === "dark" ? "light" : "dark")}
        >
          {theme === "dark" ? <Sun size={15} /> : <Moon size={15} />}
        </IconButton>
        <IconButton label="Settings" shortcut="Ctrl ," onClick={onOpenSettings}>
          <Settings size={15} />
        </IconButton>
      </div>

      {tauri && (
        <div className="nc-window-controls">
          <button type="button" className="nc-window-btn" aria-label="Minimize" onClick={() => void windowAction("minimize")}>
            <Minus size={14} />
          </button>
          <button
            type="button"
            className="nc-window-btn"
            aria-label={maximized ? "Restore" : "Maximize"}
            onClick={() => void windowAction("maximize")}
          >
            <Square size={maximized ? 11 : 12} />
          </button>
          <button type="button" className="nc-window-btn nc-close" aria-label="Close" onClick={() => void windowAction("close")}>
            <X size={14} />
          </button>
        </div>
      )}

      {vaultMenu && (
        <ContextMenu x={vaultMenu.x} y={vaultMenu.y} items={vaultItems} onClose={() => setVaultMenu(null)} />
      )}
    </header>
  );
}
