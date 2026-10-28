use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
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

fn normalize_target(target: &str) -> String {
    let target = target.trim();
    let target = target.split('#').next().unwrap_or(target);
    let target = target.split('?').next().unwrap_or(target);
    target
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}

fn dir_of(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

/// Wiki links can be written as `[[Note]]`, `[[folder/Note]]`, `[[Note.md]]`
/// or relative to the current folder. Resolve them to a real vault path so
/// graph edges always connect nodes that exist on disk.
fn resolve_target(
    target: &str,
    source: &str,
    exact: &HashMap<String, String>,
    by_stem: &HashMap<String, String>,
) -> Option<String> {
    let wanted = normalize_target(target);
    if wanted.is_empty() {
        return None;
    }
    let with_ext = if wanted.to_lowercase().ends_with(".md")
        || wanted.to_lowercase().ends_with(".markdown")
    {
        wanted.clone()
    } else {
        format!("{}.md", wanted)
    };
    if let Some(path) = exact.get(&with_ext.to_lowercase()) {
        return Some(path.clone());
    }
    let parent = dir_of(source);
    if !parent.is_empty() {
        let relative = format!("{}/{}", parent, with_ext);
        if let Some(path) = exact.get(&relative.to_lowercase()) {
            return Some(path.clone());
        }
    }
    let stem = with_ext
        .rsplit('/')
        .next()
        .unwrap_or(&with_ext)
        .trim_end_matches(".md")
        .trim_end_matches(".markdown");
    by_stem.get(&stem.to_lowercase()).cloned()
}

pub fn build(
    state: &State<AppState>,
    _scope: &str,
    _filters: &serde_json::Value,
) -> AppResult<GraphData> {
    let (root, excluded, raw_edges) = {
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

    let mut nodes_set: BTreeSet<String> = BTreeSet::new();
    if let Some(r) = &root {
        for p in crate::vault::walk_markdown(r, &excluded) {
            nodes_set.insert(crate::vault::to_rel(r, &p));
        }
    }

    let exact: HashMap<String, String> = nodes_set
        .iter()
        .map(|path| (path.to_lowercase(), path.clone()))
        .collect();
    let mut by_stem: HashMap<String, String> = HashMap::new();
    for path in &nodes_set {
        let stem = path
            .rsplit('/')
            .next()
            .unwrap_or(path)
            .trim_end_matches(".md")
            .trim_end_matches(".markdown")
            .to_lowercase();
        by_stem
            .entry(stem)
            .and_modify(|current| {
                if path.len() < current.len() {
                    *current = path.clone();
                }
            })
            .or_insert_with(|| path.clone());
    }

    let mut edges: Vec<GraphEdge> = vec![];
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for edge in raw_edges {
        let Some(source) = exact.get(&edge.source.to_lowercase()).cloned() else {
            continue;
        };
        let Some(target) = resolve_target(&edge.target, &source, &exact, &by_stem) else {
            continue;
        };
        if source == target {
            continue;
        }
        if !seen.insert((source.clone(), target.clone())) {
            continue;
        }
        edges.push(GraphEdge {
            id: format!("{}->{}", source, target),
            source,
            target,
            kind: edge.kind,
        });
    }

    let nodes: Vec<GraphNode> = nodes_set
        .into_iter()
        .map(|id| {
            let label = id
                .rsplit('/')
                .next()
                .unwrap_or(&id)
                .trim_end_matches(".md")
                .trim_end_matches(".markdown")
                .to_string();
            GraphNode {
                id,
                label,
                x: None,
                y: None,
                size: Some(6.0),
                color: None,
                tags: None,
            }
        })
        .collect();

    Ok(GraphData { nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::resolve_target;
    use std::collections::HashMap;

    fn maps() -> (HashMap<String, String>, HashMap<String, String>) {
        let paths = ["Note.md", "folder/Note.md", "folder/Deep Note.md", "other.md"];
        let exact = paths
            .iter()
            .map(|p| (p.to_lowercase(), p.to_string()))
            .collect();
        let mut by_stem = HashMap::new();
        by_stem.insert("note".to_string(), "Note.md".to_string());
        by_stem.insert("deep note".to_string(), "folder/Deep Note.md".to_string());
        by_stem.insert("other".to_string(), "other.md".to_string());
        (exact, by_stem)
    }

    #[test]
    fn resolves_bare_and_extended_links() {
        let (exact, by_stem) = maps();
        assert_eq!(
            resolve_target("Note", "folder/Source.md", &exact, &by_stem),
            Some("Note.md".into())
        );
        assert_eq!(
            resolve_target("Note.md", "other.md", &exact, &by_stem),
            Some("Note.md".into())
        );
    }

    #[test]
    fn resolves_relative_folder_links_and_headings() {
        let (exact, by_stem) = maps();
        // A bare name prefers an exact vault path when one exists.
        assert_eq!(
            resolve_target("Note", "folder/Source.md", &exact, &by_stem),
            Some("Note.md".into())
        );
        // Otherwise a note in the same folder wins.
        assert_eq!(
            resolve_target("Deep Note#Heading", "folder/Source.md", &exact, &by_stem),
            Some("folder/Deep Note.md".into())
        );
    }

    #[test]
    fn drops_unresolved_links() {
        let (exact, by_stem) = maps();
        assert_eq!(
            resolve_target("Missing", "Note.md", &exact, &by_stem),
            None
        );
        assert_eq!(resolve_target("", "Note.md", &exact, &by_stem), None);
    }
}
