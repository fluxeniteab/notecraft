# Testing

- Frontend: Vitest + Testing Library. Run `bun run test`.
- Rust: cargo test. Run in CI.
- Use temporary vaults for destructive tests; never real user notes.
- Mock Tauri APIs at boundaries in frontend tests.
