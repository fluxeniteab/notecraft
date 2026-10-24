# NoteCraft

NoteCraft is an open-source, local-first desktop knowledge-management application built with
Tauri 2, Rust, React, and TypeScript. It aims to be a comprehensive alternative to Obsidian
while keeping Markdown files as the source of truth.

## Status

Phase 1–3 in progress. Core vault, editor, search, links, graph, canvas, properties, templates,
and themes are implemented. Automation, plugin platform depth, sync providers, and advanced
recovery features are partially implemented or planned — see `build.md` and `FEATURES.md`.

## Supported platforms

- Windows 10/11 (primary). Linux and macOS supported via Tauri 2 architecture (not yet packaged).

## Features

See `FEATURES.md` for the honest implementation checklist.

## Installation (development)

Prerequisites: Bun, Rust toolchain (for Tauri), Node-compatible environment.

```bash
bun install
bun run tauri dev
```

## Build

```bash
bun install
bun run typecheck
bun run lint
bun run test
bun run build
cd src-tauri
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
bun run tauri build
```

Rust checks run in GitHub Actions (`.github/workflows/ci.yml`) because local Rust tooling may be
unavailable. Frontend checks run with Bun.

## Vault compatibility

Notes are plain Markdown (`.md`) with YAML frontmatter. Any folder can be opened as a vault.
NoteCraft writes a `.notecraft/` directory inside the vault for trash, snapshots metadata references,
and app data; note content remains readable by other Markdown editors.

## Privacy

- No mandatory account or network connection.
- No analytics or telemetry by default; telemetry is opt-in.
- No cloud upload of notes without explicit user action.
- Strict Tauri capabilities; narrow command surface.

## Plugin security

Downloaded plugins are treated as untrusted. The plugin manager enforces a declared permission
model. Native Rust plugins are trusted executable code. Full sandboxing of third-party frontend
extensions is not claimed.

## Synchronization

A provider-independent sync layer with conflict detection exists, but no remote provider is
implemented. The app is fully functional offline.

## Known limitations

- Remote sync providers not implemented.
- PDF export relies on OS print pipeline (not yet wired).
- Semantic search requires an explicit local model download (disabled by default).
- Bundle/installer generation disabled pending icon assets.

## Contributing

See `CONTRIBUTING.md`.

## License

GPL-3.0-only. See `LICENSE`.
