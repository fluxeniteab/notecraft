# Security Policy

## Reporting

Report vulnerabilities privately through GitHub security advisories. Include the
document type, the file contents if they are safe to share, and the steps that
reproduce the problem. Anything that lets a document execute code, read outside
the vault, or overwrite a file without confirmation is treated as a security bug.

## Model

- **Local-first.** No network sockets, no accounts, no telemetry, no updater.
- **Native shell.** Rust + egui; there is no webview, no IPC surface and no
  JavaScript runtime, so there is no CSP to weaken and no privileged API a
  document could reach.
- **Path containment.** Vault-relative paths are validated (no absolute paths, no
  `..`, no NUL) and canonicalised so a symlink cannot escape the vault.
- **Atomic writes; deletes go to trash.** Non-UTF-8 files are refused rather than
  re-encoded.
- **No execution of document content.** MDX is analysed and displayed, never run;
  JSON values are rendered as text; HTML is shown as source. Exports escape their
  content.
- **External links.** Only `http`, `https` and `mailto` reach the OS handler.
- **Plugins.** Not executed at all in this build; nothing is loaded and the UI
  states that rather than implying a sandbox.
- **Encryption.** Not implemented here. The previous Tauri build's encrypted
  profiles are not readable by this build. Because no key is ever derived or held,
  no memory-erasure claim is made.

The full description, including what is deliberately out of scope, is in
[docs/SECURITY.md](docs/SECURITY.md).

## Telemetry

None. `settings.json` keeps a `telemetry_opt_in` field for file compatibility
only; nothing reads it and nothing is sent.
