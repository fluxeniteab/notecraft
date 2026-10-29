# Changelog

## 0.2.1

### Added
- **NoteCraft document engine** (`src-tauri/src/engine.rs`): our own Rust renderer with no
  third-party Markdown parser. It powers the preview for `.md`, `.mdx` and `.json` and handles
  nested lists, task lists, tables, callouts, footnotes, math passthrough, wiki links, and MDX
  component blocks and `{expressions}`.
- JSON notes open as a readable outline instead of a wall of braces; invalid JSON is reported with
  the offending position and the raw source.
- Editor menu now has real exports: **Export Markdown…**, **Export HTML…** (standalone, styled
  document) and **Print / Save as PDF…** (renders to the temp folder and opens the system browser's
  print dialog).
- Settings gained a section header with a short description of each section.

### Changed
- Dark theme is now grayscale (neutral surfaces, no violet/blue background gradients) with a quiet
  blue accent; light theme is a warm-white paper palette with stone ink. Both follow code-editor
  conventions: flat rows, 1px borders, minimal elevation.
- Settings layout widened and re-spaced, with an independently scrolling content column.
- Lock screen redesigned as a two-panel sign-in: security summary on the left, form on the right,
  with a password reveal toggle and clearer create/unlock states.
- `.mdx` files are indexed, searched and opened like Markdown notes.

### Fixed
- Export buttons did nothing in the desktop webview: the old code used a blob `<a download>` and
  `window.open`, both of which the Tauri webview ignores. Exports now go through the OS save dialog
  and a real Rust file write.
- Exporting settings had the same silent failure and now writes the chosen file.
- `pulldown-cmark` removed from the dependency graph (lockfile updated).

## 0.2.0

### Added
- Local profiles: each username is a separate vault under the app data folder, unlocked with a
  password (Argon2id + ChaCha20-Poly1305). Notes and history snapshots are encrypted at rest;
  legacy plaintext notes are encrypted on first unlock. `Ctrl+Shift+L` locks the app.
- `notecraft-cli` binary with the same credentials, `--json` output, and commands for list, read,
  write, append, new, search, tasks and delete, so agents and scripts can use the vault locally.
- In-app updater: signed update artifacts per platform, `latest.json` assembled by the release
  workflows, and an install prompt inside the app.
- VirusTotal scanning of every installer (`.msi`, `.exe`, `.deb`, `.AppImage`, `.dmg`) in CI and in
  all three release workflows; the run summary lists each engine count.
- macOS and Linux release workflows alongside Windows, each publishing bundles and updater
  artifacts to the release.

### Changed
- UI reads as a document workspace: workspace block in the sidebar, page-style empty states
  instead of marketing copy, softer borders in the dark theme.
- Uses the operating system's own fonts; bundled webfont packages were removed.
- The webview's native context menu is disabled app-wide; every menu is custom.
- Version bumped to 0.2.0 across the app, crate and package manifests.

### Fixed
- Linux builds failed because the app icon was RGB without an alpha channel; the icon set is now
  RGBA with `.icns` for macOS.
- Settings modal could not scroll on short windows, cutting off the lower sections.
- Manual release runs skipped attaching bundles to the release because dispatch runs are not tag
  refs; the release workflows now accept a tag input.

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
