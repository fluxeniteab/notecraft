use crate::error::AppResult;
use crate::settings::Settings;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppInner {
    pub settings: Settings,
    pub db: Option<Connection>,
    pub vault_root: Option<PathBuf>,
    pub recent_vaults: Vec<PathBuf>,
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Mutex<AppInner>>,
}

impl AppState {
    pub fn new() -> Self {
        let settings = crate::settings::load();
        let mut inner = AppInner {
            settings,
            db: None,
            vault_root: None,
            recent_vaults: vec![],
        };
        // open db
        let db_path = crate::settings::data_dir().join("notecraft.db");
        let _ = std::fs::create_dir_all(crate::settings::data_dir());
        if let Ok(conn) = Connection::open(&db_path) {
            let _ = crate::metadata::migrate(&conn);
            inner.db = Some(conn);
        }
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
