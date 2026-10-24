use crate::error::AppResult;
use crate::state::AppState;
use crate::vault;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(serde::Serialize)]
pub struct IndexStatus { pub total: usize, pub indexed: usize, pub status: String }

// ---------- vault ----------
#[tauri::command]
pub fn open_vault(path: String, state: State<AppState>) -> AppResult<String> {
    let root = PathBuf::from(&path);
    if !root.is_dir() { return Err(crate::error::AppError::Invalid("not a directory".into())); }
    let mut g = state.inner.lock();
    g.vault_root = Some(root.clone());
    // recent
    if !g.recent_vaults.contains(&root) {
        g.recent_vaults.insert(0, root.clone());
        g.recent_vaults.truncate(10);
    }
    persist_recent(&g.recent_vaults);
    Ok(path)
}

#[tauri::command]
pub fn create_vault(path: String, state: State<AppState>) -> AppResult<String> {
    let root = PathBuf::from(&path);
    std::fs::create_dir_all(&root).map_err(crate::error::AppError::Io)?;
    std::fs::create_dir_all(root.join(".notecraft")).map_err(crate::error::AppError::Io)?;
    let mut g = state.inner.lock();
    g.vault_root = Some(root.clone());
    if !g.recent_vaults.contains(&root) {
        g.recent_vaults.insert(0, root.clone());
        g.recent_vaults.truncate(10);
    }
    persist_recent(&g.recent_vaults);
    Ok(path)
}

#[tauri::command]
pub fn close_vault(state: State<AppState>) -> AppResult<()> {
    let mut g = state.inner.lock();
    g.vault_root = None;
    Ok(())
}

#[tauri::command]
pub fn get_vault_root(state: State<AppState>) -> AppResult<Option<String>> {
    let g = state.inner.lock();
    Ok(g.vault_root.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn list_recent_vaults(state: State<AppState>) -> AppResult<Vec<String>> {
    let g = state.inner.lock();
    Ok(g.recent_vaults.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

fn persist_recent(v: &[PathBuf]) {
    let rv = crate::settings::data_dir().join("recent_vaults.json");
    let _ = std::fs::create_dir_all(crate::settings::data_dir());
    let arr: Vec<String> = v.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let _ = std::fs::write(rv, serde_json::to_string(&arr).unwrap_or_default());
}

fn with_root<F, R>(state: &State<AppState>, f: F) -> AppResult<R>
where F: FnOnce(&PathBuf) -> AppResult<R> {
    let g = state.inner.lock();
    let root = g.vault_root.clone().ok_or(crate::error::AppError::NoVault)?;
    drop(g);
    f(&root)
}

#[tauri::command]
pub fn list_dir(rel: String, state: State<AppState>) -> AppResult<Vec<vault::FileEntry>> {
    with_root(&state, |root| {
        let g = state.inner.lock();
        let excluded = g.settings.excluded_folders.clone();
        let show_hidden = g.settings.hidden_files;
        drop(g);
        vault::list_dir(root, &rel, &excluded, show_hidden)
    })
}

#[tauri::command]
pub fn read_file(rel: String, state: State<AppState>) -> AppResult<String> {
    with_root(&state, |root| vault::read_file(root, &rel))
}

#[tauri::command]
pub fn write_file(rel: String, content: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| {
        vault::write_file_atomic(root, &rel, &content)?;
        Ok(())
    })?;
    // update db links
    if rel.ends_with(".md") || rel.ends_with(".markdown") {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            // read content already; use provided
            let _ = crate::links::rebuild_for_note(conn, &rel, &content);
        }
    }
    // snapshot maybe
    Ok(())
}

#[tauri::command]
pub fn create_file(rel: String, content: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::create_file(root, &rel, &content))
}

#[tauri::command]
pub fn create_dir(rel: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::create_dir(root, &rel))
}

#[tauri::command]
pub fn delete_path(rel: String, trash: bool, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::delete_path(root, &rel, trash))
}

#[tauri::command]
pub fn rename_path(from: String, to: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::rename_path(root, &from, &to))
}

#[tauri::command]
pub fn move_path(from: String, to_dir: String, state: State<AppState>) -> AppResult<String> {
    with_root(&state, |root| vault::move_path(root, &from, &to_dir))
}

// ---------- markdown ----------
#[tauri::command]
pub fn parse_note(content: String) -> AppResult<crate::markdown::ParsedNote> {
    Ok(crate::markdown::parse(&content))
}

#[tauri::command]
pub fn render_html(content: String) -> AppResult<String> {
    Ok(crate::markdown::render_html(&content))
}

// ---------- search ----------
#[tauri::command]
pub fn search(query: String, filters: serde_json::Value, state: State<AppState>) -> AppResult<Vec<crate::search::SearchHit>> {
    crate::search::search(&state, &query, &filters)
}

#[tauri::command]
pub fn rebuild_index(state: State<AppState>) -> AppResult<()> {
    crate::search::rebuild(&state)
}

#[tauri::command]
pub fn index_status(state: State<AppState>) -> AppResult<IndexStatus> {
    let (total, indexed, status) = crate::search::status(&state);
    Ok(IndexStatus { total, indexed, status })
}

// ---------- links ----------
#[tauri::command]
pub fn rebuild_links(rel: String, content: String, state: State<AppState>) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        crate::links::rebuild_for_note(conn, &rel, &content)?;
    }
    Ok(())
}

#[tauri::command]
pub fn backlinks(target: String, state: State<AppState>) -> AppResult<Vec<String>> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        crate::links::backlinks(conn, &target)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub fn outgoing_links(source: String, state: State<AppState>) -> AppResult<Vec<crate::markdown::LinkRef>> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        crate::links::outgoing(conn, &source)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub fn unresolved_links(state: State<AppState>) -> AppResult<Vec<String>> {
    let (root, excluded) = {
        let g = state.inner.lock();
        let root = g.vault_root.clone().ok_or(crate::error::AppError::NoVault)?;
        let excluded = g.settings.excluded_folders.clone();
        (root, excluded)
    };
    let files: Vec<String> = crate::vault::walk_markdown(&root, &excluded).iter().map(|p| crate::vault::to_rel(&root, p)).collect();
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        crate::links::unresolved(conn, &files)
    } else {
        Ok(vec![])
    }
}

// ---------- graph ----------
#[tauri::command]
pub fn graph_data(scope: String, filters: serde_json::Value, state: State<AppState>) -> AppResult<crate::graph::GraphData> {
    crate::graph::build(&state, &scope, &filters)
}

// ---------- history ----------
#[tauri::command]
pub fn snapshot_now(rel: String, label: Option<String>, state: State<AppState>) -> AppResult<crate::history::Snapshot> {
    crate::history::snapshot_now(&state, &rel, label)
}

#[tauri::command]
pub fn list_snapshots(rel: String, state: State<AppState>) -> AppResult<Vec<crate::history::Snapshot>> {
    crate::history::list(&state, &rel)
}

#[tauri::command]
pub fn get_snapshot(id: String, state: State<AppState>) -> AppResult<String> {
    crate::history::get_content(&state, &id)
}

#[tauri::command]
pub fn restore_snapshot(id: String, state: State<AppState>) -> AppResult<()> {
    crate::history::restore(&state, &id)
}

#[tauri::command]
pub fn diff_snapshots(a: String, b: String, state: State<AppState>) -> AppResult<String> {
    crate::history::diff(&state, &a, &b)
}

// ---------- tasks ----------
#[tauri::command]
pub fn list_tasks(rel: Option<String>, state: State<AppState>) -> AppResult<Vec<crate::tasks::TaskItem>> {
    crate::tasks::list(&state, rel.as_deref())
}

#[tauri::command]
pub fn set_task_done(rel: String, line: i64, done: bool, state: State<AppState>) -> AppResult<()> {
    crate::tasks::set_done(&state, &rel, line, done)
}

// ---------- templates ----------
#[tauri::command]
pub fn list_templates(state: State<AppState>) -> AppResult<Vec<crate::templates::TemplateInfo>> {
    crate::templates::list(&state)
}

#[tauri::command]
pub fn render_template(path: String, vars: std::collections::HashMap<String, String>, state: State<AppState>) -> AppResult<String> {
    crate::templates::render(&state, &path, &vars)
}

// ---------- settings ----------
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<crate::settings::Settings> {
    let g = state.inner.lock();
    Ok(g.settings.clone())
}

#[tauri::command]
pub fn save_settings(s: crate::settings::Settings, state: State<AppState>) -> AppResult<()> {
    let mut g = state.inner.lock();
    g.settings = s.clone();
    drop(g);
    crate::settings::save(&s).map_err(|e| crate::error::AppError::Other(e.to_string()))?;
    Ok(())
}

// ---------- workspaces ----------
#[tauri::command]
pub fn list_workspaces(state: State<AppState>) -> AppResult<Vec<crate::workspace::Workspace>> {
    crate::workspace::list(&state)
}

#[tauri::command]
pub fn save_workspace(id: String, name: String, data: String, state: State<AppState>) -> AppResult<()> {
    crate::workspace::save(&state, &id, &name, &data)
}

#[tauri::command]
pub fn delete_workspace(id: String, state: State<AppState>) -> AppResult<()> {
    crate::workspace::delete(&state, &id)
}

// ---------- automation ----------
#[tauri::command]
pub fn list_automations(state: State<AppState>) -> AppResult<Vec<crate::automation::Automation>> {
    crate::automation::list(&state)
}

#[tauri::command]
pub fn save_automation(a: serde_json::Value, state: State<AppState>) -> AppResult<()> {
    crate::automation::save(&state, a)
}

#[tauri::command]
pub fn delete_automation(id: String, state: State<AppState>) -> AppResult<()> {
    crate::automation::delete(&state, &id)
}

#[tauri::command]
pub fn run_automation_dry_run(id: String, state: State<AppState>) -> AppResult<serde_json::Value> {
    crate::automation::dry_run(&state, &id)
}

// ---------- plugins ----------
#[tauri::command]
pub fn list_plugins(state: State<AppState>) -> AppResult<Vec<crate::plugins::PluginInfo>> {
    crate::plugins::list(&state)
}

#[tauri::command]
pub fn enable_plugin(id: String, enabled: bool, state: State<AppState>) -> AppResult<()> {
    crate::plugins::enable(&state, &id, enabled)
}

#[tauri::command]
pub fn uninstall_plugin(id: String, state: State<AppState>) -> AppResult<()> {
    crate::plugins::uninstall(&state, &id)
}

// ---------- sync ----------
#[tauri::command]
pub fn sync_status(state: State<AppState>) -> AppResult<serde_json::Value> {
    crate::sync::status(&state)
}

// ---------- security ----------
#[tauri::command]
pub fn set_vault_password(password: String, state: State<AppState>) -> AppResult<()> {
    crate::security::set_password(&state, &password)
}

// ---------- import/export ----------
#[tauri::command]
pub fn import_vault(src: String, dest: String, state: State<AppState>) -> AppResult<crate::import_export::ImportReport> {
    crate::import_export::import_vault(&src, &dest)
}

#[tauri::command]
pub fn export_html(rel: String, state: State<AppState>) -> AppResult<String> {
    with_root(&state, |root| {
        let content = vault::read_file(root, &rel)?;
        Ok(crate::markdown::render_html(&content))
    })
}

#[tauri::command]
pub fn backup_vault(dest: String, state: State<AppState>) -> AppResult<()> {
    crate::import_export::backup(&state, &dest)
}

#[tauri::command]
pub fn restore_backup(src: String, dest: String, state: State<AppState>) -> AppResult<()> {
    crate::import_export::restore_backup(&src, &dest)
}
