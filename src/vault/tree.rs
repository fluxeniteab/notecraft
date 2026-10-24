// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::error::{NoteCraftError, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
}

impl FileNode {
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self {
            path,
            name,
            is_dir,
            children: Vec::new(),
            expanded: false,
        }
    }
}

pub fn is_markdown(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let e = ext.to_ascii_lowercase();
            e == "md" || e == "markdown"
        }
        None => false,
    }
}

pub fn build_tree(root: &Path) -> Result<FileNode> {
    if !root.is_dir() {
        return Err(NoteCraftError::InvalidVault(
            root.to_string_lossy().to_string(),
        ));
    }
    let mut node = FileNode::new(root.to_path_buf(), true);
    node.expanded = true;
    populate_children(&mut node)?;
    Ok(node)
}

pub fn populate_children(node: &mut FileNode) -> Result<()> {
    if !node.is_dir {
        return Ok(());
    }
    node.children.clear();
    let mut entries: Vec<_> = fs::read_dir(&node.path)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    // directories first
    entries.sort_by(|a, b| {
        let ad = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let bd = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        bd.cmp(&ad)
    });
    for entry in entries {
        let path = entry.path();
        let ft = entry.file_type()?;
        let is_dir = ft.is_dir();
        if is_dir {
            let mut child = FileNode::new(path, true);
            // lazy: don't populate until expanded; but for simplicity populate one level
            let _ = populate_children(&mut child);
            // collapse by default except root
            child.expanded = false;
            node.children.push(child);
        } else if is_markdown(&path) {
            node.children.push(FileNode::new(path, false));
        }
    }
    Ok(())
}

pub fn refresh_node(node: &mut FileNode, target: &Path) -> Result<()> {
    if node.path == target {
        if node.is_dir {
            populate_children(node)?;
        }
        return Ok(());
    }
    for child in &mut node.children {
        if target.starts_with(&child.path) {
            refresh_node(child, target)?;
        }
    }
    Ok(())
}

pub fn find_node<'a>(node: &'a FileNode, path: &Path) -> Option<&'a FileNode> {
    if node.path == path {
        return Some(node);
    }
    for c in &node.children {
        if let Some(found) = find_node(c, path) {
            return Some(found);
        }
    }
    None
}

pub fn find_node_mut<'a>(node: &'a mut FileNode, path: &Path) -> Option<&'a mut FileNode> {
    if node.path == path {
        return Some(node);
    }
    for c in &mut node.children {
        if path.starts_with(&c.path) {
            if let Some(found) = find_node_mut(c, path) {
                return Some(found);
            }
        }
    }
    None
}
