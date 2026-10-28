use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String, // "light" | "dark"
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
    pub explorer_sort: String, // "name" | "modified" | "size"
    pub explorer_group_dirs: bool,
    pub shortcuts: std::collections::HashMap<String, String>,
    pub writing_goal_words: u32,
    pub sync_folder: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            accent: "#8b5cf6".into(),
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
            telemetry_opt_in: false,
            semantic_search_enabled: false,
            daily_note_folder: "daily".into(),
            daily_note_template: None,
            capture_note: "inbox.md".into(),
            explorer_sort: "name".into(),
            explorer_group_dirs: true,
            shortcuts: std::collections::HashMap::new(),
            writing_goal_words: 0,
            sync_folder: None,
            data_dir: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip_with_new_fields() {
        let mut s = Settings::default();
        s.capture_note = "capture/inbox.md".into();
        s.explorer_sort = "modified".into();
        s.shortcuts.insert("palette".into(), "Ctrl+K".into());
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.capture_note, "capture/inbox.md");
        assert_eq!(back.explorer_sort, "modified");
        assert_eq!(
            back.shortcuts.get("palette").map(String::as_str),
            Some("Ctrl+K")
        );
        assert_eq!(back.writing_goal_words, 0);
    }

    #[test]
    fn legacy_settings_file_uses_defaults_for_new_fields() {
        // A settings.json written before these fields existed must still load.
        let legacy = r##"{"theme":"dark","accent":"#8b5cf6"}"##;
        let s: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(s.capture_note, "inbox.md");
        assert!(s.explorer_group_dirs);
        assert!(s.sync_folder.is_none());
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
