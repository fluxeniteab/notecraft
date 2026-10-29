# Archived: the Tauri + React front end

This directory is a **frozen copy of the NoteCraft interface as it existed before
the native migration**. It is kept for reference, visual comparison, debugging
and recovery. **Do not delete it, and do not edit it to match the new
architecture** — it is a historical snapshot, not a living part of the build.

## What is in here

```
tauri-ui/
  src/            React + TypeScript UI (components, store, lib, styles.css)
  src-tauri/      The Tauri (Rust) shell: commands, vault, markdown engine, plugins
  docs/           Legacy documentation: AI assistant, CLI, plugins, signing, tutorial
  package.json    Frontend dependencies (React, CodeMirror 6, Radix, Tailwind)
  bun.lock        The lockfile that produced the last legacy build
  vite.config.ts  Vite configuration, including the Tauri dev server settings
  tsconfig*.json  TypeScript configuration
  tailwind.config.ts, postcss.config.js, eslint.config.js
  index.html
  app-icon.jpg    The icon used by the Tauri bundle
```

## What it was

- **Frontend:** React 18 + TypeScript, bundled by Vite, styled with Tailwind
  and hand-written CSS custom properties (`src/styles.css` holds the design
  tokens the native theme was ported from: `--nc-bg`, `--nc-accent`, `--nc-r-*`).
- **Editor:** CodeMirror 6 (Markdown, MDX, JSON, YAML language modes).
- **Shell:** Tauri 2 with a Rust backend exposing `#[tauri::command]` functions
  over IPC, SQLite (via `rusqlite`) for metadata and history, Tantivy for search,
  Argon2id + ChaCha20-Poly1305 for encrypted profiles, and ZIP-based plugins that
  ran inside the webview.
- **Release:** Tauri bundler produced `.msi`, `.exe`, `.dmg` and `.deb`.

## Why it was kept

1. **Design fidelity.** The native theme was ported token-for-token from
   `src/styles.css`; the original values stay readable here.
2. **Behaviour reference.** Interaction details (tab behaviour, context menus,
   settings copy, keyboard shortcuts) are still the specification for the native
   build.
3. **Recovery.** If a native feature turns out to be worse than the Tauri
   implementation, the old code is the starting point for repairing it.
4. **Data migration.** The vault layout, settings JSON and history snapshot
   naming are documented by this code.

## How it relates to the native app

The native application lives at the repository root (`Cargo.toml`, `src/`). It
is a **Rust + egui** program with no webview, no Node.js runtime and no Tauri
IPC. It reads the same vaults and the same `settings.json`, so an existing vault
opens unchanged.

See `MIGRATION.md` in this directory for the component-by-component mapping.

## Can it still be built?

It is a self-contained snapshot, so yes — with a Node/Bun toolchain and the Tauri
prerequisites installed:

```bash
cd deprecated/tauri-ui
bun install --frozen-lockfile
bun run dev          # browser preview of the UI only
bunx tauri dev       # the full desktop app
```

No CI workflow builds this directory any more; it is not part of the shipped
product.
