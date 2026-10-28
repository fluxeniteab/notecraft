# Changelog

## 0.1.0 (unreleased)

- Initial Tauri 2 + React + Rust implementation.
- Reworked the desktop workspace with a premium dark-first interface, a compatible light theme,
  persistent multi-tab Markdown editing, and source/live/reading/split editor modes.
- Added safer save handling for tab and vault changes, theme migration, and local search/link/graph
  index maintenance; the GitHub Actions pipeline (cargo fmt/check/test/clippy and a full
  Tauri build) passes on this release tag.
- Vault, editor, search, links, graph, canvas, properties, templates, history, workspaces,
  automation (basic), plugins (basic), sync (local-only), import/export (basic).

### Added in this cycle
- Quick capture (Ctrl+Shift+N) appending timestamped bullets to a configurable capture note,
  and daily notes with configurable folder/template (palette + shortcut).
- Filesystem watcher on vault open: external edits refresh the explorer and trigger a
  per-note conflict flow — a changed-on-disk note is offered for reload and its save is
  blocked so external edits are never silently overwritten.
- Search: folder filter in the UI, saved searches, and recorded search history
  (recent searches).
- Unlinked mentions, link-rename propagation with impact preview, tag rename, and
  nested-tag grouping in the tags panel.
- Dynamic collections with table/list/kanban/calendar views; note content stays the
  source of truth (views never store note data).
- Task panel (open/all/done) in the right sidebar.
- History panel: timeline, diff, restore, and inserting selected snapshot content
  back into the editor.
- Math (KaTeX) and Mermaid rendering in preview (lazy-loaded chunks).
- PDF export through the OS print pipeline (fully offline).
- Sync folder provider: mirror a vault to a folder you control with push/pull and
  per-file conflict status; conflicts are reported, never overwritten. Remote/cloud
  providers remain deferred until conflict handling proves dependable.
- Plugin marketplace UI backed by a shipped catalog (docs/plugin-catalog.json) with an
  embedded fallback so installs also work in packaged builds.
- Explorer drag-and-drop of notes into folders and configurable sort order.

### Fixed
- Tauri capability list trimmed to permissions the window actually uses.
- Settings dialog structure (missing group wrapper broke JSX nesting).
- `list_plugins` and plugin catalog typings on the frontend.
