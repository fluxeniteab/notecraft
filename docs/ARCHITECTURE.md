# NoteCraft Architecture

## Overview

Tauri 2 desktop app. React + TypeScript frontend (Vite) talks to a Rust backend via typed
Tauri commands. Markdown files are the source of truth; SQLite stores derived indexes and
app metadata.

## Frontend

- `src/store/appStore.ts` — Zustand transient UI/workspace state.
- `src/lib/tauri.ts` — typed invoke wrappers.
- `src/components/*` — UI.
- CodeMirror 6 editor, Sigma+Graphology graph, React Flow canvas.

## Backend (src-tauri/src)

Modules: error, settings, state, vault, markdown, metadata, search, links, graph, history,
tasks, templates, workspace, automation, plugins, sync, security, import_export, commands.

## Data flow

UI -> tauri invoke -> command -> module -> filesystem/SQLite/Tantivy. Results returned as
typed JSON. Long operations run on Tauri's blocking thread pool.

## Storage

- Vault: user folder of Markdown files.
- App data: OS data dir (`NoteCraft/`): settings.json, notecraft.db, search_index/,
  snapshots/, security/.
- `.notecraft/` inside vault: trash.

## Security

Strict capabilities in `src-tauri/capabilities/default.json`. Path validation in vault module.
CSP in tauri.conf.json. Markdown rendered via pulldown-cmark + ammonia sanitization.
