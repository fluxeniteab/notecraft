import { syntaxTree } from "@codemirror/language";
import { RangeSetBuilder, type Extension } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import type { SyntaxNode } from "@lezer/common";

interface Span {
  from: number;
  to: number;
}

function children(node: SyntaxNode, name: string): Span[] {
  const out: Span[] = [];
  let child = node.firstChild;
  while (child) {
    if (child.name === name) out.push({ from: child.from, to: child.to });
    child = child.nextSibling;
  }
  return out;
}

function lineRange(view: EditorView, from: number, to: number): Array<{ from: number; to: number }> {
  const lines: Array<{ from: number; to: number }> = [];
  const doc = view.state.doc;
  let line = doc.lineAt(from);
  while (line.from <= to) {
    lines.push({ from: line.from, to: line.to });
    if (line.to >= doc.length) break;
    line = doc.line(line.number + 1);
  }
  return lines;
}

function buildDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  // Collect into an array first: line decorations and mark decorations are
  // produced out of document order while walking the syntax tree.
  const ranges: Array<{ from: number; to: number; deco: Decoration }> = [];
  const seenLines = new Set<number>();
  const { state } = view;
  const selection = state.selection.main;
  const cursorFrom = Math.min(selection.from, selection.to);
  const cursorTo = Math.max(selection.from, selection.to);
  const overlaps = (from: number, to: number) => cursorFrom <= to && cursorTo >= from;

  const addLine = (from: number, className: string) => {
    if (seenLines.has(from)) return;
    seenLines.add(from);
    ranges.push({ from, to: from, deco: Decoration.line({ class: className }) });
  };

  const addReplace = (span: Span) => {
    if (span.to > span.from) {
      ranges.push({ from: span.from, to: span.to, deco: Decoration.replace({}) });
    }
  };

  const tree = syntaxTree(state);
  for (const visible of view.visibleRanges) {
    tree.iterate({
      from: visible.from,
      to: visible.to,
      enter: (node) => {
        const name = node.name;
        const from = node.from;
        const to = node.to;
        const active = overlaps(from, to);

        if (/^ATXHeading[1-6]$/.test(name) || /^SetextHeading[12]$/.test(name)) {
          const level = Number(name.slice(-1));
          const firstLine = state.doc.lineAt(from);
          addLine(firstLine.from, `cm-md-h${Math.min(level, 4)}`);
          if (!active) {
            for (const mark of children(node.node, "HeaderMark")) {
              let end = mark.to;
              if (state.doc.sliceString(end, end + 1) === " ") end += 1;
              addReplace({ from: mark.from, to: Math.min(end, firstLine.to) });
            }
          }
          return;
        }

        if (name === "StrongEmphasis" || name === "Emphasis" || name === "Strikethrough") {
          const markName = name === "Strikethrough" ? "StrikethroughMark" : "EmphasisMark";
          const marks = children(node.node, markName);
          if (marks.length >= 2) {
            const contentFrom = marks[0].to;
            const contentTo = marks[marks.length - 1].from;
            const className =
              name === "StrongEmphasis" ? "cm-md-strong" : name === "Emphasis" ? "cm-md-em" : "cm-md-strike";
            if (contentFrom < contentTo) {
              ranges.push({ from: contentFrom, to: contentTo, deco: Decoration.mark({ class: className }) });
            }
            if (!active) marks.forEach(addReplace);
          }
          return;
        }

        if (name === "InlineCode") {
          const marks = children(node.node, "CodeMark");
          if (marks.length >= 2) {
            const contentFrom = marks[0].to;
            const contentTo = marks[marks.length - 1].from;
            if (contentFrom < contentTo) {
              ranges.push({
                from: contentFrom,
                to: contentTo,
                deco: Decoration.mark({ class: "cm-md-code" }),
              });
            }
            if (!active) marks.forEach(addReplace);
          }
          return;
        }

        if (name === "Link") {
          const marks = children(node.node, "LinkMark");
          if (marks.length >= 2) {
            const labelFrom = marks[0].to;
            const labelTo = marks[1].from;
            if (labelFrom < labelTo) {
              ranges.push({
                from: labelFrom,
                to: labelTo,
                deco: Decoration.mark({ class: "cm-md-link" }),
              });
            }
            if (!active) {
              for (const mark of marks) addReplace(mark);
              for (const url of children(node.node, "URL")) addReplace(url);
              for (const title of children(node.node, "LinkTitle")) addReplace(title);
            }
          }
          return;
        }

        if (name === "QuoteMark") {
          const line = state.doc.lineAt(from);
          addLine(line.from, "cm-md-quote");
          if (!active) {
            let end = to;
            if (state.doc.sliceString(end, end + 1) === " ") end += 1;
            addReplace({ from, to: Math.min(end, line.to) });
          }
          return;
        }

        if (name === "FencedCode" || name === "CodeBlock") {
          for (const line of lineRange(view, Math.max(from, visible.from), Math.min(to, visible.to))) {
            addLine(line.from, "cm-md-codeblock");
          }
          return;
        }

        if (name === "Table") {
          for (const line of lineRange(view, Math.max(from, visible.from), Math.min(to, visible.to))) {
            addLine(line.from, "cm-md-table");
          }
        }
      },
    });
  }

  ranges.sort((a, b) => (a.from === b.from ? a.to - b.to : a.from - b.from));
  for (const range of ranges) builder.add(range.from, range.to, range.deco);
  return builder.finish();
}

class LivePreviewPlugin {
  decorations: DecorationSet;

  constructor(view: EditorView) {
    this.decorations = buildDecorations(view);
  }

  update(update: ViewUpdate) {
    if (update.docChanged || update.selectionSet || update.viewportChanged || update.geometryChanged) {
      this.decorations = buildDecorations(update.view);
    }
  }
}

/**
 * Obsidian-style live preview: markdown markers are hidden on inactive lines
 * while the rendered emphasis, links, inline code and headings stay visible.
 * Moving the cursor into a node reveals its raw syntax again.
 */
export const livePreview: Extension = ViewPlugin.fromClass(LivePreviewPlugin, {
  decorations: (plugin) => plugin.decorations,
});
