import { useEffect, useRef, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { EditorView, basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import type { ViewUpdate } from "@codemirror/view";
import { oneDark } from "@codemirror/theme-one-dark";
import { cn } from "../lib/cn";
import { Eye, Code2, Columns2, BookOpen } from "lucide-react";

export default function EditorArea() {
  const { tabs, activeTabId, setTabDirty } = useAppStore();
  const active = tabs.find((t) => t.id === activeTabId) ?? null;
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(false);
  const [html, setHtml] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const saveTimer = useRef<number | null>(null);

  // load content when active tab changes
  useEffect(() => {
    if (!active) { setContent(""); return; }
    setLoading(true);
    api.readFile(active.path).then((c) => {
      setContent(c);
      setLoading(false);
    }).catch(() => { setContent(""); setLoading(false); });
  }, [active?.path]);

  // render html for reading mode
  useEffect(() => {
    if (!active || !content) return;
    if (active.mode === "reading" || active.mode === "split") {
      api.renderHtml(content).then(setHtml).catch(() => setHtml(""));
    }
  }, [content, active?.mode]);

  // mount CodeMirror
  useEffect(() => {
    if (!containerRef.current) return;
    if (!active) return;
    if (active.mode === "reading") {
      if (viewRef.current) { viewRef.current.destroy(); viewRef.current = null; }
      return;
    }
    // destroy previous
    if (viewRef.current) { viewRef.current.destroy(); viewRef.current = null; }

    const state = EditorState.create({
      doc: content,
      extensions: [
        basicSetup,
        markdown(),
        EditorView.updateListener.of((u: ViewUpdate) => {
          if (u.docChanged) {
            const v = u.state.doc.toString();
            setContent(v);
            setTabDirty(active.id, true);
            if (saveTimer.current) window.clearTimeout(saveTimer.current);
            saveTimer.current = window.setTimeout(() => {
              api.writeFile(active.path, v).then(() => setTabDirty(active.id, false)).catch(console.error);
            }, useAppStore.getState().settings?.autosave_ms ?? 800);
          }
        }),
        // theme: follow .dark class via oneDark when dark
        document.documentElement.classList.contains("dark") ? oneDark : [],
      ],
    });
    const view = new EditorView({ state, parent: containerRef.current });
    viewRef.current = view;
    return () => { view.destroy(); viewRef.current = null; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active?.id, active?.mode]);

  // sync external content into view when loaded
  useEffect(() => {
    const v = viewRef.current;
    if (v && v.state.doc.toString() !== content && !v.hasFocus) {
      v.dispatch({ changes: { from: 0, to: v.state.doc.length, insert: content } });
    }
  }, [content]);

  if (!active) {
    return (
      <div className="h-full flex items-center justify-center text-fg-muted text-sm">
        Open a note to start editing.
      </div>
    );
  }

  const mode = active.mode;
  return (
    <div className="h-full flex flex-col min-h-0">
      <div className="flex items-center gap-1 px-2 py-1 border-b border-border bg-bg-elev">
        <ModeBtn active={mode === "source"} onClick={() => useAppStore.getState().setTabMode(active.id, "source")} icon={<Code2 size={13} />} label="Source" />
        <ModeBtn active={mode === "live"} onClick={() => useAppStore.getState().setTabMode(active.id, "live")} icon={<Eye size={13} />} label="Live" />
        <ModeBtn active={mode === "reading"} onClick={() => useAppStore.getState().setTabMode(active.id, "reading")} icon={<BookOpen size={13} />} label="Reading" />
        <ModeBtn active={mode === "split"} onClick={() => useAppStore.getState().setTabMode(active.id, "split")} icon={<Columns2 size={13} />} label="Split" />
        <div className="flex-1" />
        <span className="text-[10px] text-fg-muted">{content.split(/\s+/).filter(Boolean).length} words</span>
      </div>
      <div className="flex-1 min-h-0 flex">
        {mode !== "reading" && (
          <div ref={containerRef} className={cn("min-h-0", mode === "split" ? "w-1/2 border-r border-border" : "w-full")} />
        )}
        {(mode === "reading" || mode === "split") && (
          <div className={cn("overflow-auto nc-scroll p-6 prose max-w-none", mode === "split" ? "w-1/2" : "w-full")} dangerouslySetInnerHTML={{ __html: html }} />
        )}
      </div>
      {loading && <div className="absolute inset-0 flex items-center justify-center text-fg-muted">Loading…</div>}
    </div>
  );
}

function ModeBtn({ active, onClick, icon, label }: { active: boolean; onClick: () => void; icon: React.ReactNode; label: string }) {
  return (
    <button onClick={onClick} className={cn("flex items-center gap-1 px-2 py-0.5 rounded text-xs", active ? "bg-bg-muted text-fg" : "text-fg-muted hover:bg-bg-muted")}>
      {icon} {label}
    </button>
  );
}
