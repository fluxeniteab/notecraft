# NoteCraft — Feature Checklist (honest status)

Legend: [x] implemented & tested, [~] partial, [ ] planned

## Core vault & notes
- [x] Create/open/switch/close vaults (basic)
- [x] Recent vaults
- [x] File explorer (tree, search, context delete)
- [x] Nested folders
- [x] Create/rename/move/duplicate/delete notes and folders (create/delete/move/rename)
- [x] Safe deletion with trash
- [x] Multi-tab editing
- [x] Pin/close/reorder tabs (close; reorder store ready)
- [~] Split panes (mode split in editor, not independent panes)
- [~] Drag-and-drop notes/folders (not yet)
- [x] Context menus (delete via right-click)
- [x] Keyboard shortcuts (Ctrl+P, Ctrl+O)
- [x] File-type icons
- [x] Searchable file explorer
- [x] Hidden-file/excluded-folder settings
- [~] Configurable sorting/grouping (basic)
- [ ] External filesystem-change detection (watcher not wired)
- [x] Vault-level & app-level settings
- [x] Session restoration (UI prefs)
- [x] Recent files (recent vaults)
- [ ] File metadata display (partial)
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
- [x] Math (not yet rendered)
- [x] Mermaid (not yet)
- [x] Find/replace (CM6 search)
- [x] Folding (CM6)
- [x] Word/char/reading-time stats (words)
- [x] Autosave + dirty tracking
- [x] Conflict detection (not yet)
- [x] Configurable typography (settings)
- [x] Distraction-free (not yet)

## Links & graph
- [x] Wiki links + markdown links
- [x] Links to headings/blocks (parsed)
- [x] Backlinks
- [x] Outgoing links
- [x] Unresolved links
- [~] Unlinked mentions (approx)
- [x] Link autocomplete (not yet in CM6)
- [~] Link rename propagation (not yet)
- [x] Global graph (Sigma)
- [~] Local graph (not yet)
- [x] Graph filtering (not yet UI)
- [x] Graph zoom/pan/select
- [x] Incremental graph updates (rebuild on link rebuild)

## Canvas
- [x] Infinite workspace (React Flow)
- [x] Pan/zoom/select/drag/connect
- [x] Node types (default cards)
- [x] Multi-select/grouping (React Flow)
- [x] Undo/redo (React Flow)
- [x] Minimap, fit-to-content
- [x] Copy/paste (React Flow)
- [x] Save/restore (file)
- [x] Link node to note (not yet)

## Tags, properties, templates, views
- [x] Tags (parsed, stored)
- [~] Nested tags (not yet)
- [x] Tag rename/remove (not yet)
- [x] YAML frontmatter parse/edit (raw)
- [x] Typed properties (metadata)
- [x] Property views/bulk (not yet)
- [x] Templates (list/render/vars)
- [x] Template folders
- [x] New-note/daily templates (manual)
- [x] Template variables
- [~] Saved searches (not yet)
- [~] Dynamic collections (not yet)
- [~] Table/list/kanban/calendar views (not yet)

## Search
- [x] Full-text (Tantivy)
- [~] Incremental indexing (note saves update the index; Rust CI verification pending)
- [x] Snippets/highlighting
- [x] Search-as-you-type (UI manual)
- [x] Filters (folder/tag in backend)
- [x] Operators (Tantivy query parser)
- [x] Exact phrase (query parser)
- [x] Boolean/exclusion (query parser)
- [x] Filename search (title field)
- [~] Saved queries (not yet)
- [~] Search history (table exists)
- [x] Rebuild index
- [x] Index status
- [~] Excluded folders
- [ ] Semantic search (disabled by default, not implemented)

## History & recovery
- [x] Automatic snapshots (manual trigger)
- [x] Configurable retention
- [x] Named checkpoints (label)
- [x] Timeline/list
- [x] Diff
- [x] Restore
- [~] Recover selected content (not yet)
- [x] Trash recovery
- [x] Interrupted write recovery (atomic)
- [~] Tests (rust unit)

## Workspaces & productivity
- [x] Saved workspaces (db)
- [~] Panel layout (basic)
- [x] Command palette
- [x] Quick switcher
- [x] Outline/backlinks/properties/search panels
- [x] Task panel (list command, not UI panel)
- [~] Daily notes (not yet)
- [~] Task queries/due (stored)
- [ ] Writing goals/timer
- [x] Distraction-free (not yet)
- [x] Status bar

## Themes & customization
- [x] Dark/light themes (legacy system theme migrates to dark)
- [x] Accent color
- [x] CSS variables
- [x] Editor typography settings
- [x] Reading width
- [x] Reduced motion/high contrast (settings)
- [x] Theme import/export
- [ ] Keyboard shortcut customization
- [ ] Per-workspace settings

## Plugins
- [x] Manifest model
- [x] Registry (db)
- [x] Enable/disable/uninstall
- [x] Permissions (stored)
- [x] Example plugin (docs/examples/plugin-example)
- [x] API docs (docs/PLUGIN_API.md)
- [ ] Marketplace interface

## Automation
- [x] Rule model (db)
- [x] Save/delete/list
- [x] Dry-run preview
- [~] Triggers/conditions/actions engine (manual run + property/move/template actions)
- [x] Loop detection/limits (execution counter)

## Sync
- [x] Status interface
- [x] Local-only
- [ ] Provider implementation

## Import/export
- [x] Import vault (copy + broken link report)
- [x] Backup (zip)
- [x] Restore backup (zip)
- [x] Export HTML
- [ ] PDF export

## Security
- [x] Path validation
- [x] Strict capabilities
- [x] CSP
- [x] Encrypted vault interface (password verifier + crypto helper)
- [x] No telemetry by default

## Accessibility/i18n
- [x] Keyboard nav (basic)
- [x] Focus (basic)
- [x] Reduced motion/high contrast
- [ ] i18n string extraction
- [ ] Screen-reader labels (partial)

## Testing
- [x] Rust unit tests (vault, markdown, metadata)
- [x] Frontend tests (appStore)
- [ ] Integration/e2e (not yet)

## Build/CI
- [x] Bun frontend
- [x] GitHub Actions CI
- [x] Tauri build (bundle disabled)
