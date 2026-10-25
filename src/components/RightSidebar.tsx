import { useState, useEffect } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { ParsedNote } from "../lib/types";
import { List, Link2, Tags, Search, Network, Zap } from "lucide-react";
import AutomationPanel from "./AutomationPanel";

type Tab = "outline" | "backlinks" | "props" | "search" | "graph" | "auto";

export default function RightSidebar() {
  const [tab, setTab] = useState<Tab>("outline");
  const tabs: { id: Tab; icon: React.ReactNode; label: string }[] = [
    { id: "outline", icon: <List size={13} />, label: "Outline" },
    { id: "backlinks", icon: <Link2 size={13} />, label: "Backlinks" },
    { id: "props", icon: <Tags size={13} />, label: "Properties" },
    { id: "search", icon: <Search size={13} />, label: "Search" },
    { id: "graph", icon: <Network size={13} />, label: "Graph" },
    { id: "auto", icon: <Zap size={13} />, label: "Automation" },
  ];
  return (
    <div className="h-full flex flex-col bg-bg-elev">
      <div className="flex border-b border-border">
        {tabs.map((t) => (
          <button key={t.id} title={t.label} onClick={() => setTab(t.id)} className={`flex-1 flex items-center justify-center py-2 ${tab === t.id ? "text-accent border-b-2 border-accent" : "text-fg-muted hover:text-fg"}`}>
            {t.icon}
          </button>
        ))}
      </div>
      <div className="flex-1 min-h-0 overflow-auto nc-scroll p-2">
        {tab === "outline" && <OutlinePanel />}
        {tab === "backlinks" && <BacklinksPanel />}
        {tab === "props" && <PropertiesPanel />}
        {tab === "search" && <SearchPanel />}
        {tab === "graph" && <MiniGraph />}
        {tab === "auto" && <AutomationPanel />}
      </div>
    </div>
  );
}

function OutlinePanel() {
  const activeTab = useAppStore((s) => s.tabs.find((t) => t.id === s.activeTabId));
  const [parsed, setParsed] = useState<ParsedNote | null>(null);
  useEffect(() => {
    if (!activeTab) { setParsed(null); return; }
    api.readFile(activeTab.path).then((c) => api.parseNote(c).then(setParsed)).catch(() => setParsed(null));
  }, [activeTab?.path]);
  if (!parsed) return <p className="text-xs text-fg-muted">No note open.</p>;
  return (
    <ul className="text-xs space-y-0.5">
      {parsed.outline.map((o, i) => (
        <li key={i} style={{ paddingLeft: (o.level - 1) * 10 }} className="truncate hover:text-accent cursor-pointer" title={o.text}>
          {o.text}
        </li>
      ))}
      {parsed.outline.length === 0 && <p className="text-fg-muted">No headings.</p>}
    </ul>
  );
}

function BacklinksPanel() {
  const activeTab = useAppStore((s) => s.tabs.find((t) => t.id === s.activeTabId));
  const { openTab } = useAppStore();
  const [links, setLinks] = useState<string[]>([]);
  const [outgoing, setOutgoing] = useState<string[]>([]);
  useEffect(() => {
    if (!activeTab) { setLinks([]); setOutgoing([]); return; }
    const name = activeTab.path.replace(/\.md$/, "");
    api.backlinks(name).then(setLinks).catch(() => setLinks([]));
    api.outgoingLinks(activeTab.path).then((r) => setOutgoing(r.map((x) => x.target))).catch(() => setOutgoing([]));
  }, [activeTab?.path]);
  return (
    <div className="text-xs space-y-2">
      <div>
        <div className="font-semibold mb-1">Backlinks ({links.length})</div>
        {links.map((l) => (
          <button key={l} className="block w-full text-left truncate hover:text-accent py-0.5" onClick={() => openTab(l)}>{l}</button>
        ))}
        {links.length === 0 && <p className="text-fg-muted">None.</p>}
      </div>
      <div>
        <div className="font-semibold mb-1">Outgoing ({outgoing.length})</div>
        {outgoing.map((l, i) => (
          <div key={i} className="truncate text-fg-muted py-0.5">{l}</div>
        ))}
        {outgoing.length === 0 && <p className="text-fg-muted">None.</p>}
      </div>
    </div>
  );
}

function PropertiesPanel() {
  const activeTab = useAppStore((s) => s.tabs.find((t) => t.id === s.activeTabId));
  const [parsed, setParsed] = useState<ParsedNote | null>(null);
  useEffect(() => {
    if (!activeTab) { setParsed(null); return; }
    api.readFile(activeTab.path).then((c) => { api.parseNote(c).then(setParsed); }).catch(() => setParsed(null));
  }, [activeTab?.path]);
  if (!parsed) return <p className="text-xs text-fg-muted">No note open.</p>;
  const entries = Object.entries(parsed.frontmatter.data || {});
  return (
    <div className="text-xs space-y-1">
      {entries.map(([k, v]) => (
        <div key={k} className="flex gap-2">
          <span className="text-fg-muted shrink-0">{k}:</span>
          <span className="truncate">{String(v)}</span>
        </div>
      ))}
      {entries.length === 0 && <p className="text-fg-muted">No properties. Add frontmatter.</p>}
      <details className="mt-2">
        <summary className="cursor-pointer text-fg-muted">Raw frontmatter</summary>
        <pre className="mt-1 text-[10px] whitespace-pre-wrap break-all">{parsed.frontmatter.raw}</pre>
      </details>
    </div>
  );
}

function SearchPanel() {
  const [q, setQ] = useState("");
  const [results, setResults] = useState<{ path: string; snippet: string; title: string }[]>([]);
  const { openTab } = useAppStore();
  const run = async () => {
    if (!q.trim()) return;
    try { setResults(await api.search(q)); } catch (e) { console.error(e); }
  };
  return (
    <div className="text-xs space-y-2">
      <div className="flex gap-1">
        <input value={q} onChange={(e) => setQ(e.target.value)} onKeyDown={(e) => e.key === "Enter" && run()} placeholder="Search vault" className="flex-1 px-2 py-1 rounded bg-bg-muted outline-none" />
        <button onClick={run} className="px-2 py-1 rounded bg-accent text-accent-fg">Go</button>
      </div>
      <div className="space-y-1">
        {results.map((r) => (
          <button key={r.path} onClick={() => openTab(r.path)} className="block w-full text-left p-1.5 rounded hover:bg-bg-muted">
            <div className="font-medium truncate">{r.title || r.path}</div>
            <div className="text-fg-muted truncate">{r.snippet}</div>
          </button>
        ))}
        {results.length === 0 && q && <p className="text-fg-muted">No results.</p>}
      </div>
    </div>
  );
}

function MiniGraph() {
  return <p className="text-xs text-fg-muted">Open Graph view from command palette.</p>;
}
