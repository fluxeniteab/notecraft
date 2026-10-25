import { useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { Command } from "cmdk";
import { X } from "lucide-react";

interface Cmd { id: string; label: string; run: () => void; }

export default function CommandPalette({ open, onClose, onOpenSettings }: { open: boolean; onClose: () => void; onOpenSettings: () => void }) {
  const store = useAppStore();
  const [pages, setPages] = useState<string[]>([]);

  useEffect(() => {
    if (!open) return;
    api.listDir("").then((e) => setPages(e.filter((x) => !x.is_dir).map((x) => x.path))).catch(() => {});
  }, [open]);

  const commands: Cmd[] = [
    { id: "new-note", label: "Create new note", run: async () => {
      const name = prompt("Note name?"); if (!name) return;
      const rel = name.endsWith(".md") ? name : name + ".md";
      await api.createFile(rel, `# ${name.replace(/\.md$/, "")}\n`);
      store.openTab(rel);
    } },
    { id: "open-vault", label: "Open vault", run: async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({ directory: true });
      if (typeof picked === "string") { const r = await api.openVault(picked); store.setVaultRoot(r); }
    } },
    { id: "toggle-left", label: "Toggle left sidebar", run: () => store.togglePanel("leftSidebar") },
    { id: "toggle-right", label: "Toggle right sidebar", run: () => store.togglePanel("rightSidebar") },
    { id: "settings", label: "Open settings", run: () => onOpenSettings() },
    { id: "rebuild-index", label: "Rebuild search index", run: async () => { await api.rebuildIndex(); alert("Index rebuilt"); } },
    { id: "graph", label: "Open global graph", run: () => { (window as any).__nc_open_graph?.(); } },
    { id: "canvas", label: "Open canvas", run: () => { (window as any).__nc_open_canvas?.(); } },
  ];

  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/60 backdrop-blur-sm nc-anim-fade" onClick={onClose}>
      <div className="w-[560px] max-w-[90vw] bg-bg-elev/95 backdrop-blur-md border border-border rounded-xl shadow-2xl overflow-hidden nc-anim-scale" onClick={(e) => e.stopPropagation()}>
        <Command className="w-full">
          <div className="flex items-center px-3 border-b border-border">
            <Command.Input autoFocus placeholder="Type a command or search notes…" className="w-full py-3 bg-transparent outline-none text-sm" />
            <button onClick={onClose}><X size={14} /></button>
          </div>
          <Command.List className="max-h-80 overflow-auto nc-scroll p-1">
            <Command.Empty className="p-3 text-xs text-fg-muted">No results.</Command.Empty>
            <Command.Group heading="Commands" className="text-[10px] text-fg-muted px-2 py-1">
              {commands.map((c) => (
                <Command.Item key={c.id} onSelect={() => { c.run(); onClose(); }} className="px-2 py-1.5 rounded text-sm cursor-pointer data-[selected=true]:bg-bg-muted">
                  {c.label}
                </Command.Item>
              ))}
            </Command.Group>
            <Command.Group heading="Notes" className="text-[10px] text-fg-muted px-2 py-1">
              {pages.map((p) => (
                <Command.Item key={p} onSelect={() => { store.openTab(p); onClose(); }} className="px-2 py-1.5 rounded text-sm cursor-pointer data-[selected=true]:bg-bg-muted">
                  {p}
                </Command.Item>
              ))}
            </Command.Group>
          </Command.List>
        </Command>
      </div>
    </div>
  );
}
