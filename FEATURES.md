# NoteCraft feature status

This file is the honest checklist for the native (egui) build. A box is ticked
only when the behaviour exists in this build and is covered by a test or was
verified by hand in the running application. The Tauri build's status, module by
module, is recorded in [deprecated/tauri-ui/MIGRATION.md](deprecated/tauri-ui/MIGRATION.md).

## Documents

- [x] Create `.md`, `.mdx`, `.json`, `.yaml`, `.yml`, `.txt`, `.html`, `.css`,
      `.js`, `.ts` through a type picker; the chosen extension is used verbatim.
- [x] Open, edit, save, close and reopen any supported type.
- [x] Multi-tab workspace with dirty markers, middle-click and `Ctrl W` close,
      `Alt ←/→` navigation.
- [x] Rename and delete-to-trash from the explorer context menu; open the vault
      or settings folder in the OS file manager from Tools.
- [x] Export the active document as HTML or plain text, and copy it as text.
- [ ] Duplicate a file, and reveal a single file in the OS file manager.
- [ ] Import an external file into the vault.
- [x] Non-UTF-8 and oversized files are refused with a message instead of being
      lossily re-encoded.
- [x] Unsaved-change prompt before closing a tab or the window.
- [x] External-modification detection: clean tabs reload, edited tabs show a
      conflict banner with keep/reload choices.
- [ ] Per-tab independent split panes (one editor/reading split per window).
- [ ] Move files by drag and drop inside the explorer.

## Markdown

- [x] Headings, emphasis, strong, strikethrough, inline code, links, images,
      block quotes, bullet and ordered lists (nested), task lists, fenced code
      with syntax highlighting, tables, thematic breaks, frontmatter (hidden from
      the body, surfaced in Properties).
- [x] Wiki links `[[Note]]` and `[[Note#Heading]]` resolve against the vault.
- [x] Outline with heading levels; word/character/reading-time statistics.
- [x] Reading width, font family, size and line height are configurable.
- [x] Editing includes a live markup overlay (headings scaled, emphasis marked,
      code in mono, links accented).
- [ ] True inline WYSIWYG replacement of syntax characters.

## MDX

- [x] Dedicated document type with JSX tags, attributes and `{expressions}`
      highlighted as inert source.
- [x] Structural diagnostics with line and column: unbalanced braces, unclosed
      or mismatched components, ESM import/export, raw `<script>`.
- [x] Markdown around components still renders; component bodies are preserved
      verbatim in the source and shown as chips in the preview.
- [x] Editing and saving always work, with or without diagnostics.
- [x] Execution is disabled by default and there is no runtime to enable: the
      banner says document code is not run.
- [ ] Compilation through a real MDX compiler (no line-accurate JSX offset
      mapping; diagnostics are structural heuristics, not a parse tree).

## JSON

- [x] `serde_json` validation with byte offset, line and column of the first
      error.
- [x] Collapsible tree that keeps document key order, shows types, and handles
      both objects and arrays.
- [x] Key/value filtering, expand all, collapse all, copy JSON Pointer (RFC 6901)
      from the tree and from the cursor.
- [x] Format (2/4 spaces or tab) and Minify, disabled while the document is
      invalid so a broken file is never rewritten.
- [x] Status bar reports valid/invalid with the error position.
- [x] Large documents are limited to a bounded number of tree nodes so the UI
      stays responsive; the source view is unlimited.
- [x] No evaluation of JSON values anywhere.
- [ ] Optional validation against a user-selected JSON Schema.
- [ ] Direct editing inside the tree view (the tree is read-only by design).

## YAML

- [x] `serde_yaml` validation with line/column, shown in the status bar.
- [x] Monospace editing with line numbers.
- [ ] Structured tree view for YAML.

## Workspace and shell

- [x] Native-style menu bar: File, Edit, Format, View, Insert, Navigate, Tools,
      Plugins, Help — every item is wired to an action.
- [x] Native window decorations, minimum size, DPI scaling handled by eframe.
- [x] Vault switcher with recent vaults, explorer with folders, filters and
      sorting by name or modification time.
- [x] Collapsible sidebar and context panel, both remembered across restarts.
- [x] Command palette (`Ctrl K`) over files and commands; `Ctrl P` opens the
      command-only list.
- [x] Settings dialog: theme (dark/light/system time of day), accent colour,
      editor font, size, line height, reading width, autosave interval, editor
      line numbers, word wrap, reduced motion.
- [x] Session restore: open tabs, active tab, scroll position and panel sizes.
- [x] App settings persist atomically in the platform config directory
      (`%APPDATA%/NoteCraft/settings.json`); the legacy JSON shape still loads.
- [ ] Per-vault settings override (all settings are application-wide).
- [x] Toasts for save, export, trash and error feedback.
- [ ] Custom keyboard-shortcut remapping.
- [ ] Localization of UI strings.

## Search and linking

- [x] Vault-wide search with type scopes (All, Prose, Data, Code), snippets
      around the match, and ranking by title match then body match.
- [x] Tag extraction and a tag list; property (frontmatter) table.
- [x] Backlinks and outgoing links per note, unresolved links listed separately.
- [x] Title-aware wiki link resolution.
- [ ] Persisted on-disk search index (currently a scan per query).
- [ ] Rename propagation across the vault with a preview (link rewriting exists
      for the renamed file's own links only).

## History and data safety

- [x] Bounded per-note history snapshots under `.notecraft/history`, deduplicated
      by content hash, restorable from the History section.
- [x] Trash folder instead of permanent deletion; restore from the toast or the
      folder itself.
- [x] Atomic writes (temp file + rename) for every save and export.
- [x] `.notecraft/` is excluded from the explorer, search and indexing.
- [ ] Compression or pruning policy UI for history.
- [ ] Encrypted vaults (dropped in this build — see Security).

## Security

- [x] Vault-relative path validation with `..`, absolute path and NUL rejection,
      plus a canonicalise check to block symlink escapes.
- [x] No telemetry, no network requests, no account.
- [x] MDX/HTML/JSON content is never executed or evaluated.
- [x] Only `http`/`https`/`mailto` links are opened externally.
- [x] Plugin code is not loaded at all; the Plugins menu says so rather than
      offering a fake sandbox.
- [ ] Encrypted vaults with Argon2id + ChaCha20-Poly1305 (was in the Tauri build;
      not ported, so old encrypted profiles cannot be opened by this build).
- [ ] Signed update verification (no updater in this build).

## Not ported

AI assistant, agent tools and diff proposals; local AI runtimes; the `notecraft`
CLI; third-party plugin execution; graph view and canvas; automation rules;
Kanban/table/calendar database views; sync and conflict-resolution UI; command
palette AI actions. Each is listed here rather than presented as a placeholder
button, and the archived implementation stays available under `deprecated/`.
