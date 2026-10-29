# Contributing

1. Fork and create a feature branch.
2. Run checks before PR.
3. Keep commits focused.
4. Do not commit secrets, binaries, or personal vaults.
5. Update FEATURES.md only when a feature is implemented and tested.
6. GPL-3.0-only compatibility required for dependencies/assets.

## Stack

- **Frontend:** Svelte 5 + TypeScript, Vite, hand-written CSS tokens (`src/app.css`), CodeMirror 6.
- **Desktop/runtime:** Tauri 2 with a Rust backend in `src-tauri/`.
- **Archived UI:** the previous React interface lives in `deprecated/tauri-ui/react-src/` and must
  not be built or deleted. New UI work belongs in `src/`.

## Checks

```bash
bun install
bun run typecheck
bun run test
bun run build
cd src-tauri && cargo fmt --all -- --check && cargo check --all-targets && cargo test --all-targets
```

Rust linking may be unavailable on a machine without MSVC build tools; CI runs the Rust gates.
Frontend checks must pass locally before a pull request.
