# NoteCraft — build, test and release

NoteCraft is a native Rust application (`eframe` + `egui`). Everything needed to
build it is in the root `Cargo.toml` and `src/`.

## Local build

Requirements: a Rust toolchain (stable) and a platform linker.

| Platform | Toolchain requirement |
| --- | --- |
| Windows | MSVC build tools (`link.exe`). MinGW works too: `rustup target add x86_64-pc-windows-gnu` |
| macOS | Xcode command line tools |
| Linux | `libxkbcommon`, `libx11`, `libxcursor`, `libxi`, `libgl1`, `wayland` dev packages |

```bash
cargo run --release          # build and launch
cargo build --release        # binary in target/release/notecraft[.exe]
```

Checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo check --all-targets
```

### Windows note

If `cargo build` fails with a `link.exe` error mentioning an unknown argument,
an unrelated `link.exe` (Git for Windows ships one) is shadowing the MSVC
linker. Either use the `x86_64-pc-windows-gnu` target, or run cargo from a
"x64 Native Tools Command Prompt for VS". `cargo check` always works: it needs no
linker, so type errors and API misuse are caught even on machines that cannot
link.

## CI

| Workflow | What it does |
| --- | --- |
| [`ci.yml`](.github/workflows/ci.yml) | `fmt`, `clippy -D warnings`, `check`, `test`, and a debug build on Ubuntu, Windows and macOS |
| [`release.yml`](.github/workflows/release.yml) | On a `v*` tag or manual dispatch: builds release binaries on all three platforms, runs the tests again, and publishes archives as a GitHub Release |
| `windows_release.yml`, `linux_release.yml`, `macos_release.yml` | Removed: they built Tauri bundles, which no longer exist |

Rust verification happens in CI for this repository because the development
machine has no working MSVC linker. Nothing in this file claims a platform build
passed unless CI actually produced it.

## Release

Tag a commit and push:

```bash
git tag v0.3.0
git push origin v0.3.0
```

`release.yml` produces:

- `notecraft-windows-x64.zip` → `notecraft.exe`
- `notecraft-macos-arm64.tar.gz` → `notecraft` (the macOS runner is Apple silicon)
- `notecraft-linux-x64.tar.gz` → `notecraft`

Binaries are unsigned. There is no auto-updater, so a release is a manual
download.

## The archived interface

`deprecated/tauri-ui/` holds the previous Tauri + React implementation with its
own `package.json`. It is not built by CI and is not part of the product. To
build it for comparison, see its own `README.md`.
