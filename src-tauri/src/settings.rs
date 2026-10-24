use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String, // "system" | "light" | "dark"
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
    pub high_contrast: bool,
    pub telemetry_opt_in: bool,
    pub semantic_search_enabled: bool,
    pub data_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            accent: "#7c3aed".into(),
            editor_font_size: 15,
            editor_line_height: 1.6,
            editor_font_family: "Inter, system-ui, sans-serif".into(),
            reading_width: 760,
            autosave_ms: 800,
            snapshot_interval_min: 5,
            snapshot_retention: 50,
            excluded_folders: vec![".git".into(), "node_modules".into(), ".notecraft".into()],
            hidden_files: false,
            reduced_motion: false,
            high_contrast: false,
            telemetry_opt_in: false,
            semantic_search_enabled: false,
            data_dir: None,
        }
    }
}

pub fn data_dir() -> PathBuf {
    if let Some(p) = dirs::data_dir() {
        p.join("NoteCraft")
    } else {
        PathBuf::from(".notecraft")
    }
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn load() -> Settings {
    let p = settings_path();
    if let Ok(s) = std::fs::read_to_string(&p) {
        serde_json::from_str(&s).unwrap_or_default()
    } else {
        Settings::default()
    }
}

pub fn save(s: &Settings) -> Result<(), std::io::Error> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    let tmp = dir.join("settings.json.tmp");
    let txt = serde_json::to_string_pretty(s).unwrap_or_default();
    std::fs::write(&tmp, txt)?;
    std::fs::rename(&tmp, settings_path())?;
    Ok(())
}
