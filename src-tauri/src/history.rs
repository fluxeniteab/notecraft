use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub note_path: String,
    pub created_at: String,
    pub hash: String,
    pub size: usize,
    pub label: Option<String>,
}

fn snapshots_dir() -> PathBuf {
    crate::settings::data_dir().join("snapshots")
}

fn hash_content(c: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(c.as_bytes());
    hex::encode(h.finalize())
}

pub fn snapshot_now(
    state: &State<AppState>,
    rel: &str,
    label: Option<String>,
) -> AppResult<Snapshot> {
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone().ok_or(AppError::NoVault)?
    };
    let content = crate::vault::read_file(&root, rel)?;
    let id = uuid::Uuid::new_v4().to_string();
    let dir = snapshots_dir().join(&id);
    std::fs::create_dir_all(&dir).map_err(AppError::Io)?;
    std::fs::write(dir.join("content.md"), &content).map_err(AppError::Io)?;
    let hash = hash_content(&content);
    let size = content.len();
    let created_at = chrono::Utc::now().to_rfc3339();
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            conn.execute(
                "INSERT INTO snapshots(id,note_path,created_at,hash,size,label,data_path) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                rusqlite::params![id, rel, created_at, hash, size as i64, label, dir.to_string_lossy().to_string()],
            ).map_err(AppError::Sqlite)?;
        }
    }
    cleanup(state, rel);
    Ok(Snapshot {
        id,
        note_path: rel.to_string(),
        created_at,
        hash,
        size,
        label,
    })
}

pub fn list(state: &State<AppState>, rel: &str) -> AppResult<Vec<Snapshot>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let mut stmt = conn.prepare("SELECT id,note_path,created_at,hash,size,label FROM snapshots WHERE note_path=?1 ORDER BY created_at DESC").map_err(AppError::Sqlite)?;
        let rows = stmt
            .query_map(rusqlite::params![rel], |r| {
                Ok(Snapshot {
                    id: r.get(0)?,
                    note_path: r.get(1)?,
                    created_at: r.get(2)?,
                    hash: r.get(3)?,
                    size: r.get::<_, i64>(4)? as usize,
                    label: r.get(5)?,
                })
            })
            .map_err(AppError::Sqlite)?;
        for r in rows {
            if let Ok(s) = r {
                out.push(s);
            }
        }
    }
    Ok(out)
}

pub fn get_content(state: &State<AppState>, id: &str) -> AppResult<String> {
    let g = state.inner.lock();
    let data_path: Option<String> = g.db.as_ref().and_then(|conn| {
        conn.query_row(
            "SELECT data_path FROM snapshots WHERE id=?1",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .ok()
    });
    drop(g);
    let dp = data_path.ok_or_else(|| AppError::NotFound(id.into()))?;
    std::fs::read_to_string(PathBuf::from(dp).join("content.md")).map_err(AppError::Io)
}

pub fn restore(state: &State<AppState>, id: &str) -> AppResult<()> {
    let content = get_content(state, id)?;
    let rel = {
        let g = state.inner.lock();
        g.db.as_ref()
            .and_then(|conn| {
                conn.query_row(
                    "SELECT note_path FROM snapshots WHERE id=?1",
                    rusqlite::params![id],
                    |r| r.get::<_, String>(0),
                )
                .ok()
            })
            .unwrap_or_default()
    };
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone().ok_or(AppError::NoVault)?
    };
    let _ = snapshot_now(state, &rel, Some("pre-restore".into()));
    crate::vault::write_file_atomic(&root, &rel, &content)?;
    Ok(())
}

pub fn diff(state: &State<AppState>, a: &str, b: &str) -> AppResult<String> {
    let ca = get_content(state, a)?;
    let cb = get_content(state, b)?;
    let mut out = String::new();
    let lines_a: Vec<&str> = ca.lines().collect();
    let lines_b: Vec<&str> = cb.lines().collect();
    let max = lines_a.len().max(lines_b.len());
    for i in 0..max {
        let la = lines_a.get(i).copied().unwrap_or("");
        let lb = lines_b.get(i).copied().unwrap_or("");
        if la == lb {
            out.push_str(&format!("  {}\n", la));
        } else {
            if i < lines_a.len() {
                out.push_str(&format!("- {}\n", la));
            }
            if i < lines_b.len() {
                out.push_str(&format!("+ {}\n", lb));
            }
        }
    }
    Ok(out)
}

fn cleanup(state: &State<AppState>, rel: &str) {
    let retention = {
        let g = state.inner.lock();
        g.settings.snapshot_retention
    };
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        let ids: Vec<String> = (|| {
            let mut stmt = conn
                .prepare("SELECT id FROM snapshots WHERE note_path=?1 ORDER BY created_at DESC")
                .ok()?;
            let rows = stmt
                .query_map(rusqlite::params![rel], |r| r.get::<_, String>(0))
                .ok()?;
            rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
        })()
        .unwrap_or_default();
        if ids.len() > retention as usize {
            for id in &ids[retention as usize..] {
                let dp: Option<String> = conn
                    .query_row(
                        "SELECT data_path FROM snapshots WHERE id=?1",
                        rusqlite::params![id],
                        |r| r.get(0),
                    )
                    .ok();
                if let Some(dp) = dp {
                    let _ = std::fs::remove_dir_all(dp);
                }
                let _ = conn.execute("DELETE FROM snapshots WHERE id=?1", rusqlite::params![id]);
            }
        }
    }
}
