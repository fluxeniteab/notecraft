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
  "permissions": ["vault:read", "ui"],
  "main": "index.js",
  "style": "style.css",
  "icon": "icon.png"
}
```

`id` must use letters, digits, dashes and underscores (max 64 characters), and
is used as the install folder name. Installing the same id replaces the package.

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
