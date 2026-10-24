use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub permissions: Vec<String>,
}

pub fn list(state: &State<AppState>) -> AppResult<Vec<PluginInfo>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let mut stmt = conn
            .prepare("SELECT id,manifest,enabled FROM plugins")
            .map_err(AppError::Sqlite)?;
        let rows = stmt
            .query_map([], |r| {
                let id: String = r.get(0)?;
                let m: String = r.get(1)?;
                let enabled: i64 = r.get(2)?;
                let v: Value = serde_json::from_str(&m).unwrap_or(Value::Null);
                Ok(PluginInfo {
                    id: id.clone(),
                    name: v
                        .get("name")
                        .and_then(|x| x.as_str())
                        .unwrap_or(&id)
                        .to_string(),
                    version: v
                        .get("version")
                        .and_then(|x| x.as_str())
                        .unwrap_or("0.0.0")
                        .to_string(),
                    author: v
                        .get("author")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: v
                        .get("description")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    enabled: enabled != 0,
                    permissions: v
                        .get("permissions")
                        .and_then(|x| x.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
            })
            .map_err(AppError::Sqlite)?;
        for r in rows {
            if let Ok(p) = r {
                out.push(p);
            }
        }
    }
    Ok(out)
}

pub fn enable(state: &State<AppState>, id: &str, enabled: bool) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "UPDATE plugins SET enabled=?1 WHERE id=?2",
            rusqlite::params![if enabled { 1 } else { 0 }, id],
        )
        .map_err(AppError::Sqlite)?;
    }
    Ok(())
}

pub fn uninstall(state: &State<AppState>, id: &str) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute("DELETE FROM plugins WHERE id=?1", rusqlite::params![id])
            .map_err(AppError::Sqlite)?;
    }
    Ok(())
}
