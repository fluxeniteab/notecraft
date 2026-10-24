use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub imported: usize,
    pub broken_links: Vec<String>,
}

pub fn import_vault(src: &str, dest: &str) -> AppResult<ImportReport> {
    let src = PathBuf::from(src);
    let dest = PathBuf::from(dest);
    if !src.is_dir() {
        return Err(AppError::Invalid("source not a directory".into()));
    }
    std::fs::create_dir_all(&dest).map_err(AppError::Io)?;
    let mut imported = 0usize;
    let mut broken = Vec::new();
    for entry in walkdir::WalkDir::new(&src) {
        let entry = entry.map_err(AppError::Io)?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&src)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let target = dest.join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        std::fs::copy(entry.path(), &target).map_err(AppError::Io)?;
        imported += 1;
        if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let parsed = crate::markdown::parse(&content);
                // check links resolve within dest later; collect targets for report
                for l in &parsed.links {
                    if l.target.ends_with(".md") {
                        let candidate = dest.join(&l.target);
                        if !candidate.exists() {
                            broken.push(l.target.clone());
                        }
                    }
                }
            }
        }
    }
    broken.sort();
    broken.dedup();
    Ok(ImportReport {
        imported,
        broken_links: broken,
    })
}

pub fn backup(state: &State<AppState>, dest: &str) -> AppResult<()> {
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone().ok_or(AppError::NoVault)?
    };
    let dest = PathBuf::from(dest);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let file = std::fs::File::create(&dest).map_err(AppError::Io)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry.map_err(AppError::Io)?;
        let rel = entry
            .path()
            .strip_prefix(&root)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }
        if entry.file_type().is_dir() {
            zip.add_directory(rel_str, opts)
                .map_err(|e| AppError::Other(e.to_string()))?;
        } else {
            zip.start_file(rel_str, opts)
                .map_err(|e| AppError::Other(e.to_string()))?;
            let data = std::fs::read(entry.path()).map_err(AppError::Io)?;
            use std::io::Write;
            zip.write_all(&data)
                .map_err(|e| AppError::Other(e.to_string()))?;
        }
    }
    zip.finish().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

pub fn restore_backup(src: &str, dest: &str) -> AppResult<()> {
    let src = PathBuf::from(src);
    let dest = PathBuf::from(dest);
    std::fs::create_dir_all(&dest).map_err(AppError::Io)?;
    let file = std::fs::File::open(&src).map_err(AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Other(e.to_string()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let outpath = dest.join(entry.name());
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(AppError::Io)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).map_err(AppError::Io)?;
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(AppError::Io)?;
            use std::io::copy;
            copy(&mut entry, &mut outfile).map_err(AppError::Io)?;
        }
    }
    Ok(())
}
