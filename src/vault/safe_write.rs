// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::error::{NoteCraftError, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMeta {
    pub mtime: Option<SystemTime>,
    pub size: u64,
}

pub fn file_meta(path: &Path) -> Result<FileMeta> {
    let meta = fs::metadata(path)?;
    Ok(FileMeta {
        mtime: meta.modified().ok(),
        size: meta.len(),
    })
}

pub fn content_hash(content: &str) -> u64 {
    // simple FNV-like hash for change detection
    let mut h: u64 = 0xcbf29ce484222325;
    for b in content.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn atomic_write(path: &Path, content: &str, expected: Option<FileMeta>) -> Result<()> {
    // conflict check
    if path.exists() {
        if let Some(exp) = expected {
            if let Ok(cur) = file_meta(path) {
                if cur.mtime != exp.mtime || cur.size != exp.size {
                    return Err(NoteCraftError::Conflict);
                }
            }
        }
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("nc_tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(content.as_bytes())?;
        f.flush()?;
        f.sync_all().ok();
    }
    // atomic replace
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp, path)?;
    Ok(())
}
