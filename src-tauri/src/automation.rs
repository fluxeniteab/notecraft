use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::State;

static RUN_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
    let cfg = a.get("config").cloned().unwrap_or_else(|| a.clone());
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "INSERT INTO automations(id,name,enabled,config) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name, enabled=excluded.enabled, config=excluded.config",
            rusqlite::params![id, name, if enabled {1} else {0}, cfg.to_string()],
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
    Ok(serde_json::json!({
        "automation": a.name,
        "enabled": a.enabled,
        "preview": "Dry run: no changes applied. Configured action would execute according to rules.",
        "config": a.config,
    }))
}

pub fn execute(state: &State<AppState>, id: &str, note: Option<&str>) -> AppResult<Value> {
    // loop detection
    let n = RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
    if n > 100 {
        return Err(AppError::Other("execution limit reached".into()));
    }

    let autos = list(state)?;
    let a = autos
        .into_iter()
        .find(|x| x.id == id)
        .ok_or_else(|| AppError::NotFound(id.into()))?;
    if !a.enabled {
        return Err(AppError::Other("automation disabled".into()));
    }

    let root = {
        let g = state.inner.lock();
        g.vault_root.clone()
    };
    let started = chrono::Utc::now().to_rfc3339();
    let mut log = String::new();
    let mut status = "success";

    // evaluate action from config
    let action = a
        .config
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    match action {
        "set_property" => {
            if let Some(note_path) = note {
                if let Some(root) = &root {
                    if let Ok(content) = crate::vault::read_file(root, note_path) {
                        let key = a
                            .config
                            .get("key")
                            .and_then(|v| v.as_str())
                            .unwrap_or("status");
                        let val = a.config.get("value").cloned().unwrap_or(Value::Null);
                        match crate::markdown::update_frontmatter(&content, key, val) {
                            Ok(newc) => {
                                let _ = crate::vault::write_file_atomic(root, note_path, &newc);
                                log.push_str(&format!("set {} on {}\n", key, note_path));
                            }
                            Err(e) => {
                                status = "error";
                                log.push_str(&format!("error: {}\n", e));
                            }
                        }
                    }
                }
            }
        }
        "move_note" => {
            if let Some(note_path) = note {
                if let Some(root) = &root {
                    let to_dir = a
                        .config
                        .get("to_dir")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match crate::vault::move_path(root, note_path, to_dir) {
                        Ok(dest) => log.push_str(&format!("moved {} -> {}\n", note_path, dest)),
                        Err(e) => {
                            status = "error";
                            log.push_str(&format!("error: {}\n", e));
                        }
                    }
                }
            }
        }
        "insert_template" => {
            if let Some(note_path) = note {
                if let Some(root) = &root {
                    if let Ok(content) = crate::vault::read_file(root, note_path) {
                        let tpl = a
                            .config
                            .get("template")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let vars = a
                            .config
                            .get("vars")
                            .cloned()
                            .unwrap_or(Value::Object(serde_json::Map::new()));
                        let mut hm = std::collections::HashMap::new();
                        if let Some(obj) = vars.as_object() {
                            for (k, v) in obj {
                                hm.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                            }
                        }
                        match crate::templates::render(state, tpl, &hm) {
                            Ok(rendered) => {
                                let newc = format!("{}\n{}", content, rendered);
                                let _ = crate::vault::write_file_atomic(root, note_path, &newc);
                                log.push_str(&format!(
                                    "inserted template {} into {}\n",
                                    tpl, note_path
                                ));
                            }
                            Err(e) => {
                                status = "error";
                                log.push_str(&format!("error: {}\n", e));
                            }
                        }
                    }
                }
            }
        }
        other => {
            log.push_str(&format!("unknown action: {}\n", other));
            status = "error";
        }
    }

    // record run
    let run_id = uuid::Uuid::new_v4().to_string();
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = conn.execute(
                "INSERT INTO automation_runs(id,automation_id,started_at,status,log) VALUES(?1,?2,?3,?4,?5)",
                rusqlite::params![run_id, id, started, status, log],
            );
        }
    }

    Ok(serde_json::json!({
        "run_id": run_id,
        "automation": a.name,
        "status": status,
        "log": log,
    }))
}

pub fn list_runs(state: &State<AppState>, id: Option<&str>) -> AppResult<Vec<Value>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let sql = if id.is_some() {
            "SELECT id,automation_id,started_at,status,log FROM automation_runs WHERE automation_id=?1 ORDER BY started_at DESC"
        } else {
            "SELECT id,automation_id,started_at,status,log FROM automation_runs ORDER BY started_at DESC"
        };
        let mut stmt = conn.prepare(sql).map_err(AppError::Sqlite)?;
        let rows = if let Some(i) = id {
            stmt.query_map(rusqlite::params![i], |r| {
                Ok(serde_json::json!({
                    "id": r.get::<_,String>(0)?,
                    "automation_id": r.get::<_,String>(1)?,
                    "started_at": r.get::<_,String>(2)?,
                    "status": r.get::<_,String>(3)?,
                    "log": r.get::<_,String>(4)?,
                }))
            })
            .map_err(AppError::Sqlite)?
        } else {
            stmt.query_map([], |r| {
                Ok(serde_json::json!({
                    "id": r.get::<_,String>(0)?,
                    "automation_id": r.get::<_,String>(1)?,
                    "started_at": r.get::<_,String>(2)?,
                    "status": r.get::<_,String>(3)?,
                    "log": r.get::<_,String>(4)?,
                }))
            })
            .map_err(AppError::Sqlite)?
        };
        for r in rows {
            if let Ok(v) = r {
                out.push(v);
            }
        }
    }
    Ok(out)
}
