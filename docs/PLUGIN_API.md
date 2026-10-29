# Plugin API

## Manifest (plugin.json)

```json
{
  "id": "com.example.plugin",
  "name": "Example",
  "version": "1.0.0",
  "author": "Author",
  "description": "...",
  "api_version": "1",
  "entry": "index.js",
  "permissions": ["notes:read", "commands:register"]
}
```

## Permissions

Declared in manifest and shown to user on install. Core permissions: notes:read, notes:write,
commands:register, panels:register, search:provider, themes:register, importers, exporters.

## Lifecycle

Plugins are stored in the app data plugins directory. Enable/disable/uninstall via plugin
manager. Incompatible api_version plugins are refused.

## Trust model

Third-party plugins are untrusted code. Native Rust plugins are trusted executables.
No unrestricted filesystem/process/network access is granted; capabilities are enforced
by the host.
