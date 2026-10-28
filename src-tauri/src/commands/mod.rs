use crate::error::AppResult;
use crate::state::AppState;
use crate::vault;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[derive(serde::Serialize)]
pub struct IndexStatus {
    pub total: usize,
    pub indexed: usize,
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct UnlinkedMention {
    pub path: String,
    pub line: usize,
    pub text: String,
}

#[derive(serde::Serialize)]
pub struct RenameImpact {
    pub path: String,
    pub replacements: usize,
}

// ---------- vault ----------
#[tauri::command]
pub fn open_vault(path: String, app: AppHandle, state: State<AppState>) -> AppResult<String> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err(crate::error::AppError::Invalid("not a directory".into()));
    }
    let mut g = state.inner.lock();
    g.vault_root = Some(root.clone());
    // recent
    if !g.recent_vaults.contains(&root) {
        g.recent_vaults.insert(0, root.clone());
        g.recent_vaults.truncate(10);
    }
    persist_recent(&g.recent_vaults);
    drop(g);
    start_vault_watcher(&app, &state, root)?;
    let _ = rebuild_link_index(&state);
    crate::search::mark_stale();
    Ok(path)
}

#[tauri::command]
pub fn create_vault(path: String, app: AppHandle, state: State<AppState>) -> AppResult<String> {
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
    drop(g);
    start_vault_watcher(&app, &state, root)?;
    let _ = rebuild_link_index(&state);
    crate::search::mark_stale();
    Ok(path)
}

#[tauri::command]
pub fn close_vault(state: State<AppState>) -> AppResult<()> {
    let mut g = state.inner.lock();
    g.vault_watcher = None;
    g.vault_root = None;
    Ok(())
}

#[tauri::command]
pub fn get_vault_root(state: State<AppState>) -> AppResult<Option<String>> {
    let g = state.inner.lock();
    Ok(g.vault_root
        .as_ref()
        .map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn list_recent_vaults(state: State<AppState>) -> AppResult<Vec<String>> {
    let g = state.inner.lock();
    Ok(g.recent_vaults
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

fn start_vault_watcher(app: &AppHandle, state: &State<AppState>, root: PathBuf) -> AppResult<()> {
    use notify::{RecursiveMode, Watcher};

    let app = app.clone();
    let watched_root = root.clone();
    let mut watcher =
        notify::recommended_watcher(move |result: notify::Result<notify::Event>| match result {
            Ok(event) => {
                let paths: Vec<String> = event
                    .paths
                    .iter()
                    .filter_map(|path| path.strip_prefix(&watched_root).ok())
                    .map(|path| path.to_string_lossy().replace('\\', "/"))
                    .collect();
                if !paths.is_empty() {
                    let _ = app.emit("vault-fs-change", paths);
                }
            }
            Err(error) => log::warn!("vault filesystem watcher error: {error}"),
        })
        .map_err(|error| crate::error::AppError::Other(error.to_string()))?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|error| crate::error::AppError::Other(error.to_string()))?;

    let mut inner = state.inner.lock();
    inner.vault_watcher = Some(watcher);
    Ok(())
}

fn persist_recent(v: &[PathBuf]) {
    let rv = crate::settings::data_dir().join("recent_vaults.json");
    let _ = std::fs::create_dir_all(crate::settings::data_dir());
    let arr: Vec<String> = v.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let _ = std::fs::write(rv, serde_json::to_string(&arr).unwrap_or_default());
}

fn with_root<F, R>(state: &State<AppState>, f: F) -> AppResult<R>
where
    F: FnOnce(&PathBuf) -> AppResult<R>,
{
    let g = state.inner.lock();
    let root = g
        .vault_root
        .clone()
        .ok_or(crate::error::AppError::NoVault)?;
    drop(g);
    f(&root)
}

/// Rebuild the on-disk link/tag index. Runs when a vault is opened so
/// backlinks, tags and the graph work immediately instead of only for notes
/// that have been saved during the current session.
fn rebuild_link_index(state: &State<AppState>) -> AppResult<()> {
    let (root, excluded) = {
        let g = state.inner.lock();
        let root = g
            .vault_root
            .clone()
            .ok_or(crate::error::AppError::NoVault)?;
        (root, g.settings.excluded_folders.clone())
    };
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = conn.execute("DELETE FROM links", []);
            let _ = conn.execute("DELETE FROM note_tags", []);
            let _ = conn.execute("UPDATE tags SET count=0", []);
            let _ = conn.execute("DELETE FROM tasks", []);
            let _ = conn.execute("DELETE FROM properties", []);
        }
    }
    for path in vault::walk_markdown(&root, &excluded) {
        let rel = vault::to_rel(&root, &path);
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, &rel, &content);
        }
    }
    Ok(())
}

/// Drop link/tag rows for a deleted, renamed or moved path (and its children).
fn purge_link_index(state: &State<AppState>, rel: &str) {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        let like = format!("{}/%", rel);
        let _ = conn.execute(
            "DELETE FROM links WHERE source=?1 OR source LIKE ?2",
            rusqlite::params![rel, like],
        );
        let _ = conn.execute(
            "DELETE FROM note_tags WHERE note_path=?1 OR note_path LIKE ?2",
            rusqlite::params![rel, like],
        );
        let _ = conn.execute(
            "UPDATE tags SET count=(SELECT COUNT(*) FROM note_tags WHERE note_tags.tag=tags.name)",
            [],
        );
        let _ = conn.execute("DELETE FROM tags WHERE count=0", []);
    }
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
pub fn list_all_files(state: State<AppState>) -> AppResult<Vec<vault::FileEntry>> {
    with_root(&state, |root| {
        let g = state.inner.lock();
        let excluded = g.settings.excluded_folders.clone();
        let show_hidden = g.settings.hidden_files;
        drop(g);
        Ok(vault::list_all_files(root, &excluded, show_hidden))
    })
}

#[tauri::command]
pub fn read_file(rel: String, state: State<AppState>) -> AppResult<String> {
    with_root(&state, |root| vault::read_file(root, &rel))
}

#[tauri::command]
pub fn write_file(
    rel: String,
    content: String,
    app: AppHandle,
    state: State<AppState>,
) -> AppResult<()> {
    with_root(&state, |root| {
        vault::write_file_atomic(root, &rel, &content)?;
        Ok(())
    })?;
    // keep the search index and link database in sync with saves
    if is_markdown(&rel) {
        let _ = crate::search::index_note(&state, &rel, &content);
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, &rel, &content);
        }
    }
    let _ = app.emit("vault-file-written", rel);
    Ok(())
}

#[tauri::command]
pub fn write_file_if_unchanged(
    rel: String,
    content: String,
    expected: String,
    app: AppHandle,
    state: State<AppState>,
) -> AppResult<()> {
    with_root(&state, |root| {
        let current = vault::read_file(root, &rel)?;
        if current != expected {
            return Err(crate::error::AppError::Conflict(
                "file changed on disk".into(),
            ));
        }
        vault::write_file_atomic(root, &rel, &content)
    })?;
    if is_markdown(&rel) {
        let _ = crate::search::index_note(&state, &rel, &content);
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, &rel, &content);
        }
    }
    let _ = app.emit("vault-file-written", rel);
    Ok(())
}

#[tauri::command]
pub fn create_file(rel: String, content: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::create_file(root, &rel, &content))?;
    if is_markdown(&rel) {
        let _ = crate::search::index_note(&state, &rel, &content);
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, &rel, &content);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_dir(rel: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::create_dir(root, &rel))
}

#[tauri::command]
pub fn delete_path(rel: String, trash: bool, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::delete_path(root, &rel, trash))?;
    purge_link_index(&state, &rel);
    let _ = crate::search::remove_note(&state, &rel);
    // A deleted folder can contain any number of notes; a rebuild is the only
    // reliable way to purge those documents from the index.
    crate::search::mark_stale();
    Ok(())
}

#[tauri::command]
pub fn rename_path(from: String, to: String, state: State<AppState>) -> AppResult<()> {
    with_root(&state, |root| vault::rename_path(root, &from, &to))?;
    purge_link_index(&state, &from);
    let _ = crate::search::remove_note(&state, &from);
    if is_markdown(&to) {
        if let Ok(content) = with_root(&state, |root| vault::read_file(root, &to)) {
            let _ = crate::search::index_note(&state, &to, &content);
        }
    }
    crate::search::mark_stale();
    Ok(())
}

#[tauri::command]
pub fn move_path(from: String, to_dir: String, state: State<AppState>) -> AppResult<String> {
    let dest = with_root(&state, |root| vault::move_path(root, &from, &to_dir))?;
    purge_link_index(&state, &from);
    let _ = crate::search::remove_note(&state, &from);
    if is_markdown(&dest) {
        if let Ok(content) = with_root(&state, |root| vault::read_file(root, &dest)) {
            let _ = crate::search::index_note(&state, &dest, &content);
        }
    }
    crate::search::mark_stale();
    Ok(dest)
}

fn is_markdown(rel: &str) -> bool {
    rel.ends_with(".md") || rel.ends_with(".markdown")
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
#[derive(serde::Serialize)]
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    pub query: String,
    pub filters: serde_json::Value,
}

#[tauri::command]
pub fn save_search(
    id: String,
    name: String,
    query: String,
    filters: serde_json::Value,
    state: State<AppState>,
) -> AppResult<()> {
    if name.trim().is_empty() || (query.trim().is_empty() && filters == serde_json::json!({})) {
        return Err(crate::error::AppError::Invalid(
            "saved search needs a name and a query or filter".into(),
        ));
    }
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "INSERT INTO saved_searches(id,name,query,filters,created_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name,query=excluded.query,filters=excluded.filters",
            rusqlite::params![id, name.trim(), query, filters.to_string(), chrono::Utc::now().to_rfc3339()],
        ).map_err(crate::error::AppError::Sqlite)?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_saved_searches(state: State<AppState>) -> AppResult<Vec<SavedSearch>> {
    let g = state.inner.lock();
    let Some(conn) = g.db.as_ref() else {
        return Ok(vec![]);
    };
    let mut stmt = conn
        .prepare("SELECT id,name,query,filters FROM saved_searches ORDER BY name COLLATE NOCASE")
        .map_err(crate::error::AppError::Sqlite)?;
    let rows = stmt
        .query_map([], |row| {
            let raw: String = row.get(3)?;
            Ok(SavedSearch {
                id: row.get(0)?,
                name: row.get(1)?,
                query: row.get(2)?,
                filters: serde_json::from_str(&raw).unwrap_or(serde_json::json!({})),
            })
        })
        .map_err(crate::error::AppError::Sqlite)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::AppError::Sqlite)
}

#[tauri::command]
pub fn delete_saved_search(id: String, state: State<AppState>) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "DELETE FROM saved_searches WHERE id=?1",
            rusqlite::params![id],
        )
        .map_err(crate::error::AppError::Sqlite)?;
    }
    Ok(())
}

#[tauri::command]
pub fn search_history(state: State<AppState>) -> AppResult<Vec<String>> {
    let g = state.inner.lock();
    let Some(conn) = g.db.as_ref() else {
        return Ok(vec![]);
    };
    let mut stmt = conn
        .prepare("SELECT query FROM search_history ORDER BY id DESC LIMIT 12")
        .map_err(crate::error::AppError::Sqlite)?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(crate::error::AppError::Sqlite)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::AppError::Sqlite)
}

#[tauri::command]
pub fn search(
    query: String,
    filters: serde_json::Value,
    state: State<AppState>,
) -> AppResult<Vec<crate::search::SearchHit>> {
    if !query.trim().is_empty() {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = conn.execute(
                "INSERT INTO search_history(query,created_at) VALUES(?1,?2)",
                rusqlite::params![query.trim(), chrono::Utc::now().to_rfc3339()],
            );
            let _ = conn.execute(
                "DELETE FROM search_history WHERE id NOT IN (SELECT id FROM search_history ORDER BY id DESC LIMIT 50)",
                [],
            );
        }
    }
    crate::search::search(&state, &query, &filters)
}

#[tauri::command]
pub fn rebuild_index(state: State<AppState>) -> AppResult<()> {
    crate::search::rebuild(&state)
}

#[tauri::command]
pub fn index_status(state: State<AppState>) -> AppResult<IndexStatus> {
    let (total, indexed, status) = crate::search::status(&state);
    Ok(IndexStatus {
        total,
        indexed,
        status,
    })
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
pub fn outgoing_links(
    source: String,
    state: State<AppState>,
) -> AppResult<Vec<crate::markdown::LinkRef>> {
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
        let root = g
            .vault_root
            .clone()
            .ok_or(crate::error::AppError::NoVault)?;
        let excluded = g.settings.excluded_folders.clone();
        (root, excluded)
    };
    let files: Vec<String> = crate::vault::walk_markdown(&root, &excluded)
        .iter()
        .map(|p| crate::vault::to_rel(&root, p))
        .collect();
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        crate::links::unresolved(conn, &files)
    } else {
        Ok(vec![])
    }
}

// ---------- unlinked mentions & rename propagation ----------
#[tauri::command]
pub fn unlinked_mentions(
    target: String,
    state: State<AppState>,
) -> AppResult<Vec<UnlinkedMention>> {
    let (root, excluded) = {
        let g = state.inner.lock();
        (
            g.vault_root
                .clone()
                .ok_or(crate::error::AppError::NoVault)?,
            g.settings.excluded_folders.clone(),
        )
    };
    let target_no_ext = target.trim_end_matches(".md");
    let stem_name = std::path::Path::new(target_no_ext)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if stem_name.is_empty() {
        return Ok(vec![]);
    }
    let link_re = regex::Regex::new(r"\[\[([^\]|#]+)").unwrap();
    let word_re = regex::Regex::new(&format!(r"(?i)#?\b{}\b", regex::escape(&stem_name))).unwrap();
    let mut out = Vec::new();
    for path in crate::vault::walk_markdown(&root, &excluded) {
        let rel = crate::vault::to_rel(&root, &path);
        if rel.trim_end_matches(".md") == target_no_ext {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut in_fence = false;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence || line.trim().is_empty() {
                continue;
            }
            let linked = link_re.captures_iter(line).any(|cap| {
                let t = cap[1].trim().trim_end_matches(".md").to_lowercase();
                t == target_no_ext.to_lowercase()
                    || t.rsplit('/').next() == Some(stem_name.as_str())
            });
            if linked || !word_re.is_match(line) {
                continue;
            }
            out.push(UnlinkedMention {
                path: rel.clone(),
                line: idx + 1,
                text: line.trim().to_string(),
            });
            if out.len() >= 100 {
                return Ok(out);
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn propagate_link_rename(
    from: String,
    to: String,
    apply: bool,
    state: State<AppState>,
) -> AppResult<Vec<RenameImpact>> {
    let (root, excluded) = {
        let g = state.inner.lock();
        (
            g.vault_root
                .clone()
                .ok_or(crate::error::AppError::NoVault)?,
            g.settings.excluded_folders.clone(),
        )
    };
    let from_no_ext = from.trim_end_matches(".md").to_string();
    let from_stem = std::path::Path::new(&from_no_ext)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| from_no_ext.clone());
    let to_display = to
        .trim_end_matches(".md")
        .rsplit('/')
        .next()
        .unwrap_or(&to)
        .to_string();
    let mut impacts = Vec::new();
    for path in crate::vault::walk_markdown(&root, &excluded) {
        let rel = crate::vault::to_rel(&root, &path);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (next, n) =
            crate::links::rename_wikilink_targets(&content, &from_no_ext, &from_stem, &to_display);
        if n == 0 {
            continue;
        }
        if apply {
            vault::write_file_atomic(&root, &rel, &next)?;
            let _ = crate::search::index_note(&state, &rel, &next);
            let g = state.inner.lock();
            if let Some(conn) = g.db.as_ref() {
                let _ = crate::links::rebuild_for_note(conn, &rel, &next);
            }
        }
        impacts.push(RenameImpact {
            path: rel,
            replacements: n,
        });
    }
    Ok(impacts)
}

#[tauri::command]
pub fn rename_tag(from: String, to: String, state: State<AppState>) -> AppResult<usize> {
    let from = from.trim().trim_start_matches('#');
    let to = to.trim().trim_start_matches('#');
    if from.is_empty()
        || to.is_empty()
        || from == to
        || !to
            .chars()
            .all(|c| c.is_alphanumeric() || c == '/' || c == '_' || c == '-')
    {
        return Err(crate::error::AppError::Invalid(
            "tags may contain letters, numbers, '-', '_' and '/'".into(),
        ));
    }
    let (root, affected) = {
        let g = state.inner.lock();
        let root = g
            .vault_root
            .clone()
            .ok_or(crate::error::AppError::NoVault)?;
        let mut paths: Vec<String> = vec![];
        if let Some(conn) = g.db.as_ref() {
            let mut stmt = conn
                .prepare("SELECT DISTINCT note_path FROM note_tags WHERE tag=?1")
                .map_err(crate::error::AppError::Sqlite)?;
            let rows = stmt
                .query_map(rusqlite::params![from], |r| r.get::<_, String>(0))
                .map_err(crate::error::AppError::Sqlite)?;
            for row in rows.flatten() {
                paths.push(row);
            }
        }
        (root, paths)
    };
    let mut total = 0usize;
    for rel in affected {
        let Ok(content) = vault::read_file(&root, &rel) else {
            continue;
        };
        let (next, n) = crate::links::rename_tag_in_content(&content, from, to);
        if n == 0 || next == content {
            continue;
        }
        vault::write_file_atomic(&root, &rel, &next)?;
        total += n;
        let _ = crate::search::index_note(&state, &rel, &next);
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, &rel, &next);
        }
    }
    Ok(total)
}

// ---------- collections (dynamic views) ----------
#[derive(serde::Serialize)]
pub struct CollectionDef {
    pub id: String,
    pub name: String,
    pub config: serde_json::Value,
}

#[derive(serde::Serialize)]
pub struct CollectionNote {
    pub path: String,
    pub title: String,
    pub properties: std::collections::HashMap<String, String>,
    pub tags: Vec<String>,
    pub modified: i64,
    pub due: Option<String>,
}

#[tauri::command]
pub fn list_collections(state: State<AppState>) -> AppResult<Vec<CollectionDef>> {
    let g = state.inner.lock();
    let Some(conn) = g.db.as_ref() else {
        return Ok(vec![]);
    };
    let mut stmt = conn
        .prepare("SELECT id,name,config FROM collections ORDER BY name COLLATE NOCASE")
        .map_err(crate::error::AppError::Sqlite)?;
    let rows = stmt
        .query_map([], |row| {
            let raw: String = row.get(2)?;
            Ok(CollectionDef {
                id: row.get(0)?,
                name: row.get(1)?,
                config: serde_json::from_str(&raw).unwrap_or(serde_json::json!({})),
            })
        })
        .map_err(crate::error::AppError::Sqlite)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(crate::error::AppError::Sqlite)
}

#[tauri::command]
pub fn save_collection(
    id: String,
    name: String,
    config: serde_json::Value,
    state: State<AppState>,
) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(crate::error::AppError::Invalid(
            "collection needs a name".into(),
        ));
    }
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "INSERT INTO collections(id,name,config) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,config=excluded.config",
            rusqlite::params![id, name, config.to_string()],
        )
        .map_err(crate::error::AppError::Sqlite)?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_collection(id: String, state: State<AppState>) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute("DELETE FROM collections WHERE id=?1", rusqlite::params![id])
            .map_err(crate::error::AppError::Sqlite)?;
    }
    Ok(())
}

#[tauri::command]
pub fn collection_notes(id: String, state: State<AppState>) -> AppResult<Vec<CollectionNote>> {
    let (root, excluded, config) = {
        let g = state.inner.lock();
        let root = g
            .vault_root
            .clone()
            .ok_or(crate::error::AppError::NoVault)?;
        let excluded = g.settings.excluded_folders.clone();
        let config: serde_json::Value =
            g.db.as_ref()
                .and_then(|conn| {
                    conn.query_row(
                        "SELECT config FROM collections WHERE id=?1",
                        rusqlite::params![id],
                        |r| r.get::<_, String>(0),
                    )
                    .ok()
                })
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or(serde_json::json!({}));
        (root, excluded, config)
    };
    let folder = config.get("folder").and_then(|v| v.as_str()).unwrap_or("");
    let tag = config.get("tag").and_then(|v| v.as_str()).unwrap_or("");
    let property = config
        .get("property")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let sort = config
        .get("sort")
        .and_then(|v| v.as_str())
        .unwrap_or("name");

    // Candidate paths: folder filter via vault walk, otherwise every markdown file.
    let mut paths: Vec<String> = crate::vault::walk_markdown(&root, &excluded)
        .iter()
        .map(|p| crate::vault::to_rel(&root, p))
        .filter(|rel| {
            folder.is_empty()
                || rel
                    .trim_end_matches(".md")
                    .starts_with(&format!("{}/", folder.trim_end_matches('/')))
                || rel.trim_end_matches(".md") == folder.trim_end_matches('/')
        })
        .collect();

    // Load properties/tags/tasks in bulk; filtered afterwards in memory.
    let mut props: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
        std::collections::HashMap::new();
    let mut tags: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut modified: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let mut stmt = conn
                .prepare("SELECT note_path,key,value FROM properties")
                .map_err(crate::error::AppError::Sqlite)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                })
                .map_err(crate::error::AppError::Sqlite)?;
            for row in rows.flatten() {
                props.entry(row.0.clone()).or_default().insert(row.1, row.2);
            }
            let mut stmt = conn
                .prepare("SELECT note_path,tag FROM note_tags")
                .map_err(crate::error::AppError::Sqlite)?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(crate::error::AppError::Sqlite)?;
            for row in rows.flatten() {
                tags.entry(row.0.clone()).or_default().push(row.1);
            }
        }
    }
    for path in &paths {
        if let Ok(meta) = std::fs::metadata(root.join(path)) {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            modified.insert(path.clone(), mtime);
        }
    }

    if !tag.is_empty() {
        paths.retain(|p| {
            tags.get(p)
                .map(|list| {
                    list.iter()
                        .any(|t| t == tag || t.starts_with(&format!("{}/", tag)))
                })
                .unwrap_or(false)
        });
    }
    if !property.is_empty() {
        paths.retain(|p| {
            props
                .get(p)
                .map(|map| map.contains_key(property))
                .unwrap_or(false)
        });
    }

    let mut out: Vec<CollectionNote> = paths
        .into_iter()
        .map(|path| {
            let title = std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let properties = props.remove(&path).unwrap_or_default();
            let due = properties
                .get("due")
                .or_else(|| properties.get("date"))
                .cloned();
            CollectionNote {
                path: path.clone(),
                title,
                tags: tags.remove(&path).unwrap_or_default(),
                modified: modified.remove(&path).unwrap_or(0),
                properties,
                due,
            }
        })
        .collect();

    match sort {
        "modified" => out.sort_by(|a, b| b.modified.cmp(&a.modified)),
        "due" => out.sort_by(|a, b| {
            a.due
                .clone()
                .unwrap_or_else(|| "9999-12-31".into())
                .cmp(&b.due.clone().unwrap_or_else(|| "9999-12-31".into()))
        }),
        _ => out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
    }
    Ok(out)
}

// ---------- graph ----------
#[tauri::command]
pub fn graph_data(
    scope: String,
    filters: serde_json::Value,
    state: State<AppState>,
) -> AppResult<crate::graph::GraphData> {
    crate::graph::build(&state, &scope, &filters)
}

// ---------- history ----------
#[tauri::command]
pub fn snapshot_now(
    rel: String,
    label: Option<String>,
    state: State<AppState>,
) -> AppResult<crate::history::Snapshot> {
    crate::history::snapshot_now(&state, &rel, label)
}

#[tauri::command]
pub fn list_snapshots(
    rel: String,
    state: State<AppState>,
) -> AppResult<Vec<crate::history::Snapshot>> {
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
pub fn list_tasks(
    rel: Option<String>,
    state: State<AppState>,
) -> AppResult<Vec<crate::tasks::TaskItem>> {
    crate::tasks::list(&state, rel.as_deref())
}

#[tauri::command]
pub fn set_task_done(rel: String, line: i64, done: bool, state: State<AppState>) -> AppResult<()> {
    crate::tasks::set_done(&state, &rel, line, done)
}

#[tauri::command]
pub fn create_daily_note(date: Option<String>, state: State<AppState>) -> AppResult<String> {
    let (root, folder, template_path) = {
        let g = state.inner.lock();
        (
            g.vault_root
                .clone()
                .ok_or(crate::error::AppError::NoVault)?,
            g.settings.daily_note_folder.clone(),
            g.settings.daily_note_template.clone(),
        )
    };
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| crate::error::AppError::Invalid("date must use YYYY-MM-DD".into()))?;
    let normalized_folder = folder.replace('\\', "/");
    let rel_folder = normalized_folder.trim_matches('/');
    if rel_folder.is_empty()
        || normalized_folder.starts_with('/')
        || rel_folder
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(crate::error::AppError::Invalid(
            "daily note folder must be a safe vault-relative path".into(),
        ));
    }
    let rel = format!("{}/{}.md", rel_folder, date.format("%Y-%m-%d"));
    let content = if let Some(template) = template_path {
        let vars = std::collections::HashMap::from([
            ("title".to_string(), date.format("%Y-%m-%d").to_string()),
            ("date".to_string(), date.format("%Y-%m-%d").to_string()),
            ("folder".to_string(), folder.clone()),
        ]);
        crate::templates::render(&state, &template, &vars)?
    } else {
        format!("# {}\n\n", date.format("%Y-%m-%d"))
    };
    if !root.join(&rel).exists() {
        vault::create_file(&root, &rel, &content)?;
        let _ = crate::search::index_note(&state, &rel, &content);
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, &rel, &content);
        }
    }
    Ok(rel)
}

// ---------- templates ----------
#[tauri::command]
pub fn append_capture(text: String, state: State<AppState>) -> AppResult<String> {
    let text = text.trim();
    if text.is_empty() {
        return Err(crate::error::AppError::Invalid(
            "capture text cannot be empty".into(),
        ));
    }
    let (root, rel_setting) = {
        let g = state.inner.lock();
        (
            g.vault_root
                .clone()
                .ok_or(crate::error::AppError::NoVault)?,
            g.settings.capture_note.clone(),
        )
    };
    let rel = rel_setting.replace('\\', "/");
    let trimmed = rel.trim_matches('/');
    if trimmed.is_empty()
        || rel.starts_with('/')
        || trimmed
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == "..")
    {
        return Err(crate::error::AppError::Invalid(
            "capture note must be a safe vault-relative path".into(),
        ));
    }
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut content = if root.join(trimmed).exists() {
        vault::read_file(&root, trimmed)?
    } else {
        let title = std::path::Path::new(trimmed)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Inbox".into());
        format!("# {}\n\n", title)
    };
    if !content.ends_with('\n') {
        content.push('\n');
    }
    // Capture lines are plain bullets so the note stays editable Markdown.
    let flat = text.replace(['\r', '\n'], " ");
    content.push_str(&format!("- {} ({})\n", flat, stamp));
    vault::write_file_atomic(&root, trimmed, &content)?;
    let _ = crate::search::index_note(&state, trimmed, &content);
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            let _ = crate::links::rebuild_for_note(conn, trimmed, &content);
        }
    }
    Ok(trimmed.to_string())
}

#[tauri::command]
pub fn list_templates(state: State<AppState>) -> AppResult<Vec<crate::templates::TemplateInfo>> {
    crate::templates::list(&state)
}

#[tauri::command]
pub fn render_template(
    path: String,
    vars: std::collections::HashMap<String, String>,
    state: State<AppState>,
) -> AppResult<String> {
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
pub fn save_workspace(
    id: String,
    name: String,
    data: String,
    state: State<AppState>,
) -> AppResult<()> {
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

#[tauri::command]
pub fn run_automation(
    id: String,
    note: Option<String>,
    state: State<AppState>,
) -> AppResult<serde_json::Value> {
    crate::automation::execute(&state, &id, note.as_deref())
}

#[tauri::command]
pub fn list_automation_runs(
    id: Option<String>,
    state: State<AppState>,
) -> AppResult<Vec<serde_json::Value>> {
    crate::automation::list_runs(&state, id.as_deref())
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
pub fn import_vault(
    src: String,
    dest: String,
    state: State<AppState>,
) -> AppResult<crate::import_export::ImportReport> {
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

// ---------- sync (folder provider) ----------
#[tauri::command]
pub fn sync_push(state: State<AppState>) -> AppResult<Vec<crate::sync::SyncFileStatus>> {
    crate::sync::push(&state)
}

#[tauri::command]
pub fn sync_pull(state: State<AppState>) -> AppResult<Vec<crate::sync::SyncFileStatus>> {
    crate::sync::pull(&state)
}

#[tauri::command]
pub fn sync_status_folder(state: State<AppState>) -> AppResult<Vec<crate::sync::SyncFileStatus>> {
    crate::sync::folder_status(&state)
}

// ---------- plugin marketplace (shipped local catalog) ----------
#[derive(serde::Serialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
}

fn catalog_path() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("docs").join("plugin-catalog.json"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("docs").join("plugin-catalog.json"));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("docs").join("plugin-catalog.json"));
            candidates.push(dir.join("../docs").join("plugin-catalog.json"));
        }
    }
    candidates.into_iter().find(|p| p.exists())
}

// Bundled copies of docs/plugin-catalog.json and its example plugin sources,
// so the marketplace also works in packaged builds where docs/ is absent.
const EMBEDDED_CATALOG: &str = include_str!("../../../docs/plugin-catalog.json");
const EMBEDDED_SOURCES: &[(&str, &[(&str, &str)])] = &[(
    "examples/plugin-example",
    &[
        (
            "plugin.json",
            include_str!("../../../docs/examples/plugin-example/plugin.json"),
        ),
        (
            "index.js",
            include_str!("../../../docs/examples/plugin-example/index.js"),
        ),
    ],
)];

fn read_catalog() -> AppResult<Vec<serde_json::Value>> {
    let raw = match catalog_path() {
        Some(path) => std::fs::read_to_string(path).map_err(crate::error::AppError::Io)?,
        None => EMBEDDED_CATALOG.to_string(),
    };
    serde_json::from_str(&raw).map_err(crate::error::AppError::Json)
}

/// Files that make up a catalog plugin source: from docs/ when present in
/// development, otherwise from the copies embedded above.
fn catalog_source_files(source: &str) -> AppResult<Vec<(String, String)>> {
    if let Some(base) = catalog_path().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        let dir = base.join(source);
        if dir.is_dir() {
            let mut out: Vec<(String, String)> = Vec::new();
            for file in std::fs::read_dir(&dir).map_err(crate::error::AppError::Io)? {
                let file = file.map_err(crate::error::AppError::Io)?;
                if file
                    .file_type()
                    .map_err(crate::error::AppError::Io)?
                    .is_file()
                {
                    let name = file.file_name().to_string_lossy().to_string();
                    let content =
                        std::fs::read_to_string(file.path()).map_err(crate::error::AppError::Io)?;
                    out.push((name, content));
                }
            }
            if !out.is_empty() {
                return Ok(out);
            }
        }
    }
    EMBEDDED_SOURCES
        .iter()
        .find(|(name, _)| *name == source)
        .map(|(_, files)| {
            files
                .iter()
                .map(|(name, content)| (name.to_string(), content.to_string()))
                .collect()
        })
        .ok_or_else(|| crate::error::AppError::NotFound(format!("plugin source {source}")))
}

#[tauri::command]
pub fn plugin_catalog(state: State<AppState>) -> AppResult<Vec<CatalogEntry>> {
    let installed: std::collections::HashSet<String> = crate::plugins::list(&state)?
        .into_iter()
        .map(|p| p.id)
        .collect();
    let mut out = Vec::new();
    for entry in read_catalog()? {
        let Some(id) = entry.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        out.push(CatalogEntry {
            id: id.to_string(),
            name: entry
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string(),
            version: entry
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            description: entry
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            installed: installed.contains(id),
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn install_catalog_plugin(id: String, state: State<AppState>) -> AppResult<()> {
    let catalog = read_catalog()?;
    let entry = catalog
        .iter()
        .find(|e| e.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
        .ok_or_else(|| crate::error::AppError::NotFound(format!("plugin {id}")))?;
    let source = entry
        .get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            crate::error::AppError::Invalid("catalog entry is missing its source".into())
        })?;
    let files = catalog_source_files(source)?;
    let dest = crate::settings::data_dir().join("plugins").join(&id);
    std::fs::create_dir_all(&dest).map_err(crate::error::AppError::Io)?;
    for (name, content) in &files {
        std::fs::write(dest.join(name), content).map_err(crate::error::AppError::Io)?;
    }
    let manifest_raw = files
        .iter()
        .find(|(name, _)| name == "plugin.json")
        .map(|(_, content)| content.as_str())
        .ok_or_else(|| {
            crate::error::AppError::Invalid("plugin source is missing plugin.json".into())
        })?;
    let manifest: serde_json::Value =
        serde_json::from_str(manifest_raw).map_err(crate::error::AppError::Json)?;
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "INSERT INTO plugins(id,manifest,enabled,installed_at) VALUES(?1,?2,0,?3) \
             ON CONFLICT(id) DO UPDATE SET manifest=excluded.manifest",
            rusqlite::params![id, manifest.to_string(), chrono::Utc::now().to_rfc3339()],
        )
        .map_err(crate::error::AppError::Sqlite)?;
    }
    Ok(())
}
