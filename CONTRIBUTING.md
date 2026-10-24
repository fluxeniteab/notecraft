# Contributing to NoteCraft

## Development rules

- Rust only for application logic and UI.
- Use egui/eframe for GUI.
- Keep changes small and coherent.
- Run formatting, checks, tests, clippy before submitting.

## Commands

```bash
cargo fmt --all
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

## Pull requests

- Describe changes and tests.
- Do not include secrets or user data.
- Preserve GPL-3.0-only licensing.
