# Architecture

NoteCraft is a single Rust binary. `eframe` owns the window and the event loop,
`egui` draws the interface, and every other module is plain Rust with no
knowledge of the UI beyond the widgets it is handed.

## Module map

```
src/
  main.rs            entry point: window options, run_native
  app/
    mod.rs           App state, actions, shortcuts, autosave, eframe::App
    document.rs      Tab: text + derived data (stats, outline, JSON, MDX)
  documents/mod.rs   Kind registry: label, syntax, stats, validator, capabilities
  filesystem/mod.rs  path validation, atomic writes, trash, snapshots, scan
  markdown/
    mod.rs           pulldown-cmark events -> egui layouts; outline; stats
    highlight.rs     line-oriented tokenizer for code fences and the editor
    export.rs        HTML export, plain-text conversion
  mdx/mod.rs         structural diagnostics and the no-execution policy
  json/mod.rs        serde_json validation, tree flattening, format/minify
  links.rs           wiki links, rename propagation, backlinks, tags, properties
  search/mod.rs      vault-wide search with scopes, snippets and ranking
  settings.rs        persisted preferences (same JSON shape as the old build)
  session.rs         open tabs and panel layout
  theme/
    mod.rs           design tokens, palette, egui style installation
    fonts.rs         system font discovery, bold/italic/mono families
    icons.rs         vector icons drawn with the painter
  ui/
    mod.rs           editor id + split handle
    widgets.rs       icon buttons, chips, keycaps, toggles, segmented, rows
    titlebar.rs      brand, vault switcher, search pill, panel toggles
    menubar.rs       File/Edit/Format/View/Insert/Navigate/Tools/Plugins/Help
    sidebar.rs       explorer tree, filter, vault search results
    tabs.rs          document tab strip
    workspace.rs     tab strip + toolbar + editor/reading split + banners
    editor.rs        TextEdit with highlighting layouter and line gutter
    preview.rs       reading view and internal-link resolution
    json_tree.rs     structured JSON viewer
    context_panel.rs outline, properties, links, data, history
    status.rs        document type, save state, cursor, validation
    palette.rs       Ctrl K palette and Ctrl P command list
    dialogs.rs       settings, prompts, confirmations, context/menus
    toasts.rs        transient notifications
```

## Data flow

1. A document is a `Tab`. Its `text` is the single source of truth.
2. Editing sets `changed`, which calls `Tab::on_edited`: the revision counter
   increments and derived views are recomputed once, not per frame.
3. The reading view, outline, JSON tree and status bar are all projections of
   that text. Nothing is cached across documents, so switching tabs can never
   show stale content.
4. Saving goes through `filesystem::write_text_atomic`, which writes a sibling
   temporary file and renames it over the target.
5. On a timer, the open tabs' modification times are compared with the ones seen
   at load. A file that changed underneath clean text is reloaded; a file that
   changed underneath unsaved edits shows a conflict banner instead of silently
   overwriting either side.

## Design tokens

The palette is copied from the pre-migration stylesheet
(`deprecated/tauri-ui/src/styles.css`) so the native build keeps the same
identity:

- **Dark:** neutral charcoal surfaces (`#0b0b0d` … `#29292e`), near-black code
  wells, a restrained blue accent (`#4f86f7`).
- **Light:** warm paper (`#faf7f1`, `#fffdf8`, `#f2ece2`) rather than a cold
  grey, accent `#1f6feb`.
- Radii 4/6/9/13, 1px separators, 24pt icon buttons, 44pt title bar, 26pt status
  bar. Everything lives in `theme::space` and `theme::radius`.

Typography asks for the same stack the CSS did (Inter → Segoe UI → system-ui).
Inter is not vendored, so `theme/fonts.rs` loads the best available system face
(Segoe UI on Windows, SF on macOS, DejaVu on Linux) plus separate bold, italic
and monospace faces. When nothing is found, egui's bundled fonts remain as
fallbacks, so glyphs always resolve.

## Security boundaries

- **Paths.** Every filesystem call takes a vault-relative path and runs
  `validate_rel` (no absolute paths, no `..`, no NUL) and, when both ends exist,
  a canonicalise check that the result stays inside the vault.
- **Writes.** Atomic replace via temp file + rename. Binary files that are not
  valid UTF-8 are refused rather than guessed at, so a note cannot be corrupted
  by an encoding round-trip.
- **Deletes.** Move to `.notecraft/trash`, never `unlink`.
- **MDX.** Parsing and diagnostics only. There is no JavaScript runtime, so
  document code cannot run, and the UI says so instead of implying a sandbox.
- **JSON.** Values are rendered as text. Nothing is evaluated.
- **HTML export.** Document HTML is escaped by the exporter; the output is a
  document, not an active page.
- **External links.** Only `http`, `https` and `mailto` are handed to the OS.
- **Plugins.** The Tauri build executed plugin modules inside the webview. A
  native build has no script host, so plugin execution is not available at all
  and the UI states that plainly (Help → Plugins status).

## Deliberate limitations

- No accounts, profiles or encryption: vaults are ordinary folders, and the
  previous build's encrypted profiles cannot be decrypted by this one.
- No plugin execution, no AI assistant, no graph/canvas/automation panels yet.
  They are listed as not ported in `FEATURES.md` rather than stubbed.
- Window decorations are native, so the custom min/max/close buttons of the old
  title bar are not drawn; the header row keeps the brand, vault chip and search
  pill.
- MDX is diagnosed structurally, not compiled by an MDX compiler; there is no
  line/column-accurate JSX parse tree.
- Search is a linear scan of the vault instead of an on-disk index. That is fast
  for personal vaults and cannot drift from the files, but it does not scale to
  tens of thousands of large files.
