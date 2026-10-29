# Security Policy

## Reporting

Please report vulnerabilities privately via GitHub security advisories.

## Model

- Local-first; no mandatory network.
- Strict Tauri capabilities and narrow command surface.
- Path traversal protection.
- Markdown sanitized via ammonia.
- CSP enforced.
- Optional encrypted vault uses ChaCha20-Poly1305 with Argon2 key derivation.
  Key loss means data loss; no recovery.
- Plugins are untrusted; permissions reviewed on install.

## Telemetry

None by default; opt-in only.
