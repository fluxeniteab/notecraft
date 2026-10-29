# NoteCraft tutorial

A guided tour of the app, in the order most people need it.

## 1. Profiles and the lock screen

- A **profile** is one local user with its own encrypted vault. Two profiles never share files,
  indexes or settings.
- The first screen asks for a username and a password (minimum 8 characters). That password is
  turned into a key with Argon2id and never stored.
- `Ctrl+Shift+L` locks the app. Locking forgets the key; the vault stays on disk.
- The profile list on the lock screen only shows names, never hints about content.

Where the files live:

| Platform | Path |
| --- | --- |
| Windows | `%APPDATA%\NoteCraft\profiles\<id>\vault` |
| macOS | `~/Library/Application Support/NoteCraft/profiles/<id>/vault` |
| Linux | `~/.local/share/NoteCraft/profiles/<id>/vault` |

`profiles.json` next to that folder holds the profile names, a random salt and a verifier blob.

## 2. Writing

The editor has four modes, switchable from the toolbar:

- **Source** — raw Markdown.
- **Live** — Markdown syntax hides on lines you are not editing and reveals under the cursor.
- **Reading** — rendered note, no editing.
- **Split** — source and preview side by side.

Notes save automatically after you stop typing (`Editor → Autosave delay`). `Ctrl+S` forces a save.
If a save fails the tab stays open and the status bar shows the error instead of pretending it saved.

## 3. Linking notes

- `[[Another note]]` creates a link. `[[Another note#Heading]]` links to a heading.
- `[[#Heading]]` links inside the current note.
- The **Outline** tab in the right sidebar lists every heading in the open note; click one to jump.
- The **Links** tab shows backlinks: which notes point here.
- The **Tags** tab groups `#tags`, including nested `#area/project` tags.
- **Unlinked mentions** finds notes that name this note without linking it.

Renaming a note offers to rewrite the links that point at it.

## 4. Search

- `Ctrl+Shift+F` opens search. Results show a snippet and the note path.
- Toggle folders, tags and dates into the query from the filter row.
- Saved searches are reusable: name a query and it appears in the palette and the sidebar.
- Recent searches are remembered for the session.

## 5. Tasks

Every `- [ ]` line is a task. Tasks appear in the **Tasks** panel with their source note, so the
note stays the source of truth. Due dates come from `@2026-11-02` style dates; priorities from
`!high`, `!med`, `!low`.

Clicking a task jumps to the exact line. Ticking it in the panel rewrites the Markdown.

## 6. Daily notes, capture and templates

- `Ctrl+Alt+N` opens today's daily note, created from the configured template if one exists.
- `Ctrl+Shift+N` opens quick capture and appends to the inbox note.
- Templates live in `templates/` inside the vault. `{{title}}`, `{{date}}`, `{{time}}` and
  `{{folder}}` are filled in when a note is created from a template.

## 7. History and safety

- Snapshots are taken automatically while you edit and can be taken by hand.
- Open the **History** tab to list, diff and restore a snapshot.
- Restoring writes the snapshot back into the note and keeps the current version in history.

## 8. Graph, canvas and automations

- **Graph** draws notes and links. Click a node to open the note; `Esc` closes the view.
- **Canvas** is a free-form board stored as a `.canvas` file inside the vault.
- **Automations** run simple rules (for example "append a line to the daily note when a tag
  appears"). Every rule can be dry-run and shows its run log.

## 9. Themes and settings

- Two themes only: **dark** (default) and **light**. `Ctrl+Shift+D` switches, and the choice is
  stored in settings.
- **Appearance** controls the accent colour.
- **Editor** controls font size, line height, reading width, autosave delay, snapshot cadence and
  the editor modes shown in the toolbar.
- **Files & history** controls excluded folders, hidden files and trash behaviour.

## 10. Command line

```bash
notecraft-cli profiles
notecraft-cli list --user ada --password 'correct horse battery'
notecraft-cli new "Meeting notes" --folder meetings --user ada --password '...'
notecraft-cli search "budget" --json --user ada --password '...'
```

The CLI unlocks the same encrypted vault and writes through the same encryption path as the app.
See [CLI.md](CLI.md).

## Troubleshooting

**"Wrong password"** — the profile exists but the password did not decrypt its verifier. Check the
username you selected; usernames are case-insensitive.

**Nothing appears in backlinks** — open the note once so its links are indexed, or use
`Rebuild index` in the command palette.

**A note was edited outside the app** — the file watcher notices and reloads it. If the editor had
unsaved text, the app keeps your version and asks before overwriting.

**The vault is locked after a crash** — restart and unlock with your password. Nothing is lost;
the key only ever lived in memory.
