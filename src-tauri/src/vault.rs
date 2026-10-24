use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String, // vault-relative posix
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub extension: String,
}

pub fn validate_rel(rel: &str) -> AppResult<PathBuf> {
    if rel.is_empty() {
        return Ok(PathBuf::new());
    }
    // normalize separators
    let rel = rel.replace('\\', "/");
    if rel.starts_with('/') || rel.contains("..") {
        return Err(AppError::Path("invalid relative path".into()));
    }
    // reject drive letters / absolute
    let p = PathBuf::from(&rel);
    if p.is_absolute() {
        return Err(AppError::Path("absolute path not allowed".into()));
    }
    for comp in p.components() {
        match comp {
            std::path::Component::Normal(_) => {}
            _ => return Err(AppError::Path("invalid path component".into())),
        }
    }
    Ok(p)
}

pub fn join_safe(root: &Path, rel: &str) -> AppResult<PathBuf> {
    let relp = validate_rel(rel)?;
    let full = root.join(relp);
    // ensure canonical stays under root if root exists
    Ok(full)
}

pub fn to_rel(root: &Path, full: &Path) -> String {
    full.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

pub fn ensure_excluded(name: &str, excluded: &[String]) -> bool {
    excluded.iter().any(|e| e == name)
}

pub fn list_dir(
    root: &Path,
    rel: &str,
    excluded: &[String],
    show_hidden: bool,
) -> AppResult<Vec<FileEntry>> {
    let dir = join_safe(root, rel)?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        if ensure_excluded(&name, excluded) {
            continue;
        }
        let meta = entry.metadata().map_err(AppError::Io)?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let ext = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        out.push(FileEntry {
            path: to_rel(root, &entry.path()),
            name,
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified,
            extension: ext,
        });
    }
    out.sort_by(|a, b| {
        // dirs first, then name
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(out)
}

pub fn read_file(root: &Path, rel: &str) -> AppResult<String> {
    let p = join_safe(root, rel)?;
    std::fs::read_to_string(&p).map_err(AppError::Io)
}

pub fn write_file_atomic(root: &Path, rel: &str, content: &str) -> AppResult<()> {
    let p = join_safe(root, rel)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    // atomic: write tmp then rename
    let tmp = p.with_extension("tmp_notecraft");
    std::fs::write(&tmp, content).map_err(AppError::Io)?;
    // fsync
    if let Ok(f) = std::fs::File::open(&tmp) {
        let _ = f.sync_all();
    }
    std::fs::rename(&tmp, &p).map_err(AppError::Io)?;
    Ok(())
}

pub fn create_file(root: &Path, rel: &str, content: &str) -> AppResult<()> {
    let p = join_safe(root, rel)?;
    if p.exists() {
        return Err(AppError::Conflict("file already exists".into()));
    }
    write_file_atomic(root, rel, content)
}

pub fn delete_path(root: &Path, rel: &str, trash: bool) -> AppResult<()> {
    let p = join_safe(root, rel)?;
    if !p.exists() {
        return Err(AppError::NotFound(rel.into()));
    }
    if trash {
        let trash_dir = root.join(".notecraft").join("trash");
        std::fs::create_dir_all(&trash_dir).map_err(AppError::Io)?;
        let name = p.file_name().unwrap_or_default();
        let dest = trash_dir.join(name);
        let dest = if dest.exists() {
            trash_dir.join(format!(
                "{}_{}",
                chrono::Utc::now().timestamp_millis(),
                name.to_string_lossy()
            ))
        } else {
            dest
        };
        std::fs::rename(&p, &dest).map_err(AppError::Io)?;
    } else {
        if p.is_dir() {
            std::fs::remove_dir_all(&p).map_err(AppError::Io)?;
        } else {
            std::fs::remove_file(&p).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

pub fn rename_path(root: &Path, from: &str, to: &str) -> AppResult<()> {
    let a = join_safe(root, from)?;
    let b = join_safe(root, to)?;
    if !a.exists() {
        return Err(AppError::NotFound(from.into()));
    }
    if b.exists() {
        return Err(AppError::Conflict("target exists".into()));
    }
    if let Some(parent) = b.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    std::fs::rename(&a, &b).map_err(AppError::Io)?;
    Ok(())
}

pub fn move_path(root: &Path, from: &str, to_dir: &str) -> AppResult<String> {
    let a = join_safe(root, from)?;
    let name = a
        .file_name()
        .ok_or_else(|| AppError::Invalid("no name".into()))?;
    let dest_rel = if to_dir.is_empty() {
        name.to_string_lossy().to_string()
    } else {
        format!(
            "{}/{}",
            to_dir.trim_end_matches('/'),
            name.to_string_lossy()
        )
    };
    rename_path(root, from, &dest_rel)?;
    Ok(dest_rel)
}

pub fn create_dir(root: &Path, rel: &str) -> AppResult<()> {
    let p = join_safe(root, rel)?;
    std::fs::create_dir_all(&p).map_err(AppError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validate_rel_rejects_traversal() {
        assert!(validate_rel("../secret").is_err());
        assert!(validate_rel("/abs").is_err());
        assert!(validate_rel("a/../b").is_err());
    }
    #[test]
    fn validate_rel_ok() {
        assert!(validate_rel("notes/hello.md").is_ok());
        assert!(validate_rel("").is_ok());
    }
    #[test]
    fn join_safe_stays() {
        let root = std::path::Path::new("/tmp/v");
        let p = join_safe(root, "a/b.md").unwrap();
        assert_eq!(p, root.join("a/b.md"));
    }
}

pub fn walk_markdown(root: &Path, excluded: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        if e.depth() > 0 && name.starts_with('.') {
            return false;
        }
        if excluded.iter().any(|x| x == name.as_ref()) {
            return false;
        }
        true
    }) {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                let ext = e
                    .path()
                    .extension()
                    .map(|x| x.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if ext == "md" || ext == "markdown" {
                    out.push(e.into_path());
                }
            }
        }
    }
    out
}
