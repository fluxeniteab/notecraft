import { HighlightStyle, syntaxHighlighting, indentOnInput } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { EditorView } from "@codemirror/view";

export const ncEditorTheme = EditorView.theme(
  {
    "&": {
      height: "100%",
      backgroundColor: "transparent",
      color: "var(--nc-fg)",
    },
    "&.cm-focused": {
      outline: "none",
    },
    ".cm-scroller": {
      fontFamily: "var(--nc-font-editor)",
      overflow: "auto",
    },
    ".cm-content": {
      caretColor: "var(--nc-accent)",
    },
    ".cm-gutters": {
      display: "none",
    },
    ".cm-activeLine": {
      backgroundColor: "var(--nc-editor-active-line)",
    },
    ".cm-selectionBackground": {
      backgroundColor: "var(--nc-editor-selection) !important",
    },
    "&.cm-focused .cm-selectionBackground": {
      backgroundColor: "var(--nc-editor-selection-focus) !important",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: "var(--nc-accent)",
    },
    ".cm-tooltip": {
      backgroundColor: "var(--nc-elev)",
      border: "1px solid var(--nc-border-strong)",
      color: "var(--nc-fg)",
      borderRadius: "var(--nc-r-sm)",
      boxShadow: "var(--nc-shadow)",
    },
  },
  { dark: false }
);

export const ncHighlightStyle = HighlightStyle.define([
  {
    tag: [tags.heading1, tags.heading2, tags.heading3, tags.heading4, tags.heading5, tags.heading6, tags.heading],
    color: "var(--nc-fg)",
    fontWeight: "600",
    fontFamily: "var(--nc-font-title)",
  },
  { tag: tags.strong, color: "var(--nc-fg)", fontWeight: "700" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strikethrough, textDecoration: "line-through", color: "var(--nc-fg-muted)" },
  { tag: tags.link, color: "var(--nc-accent)", textDecoration: "underline" },
  { tag: tags.url, color: "var(--nc-fg-muted)" },
  { tag: tags.monospace, fontFamily: "var(--nc-font-mono)", color: "var(--nc-fg)" },
  { tag: tags.meta, color: "var(--nc-fg-faint)" },
  { tag: tags.processingInstruction, color: "var(--nc-fg-faint)" },
  { tag: tags.comment, color: "var(--nc-fg-muted)", fontStyle: "italic" },
  { tag: tags.quote, color: "var(--nc-fg-secondary)" },
  { tag: tags.contentSeparator, color: "var(--nc-border-strong)" },
  { tag: [tags.keyword, tags.atom, tags.bool], color: "var(--nc-blue)" },
  { tag: [tags.string, tags.regexp], color: "var(--nc-success)" },
  { tag: [tags.number, tags.integer, tags.float], color: "var(--nc-warning)" },
  { tag: [tags.typeName, tags.className], color: "var(--nc-accent)" },
  { tag: [tags.tagName, tags.angleBracket], color: "var(--nc-blue)" },
  { tag: [tags.attributeName, tags.attributeValue], color: "var(--nc-warning)" },
  { tag: [tags.propertyName, tags.definition(tags.propertyName)], color: "var(--nc-accent)" },
  { tag: tags.invalid, color: "var(--nc-danger)" },
]);

export const ncEditorExtensions = [ncEditorTheme, syntaxHighlighting(ncHighlightStyle), indentOnInput()];
