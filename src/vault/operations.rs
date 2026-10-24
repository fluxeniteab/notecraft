// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::error::{NoteCraftError, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn canonicalize_vault(root: &Path) -> Result<PathBuf> {
    let c = root.canonicalize()?;
    if !c.is_dir() {
        return Err(NoteCraftError::InvalidVault(
            c.to_string_lossy().to_string(),
        ));
    }
    Ok(c)
}

pub fn ensure_inside(root: &Path, target: &Path) -> Result<PathBuf> {
    // root should be canonical vault root
    let canon_root = root
        .canonicalize()
        .map_err(|_| NoteCraftError::InvalidVault(root.to_string_lossy().to_string()))?;
    // target may not exist yet; canonicalize parent
    let parent = target.parent().unwrap_or(target);
    let canon_parent = if parent.exists() {
        parent.canonicalize()?
    } else {
        // walk up
        parent.to_path_buf()
    };
    // Build candidate by joining file name to canonical parent if exists
    let candidate = if target.exists() {
        target.canonicalize()?
    } else {
        canon_parent.join(target.file_name().unwrap_or_default())
    };
    if !candidate.starts_with(&canon_root) {
        return Err(NoteCraftError::PathTraversal(
            target.to_string_lossy().to_string(),
        ));
    }
    Ok(candidate)
}

pub fn create_note(root: &Path, dir: &Path, name: &str) -> Result<PathBuf> {
    let name = sanitize_name(name);
    let mut path = dir.join(&name);
    if !name.to_ascii_lowercase().ends_with(".md")
        && !name.to_ascii_lowercase().ends_with(".markdown")
    {
        path = dir.join(format!("{}.md", name));
    }
    // collision detection
    if path.exists() {
        return Err(NoteCraftError::Other(
            "A file with that name already exists".to_string(),
        ));
    }
    ensure_inside(root, &path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, "")?;
    Ok(path)
}

pub fn create_dir(root: &Path, parent: &Path, name: &str) -> Result<PathBuf> {
    let name = sanitize_name(name);
    let path = parent.join(&name);
    if path.exists() {
        return Err(NoteCraftError::Other(
            "A folder with that name already exists".to_string(),
        ));
    }
    ensure_inside(root, &path)?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn rename_path(root: &Path, from: &Path, to_name: &str) -> Result<PathBuf> {
    let to_name = sanitize_name(to_name);
    let parent = from
        .parent()
        .ok_or_else(|| NoteCraftError::Other("No parent".to_string()))?;
    let new_path = parent.join(&to_name);
    if new_path.exists() && new_path != from {
        return Err(NoteCraftError::Other("Target already exists".to_string()));
    }
    ensure_inside(root, from)?;
    ensure_inside(root, &new_path)?;
    fs::rename(from, &new_path)?;
    Ok(new_path)
}

pub fn delete_path(root: &Path, target: &Path) -> Result<()> {
    ensure_inside(root, target)?;
    if target.is_dir() {
        fs::remove_dir_all(target)?;
    } else {
        fs::remove_file(target)?;
    }
    Ok(())
}

pub fn sanitize_name(name: &str) -> String {
    let trimmed = name.trim();
    // remove path separators
    trimmed
        .replace(['/', '\\'], "")
        .trim_matches('.')
        .to_string()
}

pub fn read_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub fn read_file_strict(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    String::from_utf8(bytes).map_err(|_| NoteCraftError::Other("Invalid UTF-8".to_string()))
}
