import { useEffect, useMemo, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Command } from "cmdk";
import { toast } from "sonner";
import {
  Command as CommandIcon,
  CalendarDays,
  FileText,
  FolderOpen,
  FolderPlus,
  Inbox,
  Loader2,
  LogOut,
  Maximize2,
  Moon,
  Network,
  PanelLeft,
  PanelRight,
  Plus,
  RefreshCw,
  Search,
  Settings,
  Sun,
  Workflow,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { dirName, stem } from "../lib/files";
import { createNote } from "../lib/noteActions";
import { closeCurrentVault, pickVault } from "../lib/vaultActions";
import { setThemeEverywhere } from "../lib/settingsActions";
import { Kbd } from "./ui";

interface PaletteCommand {
  id: string;
  label: string;
  hint?: string;
  icon: React.ReactNode;
  shortcut?: string;
  run: () => void | Promise<void>;
}

export default function CommandPalette({
  open,
  onClose,
  onOpenSettings,
  onOpenGraph,
  onOpenCanvas,
  onOpenCapture,
}: {
  open: boolean;
  onClose: () => void;
  onOpenSettings: () => void;
  onOpenGraph: () => void;
  onOpenCanvas: () => void;
  onOpenCapture: () => void;
}) {
  const [query, setQuery] = useState("");
  const [notes, setNotes] = useState<Array<{ path: string; title: string; snippet: string }>>([]);
  const [searching, setSearching] = useState(false);
  const requestRef = useRef(0);
  const recentNotes = useAppStore((s) => s.recentNotes);
  const allFiles = useAppStore((s) => s.allFiles);

  useEffect(() => {
    if (open) {
      setQuery("");
      setNotes([]);
      setSearching(false);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const text = query.trim();
    if (text.length < 2) {
      setNotes([]);
      setSearching(false);
      return;
    }
    const request = ++requestRef.current;
    setSearching(true);
    const handle = window.setTimeout(() => {
      api
        .search(text, {})
        .then((hits) => {
          if (requestRef.current !== request) return;
          setNotes(
            hits.slice(0, 8).map((hit) => ({
              path: hit.path,
              title: hit.title || stem(hit.path),
              snippet: hit.snippet,
            }))
          );
        })
        .catch(() => {
          if (requestRef.current === request) setNotes([]);
        })
        .finally(() => {
          if (requestRef.current === request) setSearching(false);
        });
    }, 200);
    return () => window.clearTimeout(handle);
  }, [query, open]);

  const commands = useMemo<PaletteCommand[]>(() => {
    const store = useAppStore.getState();
    return [
      {
        id: "new-note",
        label: "New note",
        hint: "Create a note at the vault root",
        icon: <Plus size={14} />,
        shortcut: "Ctrl N",
        run: () => void createNote(),
      },
      ...(store.vaultRoot
        ? [{
            id: "daily-note",
            label: "Open today’s daily note",
            hint: "Create it from your configured template if it does not exist",
            icon: <CalendarDays size={14} />,
            shortcut: "Ctrl Alt N",
            run: async () => {
              try {
                const path = await api.createDailyNote();
                await useAppStore.getState().openTab(path);
              } catch (error) {
                toast.error("Could not open daily note", { description: String(error) });
              }
            },
          } satisfies PaletteCommand]
        : []),
      {
        id: "quick-capture",
        label: "Quick capture…",
        hint: "Jot a note into your capture note without opening a tab",
        icon: <Inbox size={14} />,
        shortcut: "Ctrl ⇧ N",
        run: () => {
          if (useAppStore.getState().vaultRoot) onOpenCapture();
        },
      },
      {
        id: "quick-switch",
        label: "Go to note…",
        hint: "Quick switcher",
        icon: <Search size={14} />,
        shortcut: "Ctrl P",
        run: () => store.setQuickSwitcherOpen(true),
      },
      {
        id: "graph",
        label: "Open knowledge graph",
        icon: <Network size={14} />,
        run: onOpenGraph,
      },
      {
        id: "canvas",
        label: "Open canvas",
        icon: <Workflow size={14} />,
        run: onOpenCanvas,
      },
      {
        id: "toggle-explorer",
        label: "Toggle file explorer",
        icon: <PanelLeft size={14} />,
        shortcut: "Ctrl B",
        run: () => store.togglePanel("leftSidebar"),
      },
      {
        id: "toggle-context",
        label: "Toggle context panel",
        icon: <PanelRight size={14} />,
        shortcut: "Ctrl \\",
        run: () => store.togglePanel("rightSidebar"),
      },
      {
        id: "focus-mode",
        label: store.focusMode ? "Leave focus mode" : "Enter focus mode",
        icon: <Maximize2 size={14} />,
        shortcut: "Ctrl ⇧ F",
        run: () => store.toggleFocusMode(),
      },
      {
        id: "theme",
        label: store.theme === "dark" ? "Switch to light theme" : "Switch to dark theme",
        icon: store.theme === "dark" ? <Sun size={14} /> : <Moon size={14} />,
        shortcut: "Ctrl ⇧ D",
        run: () => void setThemeEverywhere(store.theme === "dark" ? "light" : "dark"),
      },
      {
        id: "settings",
        label: "Open settings",
        icon: <Settings size={14} />,
        shortcut: "Ctrl ,",
        run: onOpenSettings,
      },
      {
        id: "open-vault",
        label: "Open vault…",
        icon: <FolderOpen size={14} />,
        shortcut: "Ctrl O",
        run: () => void pickVault("open"),
      },
      {
        id: "create-vault",
        label: "Create vault…",
        icon: <FolderPlus size={14} />,
        run: () => void pickVault("create"),
      },
      ...(store.vaultRoot
        ? [
            {
              id: "rebuild-index",
              label: "Rebuild search index",
              hint: "Use after importing notes outside NoteCraft",
              icon: <RefreshCw size={14} />,
              run: async () => {
                try {
                  await api.rebuildIndex();
                  toast.success("Search index rebuilt");
                } catch (error) {
                  toast.error("Could not rebuild the index", { description: String(error) });
                }
              },
            } satisfies PaletteCommand,
            {
              id: "close-vault",
              label: "Close vault",
              icon: <LogOut size={14} />,
              run: () => void closeCurrentVault(),
            } satisfies PaletteCommand,
          ]
        : []),
    ];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const text = query.trim().toLowerCase();
  const visibleCommands = text
    ? commands.filter((command) => command.label.toLowerCase().includes(text))
    : commands;
  const recent = useMemo(() => {
    if (text) return [];
    return recentNotes
      .map((path) => allFiles.find((file) => file.path === path))
      .filter((file): file is NonNullable<typeof file> => !!file)
      .slice(0, 5);
  }, [allFiles, recentNotes, text]);

  const select = (action: () => void | Promise<void>) => {
    onClose();
    void action();
  };

  return (
    <Dialog.Root open={open} onOpenChange={(next) => !next && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="nc-overlay" />
        <Dialog.Content
          className="nc-palette-wrap"
          aria-describedby={undefined}
          onClick={(event) => {
            if (event.target === event.currentTarget) onClose();
          }}
        >
          <Dialog.Title className="sr-only">Command palette</Dialog.Title>
          <Command className="nc-palette" label="Command palette" shouldFilter={false} loop>
            <div className="nc-palette-input-row">
              <Search size={16} />
              <Command.Input
                autoFocus
                value={query}
                onValueChange={setQuery}
                placeholder="Search notes or run a command…"
                className="nc-palette-input"
              />
              {searching && <Loader2 size={13} className="animate-spin text-fg-faint" />}
              <Kbd>Esc</Kbd>
            </div>

            <Command.List className="nc-palette-list nc-scroll">
              <Command.Empty>
                <div className="px-3 py-8 text-center text-[11.5px] text-fg-muted">
                  No commands match “{query}”.
                </div>
              </Command.Empty>

              {notes.length > 0 && (
                <Command.Group heading="Notes">
                  {notes.map((note) => (
                    <Command.Item
                      key={`note-${note.path}`}
                      value={`note-${note.path}`}
                      className="nc-palette-item"
                      onSelect={() => select(() => useAppStore.getState().openTab(note.path))}
                    >
                      <FileIcon />
                      <span className="nc-palette-item-body">
                        <span className="nc-palette-item-title">{note.title}</span>
                        <span className="nc-palette-item-sub">{dirName(note.path) || "Vault root"}</span>
                      </span>
                    </Command.Item>
                  ))}
                </Command.Group>
              )}

              {recent.length > 0 && (
                <Command.Group heading="Recent notes">
                  {recent.map((file) => (
                    <Command.Item
                      key={`recent-${file.path}`}
                      value={`recent-${file.path}`}
                      className="nc-palette-item"
                      onSelect={() => select(() => useAppStore.getState().openTab(file.path))}
                    >
                      <FileIcon />
                      <span className="nc-palette-item-body">
                        <span className="nc-palette-item-title">{stem(file.path)}</span>
                        <span className="nc-palette-item-sub">{dirName(file.path) || "Vault root"}</span>
                      </span>
                    </Command.Item>
                  ))}
                </Command.Group>
              )}

              {visibleCommands.length > 0 && (
                <Command.Group heading="Commands">
                  {visibleCommands.map((command) => (
                    <Command.Item
                      key={command.id}
                      value={command.id}
                      className="nc-palette-item"
                      onSelect={() => select(command.run)}
                    >
                      <span className="nc-palette-icon">{command.icon}</span>
                      <span className="nc-palette-item-body">
                        <span className="nc-palette-item-title">{command.label}</span>
                        {command.hint && <span className="nc-palette-item-sub">{command.hint}</span>}
                      </span>
                      {command.shortcut && <Kbd>{command.shortcut}</Kbd>}
                    </Command.Item>
                  ))}
                </Command.Group>
              )}
            </Command.List>

            <div className="nc-palette-foot">
              <span><Kbd>↑</Kbd><Kbd>↓</Kbd> navigate</span>
              <span><Kbd>↵</Kbd> select</span>
              <span><Kbd>Esc</Kbd> close</span>
              <div className="flex-1" />
              <span className="inline-flex items-center gap-1">
                <CommandIcon size={10} /> NoteCraft
              </span>
            </div>
          </Command>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function FileIcon() {
  return <FileText size={13} className="nc-palette-icon" />;
}
