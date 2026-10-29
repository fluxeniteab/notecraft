use crate::engine::{self, Dialect, Kind};
use crate::error::AppResult;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    pub raw: String,
    pub data: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub line: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRef {
    pub target: String,
    pub alias: Option<String>,
    pub heading: Option<String>,
    pub block: Option<String>,
    pub line: usize,
    pub kind: String, // wiki | markdown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedNote {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub outline: Vec<OutlineItem>,
    pub links: Vec<LinkRef>,
    pub tags: Vec<String>,
    pub tasks: Vec<TaskRef>,
    pub word_count: usize,
    pub char_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRef {
    pub line: usize,
    pub text: String,
    pub done: bool,
    pub indent: usize,
    pub due: Option<String>,
    pub priority: Option<String>,
}

pub fn parse_frontmatter(content: &str) -> (Frontmatter, String) {
    let mut fm = Frontmatter::default();
    if !content.starts_with("---") {
        return (fm, content.to_string());
    }
    // find closing ---
    let rest = &content[3..];
    if let Some(idx) = rest.find("\n---") {
        let yaml_str = rest[..idx].trim_start_matches('\n').to_string();
        let body_start = 3 + idx + 4; // "---" + ... + "\n---"
        let body = if body_start <= content.len() {
            content[body_start..].trim_start_matches('\n').to_string()
        } else {
            String::new()
        };
        fm.raw = yaml_str.clone();
        if let Ok(v) = serde_yaml::from_str::<Value>(&yaml_str) {
            if let Some(map) = v.as_object() {
                for (k, val) in map {
                    fm.data.insert(k.clone(), val.clone());
                }
            } else if v != Value::Null {
                fm.data.insert("_root".into(), v);
            }
        }
        (fm, body)
    } else {
        (fm, content.to_string())
    }
}

pub fn extract_links(body: &str) -> Vec<LinkRef> {
    let mut out = Vec::new();
    // wiki links: [[target]] or [[target|alias]] or [[target#heading]] or [[target#^block]]
    let wiki_re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    for (i, line) in body.lines().enumerate() {
        for cap in wiki_re.captures_iter(line) {
            let inner = cap.get(1).unwrap().as_str().trim();
            let (target_part, alias) = if let Some(pos) = inner.find('|') {
                (
                    inner[..pos].trim(),
                    Some(inner[pos + 1..].trim().to_string()),
                )
            } else {
                (inner, None)
            };
            let (target, heading, block) = parse_wiki_target(target_part);
            out.push(LinkRef {
                target,
                alias,
                heading,
                block,
                line: i + 1,
                kind: "wiki".into(),
            });
        }
    }
    // markdown links: [text](url)
    let md_re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for (i, line) in body.lines().enumerate() {
        for cap in md_re.captures_iter(line) {
            let text = cap.get(1).unwrap().as_str().to_string();
            let url = cap.get(2).unwrap().as_str().trim().to_string();
            if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("mailto:")
            {
                continue;
            }
            out.push(LinkRef {
                target: url,
                alias: Some(text),
                heading: None,
                block: None,
                line: i + 1,
                kind: "markdown".into(),
            });
        }
    }
    out
}

fn parse_wiki_target(s: &str) -> (String, Option<String>, Option<String>) {
    // target may be "path#heading" or "path#^block" or "path"
    if let Some(pos) = s.find('#') {
        let (t, rest) = s.split_at(pos);
        let rest = &rest[1..];
        if rest.starts_with('^') {
            (
                t.trim().to_string(),
                None,
                Some(rest[1..].trim().to_string()),
            )
        } else {
            (t.trim().to_string(), Some(rest.trim().to_string()), None)
        }
    } else {
        (s.trim().to_string(), None, None)
    }
}

pub fn extract_tags(body: &str, fm: &Frontmatter) -> Vec<String> {
    let mut tags = Vec::new();
    // inline #tag
    let re = Regex::new(r"(?:^|\s)#([A-Za-z0-9_/\-]+)").unwrap();
    for cap in re.captures_iter(body) {
        tags.push(cap.get(1).unwrap().as_str().to_string());
    }
    // frontmatter tags
    if let Some(v) = fm.data.get("tags") {
        match v {
            Value::String(s) => tags.push(s.clone()),
            Value::Array(seq) => {
                for item in seq {
                    if let Some(s) = item.as_str() {
                        tags.push(s.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    // dedup
    let mut seen = std::collections::HashSet::new();
    tags.retain(|t| seen.insert(t.clone()));
    tags
}

pub fn extract_outline(body: &str) -> Vec<OutlineItem> {
    let mut out = Vec::new();
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            // count
            let mut level = 1u8;
            let mut chars = rest.chars();
            while let Some('#') = chars.next() {
                level += 1;
            }
            // rebuild text
            let text = trimmed.trim_start_matches('#').trim().to_string();
            if !text.is_empty() && level <= 6 {
                let id = text
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                out.push(OutlineItem {
                    level,
                    text,
                    line: i + 1,
                    id,
                });
            }
        }
    }
    out
}

pub fn extract_tasks(body: &str) -> Vec<TaskRef> {
    let mut out = Vec::new();
    let re = Regex::new(r"^(\s*)- \[([ xX])\] (.*)$").unwrap();
    for (i, line) in body.lines().enumerate() {
        if let Some(cap) = re.captures(line) {
            let indent = cap.get(1).unwrap().as_str().len();
            let done = cap.get(2).unwrap().as_str().to_lowercase() == "x";
            let text = cap.get(3).unwrap().as_str().to_string();
            let due = Regex::new(r"(?:📅|due:)\s*(\d{4}-\d{2}-\d{2})")
                .unwrap()
                .captures(&text)
                .and_then(|capture| capture.get(1).map(|value| value.as_str().to_string()));
            let priority = Regex::new(r"(?:⏫|priority:)\s*(highest|high|medium|low|lowest)")
                .unwrap()
                .captures(&text.to_lowercase())
                .and_then(|capture| capture.get(1).map(|value| value.as_str().to_string()));
            out.push(TaskRef {
                line: i + 1,
                text,
                done,
                indent,
                due,
                priority,
            });
        }
    }
    out
}

pub fn parse(content: &str) -> ParsedNote {
    let (fm, body) = parse_frontmatter(content);
    let outline = extract_outline(&body);
    let links = extract_links(&body);
    let tags = extract_tags(&body, &fm);
    let tasks = extract_tasks(&body);
    let word_count = body.split_whitespace().count();
    let char_count = body.chars().count();
    ParsedNote {
        frontmatter: fm,
        body,
        outline,
        links,
        tags,
        tasks,
        word_count,
        char_count,
    }
}

/// Defence in depth for preview HTML. The engine escapes every character of
/// user text itself, so this only has to keep the markup we deliberately emit.
fn sanitize(html: &str) -> String {
    let mut builder = ammonia::Builder::default();
    builder.add_tags([
        "input", "section", "dl", "dt", "dd", "mark", "del", "sup", "sub", "div", "span",
    ]);
    builder.add_generic_attributes(["class", "id"]);
    builder.add_tag_attributes("input", ["type", "checked", "disabled"]);
    builder.add_tag_attributes("th", ["align"]);
    builder.add_tag_attributes("td", ["align"]);
    // `rel` is managed by ammonia itself and must not be added manually.
    builder.add_tag_attributes("a", ["class", "href", "title"]);
    builder.add_tag_attributes("img", ["src", "alt", "title", "loading"]);
    builder.add_tag_attributes("code", ["class"]);
    builder.clean(html).to_string()
}

/// Render Markdown through the NoteCraft engine.
pub fn render_html(content: &str) -> String {
    sanitize(&engine::render(content, Dialect::Markdown))
}

/// Render any supported note through the NoteCraft engine, choosing the
/// dialect from the file extension (`.md`, `.mdx`, `.json`).
pub fn render_document(content: &str, path: &str) -> String {
    sanitize(&engine::render_document(content, Kind::from_path(path)))
}

/// True when the engine can render this file as a note.
pub fn is_engine_document(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".md")
        || lower.ends_with(".markdown")
        || lower.ends_with(".mdx")
        || lower.ends_with(".json")
}

pub fn update_frontmatter(content: &str, key: &str, value: Value) -> AppResult<String> {
    let (mut fm, body) = parse_frontmatter(content);
    fm.data.insert(key.to_string(), value);
    // rebuild
    let yaml = serde_yaml::to_string(&fm.data).unwrap_or_default();
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&yaml);
    out.push_str("---\n\n");
    out.push_str(&body);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frontmatter_parses() {
        let c = "---\ntitle: Hello\ntags: [a, b]\n---\nBody here";
        let (fm, body) = parse_frontmatter(c);
        assert_eq!(fm.data.get("title").unwrap().as_str().unwrap(), "Hello");
        assert!(body.contains("Body here"));
    }
    #[test]
    fn links_extracted() {
        let body = "See [[Other]] and [md](note.md) and [[X#H]]";
        let ls = extract_links(body);
        assert!(ls.len() >= 3);
    }
    #[test]
    fn tags_inline() {
        let body = "Hello #foo and #bar/baz";
        let fm = Frontmatter::default();
        let t = extract_tags(body, &fm);
        assert!(t.contains(&"foo".to_string()));
    }
    #[test]
    fn tasks_parsed() {
        let body = "- [ ] todo\n- [x] done\n";
        let ts = extract_tasks(body);
        assert_eq!(ts.len(), 2);
        assert!(ts[1].done);
    }

    #[test]
    fn tasks_parse_due_dates_and_priorities() {
        let tasks = extract_tasks("- [ ] Submit report 📅 2026-11-02 priority: high");
        assert_eq!(tasks[0].due.as_deref(), Some("2026-11-02"));
        assert_eq!(tasks[0].priority.as_deref(), Some("high"));
    }

    #[test]
    fn render_keeps_classes_used_by_the_preview() {
        let html = render_html("See [[Other]]\n\n- [ ] task");
        assert!(html.contains("nc-wikilink"), "{html}");
        assert!(html.contains("nc-task"), "{html}");
    }

    #[test]
    fn render_document_dispatches_on_extension() {
        let json = render_document(r#"{"title":"Hello"}"#, "notes/data.json");
        assert!(json.contains("Hello"), "{json}");
        let mdx = render_document("# Hi\n\n<Chart />", "notes/page.mdx");
        assert!(mdx.contains("nc-mdx-block"), "{mdx}");
    }

    #[test]
    fn engine_extensions_are_recognised() {
        assert!(is_engine_document("a.mdx"));
        assert!(is_engine_document("a.json"));
        assert!(!is_engine_document("a.png"));
    }
}

pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
