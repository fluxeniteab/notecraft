import { useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { FileEntry } from "../lib/types";
import { Folder, FileText, Plus, FolderPlus, Search, RefreshCw } from "lucide-react";
import { cn } from "../lib/cn";

export default function Sidebar() {
  const { vaultRoot, setVaultRoot, files, setFiles, openTab, selectedFile, setSelectedFile } = useAppStore();
  const [query, setQuery] = useState("");
  const [creating, setCreating] = useState<"file" | "folder" | null>(null);
  const [newName, setNewName] = useState("");

  const loadRoot = async () => {
    if (!vaultRoot) return;
    try {
      const entries = await api.listDir("");
      setFiles(entries);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadRoot();
  }, [vaultRoot]);

  const openVault = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({ directory: true, multiple: false });
      if (typeof picked === "string") {
        const root = await api.openVault(picked);
        setVaultRoot(root);
        await loadRoot();
      }
    } catch (e) {
      console.error(e);
    }
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
    } catch (e) {
      console.error(e);
    }
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
    } catch (e) {
      console.error(e);
    } finally {
      setCreating(null);
      setNewName("");
    }
  };

  const visible = files.filter((f) => !query || f.name.toLowerCase().includes(query.toLowerCase()));

  return (
    <div className="h-full flex flex-col bg-bg-elev border-r border-border">
      <div className="px-3 py-2 border-b border-border flex items-center gap-2">
        <span className="font-semibold text-sm">Files</span>
        <div className="flex-1" />
        <button title="New note" className="p-1 rounded hover:bg-bg-muted" onClick={() => setCreating("file")}><Plus size={14} /></button>
        <button title="New folder" className="p-1 rounded hover:bg-bg-muted" onClick={() => setCreating("folder")}><FolderPlus size={14} /></button>
        <button title="Refresh" className="p-1 rounded hover:bg-bg-muted" onClick={loadRoot}><RefreshCw size={14} /></button>
      </div>
      <div className="px-2 py-1.5 border-b border-border">
        <div className="flex items-center gap-1 px-2 py-1 rounded bg-bg-muted">
          <Search size={12} className="text-fg-muted" />
          <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search files" className="bg-transparent outline-none text-xs flex-1" />
        </div>
      </div>
      {!vaultRoot && (
        <div className="p-3 text-xs text-fg-muted flex flex-col gap-2">
          <p>No vault open.</p>
          <button className="px-2 py-1 rounded bg-accent text-accent-fg" onClick={openVault}>Open vault</button>
          <button className="px-2 py-1 rounded border border-border" onClick={createVault}>Create vault</button>
        </div>
      )}
      {creating && (
        <div className="px-2 py-1.5 border-b border-border flex gap-1">
          <input autoFocus value={newName} onChange={(e) => setNewName(e.target.value)} onKeyDown={(e) => { if (e.key === "Enter") handleCreate(); if (e.key === "Escape") setCreating(null); }} className="flex-1 text-xs px-2 py-1 rounded bg-bg-muted outline-none" placeholder={creating === "file" ? "note name" : "folder name"} />
        </div>
      )}
      <div className="flex-1 overflow-auto nc-scroll p-1">
        {vaultRoot && <FileTree entries={visible} depth={0} onOpen={(p) => { setSelectedFile(p); openTab(p); }} selected={selectedFile} />}
      </div>
      {vaultRoot && (
        <div className="px-2 py-1.5 border-t border-border text-[10px] text-fg-muted truncate" title={vaultRoot}>
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
            className={cn("w-full flex items-center gap-1.5 px-1.5 py-0.5 rounded text-left hover:bg-bg-muted", selected === e.path && "bg-bg-muted")}
            style={{ paddingLeft: 6 + depth * 12 }}
            onClick={() => (e.is_dir ? toggle(e) : onOpen(e.path))}
            onContextMenu={(ev) => {
              ev.preventDefault();
              // simple delete confirm
              if (confirm(`Delete ${e.name}?`)) {
                api.deletePath(e.path, true).then(() => window.location.reload()).catch(console.error);
              }
            }}
          >
            {e.is_dir ? <Folder size={13} className="text-fg-muted shrink-0" /> : <FileText size={13} className="text-fg-muted shrink-0" />}
            <span className="truncate">{e.name}</span>
            {loading[e.path] && <span className="text-[10px] text-fg-muted">…</span>}
          </button>
          {expanded[e.path] && (
            <FileTree entries={expanded[e.path]} depth={depth + 1} onOpen={onOpen} selected={selected} />
          )}
        </li>
      ))}
    </ul>
  );
}
