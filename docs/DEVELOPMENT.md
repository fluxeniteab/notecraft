# Development

## Prerequisites

- Rust stable (1.85 or newer) via [rustup](https://rustup.rs)
- A platform linker: MSVC build tools on Windows, Xcode command line tools on
  macOS, `libxkbcommon`/`libgl1`/`wayland` development packages on Linux
- Optional: a `pkg-config` setup on Linux so `rfd` and `winit` find their
  dependencies

Nothing else. There is no Node.js, Bun, or Tauri CLI involvement in this
application.

## Everyday commands

```bash
cargo run                     # launch the app
cargo check --all-targets     # fast type check (no linker needed)
cargo test --all-targets      # unit tests (needs a linker)
cargo fmt --all                # format
cargo clippy --all-targets -- -D warnings
cargo build --release         # target/release/notecraft[.exe]
```

Set `RUST_LOG=debug` and the app prints startup and filesystem decisions to the
console via `env_logger`.

## Where things live

`src/app/mod.rs` is the hub: it owns the vault, the tabs, the panels and the
settings, and every menu item calls a method on `App`. UI modules take `&mut App`
and draw; they do not own state. The Markdown, MDX, JSON, search and filesystem
modules are pure Rust and never depend on a `Ui`.

A good rule when adding something: if it decides *what* happens, it goes in
`app`; if it decides *how it looks*, it goes in `ui`; if it transforms content,
it goes in its own module and gets a test.

## Adding a document type

1. Add a variant to `documents::Kind` and a `Capabilities` constant.
2. Wire it in `Kind::capabilities`, `Kind::from_path`, `extension`, `label`,
   `export_extension`, and `is_supported_document`.
3. Give it a starter body in `app::App::new_document` if it is not plain text.
4. If it needs validation, extend `Validator` and the `Tab::validate` match.
5. Add a test for `Kind::from_path("x.ext")` and for the starter body.

Nothing else needs to change: the editor picks its syntax from `Capabilities`,
the status bar reads `stats`, and the sidebar shows the right icon.

## Adding a command

1. Add a variant to `CommandId` in `app/mod.rs`.
2. Handle it in `App::run_command`.
3. Add it to `App::command_list` so it appears in `Ctrl P`.
4. If it has a shortcut, consume the key in `App::handle_shortcuts`.
5. If it belongs in a menu, add a row in the matching `ui/menubar.rs` function.

Menu rows are ordinary functions: `row(ui, app, Some(Icon::X), "Label",
Some("Ctrl X"), enabled)`. A menu item that does nothing is a bug, not a
placeholder.

## Theme

Colours, radii and spacing are in `theme/`. `Palette` holds colour tokens for
both modes; `space` and `radius` hold metrics. Widgets read them through
`&Theme`, so light mode and a different accent do not need per-widget changes.
Do not hardcode a `Color32` in a UI module; add a token instead.

## Editor

`ui/editor.rs` wraps `egui::TextEdit` with a `Layouter` that builds a
`LayoutJob` per frame: Markdown gets scaled headings and marked emphasis,
monospace kinds get the tokenizer in `markdown/highlight.rs`. The line gutter is
painted alongside, not part of the text buffer, so selection and wrapping stay
correct. Keep it that way — a custom text widget would have to re-implement IME,
selection and undo.

## Commit hygiene

Keep the tree green: `cargo fmt --all`, `cargo clippy --all-targets -- -D
warnings` and, where a linker exists, `cargo test --all-targets`. Document user-
visible changes in `CHANGELOG.md` and update `FEATURES.md` only when the
behaviour is implemented and verified.
