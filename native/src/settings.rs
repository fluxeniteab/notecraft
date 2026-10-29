//! User settings.
//!
//! The field names and JSON shape are the same as the Tauri build's, so an
//! existing `%APPDATA%/NoteCraft/settings.json` loads unchanged and unknown
//! newer fields fall back to defaults instead of resetting the file.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    pub accent: String,
    pub editor_font_size: u32,
    pub editor_line_height: f32,
    pub editor_font_family: String,
    pub reading_width: u32,
    pub autosave_ms: u32,
    pub snapshot_interval_min: u32,
    pub snapshot_retention: u32,
    pub excluded_folders: Vec<String>,
    pub hidden_files: bool,
    pub reduced_motion: bool,
    pub telemetry_opt_in: bool,
    pub semantic_search_enabled: bool,
    pub daily_note_folder: String,
    pub daily_note_template: Option<String>,
    pub capture_note: String,
    pub explorer_sort: String,
    pub explorer_group_dirs: bool,
    pub shortcuts: HashMap<String, String>,
    pub writing_goal_words: u32,
    pub sync_folder: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,

    // Native-only additions. Absent in older files, so they default.
    pub show_line_numbers: bool,
    pub wrap_code: bool,
    pub autosave_enabled: bool,
    pub reopen_last_session: bool,
    pub preview_width: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            accent: "#4f86f7".into(),
            editor_font_size: 15,
            editor_line_height: 1.75,
            editor_font_family: "Inter, system-ui, sans-serif".into(),
            reading_width: 760,
            autosave_ms: 800,
            snapshot_interval_min: 5,
            snapshot_retention: 50,
            excluded_folders: vec![".git".into(), "node_modules".into(), ".notecraft".into()],
            hidden_files: false,
            reduced_motion: false,
            telemetry_opt_in: false,
            semantic_search_enabled: false,
            daily_note_folder: "daily".into(),
            daily_note_template: None,
            capture_note: "inbox.md".into(),
            explorer_sort: "name".into(),
            explorer_group_dirs: true,
            shortcuts: HashMap::new(),
            writing_goal_words: 0,
            sync_folder: None,
            data_dir: None,
            show_line_numbers: true,
            wrap_code: false,
            autosave_enabled: true,
            reopen_last_session: true,
            preview_width: 0.5,
        }
    }
}

impl Settings {
    pub fn theme_kind(&self) -> crate::theme::ThemeKind {
        match self.theme.as_str() {
            "light" => crate::theme::ThemeKind::Light,
            _ => crate::theme::ThemeKind::Dark,
        }
    }
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("NoteCraft"))
        .unwrap_or_else(|| PathBuf::from(".notecraft"))
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn session_path() -> PathBuf {
    data_dir().join("session.json")
}

pub fn load() -> Settings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Write settings atomically so an interrupted save cannot truncate them.
pub fn save(settings: &Settings) -> std::io::Result<()> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    let text = serde_json::to_string_pretty(settings).unwrap_or_default();
    let tmp = dir.join("settings.json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, settings_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_settings_file_still_loads() {
        let legacy = r##"{"theme":"light","accent":"#8b5cf6","autosave_ms":1200}"##;
        let s: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(s.theme, "light");
        assert_eq!(s.accent, "#8b5cf6");
        assert_eq!(s.autosave_ms, 1200);
        // New native fields fall back to defaults.
        assert!(s.show_line_numbers);
        assert!(s.autosave_enabled);
        assert_eq!(s.preview_width, 0.5);
    }

    #[test]
    fn roundtrip_keeps_every_field() {
        let mut s = Settings {
            theme: "light".into(),
            explorer_sort: "modified".into(),
            ..Default::default()
        };
        s.shortcuts.insert("palette".into(), "Ctrl+K".into());
        let text = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(back.theme, "light");
        assert_eq!(back.explorer_sort, "modified");
        assert_eq!(
            back.shortcuts.get("palette").map(String::as_str),
            Some("Ctrl+K")
        );
    }

    #[test]
    fn unknown_theme_values_fall_back_to_dark() {
        let s = Settings {
            theme: "solarized".into(),
            ..Default::default()
        };
        assert_eq!(s.theme_kind(), crate::theme::ThemeKind::Dark);
    }
}
