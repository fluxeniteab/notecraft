import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import {
  ArrowUpRight,
  Folder,
  Hash,
  History,
  LayoutGrid,
  List,
  Link2,
  Loader2,
  Network,
  Search,
  Sparkles,
  CheckSquare,
  Save,
  Trash2,
  Tags as TagsIcon,
  X,
  Zap,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { GraphData, LinkRef, ParsedNote } from "../lib/types";
import { baseName, dirName, formatRelativeTime, resolveNotePath, stem } from "../lib/files";
import AiPanel from "./AiPanel";
import AutomationPanel from "./AutomationPanel";
import CollectionsPanel from "./CollectionsPanel";
import HistoryPanel from "./HistoryPanel";

type Tab =
  | "ai"
  | "outline"
  | "links"
  | "tags"
  | "search"
  | "graph"
  | "views"
  | "tasks"
  | "history"
  | "auto";

const tabs: Array<{ id: Tab; icon: React.ReactNode; label: string }> = [
  { id: "ai", icon: <Sparkles size={14} />, label: "AI assistant" },
  { id: "outline", icon: <List size={14} />, label: "Outline" },
  { id: "links", icon: <Link2 size={14} />, label: "Links" },
  { id: "tags", icon: <TagsIcon size={14} />, label: "Tags & properties" },
  { id: "search", icon: <Search size={14} />, label: "Search" },
  { id: "graph", icon: <Network size={14} />, label: "Local graph" },
  { id: "views", icon: <LayoutGrid size={14} />, label: "Collections & views" },
  { id: "tasks", icon: <CheckSquare size={14} />, label: "Tasks" },
  { id: "history", icon: <History size={14} />, label: "Version history" },
  { id: "auto", icon: <Zap size={14} />, label: "Automations" },
];

function safeSnippet(html: string): string {
  return html.replace(/<(?!\/?b(?=[\s>]))[^>]*>/gi, "");
}

function Panel({ title, count, children }: { title: string; count?: number; children: React.ReactNode }) {
  return (
    <section className="mb-4">
      <div className="nc-panel-title">
        {title}
        {count !== undefined && <span className="nc-count">{count}</span>}
      </div>
      {children}
    </section>
  );
}

export default function RightSidebar() {
  const active = useAppStore((s) => s.tabs.find((t) => t.id === s.activeTabId) ?? null);
  const saveStatus = useAppStore((s) => s.saveStatus);
  const allFiles = useAppStore((s) => s.allFiles);
  const openTab = useAppStore((s) => s.openTab);
  const gotoLine = useAppStore((s) => s.gotoLine);
  const [tab, setTab] = useState<Tab>("outline");
  const [parsed, setParsed] = useState<ParsedNote | null>(null);
  const [backlinks, setBacklinks] = useState<string[]>([]);
  const [outgoing, setOutgoing] = useState<LinkRef[]>([]);
  const [loading, setLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [tagFilter, setTagFilter] = useState<string | null>(null);
  const [folderFilter, setFolderFilter] = useState("");
  const [results, setResults] = useState<Array<{ path: string; title: string; snippet: string }>>([]);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [savedSearches, setSavedSearches] = useState<Array<{ id: string; name: string; query: string; filters: Record<string, unknown> }>>([]);
  const [searchHistory, setSearchHistory] = useState<string[]>([]);
  const [recentSearchesLoaded, setRecentSearchesLoaded] = useState(false);
  const [taskItems, setTaskItems] = useState<import("../lib/types").TaskItem[]>([]);
  const [taskLoading, setTaskLoading] = useState(false);
  const [taskFilter, setTaskFilter] = useState<"open" | "all" | "done">("open");
  const requestRef = useRef(0);
  const searchRef = useRef(0);

  const activePath = active?.path ?? null;
  const activeStatus = active ? saveStatus[active.id] : undefined;

  const loadNote = useCallback(async (path: string) => {
    const request = ++requestRef.current;
    setLoading(true);
    try {
      const content = await api.readFile(path);
      const [note, backs, outgoingLinks] = await Promise.all([
        api.parseNote(content),
        api.backlinks(path).catch(() => []),
        api.outgoingLinks(path).catch(() => []),
      ]);
      if (requestRef.current !== request) return;
      setParsed(note);
      setBacklinks(backs);
      setOutgoing(outgoingLinks);
    } catch (error) {
      if (requestRef.current !== request) return;
      setParsed(null);
      setBacklinks([]);
      setOutgoing([]);
      console.error(error);
    } finally {
      if (requestRef.current === request) setLoading(false);
    }
  }, []);

  const refreshTasks = useCallback(async () => {
    setTaskLoading(true);
    try {
      setTaskItems(await api.listTasks());
    } catch (error) {
      toast.error("Could not load tasks", { description: String(error) });
    } finally {
      setTaskLoading(false);
    }
  }, []);

  useEffect(() => {
    if (tab === "tasks") void refreshTasks();
  }, [tab, activeStatus, refreshTasks]);

  useEffect(() => {
    if (!activePath) {
      setParsed(null);
      setBacklinks([]);
      setOutgoing([]);
      return;
    }
    void loadNote(activePath);
  }, [activePath, loadNote]);

  // Refresh outline/links once a save lands so the panel reflects the text.
  useEffect(() => {
    if (!activePath || activeStatus !== "saved") return;
    void loadNote(activePath);
  }, [activePath, activeStatus, loadNote]);

  const folders = useMemo(() => {
    const set = new Set<string>();
    allFiles.forEach((file) => {
      const dir = dirName(file.path);
      if (dir) set.add(dir);
    });
    return [...set].sort();
  }, [allFiles]);

  const search = useCallback(
    async (text: string, tag: string | null, folder: string) => {
      const request = ++searchRef.current;
      const filters: Record<string, unknown> = {};
      if (tag) filters.tag = tag;
      if (folder.trim()) filters.folder = folder.trim().replace(/[/\\]+$/, "");
      if (!text.trim() && !tag && !folder.trim()) {
        setResults([]);
        setSearchError(null);
        setSearching(false);
        return;
      }
      setSearching(true);
      setSearchError(null);
      try {
        const hits = await api.search(text, filters);
        if (searchRef.current !== request) return;
        setResults(hits.map((hit) => ({ path: hit.path, title: hit.title || stem(hit.path), snippet: hit.snippet })));
      } catch (error) {
        if (searchRef.current !== request) return;
        setResults([]);
        setSearchError(String(error).replace(/^Error:\s*/, ""));
      } finally {
        if (searchRef.current === request) setSearching(false);
      }
    },
    []
  );

  useEffect(() => {
    if (tab !== "search") return;
    setRecentSearchesLoaded(false);
    void Promise.all([api.listSavedSearches(), api.searchHistory()])
      .then(([saved, history]) => {
        setSavedSearches(saved);
        setSearchHistory(history);
      })
      .catch(() => {
        setSavedSearches([]);
        setSearchHistory([]);
      })
      .finally(() => setRecentSearchesLoaded(true));
    const handle = window.setTimeout(() => {
      void search(query, tagFilter, folderFilter);
    }, 220);
    return () => window.clearTimeout(handle);
  }, [query, tagFilter, folderFilter, tab, search]);

  const openSearchResult = (path: string) => {
    const exists = allFiles.some((file) => file.path === path);
    if (!exists) {
      toast.info("That note is no longer in the vault");
      return;
    }
    openTab(path);
  };

  const openWikiTarget = (link: LinkRef) => {
    const resolved = resolveNotePath(link.target, allFiles, active?.path);
    if (!resolved) {
      toast.info(`No note matches “${link.target}”`);
      return;
    }
    openTab(resolved);
    if (link.line > 0 && resolved === active?.path) gotoLine(resolved, link.line);
  };

  const properties = useMemo(() => {
    if (!parsed) return [];
    return Object.entries(parsed.frontmatter.data).map(([key, value]) => ({
      key,
      value:
        typeof value === "string"
          ? value
          : Array.isArray(value)
            ? value.join(", ")
            : JSON.stringify(value) ?? "",
    }));
  }, [parsed]);

  // Nested tags are grouped under their top-level parent for browsing.
  const tagGroups = useMemo(() => {
    if (!parsed) return [];
    const groups = new Map<string, Set<string>>();
    parsed.tags.forEach((tag) => {
      const idx = tag.indexOf("/");
      if (idx === -1) {
        if (!groups.has(tag)) groups.set(tag, new Set());
        return;
      }
      const parent = tag.slice(0, idx);
      if (!groups.has(parent)) groups.set(parent, new Set());
      groups.get(parent)!.add(tag);
    });
    return [...groups.entries()].map(([root, children]) => ({
      root,
      children: [...children],
    }));
  }, [parsed]);

  const [renameFrom, setRenameFrom] = useState<string | null>(null);
  const [renameTo, setRenameTo] = useState("");

  const runRenameTag = async () => {
    if (!renameFrom || !renameTo.trim()) return;
    try {
      const count = await api.renameTag(renameFrom, renameTo.trim());
      if (count === 0) {
        toast.info("No occurrences of that tag were found");
      } else {
        toast.success(`Tag renamed (${count} replacement${count === 1 ? "" : "s"})`, {
          description: `#${renameFrom} → #${renameTo.trim()}`,
        });
      }
      setRenameTo("");
      setRenameFrom(null);
      if (activePath) await loadNote(activePath);
    } catch (error) {
      toast.error("Could not rename tag", { description: String(error) });
    }
  };

  const selectTag = (tag: string) => {
    setQuery("");
    setTagFilter(tag);
    setRenameFrom(tag);
    setRenameTo("");
    setTab("search");
  };

  return (
    <aside className="nc-rightbar">
      <div className="nc-rail" role="tablist" aria-label="Context panels">
        {tabs.map((entry) => (
          <button
            key={entry.id}
            type="button"
            role="tab"
            aria-selected={tab === entry.id}
            aria-label={entry.label}
            title={entry.label}
            data-active={tab === entry.id}
            className="nc-rail-btn"
            onClick={() => setTab(entry.id)}
          >
            {entry.icon}
          </button>
        ))}
        <div className="flex-1" />
        {loading && <Loader2 size={12} className="mr-1 animate-spin text-fg-faint" />}
      </div>

      <div className="nc-panel-scroll nc-scroll" data-plain={tab === "ai" || undefined}>
        {tab === "ai" && <AiPanel />}
        {tab === "outline" && (
          <>
            {!active && <div className="nc-panel-empty">Open a note to see its outline.</div>}
            {active && parsed && parsed.outline.length === 0 && (
              <div className="nc-panel-empty">Add headings to build an outline.</div>
            )}
            {active &&
              parsed?.outline.map((item) => (
                <button
                  key={`${item.line}-${item.id}`}
                  type="button"
                  className="nc-outline-item"
                  data-level={item.level}
                  style={{ paddingLeft: 7 + (item.level - 1) * 11 }}
                  title={item.text}
                  onClick={() => {
                    openTab(active.path);
                    gotoLine(active.path, item.line);
                  }}
                >
                  {item.text}
                </button>
              ))}
            {active && parsed && (
              <div className="mt-3 border-t border-border pt-3 text-[10.5px] text-fg-faint">
                {parsed.word_count.toLocaleString()} words · {parsed.char_count.toLocaleString()} characters
              </div>
            )}
          </>
        )}

        {tab === "links" && (
          <>
            {!active && <div className="nc-panel-empty">Open a note to inspect its links.</div>}
            {active && (
              <>
                <Panel title="Linked mentions" count={backlinks.length}>
                  {backlinks.length === 0 ? (
                    <div className="nc-panel-empty">No other note links here yet.</div>
                  ) : (
                    backlinks.map((path) => (
                      <button
                        key={path}
                        type="button"
                        className="nc-panel-link"
                        onClick={() => openTab(path)}
                      >
                        <Link2 size={12} className="shrink-0 text-fg-faint" />
                        <span className="min-w-0 flex-1 truncate">{stem(path)}</span>
                        <span className="max-w-[40%] truncate text-[9.5px] text-fg-faint">{dirName(path)}</span>
                      </button>
                    ))
                  )}
                </Panel>

                <Panel title="Outgoing links" count={outgoing.length}>
                  {outgoing.length === 0 ? (
                    <div className="nc-panel-empty">
                      Link to another note with <span className="text-fg">[[double brackets]]</span>.
                    </div>
                  ) : (
                    outgoing.map((link, index) => {
                      const resolved = resolveNotePath(link.target, allFiles, active.path);
                      return (
                        <button
                          key={`${link.target}-${index}`}
                          type="button"
                          className="nc-panel-link"
                          title={resolved ? resolved : "Missing note"}
                          onClick={() => openWikiTarget(link)}
                        >
                          <ArrowUpRight
                            size={12}
                            className={resolved ? "shrink-0 text-accent" : "shrink-0 text-danger"}
                          />
                          <span className="min-w-0 flex-1 truncate">{link.target}</span>
                          {link.heading && (
                            <span className="max-w-[35%] truncate text-[9.5px] text-fg-faint">
                              #{link.heading}
                            </span>
                          )}
                        </button>
                      );
                    })
                  )}
                </Panel>
              </>
            )}
            {active && <UnlinkedMentions activePath={active.path} />}
          </>
        )}

        {tab === "tags" && (
          <>
            {!active && <div className="nc-panel-empty">Open a note to see its tags and properties.</div>}
            {active && parsed && (
              <>
                <Panel title="Tags" count={parsed.tags.length}>
                  {parsed.tags.length === 0 ? (
                    <div className="nc-panel-empty">
                      Add a tag with <span className="text-fg">#topic</span> anywhere in the note.
                    </div>
                  ) : (
                    <div className="flex flex-col gap-1.5">
                      {tagGroups.map((group) => (
                        <div key={group.root} className="flex flex-wrap items-center gap-1.5">
                          <button key={group.root} type="button" className="nc-tag" onClick={() => selectTag(group.root)}>
                            <Hash size={9} className="mr-0.5" />
                            {group.root}
                          </button>
                          {group.children.map((child) => (
                            <button
                              key={child}
                              type="button"
                              className="nc-tag opacity-75"
                              title={child}
                              onClick={() => selectTag(child)}
                            >
                              {child.slice(group.root.length + 1)}
                            </button>
                          ))}
                        </div>
                      ))}
                    </div>
                  )}
                </Panel>

                {parsed.tags.length > 0 && (
                  <Panel title="Rename tag">
                    <div className="flex items-center gap-1.5">
                      <select
                        className="nc-select !h-7 min-w-0 flex-1 text-[10px]"
                        value={renameFrom ?? ""}
                        onChange={(event) => setRenameFrom(event.target.value || null)}
                        aria-label="Tag to rename"
                      >
                        <option value="">Choose tag…</option>
                        {parsed.tags.map((tag) => (
                          <option key={tag} value={tag}>#{tag}</option>
                        ))}
                      </select>
                      <input
                        className="nc-input !h-7 min-w-0 flex-1 !text-[10px]"
                        value={renameTo}
                        onChange={(event) => setRenameTo(event.target.value)}
                        placeholder="New name"
                        aria-label="New tag name"
                        onKeyDown={(event) => {
                          if (event.key === "Enter") void runRenameTag();
                        }}
                      />
                      <button
                        type="button"
                        className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
                        disabled={!renameFrom || !renameTo.trim()}
                        onClick={() => void runRenameTag()}
                      >
                        Rename
                      </button>
                    </div>
                    <div className="nc-setting-hint mt-1.5">
                      Rewrites the tag across every note that uses it.
                    </div>
                  </Panel>
                )}

                <Panel title="Properties" count={properties.length}>
                  {properties.length === 0 ? (
                    <div className="nc-panel-empty">No frontmatter properties in this note.</div>
                  ) : (
                    properties.map((property) => (
                      <div key={property.key} className="nc-property-row">
                        <span className="nc-property-key" title={property.key}>{property.key}</span>
                        <span className="nc-property-value" title={property.value}>{property.value}</span>
                      </div>
                    ))
                  )}
                </Panel>

                <Panel title="File">
                  <div className="nc-property-row">
                    <span className="nc-property-key">Path</span>
                    <span className="nc-property-value" title={active.path}>{active.path}</span>
                  </div>
                  <div className="nc-property-row">
                    <span className="nc-property-key">Modified</span>
                    <span className="nc-property-value">
                      {(() => {
                        const modified = allFiles.find((file) => file.path === active.path)?.modified ?? 0;
                        return modified ? formatRelativeTime(modified) : "—";
                      })()}
                    </span>
                  </div>
                  <div className="nc-property-row">
                    <span className="nc-property-key">Size</span>
                    <span className="nc-property-value">
                      {(() => {
                        const size = allFiles.find((file) => file.path === active.path)?.size ?? 0;
                        return size >= 1024 ? `${(size / 1024).toFixed(1)} KB` : `${size} B`;
                      })()}
                    </span>
                  </div>
                  <div className="nc-property-row">
                    <span className="nc-property-key">Type</span>
                    <span className="nc-property-value">
                      {(() => {
                        const ext = active.path.includes(".") ? active.path.split(".").pop()!.toUpperCase() : "";
                        return ext ? `${ext} file` : "File";
                      })()}
                    </span>
                  </div>
                  <div className="nc-property-row">
                    <span className="nc-property-key">Words</span>
                    <span className="nc-property-value">{parsed.word_count.toLocaleString()}</span>
                  </div>
                </Panel>
              </>
            )}
          </>
        )}

        {tab === "search" && (
          <>
            <div className="nc-field mb-2">
              <Search size={12} />
              <input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" && results[0]) openSearchResult(results[0].path);
                  if (event.key === "Escape") setQuery("");
                }}
                placeholder="Search the vault…"
                aria-label="Search the vault"
              />
              {query && (
                <button type="button" aria-label="Clear search" onClick={() => setQuery("")}>
                  <X size={11} />
                </button>
              )}
            </div>

            <div className="nc-field mb-2">
              <Folder size={12} />
              <input
                list="nc-folder-filter"
                value={folderFilter}
                onChange={(event) => setFolderFilter(event.target.value)}
                placeholder="Filter by folder…"
                aria-label="Filter results by folder"
              />
              {folderFilter && (
                <button type="button" aria-label="Clear folder filter" onClick={() => setFolderFilter("")}>
                  <X size={11} />
                </button>
              )}
              <datalist id="nc-folder-filter">
                {folders.map((folder) => (
                  <option key={folder} value={folder} />
                ))}
              </datalist>
            </div>

            <div className="mb-2 flex items-center gap-1.5">
              <button
                type="button"
                className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
                disabled={!query.trim() && !tagFilter && !folderFilter.trim()}
                onClick={async () => {
                  const name = window.prompt("Name this saved search", query.trim() || `Tag: ${tagFilter}` || `Folder: ${folderFilter}`);
                  if (!name?.trim()) return;
                  const id = crypto.randomUUID();
                  try {
                    const filters: Record<string, unknown> = {};
                    if (tagFilter) filters.tag = tagFilter;
                    if (folderFilter.trim()) filters.folder = folderFilter.trim();
                    await api.saveSearch(id, name.trim(), query.trim(), filters);
                          setSavedSearches(await api.listSavedSearches());
                          toast.success("Search saved");
                        } catch (error) {
                    toast.error("Could not save search", { description: String(error) });
                  }
                }}
              >
                <Save size={11} /> Save search
              </button>
            </div>

            {savedSearches.length > 0 && (
              <Panel title="Saved searches" count={savedSearches.length}>
                {savedSearches.map((saved) => (
                  <div key={saved.id} className="flex items-center gap-1">
                    <button
                      type="button"
                      className="nc-panel-link min-w-0 flex-1"
                      onClick={() => {
                        setQuery(saved.query);
                        setTagFilter(typeof saved.filters.tag === "string" ? saved.filters.tag : null);
                        setFolderFilter(typeof saved.filters.folder === "string" ? saved.filters.folder : "");
                      }}
                    >
                      <Search size={11} className="shrink-0 text-fg-faint" />
                      <span className="truncate">{saved.name}</span>
                    </button>
                    <button
                      type="button"
                      aria-label={`Delete saved search ${saved.name}`}
                      className="nc-icon-btn nc-sm"
                      onClick={async () => {
                        try {
                          await api.deleteSavedSearch(saved.id);
                          setSavedSearches((current) => current.filter((item) => item.id !== saved.id));
                        } catch (error) {
                          toast.error("Could not delete saved search", { description: String(error) });
                        }
                      }}
                    >
                      <Trash2 size={11} />
                    </button>
                  </div>
                ))}
              </Panel>
            )}

            {!query.trim() && recentSearchesLoaded && searchHistory.length > 0 && (
              <Panel title="Recent searches" count={searchHistory.length}>
                {[...new Set(searchHistory)].slice(0, 8).map((item) => (
                  <button key={item} type="button" className="nc-panel-link" onClick={() => setQuery(item)}>
                    <Clock3Icon /> <span className="truncate">{item}</span>
                  </button>
                ))}
              </Panel>
            )}

            {(tagFilter || folderFilter.trim()) && (
              <div className="mb-2 flex flex-wrap items-center gap-2">
                {tagFilter && (
                  <span className="nc-chip">
                    <Hash size={9} /> {tagFilter}
                  </span>
                )}
                {folderFilter.trim() && (
                  <span className="nc-chip">
                    <Folder size={9} /> {folderFilter}
                  </span>
                )}
                <button
                  type="button"
                  className="text-[10px] text-fg-muted hover:text-fg"
                  onClick={() => {
                    setTagFilter(null);
                    setFolderFilter("");
                  }}
                >
                  Clear filters
                </button>
              </div>
            )}

            {searching && (
              <div className="flex items-center gap-2 px-1 py-3 text-[11px] text-fg-muted">
                <Loader2 size={12} className="animate-spin" /> Searching…
              </div>
            )}

            {searchError && (
              <div className="nc-panel-empty text-danger">Search failed: {searchError}</div>
            )}

            {!searching && !searchError && (query.trim() || tagFilter || folderFilter.trim()) && results.length === 0 && (
              <div className="nc-panel-empty">No notes match your search.</div>
            )}

            {results.map((result) => (
              <button
                key={result.path}
                type="button"
                className="nc-search-result"
                onClick={() => openSearchResult(result.path)}
              >
                <div className="nc-search-result-title">{result.title}</div>
                <div className="nc-search-result-snippet" dir="auto">
                  <span dangerouslySetInnerHTML={{ __html: safeSnippet(result.snippet) }} />
                </div>
                <div className="mt-1 truncate text-[9.5px] text-fg-faint">{result.path}</div>
              </button>
            ))}

            {!query.trim() && !tagFilter && !folderFilter.trim() && !searchHistory.length && !savedSearches.length && (
              <div className="nc-panel-empty">
                Search titles and note content, or pick a tag from the Tags panel.
              </div>
            )}
          </>
        )}

        {tab === "graph" && <LocalGraph activePath={active?.path ?? null} />}

        {tab === "views" && <CollectionsPanel />}

        {tab === "history" && <HistoryPanel activePath={active?.path ?? null} />}

        {tab === "tasks" && (
          <>
            <div className="flex items-center justify-between px-1 pb-2">
              <div className="nc-panel-title !mb-0">Tasks <span className="nc-count">{taskItems.filter((task) => !task.done).length}</span></div>
              <select className="nc-select !h-7 !w-auto text-[10px]" value={taskFilter} onChange={(event) => setTaskFilter(event.target.value as typeof taskFilter)} aria-label="Filter tasks">
                <option value="open">Open</option><option value="all">All</option><option value="done">Done</option>
              </select>
            </div>
            {taskLoading ? <div className="nc-panel-empty">Loading tasks…</div> : taskItems.filter((task) => taskFilter === "all" || task.done === (taskFilter === "done")).length === 0 ? (
              <div className="nc-panel-empty">No tasks match this filter.</div>
            ) : taskItems.filter((task) => taskFilter === "all" || task.done === (taskFilter === "done")).map((task) => (
              <div key={`${task.note_path}:${task.line}`} className="nc-task-item">
                <input
                  type="checkbox"
                  checked={task.done}
                  aria-label={`${task.done ? "Reopen" : "Complete"} task ${task.text}`}
                  onChange={async (event) => {
                    try {
                      await api.setTaskDone(task.note_path, task.line, event.target.checked);
                      await refreshTasks();
                    } catch (error) {
                      toast.error("Could not update task", { description: String(error) });
                    }
                  }}
                />
                <button type="button" className="min-w-0 flex-1 text-left" onClick={() => { openTab(task.note_path); gotoLine(task.note_path, task.line); }}>
                  <span className="block text-[11px] text-fg-secondary">{task.text}</span>
                  <span className="block truncate text-[9.5px] text-fg-faint">{task.note_path}{task.due ? ` · due ${task.due}` : ""}{task.priority ? ` · ${task.priority} priority` : ""}</span>
                </button>
              </div>
            ))}
          </>
        )}

        {tab === "auto" && <AutomationPanel />}
      </div>
    </aside>
  );
}

function Clock3Icon() {
  return <span className="shrink-0 text-[10px] text-fg-faint">↺</span>;
}

/** Notes that mention this note's name without linking to it. */
function UnlinkedMentions({ activePath }: { activePath: string }) {
  const openTab = useAppStore((s) => s.openTab);
  const gotoLine = useAppStore((s) => s.gotoLine);
  const [items, setItems] = useState<Array<{ path: string; line: number; text: string }> | null>(null);

  useEffect(() => {
    let alive = true;
    setItems(null);
    api
      .unlinkedMentions(activePath)
      .then((rows) => alive && setItems(rows))
      .catch(() => alive && setItems([]));
    return () => {
      alive = false;
    };
  }, [activePath]);

  if (items === null) {
    return (
      <div className="flex items-center gap-2 px-1 py-2 text-[11px] text-fg-muted">
        <Loader2 size={12} className="animate-spin" /> Scanning for mentions…
      </div>
    );
  }
  if (items.length === 0) return null;

  return (
    <Panel title="Unlinked mentions" count={items.length}>
      <div className="nc-panel-empty mb-1.5">
        These notes mention this one by name but do not link it yet.
      </div>
      {items.map((item) => (
        <button
          key={`${item.path}:${item.line}`}
          type="button"
          className="nc-panel-link"
          title={item.text}
          onClick={() => {
            openTab(item.path);
            gotoLine(item.path, item.line);
          }}
        >
          <Link2 size={12} className="shrink-0 text-fg-faint" />
          <span className="min-w-0 flex-1 truncate">{stem(item.path)}</span>
          <span className="text-[9.5px] tabular-nums text-fg-faint">:{item.line}</span>
        </button>
      ))}
    </Panel>
  );
}

function LocalGraph({ activePath }: { activePath: string | null }) {
  const openTab = useAppStore((s) => s.openTab);
  const [data, setData] = useState<GraphData | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!activePath) {
      setData(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    api
      .graphData("global")
      .then((graph) => {
        if (!cancelled) setData(graph);
      })
      .catch(() => {
        if (!cancelled) setData(null);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [activePath]);

  const local = useMemo(() => {
    if (!activePath || !data) return null;
    const neighbors = new Set<string>();
    data.edges.forEach((edge) => {
      if (edge.source === activePath) neighbors.add(edge.target);
      if (edge.target === activePath) neighbors.add(edge.source);
    });
    // Keep the active note even when it has no links yet.
    const nodes = [...neighbors]
      .slice(0, 12)
      .map((id) => ({ id, label: stem(id) }));
    return { nodes };
  }, [activePath, data]);

  if (!activePath) return <div className="nc-panel-empty">Open a note to see its neighborhood.</div>;
  if (loading) {
    return (
      <div className="flex items-center gap-2 px-1 py-3 text-[11px] text-fg-muted">
        <Loader2 size={12} className="animate-spin" /> Building graph…
      </div>
    );
  }
  if (!local || local.nodes.length === 0) {
    return (
      <div className="nc-panel-empty">
        <Network size={16} className="mx-auto mb-2 text-fg-faint" />
        No connections yet. Link to another note with [[brackets]].
      </div>
    );
  }

  const size = 240;
  const center = size / 2;
  const radius = 82;
  return (
    <div>
      <div className="nc-panel-title">
        <Network size={12} /> Neighborhood
        <span className="nc-count">{local.nodes.length}</span>
      </div>
      <svg viewBox={`0 0 ${size} ${size}`} className="w-full" role="img" aria-label="Local note graph">
        {local.nodes.map((node, index) => {
          const angle = (index / local.nodes.length) * Math.PI * 2 - Math.PI / 2;
          const x = center + Math.cos(angle) * radius;
          const y = center + Math.sin(angle) * radius;
          return (
            <g key={node.id} className="cursor-pointer" onClick={() => openTab(node.id)}>
              <line
                x1={center}
                y1={center}
                x2={x}
                y2={y}
                stroke="var(--nc-border-strong)"
                strokeWidth={1}
              />
              <circle
                cx={x}
                cy={y}
                r={6}
                fill="var(--nc-accent)"
                fillOpacity={0.85}
                stroke="var(--nc-bg)"
                strokeWidth={1.5}
              />
              <text
                x={x}
                y={y + 17}
                textAnchor="middle"
                fill="var(--nc-fg-muted)"
                fontSize={8.5}
                style={{ pointerEvents: "none" }}
              >
                {node.label.length > 18 ? `${node.label.slice(0, 17)}…` : node.label}
              </text>
            </g>
          );
        })}
        <circle cx={center} cy={center} r={11} fill="var(--nc-bg)" stroke="var(--nc-accent)" strokeWidth={2} />
        <circle cx={center} cy={center} r={4.5} fill="var(--nc-accent)" />
        <text
          x={center}
          y={center + 26}
          textAnchor="middle"
          fill="var(--nc-fg)"
          fontSize={9.5}
          fontWeight={600}
        >
          {baseName(activePath)}
        </text>
      </svg>
      <p className="px-1 text-[10px] leading-relaxed text-fg-faint">
        Direct links only. Click a node to open it.
      </p>
    </div>
  );
}
