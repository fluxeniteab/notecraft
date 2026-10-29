# Testing

## Running the checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

`cargo check` and `clippy` need no linker. `cargo test` does: MSVC build tools on
Windows, Xcode command line tools on macOS, or a MinGW toolchain with `dlltool`.

### On a machine without a working linker

`clippy --all-targets` compiles the test harness even though it does not link it,
so type and lint errors in tests still surface. Use it as the pre-push gate, and
let CI run the suite:

```bash
cargo clippy --all-targets -- -D warnings
```

Do not report a test suite as passing because clippy was clean; the two are not
the same claim.

## CI

`.github/workflows/ci.yml` runs, on Ubuntu, Windows and macOS:

1. `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`
   (Linux only),
2. `cargo check --all-targets`, `cargo test --all-targets`,
   `cargo build --release` on all three runners.

`.github/workflows/release.yml` rebuilds release binaries for tagged versions and
runs the same suite first.

## Test layout

Tests live next to the code they cover, in `#[cfg(test)] mod tests` blocks. There
are 78 of them:

| Module | Covers |
| --- | --- |
| `documents` | extension dispatch, explicit-extension creation, unknown types |
| `filesystem` | path validation (`..`, absolute, NUL, symlink escape), atomic write, UTF-8 refusal, trash, snapshot retention |
| `json` | valid/invalid parsing with line/column, escapes and Unicode, key order, flattening, pointer escaping, format/minify refusing invalid input, node cap |
| `mdx` | balanced/unbalanced braces, nested components, mismatched and unclosed tags, ESM and `<script>`, strings that contain braces, line numbers |
| `markdown` | block and inline rendering, tables, task lists, frontmatter, code fences, outline, statistics |
| `markdown::export` | HTML escaping, plain-text conversion, MDX/JSON export |
| `markdown::highlight` | token roles for JSON, YAML, JS, Markdown, MDX; no panics on odd input |
| `links` | wiki link parsing (pipelines, headings, folders), backlinks by stem, rename rewriting, frontmatter tags/properties |
| `search` | scope filtering, snippet extraction, ranking, fuzzy path matching |
| `settings` | unknown themes fall back to dark, field round-trip, defaults |
| `session` | recent-vault ordering and de-duplication |
| `app` | document creation extensions, conflict/dirty helpers, command list invariants |
| `ui::editor` | line-start lookup, layouter output for mono and prose |
| `theme` | icon geometry helpers, font family selection |

## Conventions

- Use a temporary directory per test and delete it afterwards; never touch a real
  vault. `filesystem::tests::temp_root` is the pattern.
- Assert on behaviour, not on internal counters, except where the counter is the
  contract (node caps, retention limits).
- Malformed input belongs in a test: every parser has at least one case for
  truncated, deeply nested, or non-UTF-8 input.
- Data-safety tests assert that the original bytes survive (`flatten` a bad
  document, then read the file again).

## Not covered yet

There is no UI automation: egui widgets are exercised by hand. The manual pass
for a release is: create a note of each type, edit and save, reopen after
restart, toggle every panel, run format/minify on valid and invalid JSON, open a
file with MDX errors, rename a file with incoming links, and delete then restore
from trash.
