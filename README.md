# NoteCraft

NoteCraft is a local-first, file-based Markdown editor and personal knowledge workspace built in Rust with egui/eframe.

## Features (Phase 1 & 3)

- Vault management (open folder, restore last vault)
- File explorer with create/rename/delete folders and notes
- Tabbed Markdown editor with autosave
- Edit/Split/Reading modes
- Markdown preview via pulldown-cmark
- Full-text search across vault
- Tags and frontmatter
- Templates
- Command palette
- Keyboard shortcuts

## Prerequisites

- Rust (stable) via rustup
- Windows 10/11 (primary target)

## Development

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo run
```

## Vault storage

Notes are stored as ordinary Markdown files in the chosen vault directory. Settings are stored in the OS application data directory (NoteCraft/settings.json). Indexes/caches are rebuildable.

## Keyboard shortcuts

- Ctrl+P: Command palette
- Ctrl+S: Save
- Ctrl+N: New note
- Ctrl+F: Find in document
- Ctrl+Shift+F: Search vault

## Known limitations

- Syntax highlighting is basic (editor uses egui TextEdit).
- File watcher events trigger a reload prompt for dirty tabs rather than auto-merge.
- Large files may impact UI responsiveness.
- No Phase 2 knowledge-graph features.

## Licensing

Licensed under GPL-3.0-only. See LICENSE.
