/**
 * Frontend half of the document-format registry.
 *
 * The Rust side (`src-tauri/src/doc_types.rs`) is the source of truth for how a
 * file is rendered, indexed and validated. This module mirrors the same table so
 * the UI can pick an editor language, statistics mode and starter template
 * without a round trip, and so file creation never assumes Markdown.
 */

export type DocKind =
  | "markdown"
  | "mdx"
  | "json"
  | "yaml"
  | "html"
  | "css"
  | "javascript"
  | "typescript"
  | "plain-text";

export type PreviewMode = "prose" | "json" | "source" | "none";
export type StatMode = "prose" | "data" | "code";
export type Validator = "json" | "yaml" | "mdx";

export interface DocCapabilities {
  kind: DocKind;
  label: string;
  extension: string;
  /** CodeMirror language id. */
  language: string;
  preview: PreviewMode;
  stats: StatMode;
  validator: Validator | null;
  indexable: boolean;
  hasLinks: boolean;
}

const TABLE: Record<DocKind, DocCapabilities> = {
  markdown: {
    kind: "markdown",
    label: "Markdown",
    extension: "md",
    language: "markdown",
    preview: "prose",
    stats: "prose",
    validator: null,
    indexable: true,
    hasLinks: true,
  },
  mdx: {
    kind: "mdx",
    label: "MDX",
    extension: "mdx",
    language: "mdx",
    preview: "prose",
    stats: "prose",
    validator: "mdx",
    indexable: true,
    hasLinks: true,
  },
  json: {
    kind: "json",
    label: "JSON",
    extension: "json",
    language: "json",
    preview: "json",
    stats: "data",
    validator: "json",
    indexable: true,
    hasLinks: false,
  },
  yaml: {
    kind: "yaml",
    label: "YAML",
    extension: "yaml",
    language: "yaml",
    preview: "source",
    stats: "data",
    validator: "yaml",
    indexable: true,
    hasLinks: false,
  },
  html: {
    kind: "html",
    label: "HTML",
    extension: "html",
    language: "html",
    preview: "source",
    stats: "code",
    validator: null,
    indexable: true,
    hasLinks: false,
  },
  css: {
    kind: "css",
    label: "CSS",
    extension: "css",
    language: "css",
    preview: "source",
    stats: "code",
    validator: null,
    indexable: true,
    hasLinks: false,
  },
  javascript: {
    kind: "javascript",
    label: "JavaScript",
    extension: "js",
    language: "javascript",
    preview: "source",
    stats: "code",
    validator: null,
    indexable: true,
    hasLinks: false,
  },
  typescript: {
    kind: "typescript",
    label: "TypeScript",
    extension: "ts",
    language: "typescript",
    preview: "source",
    stats: "code",
    validator: null,
    indexable: true,
    hasLinks: false,
  },
  "plain-text": {
    kind: "plain-text",
    label: "Text",
    extension: "txt",
    language: "text",
    preview: "source",
    stats: "code",
    validator: null,
    indexable: true,
    hasLinks: false,
  },
};

const EXTENSION_TO_KIND: Record<string, DocKind> = {
  md: "markdown",
  markdown: "markdown",
  mdown: "markdown",
  mkd: "markdown",
  mdx: "mdx",
  json: "json",
  jsonc: "json",
  yaml: "yaml",
  yml: "yaml",
  html: "html",
  htm: "html",
  css: "css",
  js: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  jsx: "javascript",
  ts: "typescript",
  mts: "typescript",
  cts: "typescript",
  tsx: "typescript",
  txt: "plain-text",
};

/** Kinds offered by the "new document" control, in display order. */
export const DOCUMENT_KINDS: DocKind[] = [
  "markdown",
  "mdx",
  "json",
  "yaml",
  "html",
  "css",
  "javascript",
  "typescript",
  "plain-text",
];

export function extensionOf(path: string): string {
  const clean = path.replace(/\\/g, "/");
  const name = clean.slice(clean.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

/** Kind of a path. Unknown extensions are ordinary text, never Markdown. */
export function documentKind(path: string): DocKind {
  return EXTENSION_TO_KIND[extensionOf(path)] ?? "plain-text";
}

export function capabilitiesFor(path: string): DocCapabilities {
  return TABLE[documentKind(path)];
}

export function capabilitiesOf(kind: DocKind): DocCapabilities {
  return TABLE[kind];
}

/** True when the name already ends in one of the registry's extensions. */
export function hasDocumentExtension(name: string): boolean {
  return extensionOf(name) in EXTENSION_TO_KIND;
}

/** True for formats where wiki links, tags and tasks are meaningful. */
export function isProse(path: string): boolean {
  return capabilitiesFor(path).hasLinks;
}

/**
 * Ensure a name ends in a supported extension.
 *
 * An explicit supported extension is always respected — `page.mdx` stays
 * `page.mdx` instead of becoming `page.mdx.md`.
 */
export function ensureDocumentName(name: string, fallback: DocKind = "markdown"): string {
  const trimmed = name.trim().replace(/[\\/]+/g, "-");
  if (!trimmed) return "";
  if (hasDocumentExtension(trimmed)) return trimmed;
  return `${trimmed}.${TABLE[fallback].extension}`;
}

/** Starter content for a new document, mirroring the Rust templates. */
export function documentTemplate(kind: DocKind, stem: string): string {
  const name = stem.trim() || "Untitled";
  switch (kind) {
    case "markdown":
    case "mdx":
      return `# ${name}\n\n`;
    case "json":
      return `{\n  "title": ${JSON.stringify(name)}\n}\n`;
    case "yaml":
      return `title: ${name}\n`;
    case "html":
      return `<!doctype html>\n<html lang="en">\n  <head>\n    <meta charset="utf-8" />\n    <title>${name}</title>\n  </head>\n  <body>\n  </body>\n</html>\n`;
    case "css":
      return `/* ${name} */\n\n`;
    case "javascript":
    case "typescript":
      return `// ${name}\n\n`;
    default:
      return "";
  }
}
