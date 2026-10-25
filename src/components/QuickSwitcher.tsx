import { useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";

export default function QuickSwitcher() {
  const { quickSwitcherOpen, setQuickSwitcherOpen, openTab } = useAppStore();
  const [q, setQ] = useState("");
  const [files, setFiles] = useState<string[]>([]);

  useEffect(() => {
    if (!quickSwitcherOpen) return;
    api.listDir("").then((e) => setFiles(e.filter((x) => !x.is_dir).map((x) => x.path))).catch(() => {});
  }, [quickSwitcherOpen]);

  if (!quickSwitcherOpen) return null;
  const filtered = files.filter((f) => f.toLowerCase().includes(q.toLowerCase()));
  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/60 backdrop-blur-sm nc-anim-fade" onClick={() => setQuickSwitcherOpen(false)}>
      <div className="w-[480px] max-w-[90vw] bg-bg-elev/95 backdrop-blur-md border border-border rounded-xl shadow-2xl overflow-hidden nc-anim-scale" onClick={(e) => e.stopPropagation()}>
        <input autoFocus value={q} onChange={(e) => setQ(e.target.value)} placeholder="Switch to note…" className="w-full px-3 py-3 bg-transparent outline-none text-sm border-b border-border" />
        <div className="max-h-80 overflow-auto nc-scroll p-1">
          {filtered.map((f) => (
            <button key={f} onClick={() => { openTab(f); setQuickSwitcherOpen(false); }} className="block w-full text-left px-2 py-1.5 rounded text-sm hover:bg-bg-muted truncate">
              {f}
            </button>
          ))}
          {filtered.length === 0 && <p className="p-3 text-xs text-fg-muted">No matches.</p>}
        </div>
      </div>
    </div>
  );
}
