// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::vault::tree::is_markdown;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub snippet: String,
    pub score: usize,
}

pub struct SearchIndex {
    pub root: Option<PathBuf>,
    pub docs: Vec<(PathBuf, String)>,
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            root: None,
            docs: Vec::new(),
        }
    }

    pub fn rebuild(&mut self, root: &Path) {
        self.root = Some(root.to_path_buf());
        self.docs.clear();
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && is_markdown(p) {
                if let Ok(content) = fs::read_to_string(p) {
                    self.docs.push((p.to_path_buf(), content));
                }
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::new();
        for (path, content) in &self.docs {
            let lower = content.to_lowercase();
            if let Some(pos) = lower.find(&q) {
                // snippet
                let start = pos.saturating_sub(60);
                let end = (pos + q.len() + 60).min(content.len());
                // adjust to char boundaries
                let mut s = start;
                while s > 0 && !content.is_char_boundary(s) {
                    s -= 1;
                }
                let mut e = end;
                while e < content.len() && !content.is_char_boundary(e) {
                    e += 1;
                }
                let snippet = content[s..e].replace('\n', " ");
                let score = lower.matches(&q).count();
                results.push(SearchResult {
                    path: path.clone(),
                    snippet,
                    score,
                });
            }
        }
        results.sort_by_key(|r| std::cmp::Reverse(r.score));
        results
    }

    pub fn update_file(&mut self, path: &Path, content: &str) {
        if let Some(idx) = self.docs.iter().position(|(p, _)| p == path) {
            self.docs[idx].1 = content.to_string();
        } else {
            self.docs.push((path.to_path_buf(), content.to_string()));
        }
    }
}
