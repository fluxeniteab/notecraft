use crate::error::AppResult;
use crate::markdown::{parse, LinkRef};
use crate::error::AppError;
use rusqlite::Connection;
use std::path::Path;

pub fn rebuild_for_note(conn: &Connection, note_rel: &str, content: &str) -> AppResult<()> {
    conn.execute("DELETE FROM links WHERE source=?1", rusqlite::params![note_rel])
        .map_err(AppError::Sqlite)?;
    let parsed = parse(content);
    for l in &parsed.links {
        conn.execute(
            "INSERT OR IGNORE INTO links(source,target,kind) VALUES(?1,?2,?3)",
            rusqlite::params![note_rel, l.target, l.kind],
        )
        .map_err(AppError::Sqlite)?;
    }
    conn.execute("DELETE FROM note_tags WHERE note_path=?1", rusqlite::params![note_rel])
        .map_err(AppError::Sqlite)?;
    for t in &parsed.tags {
        conn.execute(
            "INSERT OR IGNORE INTO note_tags(note_path,tag) VALUES(?1,?2)",
            rusqlite::params![note_rel, t],
        )
        .map_err(AppError::Sqlite)?;
        conn.execute(
            "INSERT INTO tags(name,count) VALUES(?1,1) ON CONFLICT(name) DO UPDATE SET count=count+1",
            rusqlite::params![t],
        )
        .map_err(AppError::Sqlite)?;
    }
    conn.execute("DELETE FROM tasks WHERE note_path=?1", rusqlite::params![note_rel])
        .map_err(AppError::Sqlite)?;
    for t in &parsed.tasks {
        conn.execute(
            "INSERT INTO tasks(note_path,line,text,done) VALUES(?1,?2,?3,?4)",
            rusqlite::params![note_rel, t.line as i64, t.text, if t.done { 1 } else { 0 }],
        )
        .map_err(AppError::Sqlite)?;
    }
    conn.execute("DELETE FROM properties WHERE note_path=?1", rusqlite::params![note_rel])
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
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => (v.to_string(), "list".into()),
        _ => (v.as_str().unwrap_or(&v.to_string()).to_string(), "text".into()),
    }
}

pub fn backlinks(conn: &Connection, target: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT source FROM links WHERE target=?1 OR target=?2")
        .map_err(AppError::Sqlite)?;
    let base = target.trim_end_matches(".md");
    let alt = format!("{}.md", base);
    let rows = stmt
        .query_map(rusqlite::params![target, alt], |r| r.get::<_, String>(0))
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
    let mut stmt = conn.prepare("SELECT DISTINCT target FROM links").map_err(AppError::Sqlite)?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(AppError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        if let Ok(t) = r {
            let tnorm = t.trim_end_matches(".md").to_string();
            let exists = vault_files.iter().any(|f| {
                f.trim_end_matches(".md") == tnorm || f.ends_with(&format!("/{}.md", tnorm)) || f == &t
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
    let rows = stmt.query_map(rusqlite::params![pattern], |r| r.get::<_, String>(0)).map_err(AppError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        if let Ok(s) = r {
            out.push(s);
        }
    }
    Ok(out)
}
