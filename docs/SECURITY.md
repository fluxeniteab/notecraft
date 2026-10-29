# Security model

NoteCraft is local-first. There is no server to send notes to, and no account that lives anywhere
but this machine.

## Profiles

Each profile is a directory:

```
%APPDATA%\NoteCraft\                       (Windows; similar per-OS data dir elsewhere)
├── profiles.json                          profile names, salts, verifiers
└── profiles\<uuid>\
    ├── vault\                             encrypted notes
    ├── notecraft.db                       links, tags, tasks, snapshots index
    └── index\                             search index
```

Two profiles never share a vault, database or search index.

## Key derivation

| Step | Detail |
| --- | --- |
| Salt | 16 random bytes per profile, stored in `profiles.json` |
| KDF | Argon2id, `m = 19 MiB`, `t = 2`, `p = 1`, 32-byte output |
| Password storage | none — only a verifier blob is stored |

The verifier is the AEAD encryption of `notecraft-profile:<profile id>` under the derived key. A
wrong password fails to decrypt it, so the app refuses to open the vault before reading any note.

## Note encryption

Every note is stored in a small container:

```
"NCV1" | nonce (12 bytes) | ChaCha20-Poly1305 ciphertext + tag
```

- A fresh random nonce is generated for every write.
- Writes are atomic: write to a temporary file, `fsync`, then rename over the target.
- A tampered or truncated file fails authentication and is reported as an error instead of being
  shown as if it were valid.
- Files without the `NCV1` magic are treated as legacy plaintext and encrypted in place the first
  time the profile is unlocked.

History snapshots are written through the same encrypted path.

## What is encrypted

| Data | Encrypted |
| --- | --- |
| Notes (`.md`) | yes |
| History snapshots | yes |
| Search index | no — derived data, stored inside the profile directory |
| `notecraft.db` (links, tags, tasks, snapshots index) | no — metadata only, no note bodies |
| `profiles.json` | no — names, salt, verifier |
| Settings | no |

The search index contains tokenised note text. It lives in the profile directory so file system
permissions match the vault, but it is not itself encrypted. If that is unacceptable, disable
search for the profile or move the data directory to an encrypted volume.

## Threat model

Protects against:

- Someone reading the vault files, a backup, or a stolen disk image without the password.
- Another local profile in the same app installation.
- A tampered note file (AEAD authentication failure is surfaced, not ignored).
- A tampered application update (updates are signed; the updater rejects unsigned payloads).

Does not protect against:

- Malware running as your user while the vault is unlocked. The key is in process memory.
- Shoulder surfing, keyloggers, or a compromised OS.
- An attacker who knows your password.
- Forensic recovery of deleted plaintext that predates encryption.

## Operational notes

- Lock the app (`Ctrl+Shift+L`) before leaving a shared machine.
- The password cannot be reset. Losing it means losing the notes in that profile.
- Passwords shorter than 8 characters are rejected.
- Changing the password re-encrypts every note under a new salt and key; if the process is
  interrupted, the old password still opens the vault because the registry is updated last.

## Reporting

Open an issue for anything that contradicts this document. Please do not attach real notes.
