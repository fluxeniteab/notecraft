# Legacy → native migration map

How the archived Tauri + React interface (`src/`, `src-tauri/`) maps onto the
native Rust + egui application at the repository root. Use this when comparing
behaviour, or when a native detail must be checked against the original.

## Shell and layout

| Legacy | Native | Notes |
| --- | --- | --- |
| `src/App.tsx` (shell, global shortcuts, conflict listeners) | `src/app/mod.rs` | `App` holds vault, tabs, panels, settings; `handle_shortcuts` replaces the `useEffect` key handlers; conflict state lives on each `Tab` |
| `src/components/TitleBar.tsx` | `src/ui/titlebar.rs` | Brand, vault chip with recent-vault menu, search pill, panel toggles. Window buttons are gone: the window keeps native decorations |
| `src/components/ui.tsx` (Button, IconButton, Segmented, Chip, Toggle…) | `src/ui/widgets.rs` | `icon_button`, `segmented`, `chip`, `toggle`, `keycap`, `section_label` |
| `src/components/TabBar.tsx` | `src/ui/tabs.rs` | Dirty dot, per-kind glyph, middle-click close, drag-free reorder not yet ported |
| `src/components/EditorArea.tsx` (toolbar + mode segmented) | `src/ui/workspace.rs` | Mode segmented control, save button, per-type tools |
| `src/components/StatusBar.tsx` | `src/ui/status.rs` | Type, save state, cursor, statistics, JSON/YAML/MDX validation |
| `src/components/Sidebar.tsx` | `src/ui/sidebar.rs` | Explorer tree, filter, sort, context menu, search results |
| `src/components/RightSidebar.tsx` | `src/ui/context_panel.rs` | Outline, Properties, Links, Data, History |
| `src/components/CommandPalette.tsx` | `src/ui/palette.rs` | `Ctrl K` palette and `Ctrl P` command list |
| `src/components/QuickSwitcher.tsx` | `src/ui/palette.rs` | Merged into the palette's file scope |
| `src/components/SettingsDialog.tsx` | `src/ui/dialogs.rs` + `src/settings.rs` | Same keys in `settings.json`; unsupported legacy sections are omitted rather than faked |
| `src/components/LockScreen.tsx` | — | No encrypted profiles in the native build |
| `src/components/AutomationPanel.tsx` | — | Not ported |
| `src/store/appStore.ts` (zustand) | `src/app/mod.rs`, `src/session.rs` | Explicit fields and methods instead of a store; persistence split into settings and session |
| `src/styles.css` (design tokens) | `src/theme/mod.rs` | Colour, radius and spacing tokens transcribed; see the table below |
| `src/lib/theme.ts` | `src/theme/mod.rs` | Theme mode + accent, resolved per frame |

## Editor

| Legacy | Native |
| --- | --- |
| CodeMirror 6 (`@codemirror/lang-markdown`, `lang-json`, …) | `src/ui/editor.rs` — `TextEdit` with a `Layouter` that styles markup and code, plus a painted line gutter |
| CM6 syntax highlighting / `highlightStyle` | `src/markdown/highlight.rs` — keyword/string/comment/number tokenizer shared by the editor and fenced code blocks |
| CM6 diagnostics gutter for JSON | `src/json/mod.rs` → status bar and JSON panel banner (`line`/`column` from `serde_json`) |
| Editor settings (font, size, wrap, line numbers) | `src/settings.rs`, applied in `theme::install` and `ui::editor` |

## Document engine

| Legacy | Native |
| --- | --- |
| `src-tauri/src/doc_types.rs`, `src/lib/documentTypes.ts` | `src/documents/mod.rs` — one `Kind` enum: label, language, capabilities (`stats`, `json_tree`, `mdx`, `readable`), validator |
| `src/lib/files.ts` (create with explicit extension) | `documents::Kind::from_path` + `dialogs::prompt_new_document`; the typed extension wins |
| `src-tauri/src/markdown.rs` (custom HTML renderer) | `src/markdown/mod.rs` — `pulldown-cmark` events rendered into egui layouts; `render_html` for export only |
| `src-tauri/src/json_doc.rs` | `src/json/mod.rs` |
| MDX tag scanning in `markdown.rs` | `src/mdx/mod.rs` — structural diagnostics with line/column, no execution |
| `src-tauri/src/vault.rs` (paths, atomic write, trash) | `src/filesystem/mod.rs` |
| `src-tauri/src/search.rs` (Tantivy index) | `src/search/mod.rs` — in-memory scan with scopes, snippets and ranking |
| `src-tauri/src/links.rs` | `src/links.rs` — plus frontmatter properties, which used to be `metadata.rs` |
| `src-tauri/src/history.rs` (SQLite snapshots) | `src/filesystem` history helpers — JSON files under `.notecraft/history` |
| `src-tauri/src/templates.rs` | Not ported (new-document templates are literal per-kind starters) |
| `src-tauri/src/sync.rs`, `automation` | Not ported |
| `src-tauri/src/plugins.rs` | Not ported: no script host exists, so the UI reports plugins as unavailable |

## Design tokens carried over

| CSS variable (legacy) | Native token |
| --- | --- |
| `--nc-bg-app` `#0b0b0d` | `palette::bg_app` |
| `--nc-bg-sidebar` `#101013` | `palette::bg_sidebar` |
| `--nc-bg-editor` `#0e0e11` | `palette::bg_editor` |
| `--nc-bg-elevated` `#17171b` | `palette::bg_elevated` |
| `--nc-bg-hover` `#1e1e23` | `palette::hover` |
| `--nc-border` `#26262c` | `palette::border` |
| `--nc-fg` `#e7e7ea` | `palette::fg` |
| `--nc-fg-muted` `#8b8b95` | `palette::fg_muted` |
| `--nc-fg-faint` `#5f5f6a` | `palette::fg_faint` |
| `--nc-accent` `#4f86f7` | `palette::accent` |
| `--nc-r-sm/md/lg` `4/6/9px` | `theme::radius::{SM, MD, LG}` |
| Light mode paper tones `#faf7f1…` | `palette::light()` |

## Shortcuts preserved

`Ctrl K`, `Ctrl P`, `Ctrl N`, `Ctrl S`, `Ctrl Shift S`, `Ctrl W`, `Ctrl B`,
`Ctrl \`, `Ctrl E`, `Ctrl 1/2/3`, `Alt ←/→`, `Ctrl ,`, `Ctrl Shift P` (focus
mode), `Esc`. Behaviour matches the legacy handlers except where noted above.

## Dependencies intentionally dropped

Tauri 2, `@tauri-apps/*`, React, Radix, Tailwind, CodeMirror, Sigma/Graphology,
React Flow, TanStack Query/Virtual, react-markdown, remark/rehype, rusqlite,
Tantivy, `tauri-plugin-*`. The native build uses `eframe`/`egui`,
`pulldown-cmark`, `serde`/`serde_json`/`serde_yaml` and small utility crates.
