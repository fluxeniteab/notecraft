use crate::error::AppResult;
use crate::settings::Settings;
use parking_lot::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The unlocked local profile. Only public profile metadata is kept in memory;
/// the encryption key lives in `secure`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSession {
    pub id: String,
    pub username: String,
}

pub struct AppInner {
    pub settings: Settings,
    pub db: Option<Connection>,
    pub vault_root: Option<PathBuf>,
    pub recent_vaults: Vec<PathBuf>,
    pub vault_watcher: Option<notify::RecommendedWatcher>,
    pub session: Option<ProfileSession>,
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Mutex<AppInner>>,
}

/// Open the app-wide database used when no encrypted profile is unlocked
/// (plain folder vaults keep their links/tasks/snapshots here).
pub fn open_global_db() -> Option<Connection> {
    let dir = crate::settings::data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let conn = Connection::open(dir.join("notecraft.db")).ok()?;
    let _ = crate::metadata::migrate(&conn);
    Some(conn)
}

/// Open (and migrate) the database stored inside a profile directory.
pub fn open_profile_db(dir: &Path) -> Option<Connection> {
    let _ = std::fs::create_dir_all(dir);
    let conn = Connection::open(dir.join("notecraft.db")).ok()?;
    let _ = crate::metadata::migrate(&conn);
    Some(conn)
}

impl AppState {
    pub fn new() -> Self {
        let settings = crate::settings::load();
        let mut inner = AppInner {
            settings,
            db: None,
            vault_root: None,
            recent_vaults: vec![],
            vault_watcher: None,
            session: None,
        };
        inner.db = open_global_db();
        // load recent vaults
        let rv = crate::settings::data_dir().join("recent_vaults.json");
        if let Ok(s) = std::fs::read_to_string(&rv) {
            if let Ok(v) = serde_json::from_str::<Vec<String>>(&s) {
                inner.recent_vaults = v.into_iter().map(PathBuf::from).collect();
            }
        }
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn with<F, R>(&self, f: F) -> AppResult<R>
    where
        F: FnOnce(&mut AppInner) -> AppResult<R>,
    {
        let mut g = self.inner.lock();
        f(&mut g)
    }
}
