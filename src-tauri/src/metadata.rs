use crate::error::AppResult;
use rusqlite::Connection;

pub fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS tags (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT UNIQUE NOT NULL, count INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS note_tags (note_path TEXT NOT NULL, tag TEXT NOT NULL, PRIMARY KEY(note_path, tag));
        CREATE TABLE IF NOT EXISTS properties (note_path TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, type TEXT NOT NULL DEFAULT 'text', PRIMARY KEY(note_path, key));
        CREATE TABLE IF NOT EXISTS links (source TEXT NOT NULL, target TEXT NOT NULL, kind TEXT NOT NULL DEFAULT 'wiki', PRIMARY KEY(source, target, kind));
        CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY AUTOINCREMENT, note_path TEXT NOT NULL, line INTEGER NOT NULL, text TEXT NOT NULL, done INTEGER NOT NULL DEFAULT 0, due TEXT, priority TEXT);
        CREATE TABLE IF NOT EXISTS collections (id TEXT PRIMARY KEY, name TEXT NOT NULL, config TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS workspaces (id TEXT PRIMARY KEY, name TEXT NOT NULL, data TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS snapshots (id TEXT PRIMARY KEY, note_path TEXT NOT NULL, created_at TEXT NOT NULL, hash TEXT NOT NULL, size INTEGER NOT NULL, label TEXT, data_path TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS automations (id TEXT PRIMARY KEY, name TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1, config TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS automation_runs (id TEXT PRIMARY KEY, automation_id TEXT NOT NULL, started_at TEXT NOT NULL, status TEXT NOT NULL, log TEXT);
        CREATE TABLE IF NOT EXISTS plugins (id TEXT PRIMARY KEY, manifest TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 0, installed_at TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS sync_state (path TEXT PRIMARY KEY, hash TEXT NOT NULL, mtime INTEGER NOT NULL, remote_hash TEXT, status TEXT NOT NULL DEFAULT 'synced');
        CREATE TABLE IF NOT EXISTS search_history (id INTEGER PRIMARY KEY AUTOINCREMENT, query TEXT NOT NULL, created_at TEXT NOT NULL);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag ON note_tags(tag);
        CREATE INDEX IF NOT EXISTS idx_links_target ON links(target);
        CREATE INDEX IF NOT EXISTS idx_tasks_note ON tasks(note_path);
        CREATE INDEX IF NOT EXISTS idx_snapshots_note ON snapshots(note_path);
        "#,
    )
    .map_err(crate::error::AppError::Sqlite)?;
    Ok(())
}

pub fn set_kv(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO kv(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        rusqlite::params![key, value],
    )
    .map_err(crate::error::AppError::Sqlite)?;
    Ok(())
}

pub fn get_kv(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM kv WHERE key=?1", rusqlite::params![key], |r| r.get(0))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrate_and_kv() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        set_kv(&conn, "k", "v").unwrap();
        assert_eq!(get_kv(&conn, "k"), Some("v".into()));
    }
}
