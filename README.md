# NoteCraft

A local-first knowledge base for Markdown, MDX and JSON — a native Rust desktop
application built with [egui](https://github.com/emilk/egui) through `eframe`.

![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-4f86f7)
![License](https://img.shields.io/badge/license-GPL--3.0-77777f)

## What it is

NoteCraft is a writing tool for people who want their notes to stay on their own
disk, in ordinary files:

- **Native, not wrapped.** One Rust binary: no webview, no Node.js runtime, no
  Tauri IPC. The window, the menus, the editor and the reading view are drawn by
  egui.
- **Real documents.** `.md`, `.mdx`, `.json`, `.yaml`, `.txt`, `.html`, `.css`,
  `.js` and `.ts` are first-class types. A JSON file is never given Markdown
  statistics, and Markdown never gets a JSON tree.
- **A Markdown engine that parses properly.** Block and inline parsing is done by
  `pulldown-cmark`; NoteCraft turns those events into native text layouts with
  real bold and italic faces, code blocks with syntax highlighting, tables,
  task checkboxes, quotes and links.
- **JSON you can inspect.** `serde_json` validation with line and column, a
  collapsible tree that preserves document key order, key filtering, expand and
  collapse, right-click to copy a JSON Pointer, and Format/Minify that refuse to
  touch invalid input.
- **MDX without pretending.** JSX and `{expressions}` are recognised, shown as
  inert source, and diagnosed (unclosed components, mismatched tags, unbalanced
  braces, ESM) with line and column. Document code is **never executed**.
- **Vaults as folders.** Open any folder; the explorer and search work on real
  files. Saves are atomic, deletes go to `.notecraft/trash`, and each save writes
  a bounded history snapshot to `.notecraft/history`.
- **No account, no server, no telemetry.**

## Install

Download the archive for your platform from
[Releases](https://github.com/fluxeniteab/notecraft/releases):

| Platform | File |
| --- | --- |
| Windows | `notecraft-windows-x64.zip` (contains `notecraft.exe`) |
| macOS (Apple silicon) | `notecraft-macos-arm64.tar.gz` |
| Linux | `notecraft-linux-x64.tar.gz` |

Extract and run the binary. There is no installer and no auto-updater in this
build; updates are a new archive.

## First run

1. Start NoteCraft. You get an empty workspace.
2. **File → Open vault…** and pick a folder. Any folder of Markdown files works.
3. `Ctrl N` for a new note, or click a file in the explorer.

The vault's `.notecraft/` folder holds trash and history. It is ignored by the
explorer and never synced by NoteCraft itself.

## Getting around

| Action | Shortcut |
| --- | --- |
| Search files and vault content | `Ctrl K` |
| Command list | `Ctrl P` |
| New Markdown note | `Ctrl N` |
| Save | `Ctrl S` |
| Save all | `Ctrl Shift S` |
| Close tab | `Ctrl W` |
| Toggle explorer / context panel | `Ctrl B` / `Ctrl \` |
| Cycle edit → split → read | `Ctrl E` |
| Edit / split / read directly | `Ctrl 1` / `Ctrl 2` / `Ctrl 3` |
| Previous / next tab | `Alt ←` / `Alt →` |
| Focus mode | `Ctrl Shift P` |
| Settings | `Ctrl ,` |
| Close dialog or menu | `Esc` |

Menus: **File**, **Edit**, **Format**, **View**, **Insert**, **Navigate**,
**Tools**, **Plugins**, **Help**. Every entry performs a real action.

## Document types

| Extension | Editor | Reading view | Validation |
| --- | --- | --- | --- |
| `.md` | proportional, marked-up | rendered Markdown | – |
| `.mdx` | proportional, JSX/expression highlighting | rendered Markdown + diagnostics | structural (never executed) |
| `.json` | monospace + line numbers | collapsible tree | `serde_json`, line/column |
| `.yaml` / `.yml` | monospace | source | `serde_yaml`, line/column |
| `.txt` | proportional | – | – |
| `.html`, `.css`, `.js`, `.ts` | monospace | source | – |
| anything else | monospace | source | – |

Unknown file types are opened as plain text and never rewritten into another
format.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — module map, design tokens, security notes
- [Feature status](FEATURES.md) — what works, what does not, and why
- [Changelog](CHANGELOG.md)
- [Build and test](build.md)

## The previous interface

The Tauri + React implementation this project started as is archived, unchanged,
in [deprecated/tauri-ui](deprecated/tauri-ui). It is kept for reference and
visual comparison, and it documents the behaviour the native build reproduces.
See its [migration map](deprecated/tauri-ui/MIGRATION.md).

## Development

```bash
cargo check --all-targets     # type-check the application and its tests
cargo test --all-targets      # unit tests
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Windows, macOS and Linux builds are produced by GitHub Actions
([.github/workflows](.github/workflows)). Rust linking needs a platform toolchain
(MSVC on Windows), so a machine without one can type-check but not link.

## License

GPL-3.0-only.
