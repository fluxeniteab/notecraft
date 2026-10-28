import { useCallback, useEffect, useRef, useState } from "react";
import { EditorState, Compartment } from "@codemirror/state";
import {
  EditorView,
  placeholder as placeholderExt,
  type ViewUpdate,
} from "@codemirror/view";
import { basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { toast } from "sonner";
import { AlertTriangle, RotateCw } from "lucide-react";
import { useAppStore, type Tab } from "../store/appStore";
import { api } from "../lib/tauri";
import { resolveNotePath, baseName } from "../lib/files";
import { registerFlusher } from "../lib/saveRegistry";
import { livePreview } from "../editor/livePreview";
import { ncEditorExtensions } from "../editor/editorTheme";

const liveCompartment = new Compartment();

function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, "")
    .replace(/\s+/g, "-");
}

function countWords(text: string): number {
  const trimmed = text.trim();
  return trimmed ? trimmed.split(/\s+/).length : 0;
}

/**
 * One persistent CodeMirror instance per open tab. The pane is hidden rather
 * than unmounted when inactive, so cursor, selection, scroll position and undo
 * history survive tab switches, mode changes and theme changes.
 */
export default function EditorPane({ tab, active }: { tab: Tab; active: boolean }) {
  const hostRef = useRef<HTMLDivElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const queueRef = useRef<Promise<void>>(Promise.resolve());
  const lastSavedRef = useRef<string>("");
  const lastSnapshotRef = useRef<string>("");
  const currentPathRef = useRef(tab.path);
  const saveTimerRef = useRef<number | null>(null);
  const previewTimerRef = useRef<number | null>(null);
  const statsTimerRef = useRef<number | null>(null);
  const modeRef = useRef(tab.mode);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [html, setHtml] = useState("");
  const [reloadKey, setReloadKey] = useState(0);

  modeRef.current = tab.mode;
  currentPathRef.current = tab.path;

  const setStatus = useCallback(
    (status: "idle" | "dirty" | "saving" | "saved" | "error") => {
      useAppStore.getState().setSaveStatus(tab.id, status);
    },
    [tab.id]
  );

  const setDirty = useCallback(
    (dirty: boolean) => {
      useAppStore.getState().setTabDirty(tab.id, dirty);
    },
    [tab.id]
  );

  const enqueueSave = useCallback(
    (explicitText?: string) => {
      const view = viewRef.current;
      const text = explicitText ?? view?.state.doc.toString() ?? "";
      const path = currentPathRef.current;
      if (!path) return queueRef.current;
      if (text === lastSavedRef.current) {
        if (useAppStore.getState().saveStatus[tab.id] !== "saved") setStatus("saved");
        return queueRef.current;
      }
      queueRef.current = queueRef.current.then(async () => {
        if (text === lastSavedRef.current) return;
        setStatus("saving");
        try {
          await api.writeFile(currentPathRef.current, text);
          lastSavedRef.current = text;
          const current = viewRef.current?.state.doc.toString() ?? text;
          if (current === text) {
            setDirty(false);
            setStatus("saved");
          } else {
            setDirty(true);
            setStatus("dirty");
          }
        } catch (err) {
          setDirty(true);
          setStatus("error");
          toast.error("Could not save note", { description: String(err) });
        }
      });
      return queueRef.current;
    },
    [setDirty, setStatus, tab.id]
  );

  const flush = useCallback(async () => {
    if (saveTimerRef.current) {
      window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
    await enqueueSave();
  }, [enqueueSave]);

  useEffect(() => {
    const unregister = registerFlusher(tab.id, flush);
    return unregister;
  }, [flush, tab.id]);

  const renderPreview = useCallback((text: string) => {
    if (previewTimerRef.current) window.clearTimeout(previewTimerRef.current);
    previewTimerRef.current = window.setTimeout(() => {
      previewTimerRef.current = null;
      if (modeRef.current !== "reading" && modeRef.current !== "split") return;
      api
        .renderHtml(text)
        .then(setHtml)
        .catch((err) => {
          setHtml("");
          console.error(err);
        });
    }, 180);
  }, []);

  // Load the file and build the editor exactly once per tab.
  useEffect(() => {
    let cancelled = false;
    viewRef.current?.destroy();
    viewRef.current = null;
    setLoading(true);
    setError(null);
    api
      .readFile(tab.path)
      .then((text) => {
        if (cancelled) return;
        lastSavedRef.current = text;
        lastSnapshotRef.current = text;
        useAppStore.getState().setTabStats(tab.id, {
          words: countWords(text),
          chars: text.length,
        });
        useAppStore.getState().setCursor(tab.id, { line: 1, col: 1 });
        if (modeRef.current === "reading" || modeRef.current === "split") {
          void api.renderHtml(text).then((next) => !cancelled && setHtml(next)).catch(() => {});
        }
        const view = new EditorView({
          parent: hostRef.current!,
          state: EditorState.create({
            doc: text,
            extensions: [
              basicSetup,
              markdown(),
              ncEditorExtensions,
              EditorView.lineWrapping,
              liveCompartment.of(modeRef.current === "live" ? livePreview : []),
              placeholderExt("Start writing…"),
              EditorView.contentAttributes.of({ "aria-label": `Editing ${baseName(tab.path)}` }),
              EditorView.updateListener.of((update: ViewUpdate) => {
                if (update.selectionSet) {
                  const head = update.state.selection.main.head;
                  const line = update.state.doc.lineAt(head);
                  const cursor = { line: line.number, col: head - line.from + 1 };
                  const previous = useAppStore.getState().cursors[tab.id];
                  if (previous?.line !== cursor.line || previous?.col !== cursor.col) {
                    useAppStore.getState().setCursor(tab.id, cursor);
                  }
                }
                if (update.docChanged) {
                  const value = update.state.doc.toString();
                  if (useAppStore.getState().saveStatus[tab.id] !== "dirty") {
                    setDirty(true);
                    setStatus("dirty");
                  }
                  const delay = Math.max(
                    200,
                    useAppStore.getState().settings?.autosave_ms ?? 800
                  );
                  if (saveTimerRef.current) window.clearTimeout(saveTimerRef.current);
                  saveTimerRef.current = window.setTimeout(() => {
                    saveTimerRef.current = null;
                    void enqueueSave(value);
                  }, delay);

                  if (statsTimerRef.current) window.clearTimeout(statsTimerRef.current);
                  statsTimerRef.current = window.setTimeout(() => {
                    statsTimerRef.current = null;
                    useAppStore.getState().setTabStats(tab.id, {
                      words: countWords(value),
                      chars: value.length,
                    });
                  }, 320);

                  if (modeRef.current === "reading" || modeRef.current === "split") {
                    renderPreview(value);
                  }
                }
              }),
              EditorView.domEventHandlers({
                mousedown(event, currentView) {
                  if (!(event.metaKey || event.ctrlKey)) return false;
                  const pos = currentView.posAtCoords({ x: event.clientX, y: event.clientY });
                  if (pos == null) return false;
                  const line = currentView.state.doc.lineAt(pos);
                  const wiki = /\[\[([^\]]+)\]\]/g;
                  let match: RegExpExecArray | null;
                  while ((match = wiki.exec(line.text))) {
                    const start = line.from + match.index;
                    const end = start + match[0].length;
                    if (pos < start || pos > end) continue;
                    event.preventDefault();
                    const resolved = resolveNotePath(
                      match[1],
                      useAppStore.getState().allFiles,
                      currentPathRef.current
                    );
                    if (resolved) useAppStore.getState().openTab(resolved);
                    else toast.info(`No note matches “${match[1].split("|")[0]}”`);
                    return true;
                  }
                  return false;
                },
              }),
            ],
          }),
        });
        viewRef.current = view;
        setLoading(false);
      })
      .catch((err) => {
        if (cancelled) return;
        setError(String(err).replace(/^Error:\s*/, ""));
        setLoading(false);
      });

    return () => {
      cancelled = true;
      if (saveTimerRef.current) window.clearTimeout(saveTimerRef.current);
      if (previewTimerRef.current) window.clearTimeout(previewTimerRef.current);
      if (statsTimerRef.current) window.clearTimeout(statsTimerRef.current);
      saveTimerRef.current = null;
      previewTimerRef.current = null;
      statsTimerRef.current = null;
      viewRef.current?.destroy();
      viewRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab.id, reloadKey]);

  // Swap the live-preview extension without recreating the editor.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: liveCompartment.reconfigure(tab.mode === "live" ? livePreview : []),
    });
    if (tab.mode === "reading" || tab.mode === "split") {
      const text = view.state.doc.toString();
      if (html) renderPreview(text);
      else void api.renderHtml(text).then(setHtml).catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab.mode]);

  // Focus-mode / split resize can change the viewport; CodeMirror needs a
  // measure to keep selection and decorations aligned.
  useEffect(() => {
    if (!active) return;
    const id = window.requestAnimationFrame(() => {
      viewRef.current?.requestMeasure();
      if (tab.mode !== "reading") viewRef.current?.focus();
    });
    return () => window.cancelAnimationFrame(id);
  }, [active, tab.mode]);

  // Periodic automatic snapshots (backend keeps the retention window).
  useEffect(() => {
    const minutes = Math.max(
      1,
      useAppStore.getState().settings?.snapshot_interval_min ?? 5
    );
    const id = window.setInterval(
      () => {
        const view = viewRef.current;
        if (!view) return;
        const text = view.state.doc.toString();
        if (!text.trim() || text === lastSnapshotRef.current) return;
        lastSnapshotRef.current = text;
        api
          .snapshotNow(currentPathRef.current, "auto")
          .catch((err) => console.error("snapshot failed", err));
      },
      minutes * 60_000
    );
    return () => window.clearInterval(id);
  }, []);

  // Navigate to a heading / line requested by the outline, search or links.
  useEffect(() => {
    const request = useAppStore.getState().gotoRequest;
    if (!request) return;
    const handler = () => {
      const current = useAppStore.getState().gotoRequest;
      if (!current || current.path !== currentPathRef.current) return;
      const view = viewRef.current;
      if (!view) return;
      if (modeRef.current === "reading") {
        useAppStore.getState().setTabMode(tab.id, "live");
      }
      window.requestAnimationFrame(() => {
        const lineNumber = Math.max(1, Math.min(view.state.doc.lines, current.line));
        const info = view.state.doc.line(lineNumber);
        view.dispatch({
          selection: { anchor: info.from },
          effects: EditorView.scrollIntoView(info.from, { y: "center" }),
        });
        view.focus();
      });
    };
    handler();
    const unsubscribe = useAppStore.subscribe((state) => {
      if (state.gotoRequest?.nonce !== request.nonce) handler();
    });
    return unsubscribe;
  }, [tab.id]);

  // Give rendered headings stable ids so `#heading` links can scroll to them.
  useEffect(() => {
    const root = previewRef.current;
    if (!root || !html) return;
    root.querySelectorAll("h1, h2, h3, h4, h5, h6").forEach((heading) => {
      if (!heading.id) heading.id = slugify(heading.textContent ?? "");
    });
  }, [html]);

  const scrollToHeading = useCallback((hash: string) => {
    const root = previewRef.current;
    if (!root) return;
    const target = decodeURIComponent(hash);
    const direct = root.querySelector(`#${CSS.escape(slugify(target))}`);
    if (direct) {
      direct.scrollIntoView({ block: "start", behavior: "smooth" });
      return;
    }
    const headings = Array.from(root.querySelectorAll("h1, h2, h3, h4, h5, h6"));
    const match = headings.find(
      (heading) => heading.textContent?.trim().toLowerCase() === target.trim().toLowerCase()
    );
    match?.scrollIntoView({ block: "start", behavior: "smooth" });
  }, []);

  const handlePreviewClick = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      const anchor = (event.target as HTMLElement).closest("a");
      if (!anchor) return;
      const href = anchor.getAttribute("href");
      if (!href) return;
      event.preventDefault();

      if (/^(https?:|mailto:)/i.test(href)) {
        void (async () => {
          try {
            const { openUrl } = await import("@tauri-apps/plugin-opener");
            await openUrl(href);
          } catch {
            window.open(href, "_blank", "noopener,noreferrer");
          }
        })();
        return;
      }

      const [rawPath, rawHash] = href.split("#");
      if (!rawPath) {
        if (rawHash) scrollToHeading(rawHash);
        return;
      }

      const resolved = resolveNotePath(
        decodeURIComponent(rawPath),
        useAppStore.getState().allFiles,
        currentPathRef.current
      );
      if (!resolved) {
        toast.info(`No note matches “${decodeURIComponent(rawPath)}”`);
        return;
      }

      useAppStore.getState().openTab(resolved);
      if (!rawHash) return;

      const hash = rawHash;
      void api
        .readFile(resolved)
        .then((content) => {
          const wanted = decodeURIComponent(hash).trim().toLowerCase();
          const lines = content.split(/\r?\n/);
          const index = lines.findIndex((line) => {
            const match = /^#{1,6}\s+(.*)$/.exec(line.trim());
            return match?.[1]?.trim().toLowerCase() === wanted;
          });
          useAppStore
            .getState()
            .gotoLine(resolved, index >= 0 ? index + 1 : 1);
        })
        .catch(() => {});
    },
    [scrollToHeading]
  );

  const previewVisible = tab.mode === "reading" || tab.mode === "split";
  const editorVisible = tab.mode !== "reading";

  return (
    <div className="nc-pane" data-active={active} aria-hidden={!active}>
      <div className="nc-pane-split">
        <div
          className="nc-pane-editor"
          data-mode={tab.mode}
          data-hidden={!editorVisible}
        >
          <div ref={hostRef} className="nc-editor-host" />
        </div>
        {previewVisible && (
          <div className="nc-preview-wrap nc-scroll" data-mode={tab.mode}>
            {error ? (
              <div className="nc-empty-editor">
                <div className="nc-empty-orb text-danger">
                  <AlertTriangle size={20} />
                </div>
                <div className="text-[13px] font-medium text-fg">This note could not be opened</div>
                <p className="max-w-sm text-[11.5px] leading-relaxed">{error}</p>
                <button
                  type="button"
                  className="nc-btn nc-btn-outline mt-2"
                  onClick={() => setReloadKey((key) => key + 1)}
                >
                  <RotateCw size={13} /> Try again
                </button>
              </div>
            ) : (
              <article
                ref={previewRef}
                className="nc-preview nc-prose"
                onClick={handlePreviewClick}
                dangerouslySetInnerHTML={{ __html: html }}
              />
            )}
          </div>
        )}
      </div>

      {!editorVisible && !previewVisible && null}

      {loading && (
        <div className="nc-loading-overlay">
          <div className="flex flex-col items-center gap-2 text-[11.5px] text-fg-muted">
            <span className="nc-loading-bar w-32" />
            <span>Opening {tab.title}…</span>
          </div>
        </div>
      )}

      {error && editorVisible && (
        <div className="nc-loading-overlay">
          <div className="nc-empty-editor">
            <div className="nc-empty-orb text-danger">
              <AlertTriangle size={20} />
            </div>
            <div className="text-[13px] font-medium text-fg">This note could not be opened</div>
            <p className="max-w-sm text-[11.5px] leading-relaxed">{error}</p>
            <button
              type="button"
              className="nc-btn nc-btn-outline mt-2"
              onClick={() => setReloadKey((key) => key + 1)}
            >
              <RotateCw size={13} /> Try again
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
