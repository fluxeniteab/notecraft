use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub data: String,
}

pub fn list(state: &State<AppState>) -> AppResult<Vec<Workspace>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let mut stmt = conn
            .prepare("SELECT id,name,data FROM workspaces")
            .map_err(AppError::Sqlite)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Workspace {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    data: r.get(2)?,
                })
            })
            .map_err(AppError::Sqlite)?;
        for r in rows {
            if let Ok(w) = r {
                out.push(w);
            }
        }
    }
    Ok(out)
}

pub fn save(state: &State<AppState>, id: &str, name: &str, data: &str) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "INSERT INTO workspaces(id,name,data) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name, data=excluded.data",
            rusqlite::params![id, name, data],
        ).map_err(AppError::Sqlite)?;
    }
    Ok(())
}

pub fn delete(state: &State<AppState>, id: &str) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute("DELETE FROM workspaces WHERE id=?1", rusqlite::params![id])
            .map_err(AppError::Sqlite)?;
    }
    Ok(())
}
