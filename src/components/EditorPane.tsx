import { useCallback, useEffect, useRef, useState } from "react";
import { EditorState, Compartment } from "@codemirror/state";
import {
  EditorView,
  keymap,
  placeholder as placeholderExt,
  type ViewUpdate,
} from "@codemirror/view";
import { basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { json as jsonLanguage } from "@codemirror/lang-json";
import { javascript as jsLanguage } from "@codemirror/lang-javascript";
import { toast } from "sonner";
import katex from "katex";
import "katex/dist/katex.min.css";
import { AlertTriangle, Bold, Code, Heading1, Heading2, Italic, Link2, List, ListOrdered, Quote, RotateCw, Strikethrough } from "lucide-react";
import { useAppStore, type Tab } from "../store/appStore";
import { api } from "../lib/tauri";
import { resolveNotePath, baseName } from "../lib/files";
import { registerFlusher } from "../lib/saveRegistry";
import { livePreview } from "../editor/livePreview";
import { ncEditorExtensions } from "../editor/editorTheme";
import { capabilitiesFor, isProse } from "../lib/documentTypes";
import JsonPanel from "./JsonPanel";

const liveCompartment = new Compartment();

/**
 * Pick the CodeMirror language for a document. Markdown is only one of the
 * supported formats: JSON, JavaScript/TypeScript and plain text must not be
 * highlighted as prose.
 */
function languageFor(path: string) {
  const kind = capabilitiesFor(path).kind;
  switch (kind) {
    case "json":
      return [jsonLanguage()];
    case "javascript":
      return [jsLanguage({ jsx: true })];
    case "typescript":
      return [jsLanguage({ jsx: /\.[jt]sx$/i.test(path), typescript: true })];
    case "markdown":
    case "mdx":
      return [markdown()];
    default:
      return [];
  }
}

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

/** Wrap the current selection (or insert a pair at the cursor). */
function wrapSelection(view: EditorView, before: string, after = before): void {
  const { from, to } = view.state.selection.main;
  const selected = view.state.sliceDoc(from, to);
  view.dispatch({
    changes: { from, to, insert: `${before}${selected}${after}` },
    selection: {
      anchor: from + before.length,
      head: from + before.length + selected.length,
    },
  });
  view.focus();
}

/** Toggle a line prefix such as `- ` or `> ` across the selected lines. */
function toggleLinePrefix(view: EditorView, prefix: string): void {
  const { from, to } = view.state.selection.main;
  const first = view.state.doc.lineAt(from);
  const last = view.state.doc.lineAt(to);
  const lines = [];
  for (let n = first.number; n <= last.number; n++) lines.push(view.state.doc.line(n));
  const allPrefixed = lines.every((line) => line.text.trimStart().startsWith(prefix));
  view.dispatch({
    changes: lines.map((line) => {
      if (allPrefixed) {
        const index = line.text.indexOf(prefix);
        return { from: line.from + index, to: line.from + index + prefix.length, insert: "" };
      }
      return { from: line.from, to: line.from, insert: prefix };
    }),
  });
  view.focus();
}

/** Replace any existing heading marker on the selected lines. */
function setHeading(view: EditorView, level: number): void {
  const { from, to } = view.state.selection.main;
  const first = view.state.doc.lineAt(from);
  const last = view.state.doc.lineAt(to);
  const insert = `${"#".repeat(level)} `;
  const changes = [];
  for (let n = first.number; n <= last.number; n++) {
    const line = view.state.doc.line(n);
    const match = line.text.match(/^(#{1,6})\s+/);
    if (match) {
      changes.push({ from: line.from, to: line.from + match[0].length, insert });
    } else {
      changes.push({ from: line.from, to: line.from, insert });
    }
  }
  view.dispatch({ changes });
  view.focus();
}

/** Insert a Markdown link and select the placeholder URL. */
function insertLink(view: EditorView): void {
  const { from, to } = view.state.selection.main;
  const text = view.state.sliceDoc(from, to) || "text";
  view.dispatch({
    changes: { from, to, insert: `[${text}](url)` },
    selection: {
      anchor: from + text.length + 3,
      head: from + text.length + 6,
    },
  });
  view.focus();
}

/** Render $…$ / $$…$$ in the preview without touching code blocks. */
function typesetMath(root: HTMLElement): void {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: (node) => {
      const parent = node.parentElement;
      if (!parent) return NodeFilter.FILTER_REJECT;
      const tag = parent.tagName;
      if (tag === "PRE" || tag === "CODE" || tag === "SCRIPT" || tag === "STYLE" || tag === "TEXTAREA") {
        return NodeFilter.FILTER_REJECT;
      }
      if (parent.closest(".katex, .nc-math")) return NodeFilter.FILTER_REJECT;
      return /\$/.test(node.nodeValue ?? "")
        ? NodeFilter.FILTER_ACCEPT
        : NodeFilter.FILTER_REJECT;
    },
  });
  const textNodes: Text[] = [];
  let current: Node | null;
  while ((current = walker.nextNode())) textNodes.push(current as Text);

  for (const textNode of textNodes) {
    const value = textNode.nodeValue ?? "";
    const parts: Array<string | { tex: string; display: boolean }> = [];
    const re = /\$\$([\s\S]+?)\$\$|\$([^$\n]+?)\$/g;
    let last = 0;
    let match: RegExpExecArray | null;
    while ((match = re.exec(value))) {
      if (match.index > last) parts.push(value.slice(last, match.index));
      parts.push({ tex: (match[1] ?? match[2]).trim(), display: match[1] !== undefined });
      last = match.index + match[0].length;
    }
    if (parts.length === 0) continue;
    if (last < value.length) parts.push(value.slice(last));

    const fragment = document.createDocumentFragment();
    let rendered = false;
    for (const part of parts) {
      if (typeof part === "string") {
        fragment.append(document.createTextNode(part));
        continue;
      }
      const span = document.createElement("span");
      span.className = "nc-math";
      try {
        katex.render(part.tex, span, { throwOnError: false, displayMode: part.display });
        rendered = true;
      } catch {
        span.textContent = `$${part.tex}$`;
      }
      fragment.append(span);
    }
    if (rendered) textNode.replaceWith(fragment);
  }
}

/**
 * One persistent CodeMirror instance per open tab. The pane is hidden rather
 * than unmounted when inactive, so cursor, selection, scroll position and undo
 * history survive tab switches, mode changes and theme changes.
 */
export default function EditorPane({
  tab,
  active,
  onExternalReload,
}: {
  tab: Tab;
  active: boolean;
  onExternalReload?: (path: string, reload: (() => void) | null) => void;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const queueRef = useRef<Promise<void>>(Promise.resolve());
  const lastSavedRef = useRef<string>("");
  const lastSnapshotRef = useRef<string>("");
  const lastWrittenByAppRef = useRef<string | null>(null);
  const externalChangeRef = useRef(false);
  const diskContentRef = useRef<string>("");
  const currentPathRef = useRef(tab.path);
  const saveTimerRef = useRef<number | null>(null);
  const previewTimerRef = useRef<number | null>(null);
  const statsTimerRef = useRef<number | null>(null);
  const modeRef = useRef(tab.mode);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [html, setHtml] = useState("");
  const [editorText, setEditorText] = useState("");
  const [reloadKey, setReloadKey] = useState(0);
  const [externalContent, setExternalContent] = useState<string | null>(null);
  const [showCompare, setShowCompare] = useState(false);
  const [keepEditorRequested, setKeepEditorRequested] = useState(false);
  const theme = useAppStore((s) => s.theme);

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
        if (externalChangeRef.current && !keepEditorRequested) return;
        setStatus("saving");
        try {
          const expectedDiskText = diskContentRef.current;
          if (externalChangeRef.current) {
            await api.writeFileIfUnchanged(currentPathRef.current, text, expectedDiskText);
          } else {
            await api.writeFile(currentPathRef.current, text);
          }
          lastSavedRef.current = text;
          diskContentRef.current = text;
          setExternalContent(null);
          setShowCompare(false);
          setKeepEditorRequested(false);
          externalChangeRef.current = false;
          lastWrittenByAppRef.current = currentPathRef.current;
          window.setTimeout(() => {
            if (lastWrittenByAppRef.current === currentPathRef.current) {
              lastWrittenByAppRef.current = null;
            }
          }, 1000);
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
          const message = String(err);
          if (/file changed on disk/i.test(message)) {
            externalChangeRef.current = true;
            const diskText = await api.readFile(currentPathRef.current).catch(() => null);
            if (diskText !== null) {
              diskContentRef.current = diskText;
              setExternalContent(diskText);
            }
            toast.warning("Save blocked to protect an external edit", {
              description: "Choose whether to reload the disk version or keep your editor version.",
              duration: 8000,
            });
          } else {
            toast.error("Could not save note", { description: message });
          }
        }
      });
      return queueRef.current;
    },
    [keepEditorRequested, setDirty, setStatus, tab.id]
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
        .renderDocument(text, currentPathRef.current)
        .then(setHtml)
        .catch((err) => {
          setHtml("");
          console.error(err);
        });
    }, 180);
  }, []);

  const reloadFromDisk = useCallback(async () => {
    if (useAppStore.getState().tabs.find((item) => item.id === tab.id)?.dirty) return;
    try {
      const diskText = await api.readFile(currentPathRef.current);
      if (viewRef.current?.state.doc.toString() === diskText) {
        setExternalContent(null);
        return;
      }
      diskContentRef.current = diskText;
      externalChangeRef.current = true;
      lastSavedRef.current = diskText;
      setDirty(false);
      setStatus("saved");
      setExternalContent(null);
      setShowCompare(false);
      useAppStore.getState().setTabStats(tab.id, {
        words: countWords(diskText),
        chars: diskText.length,
      });
      if (modeRef.current === "reading" || modeRef.current === "split") {
        void api.renderDocument(diskText, currentPathRef.current).then(setHtml).catch(() => {});
      }
      if (viewRef.current) {
        viewRef.current.dispatch({
          changes: { from: 0, to: viewRef.current.state.doc.length, insert: diskText },
        });
      } else {
        setReloadKey((key) => key + 1);
      }
      toast.info("Note updated from disk", { description: tab.title });
    } catch (error) {
      toast.error("Could not reload the changed note", { description: String(error) });
    }
  }, [setDirty, setStatus, tab.id, tab.title]);

  useEffect(() => {
    onExternalReload?.(tab.path, async () => {
      const path = currentPathRef.current;
      if (lastWrittenByAppRef.current === path) {
        lastWrittenByAppRef.current = null;
        return;
      }
      const dirty = useAppStore.getState().tabs.find((item) => item.id === tab.id)?.dirty;
      if (dirty) {
        externalChangeRef.current = true;
        void api.readFile(path).then((text) => {
          diskContentRef.current = text;
          setExternalContent(text);
        }).catch(() => setExternalContent(null));
        return;
      }
      await reloadFromDisk();
    });
    return () => onExternalReload?.(tab.path, null);
  }, [onExternalReload, reloadFromDisk, tab.id, tab.path]);

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
        diskContentRef.current = text;
        lastSnapshotRef.current = text;
        setEditorText(text);
        useAppStore.getState().setTabStats(tab.id, {
          words: countWords(text),
          chars: text.length,
          lines: text.split(/\r?\n/).length,
        });
        // Structured formats get a validity badge in the status bar.
        if (capabilitiesFor(tab.path).validator === "json") {
          void api
            .jsonAnalyze(text)
            .then((result) =>
              useAppStore.getState().setTabStats(tab.id, {
                validation: result.valid
                  ? { valid: true }
                  : {
                      valid: false,
                      line: result.error?.line,
                      column: result.error?.column,
                      message: result.error?.message,
                    },
              })
            )
            .catch(() => {});
        }
        useAppStore.getState().setCursor(tab.id, { line: 1, col: 1 });
        if (modeRef.current === "reading" || modeRef.current === "split") {
          void api
            .renderDocument(text, currentPathRef.current)
            .then((next) => !cancelled && setHtml(next))
            .catch(() => {});
        }
        const view = new EditorView({
          parent: hostRef.current!,
          state: EditorState.create({
            doc: text,
            extensions: [
              basicSetup,
              keymap.of([
                { key: "Mod-b", run: (view) => { wrapSelection(view, "**"); return true; } },
                { key: "Mod-i", run: (view) => { wrapSelection(view, "*"); return true; } },
                { key: "Mod-e", run: (view) => { wrapSelection(view, "`"); return true; } },
                { key: "Mod-Shift-x", run: (view) => { wrapSelection(view, "~~"); return true; } },
              ]),
              ...languageFor(tab.path),
              ncEditorExtensions,
              EditorView.lineWrapping,
              liveCompartment.of(
                modeRef.current === "live" && isProse(tab.path) ? livePreview : []
              ),
              placeholderExt("Start writing…"),
              EditorView.contentAttributes.of({ "aria-label": `Editing ${baseName(tab.path)}` }),
              EditorView.updateListener.of((update: ViewUpdate) => {
                if (update.docChanged && externalChangeRef.current) {
                  externalChangeRef.current = false;
                  return;
                }
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
                  if (externalChangeRef.current) externalChangeRef.current = false;
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
                      lines: value.split(/\r?\n/).length,
                    });
                    setEditorText(value);
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
      effects: liveCompartment.reconfigure(
        tab.mode === "live" && isProse(tab.path) ? livePreview : []
      ),
    });
    if (tab.mode === "reading" || tab.mode === "split") {
      const text = view.state.doc.toString();
      if (html) renderPreview(text);
      else void api.renderDocument(text, currentPathRef.current).then(setHtml).catch(() => {});
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

  // Give rendered headings stable ids so `#heading` links can scroll to them,
  // typeset math and render mermaid diagrams.
  useEffect(() => {
    const root = previewRef.current;
    if (!root || !html) return;
    let cancelled = false;

    root.querySelectorAll("h1, h2, h3, h4, h5, h6").forEach((heading) => {
      if (!heading.id) heading.id = slugify(heading.textContent ?? "");
    });
    typesetMath(root);

    root.querySelectorAll<HTMLPreElement>("pre > code.language-mermaid").forEach((code) => {
      const source = code.textContent ?? "";
      const pre = code.parentElement;
      if (!pre) return;
      const host = document.createElement("div");
      host.className = "nc-mermaid";
      pre.replaceWith(host);
      void (async () => {
        try {
          const { default: mermaid } = await import("mermaid");
          mermaid.initialize({
            startOnLoad: false,
            theme: theme === "dark" ? "dark" : "neutral",
            securityLevel: "strict",
          });
          const id = `nc-mmd-${Math.random().toString(36).slice(2, 9)}`;
          const { svg } = await mermaid.render(id, source);
          if (cancelled || !host.isConnected) return;
          host.innerHTML = svg;
        } catch (error) {
          console.warn("mermaid render failed", error);
          if (host.isConnected) host.replaceWith(pre);
        }
      })();
    });

    return () => {
      cancelled = true;
    };
  }, [html, theme]);

  // Insert recovered content requested by the history panel at the cursor.
  useEffect(() => {
    let lastNonce = 0;
    const process = () => {
      const current = useAppStore.getState().insertRequest;
      if (!current || current.nonce === lastNonce) return;
      lastNonce = current.nonce;
      if (current.path !== currentPathRef.current) return;
      useAppStore.setState({ insertRequest: null });
      const view = viewRef.current;
      if (!view) return;
      if (modeRef.current === "reading") useAppStore.getState().setTabMode(tab.id, "live");
      const { from, to } = view.state.selection.main;
      view.dispatch({
        changes: { from, to, insert: current.text },
        selection: { anchor: from + current.text.length },
        scrollIntoView: true,
      });
      view.focus();
      toast.success("Selection recovered into the note");
    };
    process();
    const unsubscribe = useAppStore.subscribe(process);
    return unsubscribe;
  }, [tab.id]);

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

  const caps = capabilitiesFor(tab.path);
  const previewVisible = tab.mode === "reading" || tab.mode === "split";
  const editorVisible = tab.mode !== "reading";

  /** Replace the whole document from a structured action (JSON format). */
  const replaceDocument = useCallback((next: string) => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: next },
      selection: { anchor: 0 },
    });
    setEditorText(next);
    view.focus();
  }, []);

  const format = (action: (view: EditorView) => void) => {
    const view = viewRef.current;
    if (view) action(view);
  };

  const showFormatBar = editorVisible && isProse(tab.path) && tab.mode !== "reading";

  return (
    <div className="nc-pane" data-active={active} aria-hidden={!active}>
      {showFormatBar && (
        <div className="nc-format-bar" role="toolbar" aria-label="Formatting">
          <button type="button" className="nc-format-btn" title="Bold (Ctrl+B)" aria-label="Bold" onClick={() => format((view) => wrapSelection(view, "**"))}>
            <Bold size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Italic (Ctrl+I)" aria-label="Italic" onClick={() => format((view) => wrapSelection(view, "*"))}>
            <Italic size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Strikethrough (Ctrl+Shift+X)" aria-label="Strikethrough" onClick={() => format((view) => wrapSelection(view, "~~"))}>
            <Strikethrough size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Inline code (Ctrl+E)" aria-label="Inline code" onClick={() => format((view) => wrapSelection(view, "`"))}>
            <Code size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Link" aria-label="Insert link" onClick={() => format(insertLink)}>
            <Link2 size={13} />
          </button>
          <span className="nc-format-sep" aria-hidden="true" />
          <button type="button" className="nc-format-btn" title="Heading 1" aria-label="Heading 1" onClick={() => format((view) => setHeading(view, 1))}>
            <Heading1 size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Heading 2" aria-label="Heading 2" onClick={() => format((view) => setHeading(view, 2))}>
            <Heading2 size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Quote" aria-label="Quote" onClick={() => format((view) => toggleLinePrefix(view, "> "))}>
            <Quote size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Bulleted list" aria-label="Bulleted list" onClick={() => format((view) => toggleLinePrefix(view, "- "))}>
            <List size={13} />
          </button>
          <button type="button" className="nc-format-btn" title="Numbered list" aria-label="Numbered list" onClick={() => format((view) => toggleLinePrefix(view, "1. "))}>
            <ListOrdered size={13} />
          </button>
        </div>
      )}
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
            ) : caps.preview === "json" ? (
              <JsonPanel content={editorText} onApply={replaceDocument} />
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

      {externalContent !== null && (
        <div className="nc-conflict-backdrop" role="dialog" aria-modal="true" aria-labelledby={`conflict-title-${tab.id}`}>
          <section className="nc-conflict-card">
            <div className="nc-conflict-eyebrow">External edit detected</div>
            <h2 id={`conflict-title-${tab.id}`} className="nc-conflict-title">{tab.title} changed outside NoteCraft</h2>
            <p className="nc-conflict-copy">Your unsaved editor version is safe. Compare both versions, reload the disk copy, or explicitly keep yours.</p>
            {showCompare && (
              <div className="nc-conflict-compare">
                <div><strong>Your editor</strong><pre>{viewRef.current?.state.doc.toString() ?? ""}</pre></div>
                <div><strong>On disk</strong><pre>{externalContent}</pre></div>
              </div>
            )}
            <div className="nc-conflict-actions">
              <button type="button" className="nc-btn nc-btn-ghost" onClick={() => setShowCompare((value) => !value)}>{showCompare ? "Hide comparison" : "Compare versions"}</button>
              <div className="flex-1" />
              <button type="button" className="nc-btn nc-btn-outline" onClick={() => {
                const diskVersion = externalContent;
                diskContentRef.current = diskVersion;
                lastSavedRef.current = diskVersion;
                externalChangeRef.current = false;
                setDirty(false);
                setStatus("saved");
                setExternalContent(null);
                setShowCompare(false);
                viewRef.current?.dispatch({ changes: { from: 0, to: viewRef.current.state.doc.length, insert: diskVersion } });
                if (modeRef.current === "reading" || modeRef.current === "split")
                  void api.renderDocument(diskVersion, currentPathRef.current).then(setHtml).catch(() => {});
              }}>Reload disk version</button>
              <button type="button" className="nc-btn nc-btn-primary" onClick={() => { diskContentRef.current = externalContent; externalChangeRef.current = true; setKeepEditorRequested(true); void flush(); }}>Keep editor version</button>
            </div>
          </section>
        </div>
      )}

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
