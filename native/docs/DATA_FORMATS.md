# Data formats

## Document registry

Every document kind is one `Capabilities` record in `src/documents/mod.rs`. The UI
asks this table what a file is; nothing guesses from the text. Adding a type
means adding a record and its arm in `Kind::capabilities`, `Kind::from_path` and
`export_extension`.

| Field | Meaning |
| --- | --- |
| `syntax` | highlighting dialect for the editor and fenced code |
| `stats` | `Prose` (words/chars), `Data` (lines/chars), `Code` (lines) |
| `validator` | `Json`, `Yaml` or `None` |
| `mono` | monospace editor with a line-number gutter |
| `preview` | a rendered reading view is meaningful |
| `indexable` | content participates in vault search |
| `metadata` | frontmatter is scanned for tags and properties |

| Extension | Kind | Syntax | Stats | Validator | Mono | Preview | Indexed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `.md`, `.markdown` | Markdown | Markdown | prose | – | no | yes | yes |
| `.mdx` | MDX | MDX | prose | MDX analysis | no | yes | yes |
| `.json` | JSON | JSON | data | `serde_json` | yes | tree | yes |
| `.yaml`, `.yml` | YAML | YAML | data | `serde_yaml` | yes | source | yes |
| `.txt` | Text | plain | prose | – | no | source | yes |
| `.html`, `.htm` | HTML | HTML | code | – | yes | source | yes |
| `.css` | CSS | CSS | code | – | yes | source | yes |
| `.js`, `.jsx` | JavaScript | code | code | – | yes | source | yes |
| `.ts`, `.tsx` | TypeScript | code | code | – | yes | source | yes |
| anything else | Unknown | plain | code | – | yes | source | no |

Unknown files are opened as plain text, keep their bytes, and are never rewritten
into a supported format.

## Vault layout

A vault is any folder. NoteCraft adds one hidden directory:

```
<vault>/
  notes/…                   the user's files, untouched
  .notecraft/
    trash/20261028-141233-old.md     deleted files, timestamped
    history/notes__plan.md.20261028-141233-412.snap
```

- **Trash names** are `<yyyyMMdd-HHmmss>-<original name>`. Nothing in trash is
  read by the explorer, search or links.
- **History names** are `<path with / → __>.<yyyyMMdd-HHmmss-mmm>.snap`. A
  snapshot is a byte copy of the file before a save that replaces it, pruned to
  `snapshot_retention` entries per document (`settings.json`, default 50).
- `.notecraft` is in `excluded_folders` by default.

## Application data

`settings.json` and `session.json` live in `dirs::data_dir()/NoteCraft`
(`%APPDATA%\NoteCraft` on Windows, `~/.local/share/NoteCraft` on Linux,
`~/Library/Application Support/NoteCraft` on macOS). Both are written via a
`.tmp` file and a rename.

### settings.json

Field names match the Tauri build so an existing file loads unchanged; unknown
fields are ignored and missing fields take defaults.

```json
{
  "theme": "dark",                 // "dark" | "light" | "system" | anything → dark
  "accent": "#4f86f7",
  "editor_font_size": 15,
  "editor_line_height": 1.75,
  "editor_font_family": "Inter, system-ui, sans-serif",
  "reading_width": 760,
  "autosave_ms": 800,
  "autosave_enabled": true,
  "snapshot_interval_min": 5,
  "snapshot_retention": 50,
  "excluded_folders": [".git", "node_modules", ".notecraft"],
  "hidden_files": false,
  "reduced_motion": false,
  "telemetry_opt_in": false,
  "semantic_search_enabled": false,
  "explorer_sort": "name",         // "name" | "modified"
  "explorer_group_dirs": true,
  "shortcuts": {},
  "show_line_numbers": true,
  "wrap_code": false,
  "reopen_last_session": true,
  "preview_width": 0.5
}
```

`telemetry_opt_in` and `semantic_search_enabled` exist for file compatibility
only: no telemetry is collected and no embedding provider is used by this build.

### session.json

```json
{
  "vault": "C:\\Users\\me\\Notes",
  "tabs": ["inbox.md", "data/config.json"],
  "active": "inbox.md",
  "left_visible": true,
  "right_visible": false,
  "left_width": 248.0,
  "right_width": 292.0,
  "recent_vaults": ["C:\\Users\\me\\Notes"]
}
```

Paths are stored as `/`-separated vault-relative strings. A tab whose file no
longer exists is dropped on load instead of showing an error.

## Export

`File → Export…` writes either:

- **HTML** — one self-contained page. Markdown is rendered by `markdown::render_html`
  (headings, lists, tables, code, quotes, task states); everything else is escaped
  source in a `<pre>` block, so exporting a JSON or MDX file cannot produce an
  active page. The filename gets the extension the user typed.
- **Plain text** — Markdown syntax stripped, JSON left as source.

`Edit → Copy document as text` puts the plain-text rendering on the clipboard.

## JSON conventions

- Object key order is preserved (`serde_json` with `preserve_order`).
- Pointers use **RFC 6901** (`/items/0/name`, `~0` for `~`, `~1` for `/`), shown
  in the tree's context menu and the status bar. JSONPath is not implemented.
- Formatting options: 2 spaces, 4 spaces or a tab; Minify removes all whitespace
  outside strings.
- Format and Minify are disabled while the document does not parse, so invalid
  source is never rewritten.
- The tree is read-only and capped at 20,000 nodes; the source editor is not.

## MDX conventions

MDX is stored exactly as typed. Analysis is structural:

- braces must balance outside strings and JSX attribute strings;
- component tags must nest correctly, with open/close locations reported;
- ESM (`import`/`export`) and `<script>` at the start of a line are flagged;

Line and column numbers are 1-based. Nothing is compiled and nothing executes,
so an MDX file always saves and reopens byte-for-byte.
