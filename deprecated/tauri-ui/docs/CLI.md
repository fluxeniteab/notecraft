# notecraft-cli

Terminal access to the same encrypted vaults the desktop app uses. Built for scripts and for AI
agents that need to read or write notes locally.

## Install

The CLI is built from the same crate as the app:

```bash
cd src-tauri
cargo build --release --bin notecraft-cli
# binary: target/release/notecraft-cli
```

## Authentication

Every command that touches notes requires a profile name and password. There is no "remember me":
each invocation unlocks the vault and drops the key when it exits.

```bash
notecraft-cli list --user ada --password 'correct horse battery'
# or
export NOTECRAFT_USER=ada
export NOTECRAFT_PASSWORD='correct horse battery'
notecraft-cli list
```

`profiles` is the only command that does not need credentials.

## Commands

| Command | What it does |
| --- | --- |
| `profiles` | List local profiles with their vault paths |
| `list [folder]` | List notes (`.md`, `.markdown`, `.canvas`) |
| `read <path>` | Print a note |
| `write <path>` | Replace a note (`--content TEXT`, `--file PATH`, or stdin) |
| `append <path>` | Append text (newline-safe) |
| `new <title>` | Create a note, print its path |
| `search <query>` | Case-insensitive search across notes |
| `tasks` | List every task with its source note and line |
| `delete <path>` | Move a note to the vault trash |

Options: `--json`, `--folder DIR`, `--user`, `--password`, `--file`, `--content`, `--stdin`.

## JSON output

Use `--json` for machine-readable results. One document per call, no logging noise on stdout:

```json
{
  "profile": "ada",
  "root": "/home/ada/.local/share/NoteCraft/profiles/<id>/vault",
  "notes": [{ "path": "Tutorial.md", "name": "Tutorial.md", "size": 2140, "modified": 1793000000 }]
}
```

```bash
notecraft-cli search "roadmap" --json | jq -r '.hits[] | "\(.path):\(.line) \(.text)"'
notecraft-cli read "Tutorial.md" --json | jq -r '.content'
```

## Errors and exit codes

- `0` — success
- `1` — failure; the reason is printed on stderr
- `2` — no command given

Wrong password, unknown profile, missing note and locked vault all fail with exit code `1` and do
not write anything.

## Agent recipe

```bash
# 1. find the note
path=$(notecraft-cli search "$QUERY" --json | jq -r '.hits[0].path')
# 2. append an agent note without destroying the existing text
printf '\n## %s\n%s\n' "$(date -I)" "$SUMMARY" | notecraft-cli append "$path" --user "$USER_NAME" --password "$PASS"
```

Keep the password in the environment, never in a script's arguments where `ps` can see it.
