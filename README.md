# NoteCraft

A desktop notes app that keeps everything on your own machine, encrypted with your password.

![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-4f86f7)
![License](https://img.shields.io/badge/license-GPL--3.0-77777f)

## What it is

NoteCraft is a Markdown knowledge base for people who want their notes to stay put:

- **Local profiles.** Every username is a separate vault in this app's data folder.
- **Encrypted at rest.** Notes are sealed with ChaCha20-Poly1305 using a key derived from your
  password with Argon2id. The password is never written to disk.
- **Fast editing.** CodeMirror 6 with source, live-preview, reading and split modes.
- **One Rust document engine.** Markdown, MDX and JSON are rendered by NoteCraft's own Rust
  engine — no third-party Markdown parser. JSON becomes a readable outline; MDX components are
  surfaced instead of mangled.
- **Real exports.** Markdown, standalone HTML, and print / save-as-PDF from the editor menu.
- **Real links.** `[[wiki links]]`, backlinks, tags, unlinked mentions and a graph view.
- **Workflow built in.** Tasks with due dates, daily notes, quick capture, templates,
  saved searches, snapshots with diff and restore, canvas boards and automations.
- **A CLI for agents.** `notecraft-cli` reads and writes the same vaults from a terminal.

No account, no server, no telemetry. Nothing leaves the machine unless you point a sync folder at it.

## Install

Download the installer for your platform from
[Releases](https://github.com/fluxeniteab/notecraft/releases):

| Platform | File |
| --- | --- |
| Windows | `.msi` or `.exe` (NSIS) |
| macOS | `.dmg` (Apple silicon) |
| Linux | `.AppImage` or `.deb` |

Builds are scanned with VirusTotal in CI and each release report is attached to the run summary.
The app checks for signed updates on launch and installs them from the in-app prompt.

## First run

1. Start NoteCraft. You land on the lock screen.
2. Choose a username and a password of at least 8 characters. That creates an encrypted vault in
   `%APPDATA%\NoteCraft\profiles\<id>\vault` (Linux: `~/.local/share/NoteCraft`, macOS:
   `~/Library/Application Support/NoteCraft`).
3. A tutorial note is seeded so you have something to edit.

The password is the only way in. There is no recovery key and no reset — if it is lost, the notes
in that profile cannot be decrypted. Export anything you cannot afford to lose.

`Ctrl+Shift+L` locks the app. Locking drops the key from memory; the vault stays on disk.

## Getting around

| Action | Shortcut |
| --- | --- |
| Command palette | `Ctrl+K` |
| Quick open | `Ctrl+P` |
| New note | `Ctrl+N` |
| Daily note | `Ctrl+Alt+N` |
| Quick capture | `Ctrl+Shift+N` |
| Save now | `Ctrl+S` |
| Search everything | `Ctrl+Shift+F` |
| Toggle sidebar / context panel | `Ctrl+B` / `Ctrl+\` |
| Focus mode | `Ctrl+Shift+F` |
| Switch theme | `Ctrl+Shift+D` |
| Lock profile | `Ctrl+Shift+L` |

Right-click menus are custom; the webview's own context menu is disabled.

## Command line

The CLI uses the same encrypted vaults and the same credentials:

```bash
notecraft-cli profiles
notecraft-cli list --user ada --password 'correct horse battery'
notecraft-cli read "Tutorial.md" --user ada --password '...'
notecraft-cli write "inbox.md" --stdin --user ada --password '...' < note.md
notecraft-cli search "graph" --json --user ada --password '...'
```

`NOTECRAFT_USER` and `NOTECRAFT_PASSWORD` work too. `--json` prints a single document, which is
what agents and scripts should use. See [docs/CLI.md](docs/CLI.md).

## Privacy and security

Read [docs/SECURITY.md](docs/SECURITY.md) for the exact scheme, what is encrypted, and the limits
of the threat model.

## Development

```bash
bun install
bun run dev          # Vite dev server on http://localhost:1420
bun run typecheck
bun run lint
bun run test
bun run build
bun run tauri dev    # full desktop app
```

Rust is built and tested in GitHub Actions (`.github/workflows/ci.yml`); the release workflows in
[.github/workflows](.github/workflows) package Windows, macOS and Linux builds.

## Documentation

- [Tutorial](docs/TUTORIAL.md) — a guided tour of every feature
- [CLI](docs/CLI.md) — terminal access for scripts and agents
- [Security](docs/SECURITY.md) — encryption scheme and threat model
- [Signing](docs/SIGNING.md) — code signing and notarization secrets
- [Feature checklist](FEATURES.md) — what is implemented and what is not
- [Changelog](CHANGELOG.md)

## License

GPL-3.0-only.
