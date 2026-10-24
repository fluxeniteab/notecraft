use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo { pub path: String, pub name: String }

pub fn list(state: &State<AppState>) -> AppResult<Vec<TemplateInfo>> {
    let g = state.inner.lock();
    let root = g.vault_root.clone().ok_or(AppError::NoVault)?;
    drop(g);
    // templates in "templates" folder or .notecraft/templates
    let mut out = vec![];
    for folder in ["templates", ".notecraft/templates"] {
        let dir = root.join(folder);
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".md") {
                        out.push(TemplateInfo { path: format!("{}/{}", folder, name), name: name.trim_end_matches(".md").to_string() });
                    }
                }
            }
        }
    }
    Ok(out)
}

pub fn render(state: &State<AppState>, path: &str, vars: &std::collections::HashMap<String, String>) -> AppResult<String> {
    let root = { let g = state.inner.lock(); g.vault_root.clone().ok_or(AppError::NoVault)? };
    let content = crate::vault::read_file(&root, path)?;
    let now = chrono::Utc::now();
    let mut out = content;
    // built-in vars
    let title = vars.get("title").cloned().unwrap_or_default();
    let folder = vars.get("folder").cloned().unwrap_or_default();
    out = out.replace("{{title}}", &title);
    out = out.replace("{{folder}}", &folder);
    out = out.replace("{{date}}", &now.format("%Y-%m-%d").to_string());
    out = out.replace("{{time}}", &now.format("%H:%M").to_string());
    out = out.replace("{{datetime}}", &now.to_rfc3339());
    for (k, v) in vars { out = out.replace(&format!("{{{{{}}}}}", k), v); }
    Ok(out)
}
