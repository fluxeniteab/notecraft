use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub size: Option<f32>,
    pub color: Option<String>,
    pub tags: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub fn build(
    state: &State<AppState>,
    _scope: &str,
    _filters: &serde_json::Value,
) -> AppResult<GraphData> {
    let (root, excluded, edges) = {
        let g = state.inner.lock();
        let root = g.vault_root.clone();
        let excluded = g.settings.excluded_folders.clone();
        let mut edges: Vec<GraphEdge> = vec![];
        if let Some(conn) = g.db.as_ref() {
            let mut stmt = conn
                .prepare("SELECT source, target, kind FROM links")
                .map_err(AppError::Sqlite)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                })
                .map_err(AppError::Sqlite)?;
            for r in rows {
                if let Ok((s, t, k)) = r {
                    edges.push(GraphEdge {
                        id: format!("{}->{}", s, t),
                        source: s,
                        target: t,
                        kind: Some(k),
                    });
                }
            }
        }
        (root, excluded, edges)
    };

    let mut files: Vec<String> = vec![];
    let mut nodes_set: BTreeSet<String> = BTreeSet::new();
    if let Some(r) = &root {
        for p in crate::vault::walk_markdown(r, &excluded) {
            let rel = crate::vault::to_rel(r, &p);
            nodes_set.insert(rel.clone());
            files.push(rel);
        }
    }
    for e in &edges {
        nodes_set.insert(e.source.clone());
        nodes_set.insert(e.target.clone());
    }

    let nodes: Vec<GraphNode> = nodes_set
        .into_iter()
        .map(|id| {
            let label = id
                .rsplit('/')
                .next()
                .unwrap_or(&id)
                .trim_end_matches(".md")
                .to_string();
            GraphNode {
                id,
                label,
                x: None,
                y: None,
                size: Some(6.0),
                color: Some("#8b5cf6".into()),
                tags: None,
            }
        })
        .collect();

    Ok(GraphData { nodes, edges })
}
