# NoteCraft

NoteCraft is a Tauri 2 + React + TypeScript desktop knowledge base (in progress).

## Development

```bash
npm ci
npm run dev
```

In another terminal:

```bash
cd src-tauri
cargo tauri dev
```

## Checks

```bash
npm ci
npm run build
cd src-tauri
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

## Licensing

GPL-3.0-only. See LICENSE.
