//! Session persistence: which documents were open and how the panels were laid
//! out. Kept separate from settings so a corrupt session file cannot reset the
//! user's preferences (and vice versa).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    /// Vault root that was open.
    pub vault: Option<PathBuf>,
    /// Vault-relative paths of open tabs, in tab order.
    pub tabs: Vec<String>,
    pub active: Option<String>,
    pub left_visible: bool,
    pub right_visible: bool,
    pub left_width: f32,
    pub right_width: f32,
    pub recent_vaults: Vec<String>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            vault: None,
            tabs: Vec::new(),
            active: None,
            left_visible: true,
            right_visible: false,
            left_width: crate::theme::space::SIDEBAR_DEFAULT,
            right_width: crate::theme::space::CONTEXT_DEFAULT,
            recent_vaults: Vec::new(),
        }
    }
}

impl Session {
    pub fn load() -> Self {
        let path = crate::settings::session_path();
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = crate::settings::data_dir();
        std::fs::create_dir_all(&dir)?;
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        let tmp = dir.join("session.json.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, dir.join("session.json"))
    }

    /// Remember a vault, most recent first, without duplicates.
    pub fn remember_vault(&mut self, path: &str) {
        let path = path.to_string();
        self.recent_vaults.retain(|p| p != &path);
        self.recent_vaults.insert(0, path);
        self.recent_vaults.truncate(8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let session: Session = serde_json::from_str(r#"{"left_width":300.0}"#).unwrap();
        assert_eq!(session.left_width, 300.0);
        assert!(session.left_visible);
        assert!(session.tabs.is_empty());
    }

    #[test]
    fn recent_vaults_are_deduplicated_and_bounded() {
        let mut session = Session::default();
        session.remember_vault("/a");
        session.remember_vault("/b");
        session.remember_vault("/a");
        assert_eq!(
            session.recent_vaults,
            vec!["/a".to_string(), "/b".to_string()]
        );
        for index in 0..20 {
            session.remember_vault(&format!("/v{index}"));
        }
        assert!(session.recent_vaults.len() <= 8);
        assert_eq!(session.recent_vaults[0], "/v19");
    }
}
