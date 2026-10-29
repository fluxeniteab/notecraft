# Development

## Prerequisites

- Bun
- Rust toolchain (rustup) with cargo
- Node 20+

## Run

```bash
bun install
bun run tauri dev
```

## Checks

```bash
bun run typecheck
bun run lint
bun run test
bun run build
cd src-tauri && cargo fmt --all -- --check && cargo check --all-targets && cargo test --all-targets && cargo clippy --all-targets -- -D warnings
```

## Project layout

See `docs/ARCHITECTURE.md`.
