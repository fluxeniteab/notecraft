use crate::error::AppError;
use crate::state::AppState;
use crate::vault;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub score: f32,
    pub snippet: String,
    pub title: String,
}

fn index_dir() -> PathBuf {
    crate::settings::data_dir().join("search_index")
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
        .writer(50_000_000)
        .map_err(|e| AppError::Tantivy(e.to_string()))?;

    let files = vault::walk_markdown(&root, &excluded);
    for f in files {
        let rel = vault::to_rel(&root, &f);
        let content = std::fs::read_to_string(&f).unwrap_or_default();
        let parsed = crate::markdown::parse(&content);
        let title = f
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let tags = parsed.tags.join(" ");
        writer
            .add_document(tantivy::doc!(
                path_f => rel.clone(),
                title_f => title,
                body_f => parsed.body.clone(),
                tags_f => tags,
            ))
            .map_err(|e| AppError::Tantivy(e.to_string()))?;
    }
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
    if !idir.exists() {
        // auto rebuild
        rebuild(state)?;
    }
    let (schema, path_f, title_f, body_f, tags_f) = schema();
    let index = tantivy::Index::open_in_dir(&idir).map_err(|e| AppError::Tantivy(e.to_string()))?;
    let reader = index
        .reader()
        .map_err(|e| AppError::Tantivy(e.to_string()))?;
    let searcher = reader.searcher();
    let qp = tantivy::query::QueryParser::for_index(&index, vec![title_f, body_f, tags_f]);
    // support filters: folder, tag, ext (ignored for now beyond query)
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
    for (_score, doc_addr) in top {
        let doc: tantivy::TantivyDocument = searcher
            .doc(doc_addr)
            .map_err(|e| AppError::Tantivy(e.to_string()))?;
        let path = doc
            .get_first(path_f)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = doc
            .get_first(title_f)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let score = 1.0f32;
        let snippet = snippet_gen.snippet_from_doc(&doc).to_html();
        // folder filter
        if let Some(folder) = filters.get("folder").and_then(|v| v.as_str()) {
            if !folder.is_empty() && !path.starts_with(folder) {
                continue;
            }
        }
        if let Some(tag) = filters.get("tag").and_then(|v| v.as_str()) {
            if !tag.is_empty() {
                let tags_val = doc.get_first(tags_f).and_then(|v| v.as_str()).unwrap_or("");
                if !tags_val.contains(tag) {
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
    // apply folder filter from query? done
    Ok(hits)
}

pub fn status(state: &State<AppState>) -> (usize, usize, String) {
    let idir = index_dir();
    if !idir.exists() {
        return (0, 0, "not built".into());
    }
    // count md files
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone()
    };
    let total = if let Some(r) = root {
        vault::walk_markdown(&r, &[]).len()
    } else {
        0
    };
    (total, total, "ready".into())
}
