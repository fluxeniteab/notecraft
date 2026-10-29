# Plugins

A plugin is a `.zip` you drop on **Settings → Plugins**. NoteCraft extracts it
under the app data folder, registers it, and runs its entry module.

## Package layout

```
plugin_xyz.zip
├── plugin.json      # required
├── index.js         # required (or the path in "main")
├── style.css        # optional, injected into the app
├── icon.png         # optional, shown in settings (SVG also works)
└── lib/…            # optional, anything your module imports
```

TypeScript is accepted **only** with a compiled sibling: ship `index.ts` and
`index.js`, and `main` may point at either — the runtime loads the `.js` file.

## plugin.json

```json
{
  "id": "word-count",
  "name": "Word count",
  "version": "1.0.0",
  "description": "Adds a word-count badge to the editor.",
  "author": "you",
  "permissions": ["notes:read", "ui:notify"],
  "main": "index.js",
  "style": "style.css",
  "icon": "icon.png"
}
```

`id` must use letters, digits, dashes and underscores (max 64 characters), and
is used as the install folder name. Installing the same id replaces the package.

## Permissions

Permissions are validated at install time: an unknown permission is rejected
instead of being silently ignored, so the list shown in Settings is complete.

| Permission | Unlocks |
| --- | --- |
| `notes:read` | read files, list folders, search, parse, tasks, snapshots |
| `notes:write` | overwrite existing files, rename, move, snapshot |
| `notes:create` | create files and folders |
| `notes:delete` | move a file or folder to the trash |
| `links:read` | backlinks, outgoing links, unresolved links, graph data |
| `ui:notify` | `context.notify(...)` |
| `editor:open` | reserved for opening documents |
| `settings:read` | read the active settings object |

`notes:write` and `notes:delete` are marked sensitive: they can overwrite or
remove user data, so Settings highlights them before you enable a plugin.

Plugins **cannot** reach profile or password commands, AI provider
configuration (which holds API keys), settings writes, plugin management,
backups, sync or import/export through the plugin context.

### Trust boundary (read this)

The plugin context is permission-scoped, but a plugin's JavaScript module shares
the webview realm with the application. It is **not** a sandbox: a determined
plugin can call the Tauri bridge directly. Treat plugins as trusted code.

Because of that, installing a plugin never runs it. A plugin only executes after
you explicitly enable it in **Settings → Plugins**, and disabling it removes its
styles and stops it from being started on the next launch.

## Runtime API

The entry module is loaded as an ES module and must export `activate` (or a
default function). It receives a context object:

```js
export async function activate({ pluginId, version, api, toast, store }) {
  toast.success(`Word count ${version} loaded`);
  const state = store.getState();      // app store (read-only use is safest)
  const files = await api.listAllFiles();
  document.body.dataset.wordCount = String(files.length);
}
```

- `api` is the same typed bridge the app uses (`readFile`, `writeFile`, `search`,
  `openTab` via `store`, …).
- `toast` is the notification helper.
- `store` is the Zustand store; `store.getState()` reads state, and its actions
  (for example `openTab`) can be called.

CSS from `style.css` is injected once. Because ES modules cannot be unloaded,
disabling a plugin removes its stylesheet but leaves already-registered event
listeners until the app restarts.

## Security

Plugins run inside the app with the app's own privileges: they can read and
write vault files through the API. There is no sandbox. Install packages you
trust, check the manifest, and prefer plugins whose source you can read.
NoteCraft never downloads plugins on its own — the catalog shipped in the app is
metadata only, and installation always requires you to pick a local `.zip`.
