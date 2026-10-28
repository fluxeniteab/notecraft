import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import {
  ArrowUpRight,
  Hash,
  List,
  Link2,
  Loader2,
  Network,
  Search,
  Tags as TagsIcon,
  X,
  Zap,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { GraphData, LinkRef, ParsedNote } from "../lib/types";
import { baseName, dirName, formatRelativeTime, resolveNotePath, stem } from "../lib/files";
import AutomationPanel from "./AutomationPanel";

type Tab = "outline" | "links" | "tags" | "search" | "graph" | "auto";

const tabs: Array<{ id: Tab; icon: React.ReactNode; label: string }> = [
  { id: "outline", icon: <List size={14} />, label: "Outline" },
  { id: "links", icon: <Link2 size={14} />, label: "Links" },
  { id: "tags", icon: <TagsIcon size={14} />, label: "Tags & properties" },
  { id: "search", icon: <Search size={14} />, label: "Search" },
  { id: "graph", icon: <Network size={14} />, label: "Local graph" },
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
  const [results, setResults] = useState<Array<{ path: string; title: string; snippet: string }>>([]);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
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

  const search = useCallback(
    async (text: string, tag: string | null) => {
      const request = ++searchRef.current;
      if (!text.trim() && !tag) {
        setResults([]);
        setSearchError(null);
        setSearching(false);
        return;
      }
      setSearching(true);
      setSearchError(null);
      try {
        const hits = await api.search(
          tag ? "" : text,
          tag ? { tag } : {}
        );
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
    const handle = window.setTimeout(() => {
      void search(query, tagFilter);
    }, 220);
    return () => window.clearTimeout(handle);
  }, [query, tagFilter, tab, search]);

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

      <div className="nc-panel-scroll nc-scroll">
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
                    <div className="flex flex-wrap gap-1.5">
                      {parsed.tags.map((tag) => (
                        <button
                          key={tag}
                          type="button"
                          className="nc-tag"
                          onClick={() => {
                            setQuery("");
                            setTagFilter(tag);
                            setTab("search");
                          }}
                        >
                          <Hash size={9} className="mr-0.5" />
                          {tag}
                        </button>
                      ))}
                    </div>
                  )}
                </Panel>

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

            {tagFilter && (
              <div className="mb-2 flex items-center gap-2">
                <span className="nc-chip">
                  <Hash size={9} /> {tagFilter}
                </span>
                <button
                  type="button"
                  className="text-[10px] text-fg-muted hover:text-fg"
                  onClick={() => setTagFilter(null)}
                >
                  Clear tag
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

            {!searching && !searchError && (query.trim() || tagFilter) && results.length === 0 && (
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

            {!query.trim() && !tagFilter && (
              <div className="nc-panel-empty">
                Search titles and note content, or pick a tag from the Tags panel.
              </div>
            )}
          </>
        )}

        {tab === "graph" && <LocalGraph activePath={active?.path ?? null} />}

        {tab === "auto" && <AutomationPanel />}
      </div>
    </aside>
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
