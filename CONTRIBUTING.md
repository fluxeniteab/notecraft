# Contributing

1. Fork and create a feature branch.
2. Run checks before PR: `cargo fmt --all -- --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`.
3. Keep commits focused.
4. Do not commit secrets, binaries, `target/`, or personal vaults.
5. Update FEATURES.md only when a feature is implemented and tested.
6. GPL-3.0-only compatibility required for dependencies/assets.
7. Do not delete `deprecated/` — it archives the previous interface.

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for architecture and module
conventions, and [docs/TESTING.md](docs/TESTING.md) for the test layout.
