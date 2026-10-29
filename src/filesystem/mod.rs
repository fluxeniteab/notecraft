//! Vault access.
//!
//! Every path the UI hands us is vault-relative and validated here before it
//! touches the disk. Absolute paths, `..` components and symlinks that escape
//! the root are rejected, and writes go through a temporary file + rename so a
//! crash mid-save cannot leave a truncated note behind.

use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("{0}")]
    Path(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, FsError>;

/// A file or folder inside the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Vault-relative path with `/` separators.
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub extension: String,
}

/// Reject anything that is not a plain relative path.
pub fn validate_rel(rel: &str) -> Result<PathBuf> {
    if rel.is_empty() {
        return Ok(PathBuf::new());
    }
    let normalised = rel.replace('\\', "/");
    if normalised.starts_with('/') || normalised.contains('\0') {
        return Err(FsError::Path("invalid relative path".into()));
    }
    let p = PathBuf::from(&normalised);
    if p.is_absolute() {
        return Err(FsError::Path("absolute path not allowed".into()));
    }
    for comp in p.components() {
        match comp {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => return Err(FsError::Path("invalid path component".into())),
        }
    }
    Ok(p)
}

/// Resolve `rel` against `root`, refusing anything that escapes it.
pub fn join_safe(root: &Path, rel: &str) -> Result<PathBuf> {
    let relp = validate_rel(rel)?;
    let full = root.join(relp);
    if let (Ok(canon_root), Ok(canon_full)) = (root.canonicalize(), full.canonicalize()) {
        if !canon_full.starts_with(&canon_root) {
            return Err(FsError::Path("path escapes the vault".into()));
        }
    }
    Ok(full)
}

pub fn to_rel(root: &Path, full: &Path) -> String {
    full.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

fn modified_secs(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Read a single directory level.
///
/// The explorer renders from the cached full scan; this stays available for
/// callers that need a fresh, validated listing of one folder.
#[allow(dead_code)]
pub fn list_dir(
    root: &Path,
    rel: &str,
    excluded: &[String],
    show_hidden: bool,
    sort: &str,
    group_dirs: bool,
) -> Result<Vec<FileEntry>> {
    let dir = join_safe(root, rel)?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        if excluded.iter().any(|e| e == &name) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let extension = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        out.push(FileEntry {
            path: to_rel(root, &entry.path()),
            name,
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified: modified_secs(&meta),
            extension,
        });
    }

    out.sort_by(|a, b| {
        if group_dirs && a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        match sort {
            "modified" => b.modified.cmp(&a.modified),
            "size" => b.size.cmp(&a.size),
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    Ok(out)
}

/// Walk the whole vault, skipping excluded folders and hidden entries.
pub fn scan_vault(root: &Path, excluded: &[String], show_hidden: bool) -> Vec<FileEntry> {
    let mut out = Vec::new();
    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                return false;
            }
            !excluded.iter().any(|x| x == &name)
        });
    for entry in walker.flatten() {
        if entry.depth() == 0 {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        out.push(FileEntry {
            path: to_rel(root, entry.path()),
            extension: Path::new(&name)
                .extension()
                .map(|e| e.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default(),
            name,
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified: modified_secs(&meta),
        });
    }
    out
}

/// Read a document as UTF-8, reporting invalid encodings instead of guessing.
pub fn read_text(root: &Path, rel: &str) -> Result<String> {
    let path = join_safe(root, rel)?;
    let bytes = std::fs::read(&path)?;
    String::from_utf8(bytes).map_err(|_| {
        FsError::Path(format!(
            "{rel} is not valid UTF-8; it was not opened to avoid corrupting it"
        ))
    })
}

/// Atomically replace a document's contents.
pub fn write_text_atomic(root: &Path, rel: &str, content: &str) -> Result<()> {
    let path = join_safe(root, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.nc-tmp",
        path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default()
    ));
    std::fs::write(&tmp, content.as_bytes())?;
    // `rename` replaces the destination on Windows and POSIX alike.
    if let Err(err) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(err.into());
    }
    Ok(())
}

pub fn create_file(root: &Path, rel: &str, content: &str) -> Result<()> {
    let path = join_safe(root, rel)?;
    if path.exists() {
        return Err(FsError::Conflict(format!("{rel} already exists")));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content.as_bytes())?;
    Ok(())
}

pub fn create_dir(root: &Path, rel: &str) -> Result<()> {
    let path = join_safe(root, rel)?;
    if path.exists() {
        return Err(FsError::Conflict(format!("{rel} already exists")));
    }
    std::fs::create_dir_all(&path)?;
    Ok(())
}

/// Rename inside the vault, refusing to overwrite an existing path.
pub fn rename(root: &Path, from: &str, to: &str) -> Result<()> {
    let src = join_safe(root, from)?;
    let dst = join_safe(root, to)?;
    if !src.exists() {
        return Err(FsError::Path(format!("{from} does not exist")));
    }
    if dst.exists() {
        return Err(FsError::Conflict(format!("{to} already exists")));
    }
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&src, &dst)?;
    Ok(())
}

/// Move a document or folder to the vault trash instead of deleting it.
pub fn trash(root: &Path, rel: &str) -> Result<String> {
    let src = join_safe(root, rel)?;
    if !src.exists() {
        return Err(FsError::Path(format!("{rel} does not exist")));
    }
    let trash_dir = root.join(".notecraft").join("trash");
    std::fs::create_dir_all(&trash_dir)?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let name = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "item".to_string());
    let target = trash_dir.join(format!("{stamp}-{name}"));
    std::fs::rename(&src, &target)?;
    Ok(to_rel(root, &target))
}

/// Snapshot a document before a destructive edit.
pub fn snapshot(root: &Path, rel: &str, retention: usize) -> Result<Option<PathBuf>> {
    let src = join_safe(root, rel)?;
    if !src.is_file() {
        return Ok(None);
    }
    let history = root.join(".notecraft").join("history");
    std::fs::create_dir_all(&history)?;
    let safe_name = rel.replace('/', "__");
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S-%3f");
    let target = history.join(format!("{safe_name}.{stamp}.snap"));
    std::fs::copy(&src, &target)?;

    // Keep each document's history bounded.
    if retention > 0 {
        let prefix = format!("{safe_name}.");
        let mut snaps: Vec<_> = std::fs::read_dir(&history)?
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(prefix.as_str()))
            .map(|e| e.path())
            .collect();
        if snaps.len() > retention {
            snaps.sort();
            for old in snaps.drain(..snaps.len() - retention) {
                let _ = std::fs::remove_file(old);
            }
        }
    }
    Ok(Some(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "notecraft-fs-{tag}-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rejects_traversal_and_absolute_paths() {
        assert!(validate_rel("../secrets").is_err());
        assert!(validate_rel("/etc/passwd").is_err());
        assert!(validate_rel("C:/Windows").is_err());
        assert!(validate_rel("notes/a.md").is_ok());
    }

    #[test]
    fn atomic_write_and_read_roundtrip() {
        let root = temp_root("write");
        write_text_atomic(&root, "a/b.md", "# hi\n").unwrap();
        assert_eq!(read_text(&root, "a/b.md").unwrap(), "# hi\n");
        write_text_atomic(&root, "a/b.md", "# bye\n").unwrap();
        assert_eq!(read_text(&root, "a/b.md").unwrap(), "# bye\n");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn create_refuses_to_clobber() {
        let root = temp_root("create");
        create_file(&root, "a.md", "one").unwrap();
        assert!(matches!(
            create_file(&root, "a.md", "two"),
            Err(FsError::Conflict(_))
        ));
        assert_eq!(read_text(&root, "a.md").unwrap(), "one");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn trash_moves_instead_of_deleting() {
        let root = temp_root("trash");
        create_file(&root, "keep.md", "data").unwrap();
        let moved = trash(&root, "keep.md").unwrap();
        assert!(moved.starts_with(".notecraft/trash/"));
        assert!(!root.join("keep.md").exists());
        assert!(root.join(&moved).exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn refused_utf8_is_reported() {
        let root = temp_root("utf8");
        std::fs::write(root.join("bad.md"), [0xff, 0xfe, 0x00]).unwrap();
        assert!(read_text(&root, "bad.md").is_err());
        // The original bytes are untouched.
        assert_eq!(
            std::fs::read(root.join("bad.md")).unwrap(),
            vec![0xff, 0xfe, 0x00]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_skips_excluded_and_hidden() {
        let root = temp_root("scan");
        create_file(&root, "a.md", "x").unwrap();
        create_file(&root, "node_modules/skip.md", "x").unwrap();
        create_file(&root, ".hidden.md", "x").unwrap();
        let found = scan_vault(&root, &["node_modules".to_string()], false);
        let names: Vec<_> = found.iter().map(|e| e.path.as_str()).collect();
        assert!(names.contains(&"a.md"));
        assert!(!names.iter().any(|p| p.contains("skip.md")));
        assert!(!names.iter().any(|p| p.contains("hidden")));
        let _ = std::fs::remove_dir_all(&root);
    }
}
