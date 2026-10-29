# Deprecated: the archived Tauri UI (now promoted back)

This directory holds the history of the 0.3.0 detour, not a copy of the
application.

## What happened

1. Up to **0.2.1**, NoteCraft was the Tauri 2 + React + TypeScript application.
2. **0.3.0** replaced it with a native `eframe`/`egui` rewrite. To keep the old
   interface, the whole Tauri app was moved into this directory as a frozen
   snapshot (`src/`, `src-tauri/`, `package.json`, docs, icons).
3. **0.4.0** promoted that snapshot back to the repository root: the Tauri
   application is the shipped app again, with all of its features. The native
   rewrite is kept, still buildable, in [`native/`](../../native).

## Where things live now

| What | Where |
| --- | --- |
| The application (Tauri 2, React, CodeMirror, Rust core) | repository root: `src/`, `src-tauri/` |
| The native eframe/egui rewrite from 0.3.0 | [`native/`](../../native) |
| How the two relate, component by component | [MIGRATION.md](MIGRATION.md) |
| Legacy documentation for the archived snapshot | the root `docs/` folder, restored with the app |

Nothing was deleted. The files that were archived here were moved back to their
original locations, so git history and paths are continuous: `git log --follow`
on any file in `src/` or `src-tauri/` crosses the archive boundary unchanged.

## Why the detour was reverted

The native build lost the interface people were using and did not implement the
AI assistant, plugin execution, encrypted profiles, graph view, canvas, database
views, automation or the CLI. Rather than ship an incomplete replacement, 0.4.0
returns to the Tauri app and keeps the native work as an experimental side
build, checked by `.github/workflows/native.yml`.

## Can the native build still be run?

Yes, if a Rust linker is installed:

```bash
cd native
cargo run --release
```

It opens real vault folders and never executes document code, but it has no
encryption, no plugins and no AI assistant. See
[`native/docs/ARCHITECTURE.md`](../../native/docs/ARCHITECTURE.md).
