# Changelog

## 0.4.0

### Changed
- **The Tauri application is the shipped app again.** 0.3.0 replaced it with a
  native eframe/egui rewrite; the interface drifted away from the design people
  were using and the advanced modules (AI assistant, plugins, encrypted
  profiles, graph, canvas, database views, automation, CLI) were missing. This
  release returns to the Tauri + React application with all of them intact.
- **The native rewrite is kept, not deleted.** It lives in `native/` as an
  experimental build with its own CI checks, and 0.3.0 is marked as a
  pre-release. Nothing from it was thrown away.
- Releases can now be cut from the Actions tab (`.github/workflows/release.yml`):
  the version is read from `tauri.conf.json`, the tag is created and the three
  platform builds attach their installers.

## 0.3.0 — native rewrite (experimental)

> Superseded by 0.4.0 and marked as a pre-release. The code is preserved in
> `native/`.

NoteCraft is a single native Rust binary (eframe + egui). There is no webview,
no Node.js runtime and no Tauri IPC. The Tauri + React implementation this was
forked from is preserved; it was later promoted back to the primary app in
0.4.0 and now lives at the repository root.

### Added
- **Native application shell.** Menu bar (File, Edit, Format, View, Insert,
  Navigate, Tools, Plugins, Help), title row with vault switcher and search pill,
  explorer, document tabs, editor/reading split, context panel, status bar,
  command palette and toasts — all drawn with egui and driven by design tokens
  ported from the legacy stylesheet.
- **Markdown rendered by `pulldown-cmark`** into native text layouts: headings,
  emphasis, lists, task checkboxes, quotes, tables, fenced code with
  highlighting, frontmatter moved to Properties, wiki links resolved against the
  vault.
- **JSON document type.** `serde_json` validation with line/column, collapsible
  tree in document order, key/value filter, expand/collapse, RFC 6901 JSON
  Pointer copy, Format (2/4 spaces, tab) and Minify that refuse to rewrite
  invalid source.
- **MDX document type.** JSX tags, attributes and `{expressions}` highlighted as
  inert source with structural diagnostics (unbalanced braces, unclosed or
  mismatched components, ESM, `<script>`) at line and column. Document code is
  never executed.
- **YAML validation** with line/column in the status bar.
- **Document format registry** (`documents::Kind`) with explicit capabilities, so
  statistics, validation, preview and toolbar adapt per type.
- **History snapshots** under `.notecraft/history`, deduplicated by content hash
  and restorable from the context panel.
- **Atomic saves**, trash instead of deletion, external-modification detection
  with a conflict banner, and unsaved-change prompts.
- Session restore (tabs, active note, panel sizes) and per-vault settings.
- Settings for theme, accent, editor typography, reading width, autosave,
  line numbers, word wrap and reduced motion.

### Changed
- Editing is a native `TextEdit` with a markup layouter and line gutter instead
  of CodeMirror 6.
- Search is an in-process scan with scopes, snippets and ranking instead of a
  Tantivy index.
- Settings keep the same JSON keys, so an existing `settings.json` still loads;
  unsupported legacy sections are ignored rather than rejected.

### Removed from the shipped build
AI assistant and agent tools, the `notecraft` CLI, third-party plugin execution,
graph and canvas views, automation rules, database views, sync, and encrypted
profiles. The source for each remains available in `deprecated/tauri-ui/`; see
`FEATURES.md` for status.

### Fixed
- Creating `page.mdx` no longer produces `page.mdx.md`.
- New `.json` files start from a valid JSON starter instead of a Markdown heading.
- Duplicating a non-Markdown file keeps its extension.

## Unreleased (Tauri build, 0.2.x)

### Added
- **Document-format registry.** One table (Rust `doc_types.rs`, mirrored in
  `documentTypes.ts`) decides the editor language, preview mode, statistics mode and
  validator for every supported extension: `.md`, `.mdx`, `.json`, `.yaml`/`.yml`,
  `.html`, `.css`, `.js`/`.jsx`, `.ts`/`.tsx`, `.txt`.
- **Dedicated JSON view.** Validation with line/column, a collapsible tree with key/value
  filter and RFC 6901 JSON Pointer copy, and Format/Minify actions that refuse to touch
  invalid source. JSON keys are shown in document order.
- JSON and the other supported text formats are now indexed for search.
- Status bar shows the active document type, lines for data/code files and a JSON
  validity badge, instead of always reporting Markdown statistics.

### Changed
- **Plugins are permission-scoped.** The plugin context only exposes commands covered by
  the manifest's declared permissions, unknown permissions are rejected at install time,
  the app store is read-only for plugins, and installing a plugin no longer runs it —
  enabling is an explicit action in Settings.
- JSON parsing uses `serde_json` (with `preserve_order`), so diagnostics are accurate and
  the structured view matches the file.

### Fixed
- Creating `page.mdx` produced `page.mdx.md`; explicit supported extensions are respected.
- New `.json` files were created with a Markdown heading and were therefore invalid.
- Duplicating a non-Markdown file changed its extension.
- JSON tabs were highlighted and previewed as Markdown.
- A JSON tab's preview replaced the source outline with Markdown headings.

## 0.2.1

### Added
- **AI assistant sidebar.** Provider presets with inline SVG marks for DeepSeek, Google Gemini,
  Mistral, OpenAI, OpenRouter, Groq, Ollama, LM Studio and llama.cpp; editable endpoint and model;
  keys stored encrypted with the profile and requests made from Rust (the webview stays offline).
  The agent can list, read, search, create, rename and open notes, create daily notes, list tasks
  and switch the theme. `propose_edit` shows a line diff with **Accept edit** / **Reject**.
- **Plugin packages.** Drop a `.zip` with `plugin.json`, a JavaScript entry, optional `style.css`
