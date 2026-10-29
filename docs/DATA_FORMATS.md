# Data Formats

NoteCraft gives every supported file type an explicit capability record
(`src-tauri/src/doc_types.rs`, mirrored in `src/lib/documentTypes.ts`): editor
language, preview mode, statistics mode, validator, and whether the file is
indexed or can hold wiki links.

| Extension | Kind | Preview | Status bar | Validated | Wiki links |
| --- | --- | --- | --- | --- | --- |
| `.md`, `.markdown` | Markdown | rendered | words / chars | — | yes |
| `.mdx` | MDX | rendered | words / chars | MDX markers | yes |
| `.json` | JSON | structured tree | lines / validity | yes (serde_json) | no |
| `.yaml`, `.yml` | YAML | escaped source | lines | — | no |
| `.html`, `.htm` | HTML | escaped source | lines | — | no |
| `.css` | CSS | escaped source | lines | — | no |
| `.js`, `.mjs`, `.cjs`, `.jsx` | JavaScript | escaped source | lines | — | no |
| `.ts`, `.mts`, `.cts`, `.tsx` | TypeScript | escaped source | lines | — | no |
| `.txt` | plain text | escaped source | lines | — | no |

Unknown extensions are kept as ordinary files. They are never reinterpreted as
Markdown, never rewritten, and never added to the text index.

## Creating files

A new file's extension decides its content:

- `.md` / `.mdx` → `# <name>`
- `.json` → `{ "title": "<name>" }` (valid JSON)
- `.yaml` → `title: <name>`
- `.html`, `.css`, `.js`, `.ts` → minimal file with the name in a comment
- `.txt` → empty

Typing `page.mdx` creates `page.mdx`, not `page.mdx.md`.

## JSON documents

JSON is data, not a programming language: parsing, viewing and formatting never
execute anything.

- Validation uses `serde_json`, so errors report a line and column.
- The JSON tab shows a collapsible tree with keys, types and values, a filter for
  keys/values, and a **Copy JSON Pointer** action per node (RFC 6901).
- **Format** pretty-prints with 2 spaces; **Minify** collapses to one line. Both
  parse first and refuse to touch invalid source.
- The preview tree is capped (20 000 nodes, depth 64) so a pathological file
  cannot freeze the window. The document itself is unchanged.

## MDX documents

MDX is edited as source and rendered non-executing. JSX component blocks render
as labelled cards, `{expressions}` and ESM lines are surfaced rather than
dropped, and Markdown around them still renders. **No MDX is executed**: there is
no JSX runtime, no import resolution and no access to Tauri commands from a
document. This is a deliberate security boundary, not a missing feature.

## Note (.md)

Markdown with optional YAML frontmatter delimited by `---`.

## Canvas (.canvas.json)

```json
{ "version": 1, "nodes": [...], "edges": [...] }
```

## Collection config

Stored as `.notecraft/collections.json` in vault or app data (versioned).

## Settings

`settings.json` in app data dir.

## Snapshot

Stored under app data `snapshots/<uuid>/content.md` with DB metadata row.

## Search index

Tantivy index in app data `search_index/`. Rebuildable from vault.
