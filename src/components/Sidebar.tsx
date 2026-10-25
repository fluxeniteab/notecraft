import { useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { FileEntry } from "../lib/types";
import { FileText, Plus, FolderPlus, Search, RefreshCw, ChevronRight } from "lucide-react";
import { cn } from "../lib/cn";

export default function Sidebar() {
  const { vaultRoot, setVaultRoot, files, setFiles, openTab, selectedFile, setSelectedFile } = useAppStore();
  const [query, setQuery] = useState("");
  const [creating, setCreating] = useState<"file" | "folder" | null>(null);
  const [newName, setNewName] = useState("");

  const loadRoot = async () => {
    if (!vaultRoot) return;
    try { setFiles(await api.listDir("")); } catch (e) { console.error(e); }
  };

  useEffect(() => { loadRoot(); }, [vaultRoot]);

  const openVault = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({ directory: true, multiple: false });
      if (typeof picked === "string") {
        const root = await api.openVault(picked);
        setVaultRoot(root);
        await loadRoot();
      }
    } catch (e) { console.error(e); }
  };

  const createVault = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({ directory: true, multiple: false });
      if (typeof picked === "string") {
        const root = await api.createVault(picked);
        setVaultRoot(root);
        await loadRoot();
      }
    } catch (e) { console.error(e); }
  };

  const handleCreate = async () => {
    if (!newName.trim()) { setCreating(null); return; }
    try {
      if (creating === "file") {
        const rel = newName.endsWith(".md") ? newName : newName + ".md";
        await api.createFile(rel, `# ${newName.replace(/\.md$/, "")}\n`);
        await loadRoot();
        openTab(rel);
      } else if (creating === "folder") {
        await api.createDir(newName);
        await loadRoot();
      }
    } catch (e) { console.error(e); }
    finally { setCreating(null); setNewName(""); }
  };

  const visible = files.filter((f) => !query || f.name.toLowerCase().includes(query.toLowerCase()));

  return (
    <div className="h-full flex flex-col bg-bg-elev border-r border-border">
      <div className="px-3 py-2.5 border-b border-border flex items-center gap-2">
        <span className="font-semibold text-xs uppercase tracking-wider text-fg-muted">Files</span>
        <div className="flex-1" />
        <button title="New note" className="p-1.5 rounded-md hover:bg-bg-muted transition-colors nc-btn text-fg-muted hover:text-fg" onClick={() => setCreating("file")}><Plus size={14} /></button>
        <button title="New folder" className="p-1.5 rounded-md hover:bg-bg-muted transition-colors nc-btn text-fg-muted hover:text-fg" onClick={() => setCreating("folder")}><FolderPlus size={14} /></button>
        <button title="Refresh" className="p-1.5 rounded-md hover:bg-bg-muted transition-colors nc-btn text-fg-muted hover:text-fg" onClick={loadRoot}><RefreshCw size={14} /></button>
      </div>
      <div className="px-2 py-2 border-b border-border">
        <div className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-md bg-bg-muted/60 border border-border/60 focus-within:border-accent transition-colors">
          <Search size={12} className="text-fg-muted" />
          <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search files" className="bg-transparent outline-none text-xs flex-1 text-fg placeholder:text-fg-muted" />
        </div>
      </div>
      {!vaultRoot && (
        <div className="p-3 text-xs text-fg-muted flex flex-col gap-2">
          <p>No vault open.</p>
          <button className="px-3 py-1.5 rounded-md bg-accent text-accent-fg nc-btn text-xs" onClick={openVault}>Open vault</button>
          <button className="px-3 py-1.5 rounded-md border border-border nc-btn text-xs hover:bg-bg-muted" onClick={createVault}>Create vault</button>
        </div>
      )}
      {creating && (
        <div className="px-2 py-2 border-b border-border">
          <input autoFocus value={newName} onChange={(e) => setNewName(e.target.value)} onKeyDown={(e) => { if (e.key === "Enter") handleCreate(); if (e.key === "Escape") setCreating(null); }} className="w-full text-xs px-2.5 py-1.5 rounded-md bg-bg-muted border border-border outline-none focus:border-accent transition-colors" placeholder={creating === "file" ? "note name" : "folder name"} />
        </div>
      )}
      <div className="flex-1 overflow-auto nc-scroll p-1">
        {vaultRoot && <FileTree entries={visible} depth={0} onOpen={(p) => { setSelectedFile(p); openTab(p); }} selected={selectedFile} />}
      </div>
      {vaultRoot && (
        <div className="px-3 py-2 border-t border-border text-[10px] text-fg-muted truncate" title={vaultRoot}>
          {vaultRoot}
        </div>
      )}
    </div>
  );
}

function FileTree({ entries, depth, onOpen, selected }: { entries: FileEntry[]; depth: number; onOpen: (p: string) => void; selected: string | null }) {
  const [expanded, setExpanded] = useState<Record<string, FileEntry[]>>({});
  const [loading, setLoading] = useState<Record<string, boolean>>({});

  const toggle = async (e: FileEntry) => {
    if (!e.is_dir) return;
    if (expanded[e.path]) {
      setExpanded((s) => { const c = { ...s }; delete c[e.path]; return c; });
      return;
    }
    setLoading((s) => ({ ...s, [e.path]: true }));
    try {
      const kids = await api.listDir(e.path);
      setExpanded((s) => ({ ...s, [e.path]: kids }));
    } finally {
      setLoading((s) => ({ ...s, [e.path]: false }));
    }
  };

  return (
    <ul className="text-sm">
      {entries.map((e) => (
        <li key={e.path}>
          <button
            className={cn(
              "w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-left transition-colors duration-100",
              selected === e.path ? "bg-accent/15 text-fg" : "text-fg-muted hover:bg-bg-muted hover:text-fg"
            )}
            style={{ paddingLeft: 8 + depth * 14 }}
            onClick={() => (e.is_dir ? toggle(e) : onOpen(e.path))}
            onContextMenu={(ev) => {
              ev.preventDefault();
              if (confirm(`Delete ${e.name}?`)) {
                api.deletePath(e.path, true).then(() => window.location.reload()).catch(console.error);
              }
            }}
          >
            {e.is_dir ? (
              <span className="transition-transform duration-150" style={{ transform: expanded[e.path] ? "rotate(90deg)" : "rotate(0deg)" }}>
                <ChevronRight size={12} className="text-fg-muted shrink-0" />
              </span>
            ) : (
              <FileText size={13} className="text-fg-muted shrink-0" />
            )}
            <span className="truncate">{e.name}</span>
            {loading[e.path] && <span className="text-[10px] text-fg-muted ml-auto">…</span>}
          </button>
          {expanded[e.path] && (
            <div className="nc-anim-slide">
              <FileTree entries={expanded[e.path]} depth={depth + 1} onOpen={onOpen} selected={selected} />
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}
