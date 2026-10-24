use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
