use crate::error::AppError;
use crate::error::AppResult;
use crate::markdown::{parse, LinkRef};
use rusqlite::Connection;
use std::collections::HashSet;
use std::path::Path;

pub fn rebuild_for_note(conn: &Connection, note_rel: &str, content: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM links WHERE source=?1",
        rusqlite::params![note_rel],
    )
    .map_err(AppError::Sqlite)?;
    let parsed = parse(content);
    for l in &parsed.links {
        conn.execute(
            "INSERT OR IGNORE INTO links(source,target,kind) VALUES(?1,?2,?3)",
            rusqlite::params![note_rel, l.target, l.kind],
        )
        .map_err(AppError::Sqlite)?;
    }

    // Tags removed from a note must also have their counts recalculated;
    // otherwise a tag keeps its old count after the last mention is deleted.
    let mut affected_tags: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn
            .prepare("SELECT tag FROM note_tags WHERE note_path=?1")
            .map_err(AppError::Sqlite)?;
        let rows = stmt
            .query_map(rusqlite::params![note_rel], |r| r.get::<_, String>(0))
            .map_err(AppError::Sqlite)?;
        for tag in rows.flatten() {
            affected_tags.insert(tag);
        }
    }
    conn.execute(
        "DELETE FROM note_tags WHERE note_path=?1",
        rusqlite::params![note_rel],
    )
    .map_err(AppError::Sqlite)?;
    for t in &parsed.tags {
        conn.execute(
            "INSERT OR IGNORE INTO note_tags(note_path,tag) VALUES(?1,?2)",
            rusqlite::params![note_rel, t],
        )
        .map_err(AppError::Sqlite)?;
        conn.execute(
            "INSERT OR IGNORE INTO tags(name,count) VALUES(?1,0)",
            rusqlite::params![t],
        )
        .map_err(AppError::Sqlite)?;
        affected_tags.insert(t.clone());
    }
    for t in &affected_tags {
        conn.execute(
            "UPDATE tags SET count=(SELECT COUNT(*) FROM note_tags WHERE note_tags.tag=?1) WHERE name=?1",
            rusqlite::params![t],
        )
        .map_err(AppError::Sqlite)?;
        conn.execute(
            "DELETE FROM tags WHERE name=?1 AND count=0",
            rusqlite::params![t],
        )
        .map_err(AppError::Sqlite)?;
    }
    conn.execute(
        "DELETE FROM tasks WHERE note_path=?1",
        rusqlite::params![note_rel],
    )
    .map_err(AppError::Sqlite)?;
    for t in &parsed.tasks {
        conn.execute(
            "INSERT INTO tasks(note_path,line,text,done,due,priority) VALUES(?1,?2,?3,?4,?5,?6)",
            rusqlite::params![
                note_rel,
                t.line as i64,
                t.text,
                if t.done { 1 } else { 0 },
                t.due,
                t.priority
            ],
        )
        .map_err(AppError::Sqlite)?;
    }
    conn.execute(
        "DELETE FROM properties WHERE note_path=?1",
        rusqlite::params![note_rel],
    )
    .map_err(AppError::Sqlite)?;
    for (k, v) in &parsed.frontmatter.data {
        let (val, ty) = value_to_prop(v);
        conn.execute(
            "INSERT OR REPLACE INTO properties(note_path,key,value,type) VALUES(?1,?2,?3,?4)",
            rusqlite::params![note_rel, k, val, ty],
        )
        .map_err(AppError::Sqlite)?;
    }
    Ok(())
}

fn value_to_prop(v: &serde_json::Value) -> (String, String) {
    match v {
        serde_json::Value::Bool(b) => (b.to_string(), "checkbox".into()),
        serde_json::Value::Number(n) => (n.to_string(), "number".into()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            (v.to_string(), "list".into())
        }
        _ => (
            v.as_str().unwrap_or(&v.to_string()).to_string(),
            "text".into(),
        ),
    }
}

pub fn backlinks(conn: &Connection, target: &str) -> AppResult<Vec<String>> {
    // Links can be written as `[[Note]]`, `[[folder/Note]]` or `[[Note.md]]`.
    // Match every spelling instead of only the exact vault path.
    let no_ext = target.trim_end_matches(".md");
    let base = no_ext.rsplit('/').next().unwrap_or(no_ext);
    let mut stmt = conn
        .prepare("SELECT DISTINCT source FROM links WHERE target IN (?1,?2,?3,?4,?5)")
        .map_err(AppError::Sqlite)?;
    let rows = stmt
        .query_map(
            rusqlite::params![
                target,
                no_ext,
                format!("{}.md", no_ext),
                base,
                format!("{}.md", base)
            ],
            |r| r.get::<_, String>(0),
        )
        .map_err(AppError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        if let Ok(s) = r {
            out.push(s);
        }
    }
    Ok(out)
}

pub fn outgoing(conn: &Connection, source: &str) -> AppResult<Vec<LinkRef>> {
    let mut stmt = conn
        .prepare("SELECT target, kind FROM links WHERE source=?1")
        .map_err(AppError::Sqlite)?;
    let rows = stmt
        .query_map(rusqlite::params![source], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(AppError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        if let Ok((t, k)) = r {
            out.push(LinkRef {
                target: t,
                alias: None,
                heading: None,
                block: None,
                line: 0,
                kind: k,
            });
        }
    }
    Ok(out)
}

pub fn unresolved(conn: &Connection, vault_files: &[String]) -> AppResult<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT target FROM links")
        .map_err(AppError::Sqlite)?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(AppError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        if let Ok(t) = r {
            let tnorm = t.trim_end_matches(".md").to_string();
            let exists = vault_files.iter().any(|f| {
                f.trim_end_matches(".md") == tnorm
                    || f.ends_with(&format!("/{}.md", tnorm))
                    || f == &t
            });
            if !exists {
                out.push(t);
            }
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

pub fn unlinked_mentions(conn: &Connection, note_name: &str) -> AppResult<Vec<String>> {
    let pattern = format!("%{}%", note_name);
    let mut stmt = conn
        .prepare("SELECT DISTINCT source FROM links WHERE target LIKE ?1")
        .map_err(AppError::Sqlite)?;
    let rows = stmt
        .query_map(rusqlite::params![pattern], |r| r.get::<_, String>(0))
        .map_err(AppError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        if let Ok(s) = r {
            out.push(s);
        }
    }
    Ok(out)
}

/// Replace `[[Old]]`-style references (any spelling: with folder, `.md`,
/// heading or alias) with the new target, preserving heading/alias suffixes.
/// Returns the new content and how many links were rewritten.
pub fn rename_wikilink_targets(
    content: &str,
    from_no_ext: &str,
    from_stem: &str,
    to_display: &str,
) -> (String, usize) {
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    let mut count = 0usize;
    let out = re.replace_all(content, |caps: &regex::Captures| {
        let inner = &caps[1];
        let split_at = inner.find(['#', '|']).unwrap_or(inner.len());
        let target = inner[..split_at].trim();
        let suffix = &inner[split_at..];
        let target_no_ext = target.trim_end_matches(".md");
        let matches = target_no_ext == from_no_ext
            || target_no_ext == from_stem
            || (from_stem.contains('/')
                && target_no_ext == from_stem.rsplit('/').next().unwrap_or(target_no_ext));
        if !matches {
            return caps[0].to_string();
        }
        count += 1;
        format!("[[{}{}]]", to_display, suffix)
    });
    (out.into_owned(), count)
}

/// Rename an inline/frontmatter tag. Handles `#tag` in the body (including
/// nested `#parent/child` prefixes), `tags:` scalar lines and `- item` tag list
/// entries. Returns the new content and the number of replacements.
pub fn rename_tag_in_content(content: &str, from: &str, to: &str) -> (String, usize) {
    let mut count = 0usize;
    // Inline #tag; a parent rename also rewrites its nested children.
    let inline = regex::Regex::new(&format!(
        r"(?m)(^|[\s(])#{}(?=[\s\]#/,]|$)",
        regex::escape(from)
    ))
    .unwrap();
    let mut out = inline
        .replace_all(content, |caps: &regex::Captures| {
            count += 1;
            format!("{}#{}", &caps[1], to)
        })
        .into_owned();
    // `tags: one, two` scalar / flow-list values: keep everything before the
    // tag intact so `tags: [work, old]` becomes `tags: [work, archived]`.
    let scalar = regex::Regex::new(&format!(
        r"(?m)^(tags:\s*[^\n#]*?)\b{}\b",
        regex::escape(from)
    ))
    .unwrap();
    out = scalar
        .replace_all(&out, |caps: &regex::Captures| {
            count += 1;
            format!("{}{}", &caps[1], to)
        })
        .into_owned();
    // `- tag` entries inside a block list.
    let list = regex::Regex::new(&format!(r"(?m)^(\s*-\s*){}\s*$", regex::escape(from))).unwrap();
    let out = list.replace_all(&out, |caps: &regex::Captures| {
        count += 1;
        format!("{}{}", &caps[1], to)
    });
    (out.into_owned(), count)
}

#[cfg(test)]
mod rename_tests {
    use super::*;

    #[test]
    fn renames_every_wikilink_spelling() {
        let content = "See [[Old]] and [[folder/Old.md]] and [[Old#Heading|Alias]] and [[New]].";
        let (out, n) = rename_wikilink_targets(content, "folder/Old", "Old", "New");
        assert_eq!(n, 3);
        assert!(
            out.contains("See [[New]] and [[New]] and [[New#Heading|Alias]] and [[New]]."),
            "unexpected: {out}"
        );
        assert!(!out.contains("[[Old"));
    }

    #[test]
    fn renames_inline_and_frontmatter_tags() {
        let content = "---\ntags: [work, old]\n---\n\n#old and #old/child done\n";
        let (out, n) = rename_tag_in_content(content, "old", "archived");
        assert!(n >= 3, "expected at least 3 replacements, got {n}: {out}");
        assert!(out.contains("#archived"));
        assert!(out.contains("#archived/child"));
        assert!(!out.contains("old"));
    }

    #[test]
    fn renames_nested_tag_parent_prefix() {
        let (out, n) = rename_tag_in_content("topic #project/alpha here", "project/alpha", "alpha");
        assert_eq!(n, 1);
        assert_eq!(out, "topic #alpha here");
    }

    #[test]
    fn does_not_touch_lookalike_tags() {
        let (out, n) = rename_tag_in_content("#older and #bold", "old", "new");
        assert_eq!(n, 0);
        assert_eq!(out, "#older and #bold");
    }
}
