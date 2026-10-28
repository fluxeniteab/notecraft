use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use crate::vault;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use tantivy::doc;
use tauri::State;

/// Serializes writers/readers. Tantivy only allows one writer at a time, and
/// re-creating an index while a reader is open fails on Windows.
static INDEX_LOCK: Mutex<()> = Mutex::new(());
/// Set when notes are deleted or moved between folders; the next search
/// rebuilds instead of serving stale documents.
static STALE: AtomicBool = AtomicBool::new(false);

fn index_lock() -> MutexGuard<'static, ()> {
    INDEX_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn mark_stale() {
    STALE.store(true, Ordering::SeqCst);
}

fn is_markdown(rel: &str) -> bool {
    rel.ends_with(".md") || rel.ends_with(".markdown")
}

fn matches_folder(path: &str, folder: &str) -> bool {
    path == folder || path.starts_with(&format!("{}/", folder.trim_end_matches('/')))
}

fn note_document(
    rel: &str,
    content: &str,
    path_f: tantivy::schema::Field,
    title_f: tantivy::schema::Field,
    body_f: tantivy::schema::Field,
    tags_f: tantivy::schema::Field,
) -> tantivy::TantivyDocument {
    let parsed = crate::markdown::parse(content);
    let title = Path::new(rel)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let tags = parsed.tags.join(" ");
    doc!(
        path_f => rel.to_string(),
        title_f => title,
        body_f => parsed.body,
        tags_f => tags,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub score: f32,
    pub snippet: String,
    pub title: String,
}

fn index_dir() -> PathBuf {
    // An unlocked profile keeps its index beside its own encrypted vault so two
    // accounts never share documents.
    match crate::secure::active_profile() {
        Some(id) => crate::secure::profile_dir(&id).join("index"),
        None => crate::settings::data_dir().join("search_index"),
    }
}

fn schema() -> (
    tantivy::schema::Schema,
    tantivy::schema::Field,
    tantivy::schema::Field,
    tantivy::schema::Field,
    tantivy::schema::Field,
) {
    use tantivy::schema::{Field, Schema, SchemaBuilder, STORED, STRING, TEXT};
    let mut b: SchemaBuilder = Schema::builder();
    let path = b.add_text_field("path", STRING | STORED);
    let title = b.add_text_field("title", TEXT | STORED);
    let body = b.add_text_field("body", TEXT);
    let tags = b.add_text_field("tags", TEXT | STORED);
    (b.build(), path, title, body, tags)
}

pub fn rebuild(state: &State<AppState>) -> AppResult<()> {
    let _guard = index_lock();
    rebuild_locked(state)
}

fn rebuild_locked(state: &State<AppState>) -> AppResult<()> {
    let (root, excluded) = {
        let g = state.inner.lock();
        let root = g.vault_root.clone().ok_or(AppError::NoVault)?;
        let excluded = g.settings.excluded_folders.clone();
        (root, excluded)
    };
    let idir = index_dir();
    let _ = std::fs::remove_dir_all(&idir);
    std::fs::create_dir_all(&idir).map_err(AppError::Io)?;
    let (schema, path_f, title_f, body_f, tags_f) = schema();
    let index = tantivy::Index::create_in_dir(&idir, schema)
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    let mut writer = index
        .writer::<tantivy::TantivyDocument>(50_000_000)
        .map_err(|e| AppError::Tantivy(e.to_string()))?;

    let files = vault::walk_markdown(&root, &excluded);
    for f in files {
        let rel = vault::to_rel(&root, &f);
        let content = crate::secure::read_note_text(&f).unwrap_or_default();
        writer
            .add_document(note_document(
                &rel, &content, path_f, title_f, body_f, tags_f,
            ))
            .map_err(|e| AppError::Tantivy(e.to_string()))?;
    }
    writer
        .commit()
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    STALE.store(false, Ordering::SeqCst);
    Ok(())
}

/// Update a single note in the index after it is saved or created. Falls back
/// to a rebuild on the next search when the index does not exist yet.
pub fn index_note(_state: &State<AppState>, rel: &str, content: &str) -> AppResult<()> {
    if !is_markdown(rel) {
        return Ok(());
    }
    let _guard = index_lock();
    let idir = index_dir();
    if !idir.exists() {
        mark_stale();
        return Ok(());
    }
    let (schema, path_f, title_f, body_f, tags_f) = schema();
    let index = tantivy::Index::open_in_dir(&idir).map_err(|e| AppError::Tantivy(e.to_string()))?;
    let mut writer = index
        .writer::<tantivy::TantivyDocument>(15_000_000)
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    writer.delete_term(tantivy::schema::Term::from_field_text(path_f, rel));
    writer
        .add_document(note_document(rel, content, path_f, title_f, body_f, tags_f))
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    writer
        .commit()
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    Ok(())
}

/// Remove a single document. Deleting folders calls `mark_stale` as well, so
/// nested documents are cleaned up by the next rebuild.
pub fn remove_note(_state: &State<AppState>, rel: &str) -> AppResult<()> {
    if !is_markdown(rel) {
        return Ok(());
    }
    let _guard = index_lock();
    let idir = index_dir();
    if !idir.exists() {
        return Ok(());
    }
    let (_, path_f, _, _, _) = schema();
    let index = tantivy::Index::open_in_dir(&idir).map_err(|e| AppError::Tantivy(e.to_string()))?;
    let mut writer = index
        .writer::<tantivy::TantivyDocument>(15_000_000)
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    writer.delete_term(tantivy::schema::Term::from_field_text(path_f, rel));
    writer
        .commit()
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    Ok(())
}

pub fn search(
    state: &State<AppState>,
    query: &str,
    filters: &serde_json::Value,
) -> AppResult<Vec<SearchHit>> {
    let idir = index_dir();
    let _guard = index_lock();
    if !idir.exists() || STALE.load(Ordering::SeqCst) {
        rebuild_locked(state)?;
    }
    let (_, path_f, title_f, body_f, tags_f) = schema();
    let index = tantivy::Index::open_in_dir(&idir).map_err(|e| AppError::Tantivy(e.to_string()))?;
    let reader = index
        .reader()
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    let searcher = reader.searcher();
    let qp = tantivy::query::QueryParser::for_index(&index, vec![title_f, body_f, tags_f]);
    let q = if query.trim().is_empty() {
        "*".to_string()
    } else {
        query.to_string()
    };
    let parsed_query = if q == "*" {
        Box::new(tantivy::query::AllQuery) as Box<dyn tantivy::query::Query>
    } else {
        qp.parse_query(&q)
            .map_err(|e| AppError::Tantivy(e.to_string()))?
    };
    let top = searcher
        .search(&parsed_query, &tantivy::collector::TopDocs::with_limit(50))
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    let snippet_gen = tantivy::snippet::SnippetGenerator::create(&searcher, &parsed_query, body_f)
        .map_err(|e| AppError::Tantivy(e.to_string()))?;

    let mut hits = Vec::new();
    for (score, doc_addr) in top {
        let doc: tantivy::TantivyDocument = searcher
            .doc(doc_addr)
            .map_err(|e| AppError::Tantivy(e.to_string()))?;
        let path = match doc.get_first(path_f) {
            Some(tantivy::schema::OwnedValue::Str(t)) => t,
            _ => "",
        }
        .to_string();
        let title = match doc.get_first(title_f) {
            Some(tantivy::schema::OwnedValue::Str(t)) => t,
            _ => "",
        }
        .to_string();
        let snippet = snippet_gen.snippet_from_doc(&doc).to_html();
        if let Some(folder) = filters.get("folder").and_then(|v| v.as_str()) {
            if !folder.is_empty() && !matches_folder(&path, folder) {
                continue;
            }
        }
        if let Some(tag) = filters.get("tag").and_then(|v| v.as_str()) {
            if !tag.is_empty() {
                let tags_val = match doc.get_first(tags_f) {
                    Some(tantivy::schema::OwnedValue::Str(t)) => t.as_str(),
                    _ => "",
                };
                if !tags_val.split_whitespace().any(|t| t == tag) {
                    continue;
                }
            }
        }
        hits.push(SearchHit {
            path,
            score,
            snippet,
            title,
        });
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::matches_folder;

    #[test]
    fn folder_filter_matches_only_the_selected_folder_boundary() {
        assert!(matches_folder("notes/a.md", "notes"));
        assert!(matches_folder("notes", "notes"));
        assert!(matches_folder("notes/a.md", "notes/"));
        assert!(!matches_folder("notes2/a.md", "notes"));
        assert!(!matches_folder("other/a.md", "notes"));
    }
}

pub fn status(state: &State<AppState>) -> (usize, usize, String) {
    let idir = index_dir();
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone()
    };
    let total = if let Some(r) = root {
        vault::walk_markdown(&r, &[]).len()
    } else {
        0
    };
    if !idir.exists() {
        return (total, 0, "not built".into());
    }
    let state_str = if STALE.load(Ordering::SeqCst) {
        "stale"
    } else {
        "ready"
    };
    (total, total, state_str.into())
}
