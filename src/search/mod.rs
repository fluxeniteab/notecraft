//! Full-text search.
//!
//! The Tauri build kept a Tantivy index on disk. The native build searches the
//! vault directly: notes are plain files, the walk is cached by the explorer,
//! and a linear scan of a personal vault is both fast enough and impossible to
//! get out of sync with the files on disk. Results carry a line number and a
//! snippet so the editor can jump straight to the match.

use std::path::Path;

use crate::documents::Kind;
use crate::filesystem::FileEntry;

/// Files larger than this are skipped: they are data dumps, not notes.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// Which kinds a search covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    All,
    Prose,
    Data,
    Code,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Scope::All => "All",
            Scope::Prose => "Prose",
            Scope::Data => "Data",
            Scope::Code => "Code",
        }
    }

    fn matches(self, kind: Kind) -> bool {
        use crate::documents::Stats;
        let stats = kind.capabilities().stats;
        match self {
            Scope::All => true,
            Scope::Prose => stats == Stats::Prose,
            Scope::Data => stats == Stats::Data,
            Scope::Code => stats == Stats::Code,
        }
    }
}

/// One match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub score: i64,
}

/// Restrict a search to a folder (vault-relative, empty means the whole vault).
pub struct Filter<'a> {
    pub folder: &'a str,
    pub scope: Scope,
}

impl Default for Filter<'_> {
    fn default() -> Self {
        Self {
            folder: "",
            scope: Scope::All,
        }
    }
}

fn snippet_for(line: &str, needle_start: usize, needle_len: usize) -> String {
    const CONTEXT: usize = 46;
    let chars: Vec<char> = line.chars().collect();
    let start_char = line[..needle_start].chars().count();
    let needle_chars = line[needle_start..needle_start + needle_len]
        .chars()
        .count()
        .max(1);
    let from = start_char.saturating_sub(CONTEXT);
    let to = (start_char + needle_chars + CONTEXT).min(chars.len());
    let mut out = String::new();
    if from > 0 {
        out.push('…');
    }
    out.extend(&chars[from..to]);
    if to < chars.len() {
        out.push('…');
    }
    out
}

/// Score a match: filename hits and word-boundary hits rank higher.
fn score_of(entry: &FileEntry, line: &str, query: &str, column: usize) -> i64 {
    let mut score = 10;
    if entry.name.to_lowercase().contains(&query.to_lowercase()) {
        score += 40;
    }
    let before = line[..column].chars().next_back();
    let after = line[column + query.len()..].chars().next();
    let boundary = |c: Option<char>| {
        matches!(
            c,
            None | Some(' ' | '\t' | '(' | '[' | '"' | '\'' | '*' | '_' | '#' | '-' | '.' | ',')
        )
    };
    if boundary(before) && boundary(after) {
        score += 25;
    }
    if line.trim_start().starts_with('#') {
        score += 15;
    }
    score
}

/// Search the vault.
pub fn search(root: &Path, files: &[FileEntry], query: &str, filter: &Filter<'_>) -> Vec<Hit> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    let needle = query.to_lowercase();
    let mut hits = Vec::new();

    for entry in files {
        if entry.is_dir || entry.size > MAX_FILE_BYTES {
            continue;
        }
        let kind = Kind::from_path(&entry.path);
        if !kind.capabilities().indexable || !filter.scope.matches(kind) {
            continue;
        }
        if !filter.folder.is_empty() && !entry.path.starts_with(filter.folder.trim_end_matches('/'))
        {
            continue;
        }
        let Ok(text) = crate::filesystem::read_text(root, &entry.path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            let haystack = line.to_lowercase();
            let mut from = 0;
            while let Some(offset) = haystack[from..].find(&needle) {
                let column = from + offset;
                if column <= line.len() {
                    hits.push(Hit {
                        path: entry.path.clone(),
                        line: index + 1,
                        column,
                        snippet: snippet_for(line, column, query.len()),
                        score: score_of(entry, line, query, column),
                    });
                }
                from = column + needle.len().max(1);
                if from >= haystack.len() {
                    break;
                }
            }
        }
    }

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.path.cmp(&b.path))
            .then(a.line.cmp(&b.line))
    });
    hits.truncate(500);
    hits
}

/// Quick file-name fuzzy match for the command palette.
pub fn fuzzy_path(query: &str, path: &str) -> Option<usize> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Some(0);
    }
    let haystack = path.to_lowercase();
    let mut score = 0usize;
    let mut index = 0usize;
    for ch in query.chars() {
        let found = haystack[index..].find(ch)?;
        index += found + ch.len_utf8();
        score += 1;
    }
    // Shorter paths that contain the name rank first.
    Some(score * 100 - haystack.len().min(99))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::{create_file, scan_vault};

    fn vault(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "notecraft-search-{tag}-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn finds_matches_with_line_numbers() {
        let root = vault("basic");
        create_file(&root, "a.md", "# Alpha\n\nThe alpha bet.\n").unwrap();
        create_file(&root, "b.json", "{\n  \"alpha\": 1\n}\n").unwrap();
        let files = scan_vault(&root, &[], false);
        let hits = search(&root, &files, "alpha", &Filter::default());
        assert_eq!(hits.len(), 3);
        // Heading matches rank above body matches.
        assert_eq!(hits[0].line, 1);
        assert!(hits.iter().any(|h| h.path == "b.json" && h.line == 2));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scope_filters_by_document_family() {
        let root = vault("scope");
        create_file(&root, "note.md", "needle\n").unwrap();
        create_file(&root, "data.json", "\"needle\"\n").unwrap();
        let files = scan_vault(&root, &[], false);
        let only_prose = search(
            &root,
            &files,
            "needle",
            &Filter {
                folder: "",
                scope: Scope::Prose,
            },
        );
        assert_eq!(only_prose.len(), 1);
        assert_eq!(only_prose[0].path, "note.md");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn folder_filter_is_a_path_boundary() {
        let root = vault("folder");
        create_file(&root, "notes/a.md", "x\n").unwrap();
        create_file(&root, "notes2/b.md", "x\n").unwrap();
        let files = scan_vault(&root, &[], false);
        let hits = search(
            &root,
            &files,
            "x",
            &Filter {
                folder: "notes",
                scope: Scope::All,
            },
        );
        // "notes2" must not match the "notes" scope.
        assert!(hits.iter().all(|h| h.path.starts_with("notes/")));
        assert_eq!(hits.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn case_insensitive_and_multiple_hits_per_line_are_bounded() {
        let root = vault("case");
        create_file(&root, "a.md", "Foo foo FOO\n").unwrap();
        let files = scan_vault(&root, &[], false);
        let hits = search(&root, &files, "foo", &Filter::default());
        assert_eq!(hits.len(), 3);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn empty_query_returns_nothing() {
        let root = vault("empty");
        create_file(&root, "a.md", "text\n").unwrap();
        let files = scan_vault(&root, &[], false);
        assert!(search(&root, &files, "   ", &Filter::default()).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn fuzzy_matching_prefers_ordered_subsequences() {
        assert!(fuzzy_path("nd", "notes/daily.md").is_some());
        assert!(fuzzy_path("zz", "notes/daily.md").is_none());
    }
}
