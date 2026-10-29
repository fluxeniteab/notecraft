import {
  EditorView,
  crosshairCursor,
  drawSelection,
  dropCursor,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  rectangularSelection,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab, redo, undo } from "@codemirror/commands";
import { searchKeymap } from "@codemirror/search";
import { markdown } from "@codemirror/lang-markdown";
import { json as jsonLanguage } from "@codemirror/lang-json";
import { openSearchPanel } from "@codemirror/search";
import { javascript as jsLanguage } from "@codemirror/lang-javascript";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { bracketMatching, foldGutter, foldKeymap } from "@codemirror/language";
import type { Extension } from "@codemirror/state";
import { capabilitiesFor } from "./documentTypes";
import { ncEditorExtensions } from "./editorTheme";

export function languageFor(path: string): Extension[] {
  switch (capabilitiesFor(path).kind) {
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

export function editorExtensions(path: string): Extension[] {
  const code = capabilitiesFor(path).stats !== "prose";
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    highlightActiveLine(),
    drawSelection(),
    dropCursor(),
    rectangularSelection(),
    crosshairCursor(),
    code ? [] : EditorView.lineWrapping,
    history(),
    autocompletion(),
    closeBrackets(),
    bracketMatching(),
    foldGutter({ openText: "▾", closedText: "▸" }),
    keymap.of([
      ...closeBracketsKeymap,
      ...completionKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...defaultKeymap,
      indentWithTab,
    ]),
    EditorView.contentAttributes.of({ spellcheck: code ? "false" : "true" }),
    ...languageFor(path),
    ...ncEditorExtensions,
  ];
}

/* --------------------------------------------------------- active editor --- */

let activeView: EditorView | null = null;

export function setActiveEditor(view: EditorView | null): void {
  activeView = view;
}

export function focusEditor(): void {
  activeView?.focus();
}

export function undoEdit(): void {
  if (activeView) undo(activeView);
}

export function redoEdit(): void {
  if (activeView) redo(activeView);
}

export function openFind(): void {
  if (activeView) {
    activeView.focus();
    openSearchPanel(activeView);
  }
}

export function selectAllText(): void {
  const view = activeView;
  if (!view) return;
  view.dispatch({ selection: { anchor: 0, head: view.state.doc.length } });
  view.focus();
}

export function wrapSelection(before: string, after = before): void {
  const view = activeView;
  if (!view) return;
  const { from, to } = view.state.selection.main;
  const selected = view.state.sliceDoc(from, to);
  view.dispatch({
    changes: { from, to, insert: `${before}${selected}${after}` },
    selection: { anchor: from + before.length, head: from + before.length + selected.length },
  });
  view.focus();
}

export function toggleLinePrefix(prefix: string): void {
  const view = activeView;
  if (!view) return;
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
        return { from: line.from + Math.max(0, index), to: line.from + Math.max(0, index) + prefix.length, insert: "" };
      }
      return { from: line.from, to: line.from, insert: prefix };
    }),
  });
  view.focus();
}

export function setHeading(level: number): void {
  const view = activeView;
  if (!view) return;
  const { from, to } = view.state.selection.main;
  const first = view.state.doc.lineAt(from);
  const last = view.state.doc.lineAt(to);
  const insert = `${"#".repeat(level)} `;
  const changes = [];
  for (let n = first.number; n <= last.number; n++) {
    const line = view.state.doc.line(n);
    const match = line.text.match(/^(#{1,6})\s+/);
    changes.push(
      match
        ? { from: line.from, to: line.from + match[0].length, insert }
        : { from: line.from, to: line.from, insert }
    );
  }
  view.dispatch({ changes });
  view.focus();
}

export function insertLink(): void {
  const view = activeView;
  if (!view) return;
  const { from, to } = view.state.selection.main;
  const text = view.state.sliceDoc(from, to) || "text";
  view.dispatch({
    changes: { from, to, insert: `[${text}](url)` },
    selection: { anchor: from + text.length + 3, head: from + text.length + 6 },
  });
  view.focus();
}

export function insertText(text: string): void {
  const view = activeView;
  if (!view) return;
  const { from, to } = view.state.selection.main;
  view.dispatch({ changes: { from, to, insert: text }, selection: { anchor: from + text.length } });
  view.focus();
}

export function goToLine(line: number): void {
  const view = activeView;
  if (!view || line < 1) return;
  const target = view.state.doc.line(Math.min(line, view.state.doc.lines));
  view.dispatch({ selection: { anchor: target.from }, scrollIntoView: true });
  view.focus();
}
