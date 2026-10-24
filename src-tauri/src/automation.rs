use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Automation {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub config: Value,
}

pub fn list(state: &State<AppState>) -> AppResult<Vec<Automation>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let mut stmt = conn
            .prepare("SELECT id,name,enabled,config FROM automations")
            .map_err(AppError::Sqlite)?;
        let rows = stmt
            .query_map([], |r| {
                let cfg: String = r.get(3)?;
                let v: Value = serde_json::from_str(&cfg).unwrap_or(Value::Null);
                Ok(Automation {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    enabled: r.get::<_, i64>(2)? != 0,
                    config: v,
                })
            })
            .map_err(AppError::Sqlite)?;
        for r in rows {
            if let Ok(a) = r {
                out.push(a);
            }
        }
    }
    Ok(out)
}

pub fn save(state: &State<AppState>, a: Value) -> AppResult<()> {
    let id = a
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let name = a
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let enabled = a.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "INSERT INTO automations(id,name,enabled,config) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name, enabled=excluded.enabled, config=excluded.config",
            rusqlite::params![id, name, if enabled {1} else {0}, a.to_string()],
        ).map_err(AppError::Sqlite)?;
    }
    Ok(())
}

pub fn delete(state: &State<AppState>, id: &str) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute("DELETE FROM automations WHERE id=?1", rusqlite::params![id])
            .map_err(AppError::Sqlite)?;
    }
    Ok(())
}

pub fn dry_run(state: &State<AppState>, id: &str) -> AppResult<Value> {
    let autos = list(state)?;
    let a = autos
        .into_iter()
        .find(|x| x.id == id)
        .ok_or_else(|| AppError::NotFound(id.into()))?;
    // preview: describe configured action without executing
    Ok(serde_json::json!({
        "automation": a.name,
        "enabled": a.enabled,
        "preview": "Dry run: no changes applied. Configured action would execute according to rules.",
        "config": a.config,
    }))
}
