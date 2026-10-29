# Deprecated: the archived React UI

This directory preserves the previous frontends for reference. The shipped
application now uses a Svelte 5 interface in the repository root.

## The React snapshot (`react-src/`)

`react-src/` is the complete React + TypeScript frontend as it shipped in
0.4.0: every component, stylesheet, store, editor extension, test, Vite config
and asset. It is kept so the previous design, interaction details and feature
implementations can be compared or recovered. It is **not built** by the Tauri
configuration and its toolchain is deliberately not installed. To inspect a
file, read it in place; to run it, its original `package.json` dependencies must
be restored in an isolated copy.

Do not delete this directory as part of frontend clean-up. `MIGRATION.md` maps
its components to the native experiment; the authoritative current UI lives in
`src/` at the repository root.

## The 0.3.0 detour

This directory also holds the history of the 0.3.0 native experiment, not a
copy of the application.

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
| The current application (Tauri 2, Svelte 5, CodeMirror, Rust core) | repository root: `src/`, `src-tauri/` |
| The archived React frontend from 0.4.0 | this directory: `react-src/` |
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
