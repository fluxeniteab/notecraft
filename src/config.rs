// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub last_vault: Option<PathBuf>,
    pub autosave_debounce_ms: u64,
    pub sidebar_width: f32,
    pub show_line_numbers: bool,
    pub templates_subdir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_vault: None,
            autosave_debounce_ms: 1200,
            sidebar_width: 260.0,
            show_line_numbers: true,
            templates_subdir: "templates".to_string(),
        }
    }
}

pub fn settings_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("NoteCraft").join("settings.json")
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    if !path.exists() {
        return Settings::default();
    }
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    // safe write
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn is_vault_valid(path: &Path) -> bool {
    path.is_dir()
}
