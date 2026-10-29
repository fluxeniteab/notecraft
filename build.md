# NoteCraft — Build Plan (5 Phases)

This file splits the full NoteCraft implementation (see the master prompt) into five coherent phases.
Each phase is a complete, runnable unit. A phase is only marked done when its checks pass and
its features are genuinely implemented (no placeholders). Work proceeds phase by phase; later
phases depend on earlier ones being green.

---

## Current frontend (supersedes the original React plan below)

The shipped interface is **Svelte 5 + TypeScript** built with Vite. There is no Tailwind, Radix or
React in the app. Styling is a hand-written token system in `src/app.css`; CodeMirror 6 provides the
editor. The archived React UI is kept under `deprecated/tauri-ui/` for reference only and is not
built. Commands:

```bash
bun install
bun run dev        # Vite at http://localhost:1420
bun run typecheck  # svelte-check
bun run test       # vitest
bun run build      # svelte-check + production bundle
bun run tauri dev
```

## Phase 1 — Foundation & Shell (Tauri + React + Rust core)

> Historical plan. The shell described here was implemented in React in 0.4.0 and has since been
> replaced by the Svelte interface above; the Rust commands are unchanged.

Goal: A launchable Tauri 2 desktop app with a real React shell wired to a real Rust backend,
vault file operations, Markdown parsing, and a working editor.

Scope:
- Tooling: bun-based frontend, Tauri 2, Rust workspace, Tailwind + CSS variables, Radix UI primitives.
- CI: GitHub Actions running bun install/typecheck/lint/test/build + cargo fmt/check/test/clippy + tauri build.
- Rust modules: `error`, `settings`, `state`, `vault`, `markdown`, `metadata` (SQLite migrate).
- Tauri commands: open/create/switch vault, list/read/write/create/rename/move/delete files and folders,
  read/write note, parse note (frontmatter/outline/links/tags/tasks), render markdown HTML.
- Frontend: app shell with resizable panels, sidebar file explorer (tree, search, context menu),
  tab bar (open/close/pin/reorder), CodeMirror 6 editor (source mode), status bar,
  command palette skeleton, theme provider (light/dark/system + accent), settings dialog.
- Storage: atomic writes, path validation, trash, recent vaults, session restore (localStorage).
- Tests: Rust unit tests for path validation, atomic write, frontmatter parse, link/tag/task extraction;
  frontend vitest for file explorer + tab store.
- Docs: README (accurate status), CONTRIBUTING, SECURITY, LICENSE check.

Exit criteria:
- `bun run typecheck`, `bun run lint`, `bun run test`, `bun run build` pass locally.
- CI green on push (rust fmt/check/test/clippy + tauri build).
- App launches (`bun run tauri dev`) and can open a folder vault, create/edit/save a note.

---

## Phase 2 — Knowledge Graph & Search

Goal: Real Tantivy-backed full-text search, internal links, backlinks, unresolved links,
unlinked mentions, and a Sigma.js knowledge graph.

Scope:
- Rust: `search` (Tantivy index, incremental indexing, query parser with operators/filters,
  snippets/highlighting, rebuild, status), `links` (rebuild per note, backlinks, outgoing,
  unresolved, mentions), `graph` (graph data generation from links, filtering).
- Frontend: global search panel (search-as-you-type, filters, snippets, saved queries),
  backlinks panel, outgoing links panel, unresolved links view, unlinked mentions,
  local + global graph views (Sigma + Graphology) with filtering, zoom/pan, navigation,
  link autocomplete in editor, link rename propagation with preview/confirm.
- Editor: live-preview mode (CM6 decorations from markdown syntax tree), reading mode
  (sanitized HTML), wikilinks + standard links, heading/block link targets.
- Tests: search indexing/query, link extraction/backlinks, graph generation.

Exit criteria:
- Search returns ranked results with snippets across a 10k-note test vault in CI.
- Graph renders real nodes/edges and updates incrementally on edit.
- Link rename propagation updates references with confirmation.

---

## Phase 3 — Canvas, Properties, Templates, Views

Goal: Interactive React Flow canvas, typed frontmatter properties, templates, and
database views (table/list/kanban/calendar).

Scope:
- Rust: `tasks` (parse/state/due), `templates` (discovery, render, variables),
  `metadata` extensions (collections, saved searches), property bulk ops.
- Frontend: Canvas with infinite workspace, node types (note/text/file/image/link),
  pan/zoom, select/drag/resize/connect, grouping, undo/redo, minimap, persistence
  (versioned schema), export.
- Properties panel + frontmatter editor (typed: text/number/checkbox/date/tags/list),
  validation, bulk editing.
- Templates: folder, insert/new-note/daily-note templates, variables (date/time/title/folder),
  user templates.
- Tags: nested tags, rename/remove, tag search/filter.
- Views: saved searches, dynamic collections, table/list/kanban/calendar views with
  filter/sort/group/column config; collection config in versioned user file.
- Daily notes, task panel, task queries, due dates/statuses.
- Tests: canvas persistence schema, template rendering, property serialization, task parsing.

Exit criteria:
- Canvas content persists and restores exactly (no hardcoded nodes).
- Properties round-trip through frontmatter Markdown.
- At least table + kanban views functional on real notes.

---

## Phase 4 — History, Workspaces, Automation, Plugins, Sync Interfaces

Goal: Time Machine version history, multi-workspace persistence, automation engine,
plugin platform with permissions, and local sync/conflict foundation.

Scope:
- Rust: `history` (snapshots, retention/dedup, diffs, restore, recovery), `workspace`
  (serialization/restoration, panels), `automation` (triggers/conditions/actions, limits,
  loop detection, dry-run, history), `plugins` (manifest, registry, permissions, lifecycle,
  example plugin), `sync` (change tracking, hashes, conflict detection, offline queue,
  resolution data), `import_export` (import vault, export HTML, backup/restore zip),
  `security` (path checks, CSP, encrypted vault via chacha20poly1305+argon2).
- Frontend: version timeline + diff viewer + restore preview, workspace save/switch/restore,
  automation rule editor + run history, plugin manager (install/enable/disable/uninstall,
  permissions review, example plugin), sync status + conflict resolution UI,
  import/export wizards with validation reports, encrypted vault setup/recovery docs.
- Tests: snapshot/restore/retention, automation limits, plugin lifecycle/error isolation,
  conflict detection, import/export round-trip.

Exit criteria:
- Restore a note to a previous version works and is tested.
- Automation dry-run previews destructive changes; loop detection enforced.
- Plugin manager handles a deliberately failing plugin without crashing the app.
- Encrypted vault documents key-loss consequences.

---

## Phase 5 — Polish, Accessibility, i18n, Performance, Release

Goal: Production-quality UX, accessibility, localization readiness, performance on large
vaults, docs, and release packaging.

Scope:
- Themes: theme import/export + validation, reset, per-workspace settings, reduced motion,
  high contrast, UI scaling, density/spacing, editor typography controls.
- Accessibility: keyboard navigation everywhere, focus indicators, screen-reader labels,
  high-DPI, Unicode-safe file handling, i18n date/number formatting, localized string
  extraction (no hardcoded UI strings in components where feasible).
- Performance: virtualized lists/search results, incremental indexing/watching, bounded
  concurrency, cancellation, progress UI, memory profiling notes, cold/warm startup tests.
- Optional semantic search: local embedding model with explicit download, disk/mem
  requirements, disable toggle, no upload without consent (documented, off by default).
- Docs: full README, ARCHITECTURE, PLUGIN_API, DATA_FORMATS, DEVELOPMENT, TESTING;
  CHANGELOG; known limitations; upgrade/migration guidance.
- Release: Windows installer config (tauri bundle), icons, versioning, CI release workflow
  (only if signing available; otherwise document).
- Final verification: all checks green; feature checklist updated honestly.

Exit criteria:
- Lighthouse/accessibility manual pass on key flows; reduced-motion respected.
- Performance test notes on 50k-note generated vault documented (no fabricated numbers).
- Release build produces installer on Windows runner.
- README feature status matches implementation.

---

## Tracking

Maintain `FEATURES.md` (checklist) updated only when a feature is implemented AND tested.
Record blockers and infrastructure gaps honestly. Never mark unexecuted checks as passed.
Rust checks run only in GitHub CI (no local cargo). Frontend checks run with bun locally.

---

## Releases

`Release` in the Actions tab, or a pushed `v*` tag, produces the installers.

**One-click:** *Actions → Release → Run workflow*, optional `version` (defaults to
`src-tauri/tauri.conf.json`). It creates the tag and starts
`windows_release.yml`, `macos_release.yml` and `linux_release.yml`, which build the
signed bundles, attach them to the release for that tag, and merge that platform
into the updater's `latest.json`. Set `publish` to false for a dry run that only
uploads artifacts.

**From a tag:**

```bash
git tag v0.4.0
git push origin v0.4.0
```

The three platform workflows listen for tag pushes, so this needs no extra step.
Signing secrets and what happens without them are described in
[docs/SIGNING.md](docs/SIGNING.md).

## The native experimental build

`native/` holds the eframe/egui rewrite from 0.3.0. It is deliberately outside the
default build:

```bash
cd native
cargo run --release      # needs a platform linker, like any Rust binary
cargo test --all-targets
```

`.github/workflows/native.yml` runs its format, clippy, test and release-build
checks on Linux and Windows so it cannot rot. It is not released and it is not a
substitute for the application above: see its [limits](native/docs/ARCHITECTURE.md).
