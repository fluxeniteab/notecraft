import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import {
  ChevronRight,
  Clock,
  Copy,
  FilePlus2,
  FileText,
  Folder,
  FolderOpen,
  FolderPlus,
  MoreHorizontal,
  Pencil,
  Plus,
  RefreshCw,
  Search,
  Star,
  Trash2,
  X,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { FileEntry } from "../lib/types";
import {
  baseName,
  dirName,
  ensureMarkdown,
  highlightParts,
  isDocument,
  isMarkdown,
  joinRel,
  stem,
  uniqueName,
} from "../lib/files";
import { refreshVault, pickVault } from "../lib/vaultActions";
import { flushTab } from "../lib/saveRegistry";
import { isFavorite, toggleFavorite, useFavorites } from "../lib/favorites";
import {
  ConfirmDialog,
  ContextMenu,
  IconButton,
  PromptDialog,
  Spinner,
  type MenuEntry,
} from "./ui";
import { cn } from "../lib/cn";

type PromptState =
  | { mode: "create-file"; dir: string }
  | { mode: "create-folder"; dir: string }
  | { mode: "rename"; entry: FileEntry };

interface MenuState {
  x: number;
  y: number;
  entry: FileEntry;
}

export default function Sidebar() {
  const profile = useAppStore((s) => s.profile);
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const settings = useAppStore((s) => s.settings);
  const files = useAppStore((s) => s.files);
  const allFiles = useAppStore((s) => s.allFiles);
  const tabs = useAppStore((s) => s.tabs);
  const activeTabId = useAppStore((s) => s.activeTabId);
  const recentNotes = useAppStore((s) => s.recentNotes);
  const favorites = useFavorites();
  const openTab = useAppStore((s) => s.openTab);
  const closeTab = useAppStore((s) => s.closeTab);
  const closeTabsUnder = useAppStore((s) => s.closeTabsUnder);
  const remapPaths = useAppStore((s) => s.remapPaths);
  const [query, setQuery] = useState("");
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [children, setChildren] = useState<Record<string, FileEntry[]>>({});
  const [loadingDirs, setLoadingDirs] = useState<Set<string>>(new Set());
  const [prompt, setPrompt] = useState<PromptState | null>(null);
  const [confirm, setConfirm] = useState<FileEntry | null>(null);
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [dropTarget, setDropTarget] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [pendingLinks, setPendingLinks] = useState<{
    from: string;
    to: string;
    impacts: Array<{ path: string; replacements: number }>;
  } | null>(null);

  const activePath = tabs.find((t) => t.id === activeTabId)?.path ?? null;

  useEffect(() => {
    setExpanded(new Set());
    setChildren({});
    if (vaultRoot) void refreshVault();
  }, [vaultRoot]);

  const refreshAll = useCallback(async () => {
    setRefreshing(true);
    try {
      await refreshVault();
      const dirs = [...expanded];
      const next: Record<string, FileEntry[]> = {};
      await Promise.all(
        dirs.map(async (dir) => {
          try {
            next[dir] = await api.listDir(dir);
          } catch {
            next[dir] = [];
          }
        })
      );
      setChildren(next);
    } catch (error) {
      toast.error("Could not refresh the vault", { description: String(error) });
    } finally {
      setRefreshing(false);
    }
  }, [expanded]);

  const toggleDir = useCallback(
    async (entry: FileEntry) => {
      setExpanded((current) => {
        const next = new Set(current);
        if (next.has(entry.path)) next.delete(entry.path);
        else next.add(entry.path);
        return next;
      });
      if (children[entry.path] || expanded.has(entry.path)) return;
      setLoadingDirs((current) => new Set(current).add(entry.path));
      try {
        const kids = await api.listDir(entry.path);
        setChildren((current) => ({ ...current, [entry.path]: kids }));
      } catch (error) {
        toast.error("Could not open folder", { description: String(error) });
      } finally {
        setLoadingDirs((current) => {
          const next = new Set(current);
          next.delete(entry.path);
          return next;
        });
      }
    },
    [children, expanded]
  );

  const closeTabsFor = useCallback(
    async (entry: FileEntry) => {
      const affected = entry.is_dir
        ? tabs.filter((t) => t.path === entry.path || t.path.startsWith(`${entry.path}/`))
        : tabs.filter((t) => t.path === entry.path);
      await Promise.all(affected.map((t) => flushTab(t.id)));
      return affected;
    },
    [tabs]
  );

  const runCreate = async (dir: string, value: string) => {
    if (!prompt) return;
    if (prompt.mode === "create-folder") {
      const rel = joinRel(dir, value.trim());
      if (!rel) throw new Error("Enter a folder name");
      await api.createDir(rel);
      setExpanded((current) => new Set(current).add(rel));
      await refreshAll();
      return;
    }
    if (prompt.mode === "create-file") {
      // An explicit extension is honoured: `page.mdx` stays `.mdx`, and the
      // template matches the format instead of being Markdown for everything.
      const rel = joinRel(dir, uniqueName(value, allFiles, dir));
      if (!rel) throw new Error("Enter a file name");
      await api.createDocument(rel);
      if (dir) {
        setExpanded((current) => new Set(current).add(dir));
        const kids = await api.listDir(dir).catch(() => []);
        setChildren((current) => ({ ...current, [dir]: kids }));
      }
      await refreshVault();
      openTab(rel);
      return;
    }
    // rename
    const entry = prompt.entry;
    const parent = dirName(entry.path);
    // Only Markdown notes get the .md default; renaming a .mdx or .json file
    // must not append a second extension.
    const nextName = entry.is_dir
      ? value.trim().replace(/[\\/]+/g, "-")
      : isMarkdown(entry.path)
        ? ensureMarkdown(value)
        : value.trim().replace(/[\\/]+/g, "-");
    const to = joinRel(parent, nextName);
    if (!to || to === entry.path) return;
    await closeTabsFor(entry);
    await api.renamePath(entry.path, to);
    remapPaths(entry.path, to);
    await refreshAll();
    toast.success("Renamed", { description: to });
    if (!entry.is_dir && isMarkdown(to)) {
      const impacts = await api.propagateLinkRename(entry.path, to, false).catch(() => []);
      if (impacts.length) setPendingLinks({ from: entry.path, to, impacts });
    }
  };

  const runDelete = async () => {
    if (!confirm) return;
    const entry = confirm;
    const affected = await closeTabsFor(entry);
    await api.deletePath(entry.path, true);
    if (entry.is_dir) closeTabsUnder(entry.path);
    else affected.forEach((tab) => closeTab(tab.id));
    await refreshAll();
    toast.success("Moved to trash", { description: baseName(entry.path) });
  };

  const duplicate = async (entry: FileEntry) => {
    try {
      const content = await api.readFile(entry.path);
      const ext = entry.path.slice(entry.path.lastIndexOf("."));
      const name = uniqueName(stem(entry.path) + ext, allFiles, dirName(entry.path));
      const rel = joinRel(dirName(entry.path), name);
      await api.createFile(rel, content);
      await refreshVault();
      openTab(rel);
      toast.success("Note duplicated");
    } catch (error) {
      toast.error("Could not duplicate note", { description: String(error) });
    }
  };

  const moveEntry = async (entry: FileEntry, targetDir: string) => {
    if (entry.is_dir && (targetDir === entry.path || targetDir.startsWith(`${entry.path}/`))) return;
    if (dirName(entry.path) === targetDir) return;
    try {
      const affected = entry.is_dir
        ? tabs.filter((t) => t.path === entry.path || t.path.startsWith(`${entry.path}/`))
        : tabs.filter((t) => t.path === entry.path);
      await Promise.all(affected.map((t) => flushTab(t.id)));
      const dest = await api.movePath(entry.path, targetDir);
      remapPaths(entry.path, dest);
      setExpanded((current) => new Set(current).add(targetDir));
      await refreshAll();
      toast.success("Moved", { description: dest });
    } catch (error) {
      toast.error("Could not move item", { description: String(error) });
    }
  };

  const menuItems = (entry: FileEntry): MenuEntry[] => {
    if (entry.is_dir) {
      return [
        { label: "New note here", icon: <FilePlus2 size={13} />, onSelect: () => setPrompt({ mode: "create-file", dir: entry.path }) },
        { label: "New folder here", icon: <FolderPlus size={13} />, onSelect: () => setPrompt({ mode: "create-folder", dir: entry.path }) },
        { separator: true },
        { label: "Rename", icon: <Pencil size={13} />, onSelect: () => setPrompt({ mode: "rename", entry }) },
        { label: "Delete", icon: <Trash2 size={13} />, danger: true, onSelect: () => setConfirm(entry) },
      ];
    }
    return [
      { label: "Open", icon: <FileText size={13} />, onSelect: () => openTab(entry.path) },
      {
        label: isFavorite(entry.path) ? "Unpin from favorites" : "Pin to favorites",
        icon: <Star size={13} />,
        onSelect: () => {
          toggleFavorite(entry.path);
          toast.success(
            isFavorite(entry.path) ? "Pinned to favorites" : "Removed from favorites",
            { description: entry.name }
          );
        },
      },
      { label: "Duplicate", icon: <Copy size={13} />, onSelect: () => void duplicate(entry) },
      { label: "Rename", icon: <Pencil size={13} />, onSelect: () => setPrompt({ mode: "rename", entry }) },
      { separator: true },
      { label: "Delete", icon: <Trash2 size={13} />, danger: true, onSelect: () => setConfirm(entry) },
    ];
  };

  // Explorer ordering is configurable: name/modified/size, folders first or mixed.
  const sortEntries = useCallback(
    (entries: FileEntry[]): FileEntry[] => {
      const mode = settings?.explorer_sort ?? "name";
      const groupDirs = settings?.explorer_group_dirs ?? true;
      return [...entries].sort((a, b) => {
        if (groupDirs && a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        if (mode === "modified") return b.modified - a.modified;
        if (mode === "size") return b.size - a.size;
        return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
      });
    },
    [settings]
  );

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return allFiles
      .filter((entry) => isDocument(entry.path) && (entry.path.toLowerCase().includes(q) || entry.name.toLowerCase().includes(q)))
      .slice(0, 80);
  }, [allFiles, query]);

  const noteCount = allFiles.filter((f) => isDocument(f.path)).length;

  const pinnedEntries = favorites
    .map((path) => allFiles.find((file) => file.path === path))
    .filter((entry): entry is FileEntry => !!entry && !entry.is_dir)
    .slice(0, 8);
  const recentEntries = recentNotes
    .map((path) => allFiles.find((file) => file.path === path))
    .filter(
      (entry): entry is FileEntry =>
        !!entry && !entry.is_dir && !favorites.includes(entry.path)
    )
    .slice(0, 5);

  const renderNode = (entry: FileEntry, depth: number): React.ReactNode => {
    const isDir = entry.is_dir;
    const isOpen = expanded.has(entry.path);
    const isActive = !isDir && activePath === entry.path;
    const isLoading = loadingDirs.has(entry.path);
    return (
      <div key={entry.path}>
        <div
          role="button"
          tabIndex={0}
          className="nc-tree-row group"
          data-active={isActive || undefined}
          data-expanded={isOpen || undefined}
          data-drop={dropTarget === entry.path || undefined}
          style={{ paddingLeft: 6 + depth * 13 }}
          onClick={() => (isDir ? void toggleDir(entry) : openTab(entry.path))}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              if (isDir) void toggleDir(entry);
              else openTab(entry.path);
            }
          }}
          onContextMenu={(event) => {
            event.preventDefault();
            setMenu({ x: event.clientX, y: event.clientY, entry });
          }}
          draggable={!isDir}
          onDragStart={(event) => {
            event.dataTransfer.setData("text/notecraft-path", entry.path);
            event.dataTransfer.effectAllowed = "move";
          }}
          onDragOver={(event) => {
            if (!isDir) return;
            event.preventDefault();
            event.dataTransfer.dropEffect = "move";
            if (dropTarget !== entry.path) setDropTarget(entry.path);
          }}
          onDragLeave={() => setDropTarget((current) => (current === entry.path ? null : current))}
          onDrop={(event) => {
            if (!isDir) return;
            event.preventDefault();
            setDropTarget(null);
            const from = event.dataTransfer.getData("text/notecraft-path");
            const source = allFiles.find((f) => f.path === from) ?? files.find((f) => f.path === from);
            if (source) void moveEntry(source, entry.path);
          }}
          onDoubleClick={() => {
            if (!isDir) return;
            setExpanded((current) => new Set(current).add(entry.path));
          }}
        >
          {isDir ? (
            <ChevronRight size={11} className="nc-tree-chevron shrink-0" />
          ) : (
            <span className="w-[11px] shrink-0" />
          )}
          {isDir ? (
            isOpen ? (
              <FolderOpen size={14} className="shrink-0 text-accent/80" />
            ) : (
              <Folder size={14} className="shrink-0 text-fg-muted" />
            )
          ) : (
            <FileText
              size={13}
              className={cn("shrink-0", isActive ? "text-accent" : isDocument(entry.path) ? "text-fg-muted" : "text-fg-faint")}
            />
          )}
          <span className="nc-tree-label">{entry.name}</span>
          {isLoading && <Spinner size={11} />}
          <span className="nc-tree-actions">
            <button
              type="button"
              aria-label={`Actions for ${entry.name}`}
              onClick={(event) => {
                event.stopPropagation();
                const rect = event.currentTarget.getBoundingClientRect();
                setMenu({ x: rect.left, y: rect.bottom + 4, entry });
              }}
            >
              <MoreHorizontal size={13} />
            </button>
          </span>
        </div>
        {isDir && isOpen && (
          <div className="nc-tree-children">
            {sortEntries(children[entry.path] ?? []).map((child) => renderNode(child, depth + 1))}
            {!children[entry.path]?.length && !isLoading && (
              <div className="px-2 py-1 text-[10.5px] text-fg-faint" style={{ paddingLeft: 26 + depth * 13 }}>
                Empty folder
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  if (!vaultRoot) {
    return (
      <aside className="nc-sidebar">
        <div className="nc-sidebar-header">
          <span className="nc-section-label">Explorer</span>
        </div>
        <div className="flex flex-1 flex-col items-center justify-center gap-4 px-6 text-center">
          <div className="grid h-12 w-12 place-items-center rounded-2xl border border-border bg-elev text-accent shadow-sm">
            <FolderOpen size={20} />
          </div>
          <div>
            <div className="font-title text-[18px] font-semibold text-fg">No vault open</div>
            <p className="mt-1 text-[11.5px] leading-relaxed text-fg-muted">
              A vault is just a folder of Markdown files. Open an existing one or start fresh.
            </p>
          </div>
          <div className="flex w-full flex-col gap-2">
            <button type="button" className="nc-btn nc-btn-primary w-full" onClick={() => void pickVault("open")}>
              <FolderOpen size={13} /> Open vault
            </button>
            <button type="button" className="nc-btn nc-btn-outline w-full" onClick={() => void pickVault("create")}>
              <FolderPlus size={13} /> Create vault
            </button>
          </div>
        </div>
        <div className="nc-sidebar-footer">
          <span>Local-first · nothing leaves your machine</span>
        </div>
      </aside>
    );
  }

  return (
    <aside className="nc-sidebar">
      <div className="nc-workspace">
        <span className="nc-workspace-avatar">
          {(profile?.username ?? baseName(vaultRoot)).slice(0, 1).toUpperCase()}
        </span>
        <div className="min-w-0 flex-1">
          <div className="nc-workspace-name">{profile?.username ?? baseName(vaultRoot)}</div>
          <div className="nc-workspace-sub">
            {noteCount} {noteCount === 1 ? "note" : "notes"}
          </div>
        </div>
      </div>

      <div className="nc-sidebar-header">
        <span className="nc-section-label">Notes</span>
        {refreshing && <Spinner size={11} />}
        <div className="flex-1" />
        <IconButton label="New note" shortcut="Ctrl N" small onClick={() => setPrompt({ mode: "create-file", dir: "" })}>
          <Plus size={14} />
        </IconButton>
        <IconButton label="New folder" small onClick={() => setPrompt({ mode: "create-folder", dir: "" })}>
          <FolderPlus size={14} />
        </IconButton>
        <IconButton label="Refresh" small onClick={() => void refreshAll()}>
          <RefreshCw size={13} />
        </IconButton>
      </div>

      <div className="px-2.5 pb-2.5">
        <div className="nc-field">
          <Search size={12} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") setQuery("");
              if (event.key === "Enter" && filtered[0]) {
                openTab(filtered[0].path);
                setQuery("");
              }
            }}
            placeholder="Filter notes…"
            aria-label="Filter notes"
          />
          {query && (
            <button type="button" aria-label="Clear filter" className="text-fg-faint hover:text-fg" onClick={() => setQuery("")}>
              <X size={11} />
            </button>
          )}
        </div>
      </div>

      <div className="nc-scroll flex-1 overflow-auto">
        {query ? (
          <div className="px-1.5 pb-4">
            <div className="px-2 py-1.5 text-[10px] text-fg-faint">
              {filtered.length} {filtered.length === 1 ? "note" : "notes"}
            </div>
            {filtered.map((entry) => (
              <button
                key={entry.path}
                type="button"
                className="nc-tree-row"
                data-active={activePath === entry.path || undefined}
                style={{ paddingLeft: 8 }}
                onClick={() => openTab(entry.path)}
              >
                <FileText size={13} className="shrink-0 text-fg-muted" />
                <span className="nc-tree-label">
                  {highlightParts(entry.name, query).map((part, index) =>
                    part.match ? <mark key={index} className="nc-tree-mark">{part.text}</mark> : <span key={index}>{part.text}</span>
                  )}
                </span>
                <span className="max-w-[45%] truncate text-[9.5px] text-fg-faint">{dirName(entry.path)}</span>
              </button>
            ))}
            {filtered.length === 0 && (
              <div className="nc-panel-empty m-1">No notes match “{query}”.</div>
            )}
          </div>
        ) : (
          <div className="nc-tree">
            {(pinnedEntries.length > 0 || recentEntries.length > 0) && (
              <div className="nc-sidebar-pinned">
                {pinnedEntries.length > 0 && (
                  <>
                    <div className="nc-sidebar-group-label">
                      <Star size={10} /> Pinned
                    </div>
                    {pinnedEntries.map((entry) => (
                      <div
                        key={`pin-${entry.path}`}
                        role="button"
                        tabIndex={0}
                        className="nc-tree-row"
                        data-active={activePath === entry.path || undefined}
                        style={{ paddingLeft: 8 }}
                        onClick={() => openTab(entry.path)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            event.preventDefault();
                            openTab(entry.path);
                          }
                        }}
                        title={entry.path}
                      >
                        <Star size={12} className="shrink-0 text-accent" />
                        <span className="nc-tree-label">{entry.name}</span>
                        <button
                          type="button"
                          className="nc-pin-remove"
                          aria-label={`Unpin ${entry.name}`}
                          onClick={(event) => {
                            event.stopPropagation();
                            toggleFavorite(entry.path);
                          }}
                        >
                          <X size={10} />
                        </button>
                      </div>
                    ))}
                  </>
                )}
                {recentEntries.length > 0 && (
                  <>
                    <div className="nc-sidebar-group-label">
                      <Clock size={10} /> Recent
                    </div>
                    {recentEntries.map((entry) => (
                      <div
                        key={`recent-${entry.path}`}
                        role="button"
                        tabIndex={0}
                        className="nc-tree-row"
                        data-active={activePath === entry.path || undefined}
                        style={{ paddingLeft: 8 }}
                        onClick={() => openTab(entry.path)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            event.preventDefault();
                            openTab(entry.path);
                          }
                        }}
                        title={entry.path}
                      >
                        <FileText size={12} className="shrink-0 text-fg-faint" />
                        <span className="nc-tree-label">{entry.name}</span>
                        <span className="max-w-[40%] truncate text-[9px] text-fg-faint">
                          {dirName(entry.path)}
                        </span>
                      </div>
                    ))}
                  </>
                )}
                <div className="nc-sidebar-group-divider" />
              </div>
            )}
            {sortEntries(files).map((entry) => renderNode(entry, 0))}
            {files.length === 0 && (
              <div className="px-3 py-6 text-center">
                <p className="text-[11.5px] text-fg-muted">This vault is empty.</p>
                <button type="button" className="nc-btn nc-btn-primary mt-3" onClick={() => setPrompt({ mode: "create-file", dir: "" })}>
                  <Plus size={13} /> Create your first note
                </button>
              </div>
            )}
          </div>
        )}        </div>

      <div className="nc-workspace-wrapper">
      <div className="nc-sidebar-footer">
        <FileText size={11} className="shrink-0" />
        <span className="shrink-0 tabular-nums">{noteCount} notes</span>
        <div className="flex-1" />
        <span className="nc-vault-path" title={vaultRoot}>{vaultRoot}</span>
      </div>
      </div>

      {menu && (
        <ContextMenu x={menu.x} y={menu.y} items={menuItems(menu.entry)} onClose={() => setMenu(null)} />
      )}

      <PromptDialog
        open={!!prompt}
        title={prompt?.mode === "rename" ? "Rename" : prompt?.mode === "create-folder" ? "New folder" : "New file"}
        label={prompt?.mode === "create-folder" ? "Folder name" : "File name"}
        initial={prompt?.mode === "rename" ? baseName(prompt.entry.path) : ""}
        placeholder={
          prompt?.mode === "create-folder"
            ? "Research"
            : "Untitled note.md, page.mdx, data.json…"
        }
        hint={
          prompt?.mode === "create-file"
            ? "Any supported extension works: .md, .mdx, .json, .yaml, .txt, .css, .js, .ts"
            : undefined
        }
        confirmLabel={prompt?.mode === "rename" ? "Rename" : "Create"}
        onClose={() => setPrompt(null)}
        onSubmit={(value) => runCreate(prompt?.mode === "rename" ? dirName(prompt.entry.path) : prompt?.dir ?? "", value)}
      />

      <ConfirmDialog
        open={!!confirm}
        title={confirm?.is_dir ? "Delete folder?" : "Delete note?"}
        message={
          <>
            <span className="text-fg">{confirm ? baseName(confirm.path) : ""}</span> will be moved to the vault trash.
            {confirm?.is_dir ? " Everything inside it will move too." : ""} Open tabs for this item will close.
          </>
        }
        confirmLabel="Move to trash"
        onClose={() => setConfirm(null)}
        onConfirm={runDelete}
      />

      <ConfirmDialog
        open={!!pendingLinks}
        title="Update links to this note?"
        danger={false}
        message={
          pendingLinks ? (
            <>
              <span className="text-fg">{pendingLinks.impacts.length}</span>{" "}
              {pendingLinks.impacts.length === 1 ? "note links" : "notes link"} to{" "}
              <span className="text-fg">{stem(pendingLinks.from)}</span>. Rewrite those references
              to <span className="text-fg">{stem(pendingLinks.to)}</span>? The note itself has
              already been renamed.
            </>
          ) : null
        }
        confirmLabel="Update links"
        onClose={() => setPendingLinks(null)}
        onConfirm={async () => {
          if (!pendingLinks) return;
          const impacts = await api.propagateLinkRename(pendingLinks.from, pendingLinks.to, true);
          const total = impacts.reduce((sum, impact) => sum + impact.replacements, 0);
          if (total > 0) {
            toast.success(`Updated ${total} link${total === 1 ? "" : "s"}`, {
              description: pendingLinks.to,
            });
          }
        }}
      />
    </aside>
  );
}
