use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: Option<i64>,
    pub note_path: String,
    pub line: i64,
    pub text: String,
    pub done: bool,
    pub due: Option<String>,
    pub priority: Option<String>,
}

pub fn list(state: &State<AppState>, rel: Option<&str>) -> AppResult<Vec<TaskItem>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let sql = if rel.is_some() {
            "SELECT note_path,line,text,done,due,priority FROM tasks WHERE note_path=?1 ORDER BY COALESCE(due,'9999-12-31'), note_path, line"
        } else {
            "SELECT note_path,line,text,done,due,priority FROM tasks ORDER BY done, COALESCE(due,'9999-12-31'), note_path, line"
        };
        let mut stmt = conn.prepare(sql).map_err(AppError::Sqlite)?;
        if let Some(r) = rel {
            let rows = stmt
                .query_map(rusqlite::params![r], |row| {
                    Ok(TaskItem {
                        id: None,
                        note_path: row.get(0)?,
                        line: row.get(1)?,
                        text: row.get(2)?,
                        done: row.get::<_, i64>(3)? != 0,
                        due: row.get(4)?,
                        priority: row.get(5)?,
                    })
                })
                .map_err(AppError::Sqlite)?;
            for row in rows {
                if let Ok(t) = row {
                    out.push(t);
                }
            }
        } else {
            let rows = stmt
                .query_map([], |row| {
                    Ok(TaskItem {
                        id: None,
                        note_path: row.get(0)?,
                        line: row.get(1)?,
                        text: row.get(2)?,
                        done: row.get::<_, i64>(3)? != 0,
                        due: row.get(4)?,
                        priority: row.get(5)?,
                    })
                })
                .map_err(AppError::Sqlite)?;
            for row in rows {
                if let Ok(t) = row {
                    out.push(t);
                }
            }
        }
    }
    Ok(out)
}

pub fn set_done(state: &State<AppState>, rel: &str, line: i64, done: bool) -> AppResult<()> {
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone().ok_or(AppError::NoVault)?
    };
    let content = crate::vault::read_file(&root, rel)?;
    let lines: Vec<&str> = content.lines().collect();
    if line < 1 {
        return Err(AppError::Invalid("line must be positive".into()));
    }
    let idx = (line as usize).saturating_sub(1);
    if idx >= lines.len() {
        return Err(AppError::NotFound("line".into()));
    }
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let l = &new_lines[idx];
    let marker = Regex::new(r"^(\s*- \[)[ xX](\])").unwrap().captures(l);
    let Some(marker) = marker else {
        return Err(AppError::Invalid("line is not a Markdown task".into()));
    };
    let replacement = if done { "x" } else { " " };
    let marker_range = marker.get(0).unwrap().range();
    let mut new = l.to_string();
    new.replace_range(
        marker_range.start + marker.get(1).unwrap().as_str().len()..marker_range.end - 1,
        replacement,
    );
    new_lines[idx] = new;
    let new_content = new_lines.join("\n");
    crate::vault::write_file_atomic(&root, rel, &new_content)?;
    let _ = crate::search::index_note(state, rel, &new_content);
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            crate::links::rebuild_for_note(conn, rel, &new_content)?;
        }
    }
    Ok(())
}
