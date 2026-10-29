//! Wiki links and backlinks.
//!
//! The original build stored a link graph in SQLite. Recomputing it from the
//! files is cheap for a personal vault and can never drift out of date, so the
//! native build scans on demand and caches nothing between edits.

use std::collections::HashSet;

/// A `[[target]]` or `[[target|label]]` reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The note reference without its `|label` or `#heading` annotation.
    pub target: String,
    /// `[[target|label]]` keeps its label so a rename does not discard it.
    pub label: Option<String>,
    /// `[[target#heading]]` keeps the fragment for the same reason.
    pub fragment: Option<String>,
    /// Byte range of the whole `[[...]]` token, for rewriting on rename.
    pub start: usize,
    pub end: usize,
}

/// Extract wiki links, ignoring fenced code and inline code.
pub fn wiki_links(text: &str) -> Vec<Link> {
    let mut links = Vec::new();
    let mut in_fence = false;
    let bytes = text.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        // Track fences line by line.
        if index == 0 || bytes[index - 1] == b'\n' {
            let line_end = text[index..]
                .find('\n')
                .map(|offset| index + offset)
                .unwrap_or(text.len());
            let line = &text[index..line_end];
            if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
                in_fence = !in_fence;
            }
        }
        if !in_fence && index + 1 < bytes.len() && bytes[index] == b'[' && bytes[index + 1] == b'['
        {
            if let Some(close) = text[index + 2..].find("]]") {
                let inner = &text[index + 2..index + 2 + close];
                let (reference, label) = match inner.split_once('|') {
                    Some((reference, label)) => (reference.trim(), Some(label.trim().to_string())),
                    None => (inner.trim(), None),
                };
                let (target, fragment) = match reference.split_once('#') {
                    Some((target, fragment)) => (target.trim(), Some(fragment.trim().to_string())),
                    None => (reference, None),
                };
                if !target.is_empty() && !target.contains('\n') {
                    links.push(Link {
                        target: target.to_string(),
                        label: label.filter(|l| !l.is_empty()),
                        fragment: fragment.filter(|f| !f.is_empty()),
                        start: index,
                        end: index + 2 + close + 2,
                    });
                }
                index = index + 2 + close + 2;
                continue;
            }
        }
        index += 1;
    }
    links
}

/// Rewrite every spelling of a link target after a rename.
///
/// `[[Note]]`, `[[Note|label]]`, `[[folder/Note]]` and `[[Note#heading]]` all
/// point at the same document and must all be updated.
pub fn rename_targets(text: &str, old: &str, new: &str) -> String {
    let links = wiki_links(text);
    if links.is_empty() {
        return text.to_string();
    }
    let old_stem = stem(old);
    let new_stem = stem(new);
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for link in links {
        if cursor > link.start {
            continue;
        }
        // `[[Note]]`, `[[Note.md]]`, `[[folder/Note]]` and `[[Note#heading]]` all
        // point at the same document; the label and heading survive the rename.
        let matches = link.target == old || stem(&link.target) == old_stem;
        if !matches {
            continue;
        }
        out.push_str(&text[cursor..link.start]);
        let replacement_target = if link.target.contains('/') {
            new.to_string()
        } else {
            new_stem.clone()
        };
        out.push_str("[[");
        out.push_str(&replacement_target);
        if let Some(fragment) = &link.fragment {
            out.push('#');
            out.push_str(fragment);
        }
        if let Some(label) = &link.label {
            out.push('|');
            out.push_str(label);
        }
        out.push_str("]]");
        cursor = link.end;
    }
    out.push_str(&text[cursor..]);
    out
}

fn stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem.to_string(),
        _ => name.to_string(),
    }
}

/// Notes that link to `target`, as vault-relative paths.
pub fn backlinks<'a, I>(target: &str, documents: I) -> Vec<String>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let wanted = stem(target);
    let mut out = Vec::new();
    for (path, body) in documents {
        if stem(path) == wanted {
            continue;
        }
        if wiki_links(body)
            .iter()
            .any(|link| stem(&link.target) == wanted)
        {
            out.push(path.to_string());
        }
    }
    out
}

/// Tags used in a document: `#tag` and frontmatter `tags:`.
pub fn tags(text: &str) -> Vec<String> {
    let mut found: HashSet<String> = HashSet::new();
    let mut in_frontmatter = false;
    let mut lines = text.lines().peekable();

    if lines.peek().is_some_and(|l| l.trim() == "---") {
        lines.next();
        in_frontmatter = true;
    }
    let mut collecting_tags = false;
    for line in lines {
        let trimmed = line.trim();
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
                collecting_tags = false;
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("tags:") {
                collecting_tags = true;
                // `tags: one, two`, `tags: [one, two]` and `tags: "one"` are
                // all written in the wild, so strip list brackets and quotes
                // before deciding what the tag is.
                for part in rest.split(',') {
                    if let Some(tag) = clean_tag(part) {
                        found.insert(tag);
                    }
                }
                continue;
            }
            if collecting_tags {
                if let Some(item) = trimmed.strip_prefix('-') {
                    if let Some(tag) = clean_tag(item) {
                        found.insert(tag);
                    }
                    continue;
                }
                collecting_tags = false;
            }
            continue;
        }
        // Inline #tags, but not headings and not inside code.
        if trimmed.starts_with('#') {
            continue;
        }
        let mut word = String::new();
        for ch in trimmed.chars() {
            if ch == '#' {
                word.clear();
                continue;
            }
            if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '/' {
                if !word.is_empty() || ch.is_alphanumeric() {
                    word.push(ch);
                }
            } else {
                if word.chars().any(|c| c.is_alphabetic()) {
                    found.insert(word.clone());
                }
                word.clear();
            }
        }
        if word.chars().any(|c| c.is_alphabetic()) {
            found.insert(word);
        }
    }
    let mut out: Vec<String> = found.into_iter().collect();
    out.sort();
    out
}

/// Normalise one frontmatter tag: drop list brackets, quotes and a leading `#`.
fn clean_tag(raw: &str) -> Option<String> {
    let tag = raw
        .trim()
        .trim_matches(|c: char| matches!(c, '[' | ']' | '{' | '}' | '"' | '\'' | ','))
        .trim_start_matches('#')
        .trim();
    (!tag.is_empty()).then(|| tag.to_string())
}

/// Key/value pairs from YAML frontmatter, in document order.
pub fn properties(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut lines = text.lines();
    if !lines.next().is_some_and(|l| l.trim() == "---") {
        return out;
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            if key.is_empty() || key.starts_with('-') {
                continue;
            }
            out.push((key.to_string(), value.trim().trim_matches('"').to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_links_are_extracted() {
        let links = wiki_links("See [[Alpha]] and [[folder/Beta|the beta]].");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "Alpha");
        assert_eq!(links[1].target, "folder/Beta");
    }

    #[test]
    fn code_blocks_are_ignored() {
        let links = wiki_links("```\n[[NotALink]]\n```\n[[Real]]\n");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Real");
    }

    #[test]
    fn rename_updates_every_spelling() {
        let text = "[[Note]] [[Note|label]] [[Note#Heading]] [[Other]]";
        let renamed = rename_targets(text, "Note.md", "Renamed.md");
        assert_eq!(
            renamed, "[[Renamed]] [[Renamed|label]] [[Renamed#Heading]] [[Other]]",
            "labels and headings must survive the rename"
        );
    }

    #[test]
    fn backlinks_resolve_by_stem() {
        let docs = vec![
            ("a.md", "links to [[Target]]"),
            ("b.md", "nothing"),
            ("Target.md", "self"),
        ];
        assert_eq!(backlinks("Target.md", docs), vec!["a.md".to_string()]);
    }

    #[test]
    fn tags_come_from_frontmatter_and_body() {
        let text = "---\ntags: [one, two]\n---\n\nBody with #three and a #four/five tag.\n";
        let tags = tags(text);
        assert!(tags.contains(&"one".to_string()));
        assert!(tags.contains(&"three".to_string()));
        assert!(tags.contains(&"four/five".to_string()));
    }

    #[test]
    fn properties_keep_document_order() {
        let props = properties("---\ntitle: Hello\ntags: a\n---\nbody\n");
        assert_eq!(props[0].0, "title");
        assert_eq!(props[0].1, "Hello");
        assert_eq!(props[1].0, "tags");
    }
}
