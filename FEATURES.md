# NoteCraft — Feature Checklist (honest status)

Legend: [x] implemented & tested, [~] partial, [ ] planned

> **Svelte interface status.** The frontend was rebuilt in Svelte 5 + TypeScript. The Rust
> document engine, profiles, vaults and file formats are unchanged and covered below. Panels that
> are marked **archived UI only** still exist in the Rust backend and in the archived React UI
> under `deprecated/tauri-ui/`, but the new shell does not expose them yet. They are not claimed
> as working in the shipped UI.

## Document engine
- [x] Own Rust renderer for Markdown (no third-party Markdown parser)
- [x] Explicit document-format registry (`doc_types.rs` + `documentTypes.ts`): kind, editor language, preview mode, statistics mode and validator per extension
- [x] Supported types: `.md`, `.mdx`, `.json`, `.yaml`/`.yml`, `.html`, `.css`, `.js`/`.jsx`, `.ts`/`.tsx`, `.txt`
- [x] Creating a file keeps the extension the user typed (`page.mdx`, never `page.mdx.md`)
- [x] Type-specific starter content, so a new `.json` file is valid JSON
- [x] MDX: JSX component blocks, `{expressions}` and ESM lines surfaced, rest rendered
- [x] MDX is never executed — no JSX runtime, no imports, no Tauri access from a document
- [x] JSON: dedicated structured view with validation (line/column), collapsible tree, key/value filter and JSON Pointer copy
- [x] JSON Format / Minify actions that parse first and refuse to touch invalid source
- [x] JSON object keys displayed in document order (not alphabetised)
- [x] Code-like documents preview as escaped source instead of being rendered as Markdown
- [x] Editor language follows the file type (JSON, JavaScript/TypeScript, Markdown)
- [x] Status bar reports the real type: words/chars for prose, lines and JSON validity for data
- [x] Text index covers every supported type; wiki links/tags stay Markdown-only
- [x] Nested lists, task lists, tables, callouts, footnotes, frontmatter card, wiki links
- [x] Preview HTML escapes all user text and is sanitised again before insertion
- [x] `render_document` command dispatches on the file extension; never silently falls back to Markdown

## AI assistant (0.2.1) — backend retained, no Svelte panel yet

> The commands and encrypted key storage exist in Rust. The redesigned shell does not include
> the chat panel, so this section documents the backend/archived UI only.
- [x] Provider presets: DeepSeek, Google Gemini, Mistral, OpenAI, OpenRouter, Groq
- [x] Local runtimes: Ollama, LM Studio, llama.cpp (no API key)
- [x] Inline SVG mark for every provider, in the panel and the setup grid
- [x] Editable endpoint and model for any OpenAI-compatible gateway
- [x] Keys stored in the app data folder, encrypted with the unlocked profile
- [x] Requests executed in Rust; the webview CSP keeps the page offline
- [x] Agent tools: list/read/search/create/rename/open notes, daily note, tasks, theme
- [x] Edit proposals shown as a diff with Accept/Reject (never applied automatically)
- [x] Eight-step tool loop cap and a new-conversation reset
- [ ] Streaming token output (responses arrive complete)

## Plugin packages (0.2.1) — manager ported, runtime not loaded yet

> Settings can list, enable, disable, install and uninstall plugins. The new shell does not load
> plugin JavaScript into the page, so third-party UI extensions do not run yet.
- [x] Drag-and-drop or file-picker install of `plugin_xyz.zip`
- [x] Manifest validation (`plugin.json`), id/folder sanitisation, zip-slip guard
- [x] Optional `style.css` injected, optional `icon.png`/SVG shown in settings
- [x] ES module entry with a typed app context (`api`, `notify`, read-only `store`)
- [x] Permission-scoped API: `notes:read`, `notes:write`, `notes:create`, `notes:delete`, `links:read`, `ui:notify`, `editor:open`, `settings:read`
- [x] Unknown permissions rejected at install time, so the list shown is complete
- [x] Plugins never reach profile/password, AI keys, settings writes or plugin management
- [x] Installed plugins start disabled; enabling is an explicit user action
- [x] Enable/disable (styles removed), uninstall removes the folder
- [x] TypeScript accepted when the compiled `.js` is shipped
- [x] Documentation in `docs/PLUGINS.md`
- [ ] True sandboxing (plugin modules still share the webview realm; documented as trusted code)

## Local accounts, encryption and distribution (0.2.0)
- [x] Local profiles: one encrypted vault per username in the app data folder
- [x] Password unlock with Argon2id key derivation; a verifier is stored, never the password
- [x] Notes and history snapshots encrypted at rest (ChaCha20-Poly1305, fresh nonce per write)
- [x] Legacy plaintext notes encrypted in place on first unlock
- [x] Lock profile (Ctrl+Shift+L) drops the in-memory key
- [x] Change password re-encrypts the vault under a new salt and key
- [x] Delete profile (requires the password) removes the vault and registry entry
- [ ] Password recovery (intentionally none: the password is the only key)
- [x] Per-profile search index and database, so accounts never share documents
- [x] CLI (notecraft-cli) with username/password auth and --json output
- [x] In-app updater with signed artifacts and a per-platform latest.json
- [x] VirusTotal scan of every installer in CI and in each release workflow
- [x] macOS, Linux and Windows release workflows with artifact upload

## Core vault & notes
- [x] Create/open/switch/close vaults (basic)
- [x] Recent vaults
- [x] File explorer (tree, search, context delete)
- [x] Nested folders
- [x] Create/rename/move/duplicate/delete notes and folders
- [x] Safe deletion with trash
- [x] Multi-tab editing
- [x] Pin/close/reorder tabs (close; reorder store ready)
- [~] Split panes (mode split in editor, not independent panes)
- [x] Drag-and-drop notes into folders (explorer)
- [x] Context menus (delete via right-click)
- [x] Keyboard shortcuts (Ctrl+P, Ctrl+O, Ctrl+K, Ctrl+S, …)
- [x] File-type icons
- [x] Searchable file explorer
- [x] Hidden-file/excluded-folder settings
- [x] Configurable explorer sorting (name/modified via settings)
- [x] External filesystem-change detection (notify watcher → explorer refresh + reload)
- [x] Vault-level & app-level settings
- [x] Session restoration (UI prefs)
- [x] Recent files (recent vaults)
- [x] File metadata display (modified time in views)
- [~] Safe import/export

## Editor
- [x] Source mode
- [x] Live-preview mode (CodeMirror decorations reveal Markdown syntax on the active line)
- [x] Reading mode (sanitized HTML)
- [x] Split editing/preview
- [x] Syntax highlighting (CM6)
- [x] Headings/outline
- [x] Lists, tasks, tables (markdown lang)
- [x] Blockquotes, callouts (rendered)
- [x] Fenced code blocks
- [x] Inline code
- [x] Links and images
- [x] Footnotes (pulldown-cmark)
- [x] Frontmatter editing (raw)
- [x] Tags/properties integration
- [x] Math (KaTeX rendered in preview)
- [x] Mermaid diagrams (rendered in preview, lazy-loaded)
- [x] Find/replace (CM6 search)
- [x] Folding (CM6)
- [x] Word/char/reading-time stats (words)
- [x] Autosave + dirty tracking
- [x] Conflict protection (external edits block/replace saves with a toast)
- [x] Configurable typography (settings)
- [x] Distraction-free focus mode (toolbar toggle)

## Links & graph
- [x] Wiki links + markdown links
- [x] Links to headings/blocks (parsed)
- [x] Backlinks
- [x] Outgoing links
- [x] Unresolved links
- [x] Unlinked mentions (scan for plain-text references, insert link)
- [x] Link rename propagation (impact preview + apply across vault)
- [x] Global graph (Sigma)
- [~] Local graph (backend scope, no UI toggle yet)
- [~] Graph filtering (backend filters, limited UI)
- [x] Graph zoom/pan/select
- [x] Incremental graph updates (rebuild on link rebuild)

## Canvas — archived UI only
- [~] Infinite workspace (archived React Flow implementation)
- [~] Pan/zoom/select/drag/connect
- [~] Node types (default cards)
- [~] Multi-select/grouping
- [~] Undo/redo
- [~] Minimap, fit-to-content
- [~] Copy/paste
- [~] Save/restore (file)
- [ ] Link node to note (not yet)
- [ ] Svelte canvas view (not ported)

## Tags, properties, templates, views — note views are archived UI only

> The new shell shows tags and frontmatter properties for the open document. Collections and
> saved database-style views are not ported yet.
- [x] Tags (parsed, stored)
- [x] Nested tags (grouped under top-level parent in the tags panel)
- [x] Tag rename (rewrites occurrences, rebuilds index)
- [x] YAML frontmatter parse/edit (raw)
- [x] Typed properties (metadata)
- [~] Property views/bulk (shown in collections; no bulk editor)
- [x] Templates (list/render/vars)
- [x] Template folders
- [x] New-note/daily templates (settings-driven)
- [x] Template variables
- [x] Saved searches (save/reuse/delete from the search panel)
- [x] Dynamic collections (folder/tag/property queries stored in db)
- [x] Table/list/kanban/calendar views (collections; vault files stay source of truth)

## Search
- [x] Full-text (Tantivy)
- [x] Incremental indexing (note saves update the index; cargo check/test green in CI)
- [x] Snippets/highlighting
- [x] Search-as-you-type (UI manual)
- [x] Filters (folder + tag in the UI, folder/tag in backend)
- [x] Operators (Tantivy query parser)
- [x] Exact phrase (query parser)
- [x] Boolean/exclusion (query parser)
- [x] Filename search (title field)
- [x] Saved queries
- [x] Search history (recorded on search, shown as recent searches)
- [x] Rebuild index
- [x] Index status
- [x] Excluded folders (settings UI)
- [ ] Semantic search (disabled by default, not implemented)

## History & recovery
- [x] Automatic snapshots (manual trigger)
- [x] Configurable retention
- [x] Named checkpoints (label)
- [x] Timeline/list (History panel)
- [x] Diff
- [x] Restore
- [x] Recover selected content (insert snapshot lines into the editor)
- [x] Trash recovery
- [x] Interrupted write recovery (atomic)
- [x] Tests (rust unit suite green in GitHub CI)

## Workspaces & productivity
- [x] Saved workspaces (db)
- [~] Panel layout (basic)
- [x] Command palette
- [x] Quick switcher
- [x] Outline/backlinks/properties/search/tags/tasks/history/automation panels
- [x] Task panel (open/all/done filters; note content is the source of truth)
- [x] Daily notes (settings folder/template, palette + shortcut)
- [x] Quick capture (append timestamped bullets to a capture note, Ctrl+Shift+N)
- [~] Task queries/due (stored; due-date filter not in UI)
- [ ] Writing goals/timer
- [x] Distraction-free focus mode
- [x] Status bar

## Themes & customization
- [x] Two themes only: grayscale dark and warm-white light (legacy system theme migrates to dark)
- [x] Flat, code-editor-like chrome: 1px borders, neutral surfaces, quiet blue accent
- [x] Accent color
- [x] CSS variables
- [x] Editor typography settings
- [x] Reading width
- [x] Reduced motion (settings)
- [x] Theme import/export
- [ ] Keyboard shortcut customization
- [ ] Per-workspace settings

## Plugins
- [x] Plugin manager in Settings (list, enable, disable, install, uninstall)
- [ ] Plugin JavaScript runtime in the Svelte shell (intentionally disabled until sandboxed)
- [x] Manifest model
- [x] Registry (db)
- [x] Enable/disable/uninstall
- [x] Permissions (stored)
- [x] Example plugin (docs/examples/plugin-example)
- [x] API docs (docs/PLUGIN_API.md)
- [x] Marketplace interface (shipped catalog in Settings; embedded fallback for packaged builds)

## Automation — backend retained, no Svelte panel yet
- [x] Rule model (db)
- [x] Save/delete/list
- [x] Dry-run preview
- [~] Triggers/conditions/actions engine (manual run + property/move/template actions)
- [x] Loop detection/limits (execution counter)

## Sync
- [x] Status interface
- [x] Local-only
- [x] Folder provider (mirror to a folder you control; push/pull; conflicts reported, never overwritten)
- [ ] Remote/cloud provider (deliberately deferred until conflict handling proves dependable)

## Import/export
- [x] Import vault (copy + broken link report)
- [x] Backup (zip)
- [x] Restore backup (zip)
- [x] Export Markdown file (OS save dialog)
- [x] Export standalone HTML file (styled, offline)
- [x] PDF via the system browser print dialog (offline)
- [x] Export/import settings JSON through a real file write

## Security
- [x] Path validation
- [x] Strict capabilities
- [x] CSP
- [x] Encrypted vault interface (password verifier + crypto helper)
- [x] No telemetry by default

## Accessibility/i18n
- [x] Keyboard nav (basic)
- [x] Focus (basic)
- [x] Reduced motion
- [ ] i18n string extraction
- [ ] Screen-reader labels (partial)

## Testing
- [x] Rust unit tests (vault, markdown, metadata, graph, links, settings)
- [x] Frontend tests (vitest: document registry, rename/extension preservation, explorer tree)
- [ ] Frontend tests for tab saving and conflict handling (planned)
- [ ] Integration/e2e (not yet)

## Build/CI
- [x] Bun frontend
- [x] GitHub Actions CI (tests, then signed installers per platform)
- [x] Tauri build (bundle disabled)
- [x] One-click release from the Actions tab (tag + installers + updater manifest)

## Native experimental build (`native/`)
Kept for reference, built and tested by `.github/workflows/native.yml`. Not shipped.
- [x] Native eframe/egui shell (menubar, explorer, tabs, editor, reading view, JSON tree, status bar)
- [x] JSON tree with JSON Pointer copy, MDX structural diagnostics, YAML validation
- [x] Atomic saves, trash, history snapshots, conflict banner
- [ ] Interface parity with the app above
- [ ] Encryption, AI assistant, plugins, graph, canvas, automation, CLI (not ported)
