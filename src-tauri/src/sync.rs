use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::State;

pub fn status(state: &State<AppState>) -> AppResult<Value> {
    let g = state.inner.lock();
    let root = g.vault_root.clone();
    let mut files = 0usize;
    if let Some(r) = root {
        let excluded = g.settings.excluded_folders.clone();
        files = crate::vault::walk_markdown(&r, &excluded).len();
    }
    Ok(serde_json::json!({
        "provider": "none",
        "status": "local-only",
        "pending": 0,
        "files": files,
        "note": "Remote sync provider not configured. Local conflict handling foundation available."
    }))
}

// ---------- folder provider ----------
// A deliberately boring, local-first provider: mirror Markdown files into a
// folder the user controls (a mounted drive, a Dropbox/Syncthing folder, ...).
// Conflicts are reported and never overwritten automatically.

#[derive(Debug, Clone, Serialize)]
pub struct SyncFileStatus {
    pub path: String,
    pub status: String, // in_sync | local_newer | remote_newer | conflict | local_only | remote_only
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn mtime_secs(path: &Path) -> i64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

struct Context {
    root: PathBuf,
    remote: PathBuf,
    excluded: Vec<String>,
}

fn context(state: &State<AppState>) -> AppResult<Context> {
    let g = state.inner.lock();
    let root = g.vault_root.clone().ok_or(AppError::NoVault)?;
    let remote = g
        .settings
        .sync_folder
        .clone()
        .ok_or_else(|| AppError::Invalid("sync folder is not configured".into()))?;
    if remote.as_os_str().is_empty() {
        return Err(AppError::Invalid("sync folder is not configured".into()));
    }
    Ok(Context {
        root,
        remote,
        excluded: g.settings.excluded_folders.clone(),
    })
}

fn recorded(state: &State<AppState>, rel: &str) -> Option<(String, String)> {
    let g = state.inner.lock();
    let conn = g.db.as_ref()?;
    conn.query_row(
        "SELECT hash, remote_hash FROM sync_state WHERE path=?1",
        rusqlite::params![rel],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .ok()
}

fn record(state: &State<AppState>, rel: &str, hash: &str) {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        let _ = conn.execute(
            "INSERT INTO sync_state(path,hash,mtime,remote_hash,status) VALUES(?1,?2,?3,?4,'synced') \
             ON CONFLICT(path) DO UPDATE SET hash=excluded.hash,mtime=excluded.mtime,remote_hash=excluded.remote_hash,status='synced'",
            rusqlite::params![rel, hash, chrono::Utc::now().timestamp(), hash],
        );
    }
}

fn classify(state: &State<AppState>, rel: &str, local: &Path, remote: &Path) -> SyncFileStatus {
    let local_exists = local.exists();
    let remote_exists = remote.exists();
    let status = match (local_exists, remote_exists) {
        (false, false) => "in_sync",
        (true, false) => "local_only",
        (false, true) => "remote_only",
        (true, true) => {
            let local_hash = std::fs::read(local)
                .map(|b| sha256_hex(&b))
                .unwrap_or_default();
            let remote_hash = std::fs::read(remote)
                .map(|b| sha256_hex(&b))
                .unwrap_or_default();
            if local_hash == remote_hash {
                "in_sync"
            } else {
                match recorded(state, rel) {
                    // Baseline says only one side moved since the last sync.
                    Some((last_local, _)) if last_local == local_hash => "remote_newer",
                    Some((_, last_remote)) if last_remote == remote_hash => "local_newer",
                    Some(_) => "conflict",
                    // No baseline yet: fall back to modification times.
                    None if mtime_secs(local) >= mtime_secs(remote) => "local_newer",
                    None => "remote_newer",
                }
            }
        }
    };
    SyncFileStatus {
        path: rel.to_string(),
        status: status.to_string(),
    }
}

fn pairs(state: &State<AppState>) -> AppResult<Vec<(String, PathBuf, PathBuf)>> {
    let ctx = context(state)?;
    let mut out = Vec::new();
    for path in crate::vault::walk_markdown(&ctx.root, &ctx.excluded) {
        let rel = crate::vault::to_rel(&ctx.root, &path);
        if rel.is_empty() {
            continue;
        }
        let remote_path = ctx.remote.join(&rel);
        out.push((rel, path, remote_path));
    }
    Ok(out)
}

pub fn folder_status(state: &State<AppState>) -> AppResult<Vec<SyncFileStatus>> {
    pairs(state)?
        .iter()
        .map(|(rel, local, remote)| classify(state, rel, local, remote))
        .collect()
}

pub fn push(state: &State<AppState>) -> AppResult<Vec<SyncFileStatus>> {
    for (rel, local, remote) in pairs(state)? {
        let status = classify(state, &rel, &local, &remote);
        match status.status.as_str() {
            "local_only" | "local_newer" => {
                if let Some(parent) = remote.parent() {
                    std::fs::create_dir_all(parent).map_err(AppError::Io)?;
                }
                std::fs::copy(&local, &remote).map_err(AppError::Io)?;
                let hash = std::fs::read(&local)
                    .map(|b| sha256_hex(&b))
                    .unwrap_or_default();
                record(state, &rel, &hash);
            }
            // Conflicts need a human decision; never overwrite either side.
            _ => {}
        }
    }
    folder_status(state)
}

pub fn pull(state: &State<AppState>) -> AppResult<Vec<SyncFileStatus>> {
    for (rel, local, remote) in pairs(state)? {
        let status = classify(state, &rel, &local, &remote);
        match status.status.as_str() {
            "remote_only" | "remote_newer" => {
                let content = std::fs::read_to_string(&remote).map_err(AppError::Io)?;
                crate::vault::write_file_atomic(&ctx_root(state)?, &rel, &content)?;
                let _ = crate::search::index_note(state, &rel, &content);
                let g = state.inner.lock();
                if let Some(conn) = g.db.as_ref() {
                    let _ = crate::links::rebuild_for_note(conn, &rel, &content);
                }
                drop(g);
                let hash = sha256_hex(content.as_bytes());
                record(state, &rel, &hash);
            }
            _ => {}
        }
    }
    folder_status(state)
}

fn ctx_root(state: &State<AppState>) -> AppResult<PathBuf> {
    let g = state.inner.lock();
    g.vault_root.clone().ok_or(AppError::NoVault)
}
