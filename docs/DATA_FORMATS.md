# Data Formats

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
