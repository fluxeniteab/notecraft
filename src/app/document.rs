//! An open document.
//!
//! Derived data (statistics, outline, JSON tree, MDX diagnostics) is recomputed
//! only when the text changes, and the text is the single source of truth: the
//! preview, the outline and the status bar are all projections of it.

use std::collections::HashSet;

use crate::documents::{Capabilities, Kind};
use crate::json::JsonDoc;
use crate::markdown::{self, DocStats, Heading};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Edit,
    Split,
    Read,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveState {
    /// In sync with the file on disk.
    Clean,
    /// Edited since the last successful save.
    Dirty,
    Saved,
    SaveFailed(String),
}

/// A validation problem for a data document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validation {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct Tab {
    /// Vault-relative path, `/` separated.
    pub path: String,
    pub kind: Kind,
    pub text: String,
    /// Contents as last written to (or read from) disk.
    pub saved: String,
    pub mode: ViewMode,
    pub save: SaveState,
    /// 1-based cursor position.
    pub cursor: (usize, usize),
    pub stats: DocStats,
    pub outline: Vec<Heading>,
    pub json: JsonDoc,
    pub json_collapsed: HashSet<String>,
    pub mdx: crate::mdx::Report,
    pub validation: Option<Validation>,
    /// Modification time observed on disk, in epoch seconds.
    pub disk_mtime: i64,
    /// Set when the file changed underneath unsaved edits.
    pub external_change: bool,
    /// Incremented whenever the text changes, so views can cache layouts.
    pub revision: u64,
    /// Cached preview request (scroll target).
    pub goto_line: Option<usize>,
}

impl Tab {
    pub fn new(path: &str, text: String, disk_mtime: i64) -> Self {
        let mut tab = Self {
            path: path.to_string(),
            kind: Kind::from_path(path),
            text,
            saved: String::new(),
            mode: ViewMode::Edit,
            save: SaveState::Clean,
            cursor: (1, 1),
            stats: DocStats::default(),
            outline: Vec::new(),
            json: JsonDoc::default(),
            json_collapsed: HashSet::new(),
            mdx: crate::mdx::Report::default(),
            validation: None,
            disk_mtime,
            external_change: false,
            revision: 1,
            goto_line: None,
        };
        tab.saved = tab.text.clone();
        tab.recompute();
        tab
    }

    pub fn capabilities(&self) -> &'static Capabilities {
        self.kind.capabilities()
    }

    pub fn title(&self) -> String {
        self.path
            .rsplit('/')
            .next()
            .map(|name| match name.rsplit_once('.') {
                Some((stem, _)) if !stem.is_empty() => stem.to_string(),
                _ => name.to_string(),
            })
            .unwrap_or_else(|| self.path.clone())
    }

    pub fn file_name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    pub fn folder(&self) -> String {
        match self.path.rsplit_once('/') {
            Some((dir, _)) => dir.to_string(),
            None => String::new(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved
    }

    pub fn default_mode(&self) -> ViewMode {
        if self.capabilities().preview {
            ViewMode::Split
        } else {
            ViewMode::Edit
        }
    }

    /// Recompute everything derived from the text.
    pub fn recompute(&mut self) {
        self.stats = markdown::stats(&self.text);
        match self.kind {
            Kind::Markdown | Kind::Mdx => {
                self.outline = markdown::outline(&self.text);
            }
            _ => self.outline.clear(),
        }
        match self.kind {
            Kind::Json => {
                self.json = JsonDoc::parse(&self.text);
                self.validation = self.json.error.as_ref().map(|err| Validation {
                    message: err.message.clone(),
                    line: err.line,
                    column: err.column,
                });
            }
            Kind::Yaml => {
                self.json = JsonDoc::default();
                self.validation = match serde_yaml::from_str::<serde_yaml::Value>(&self.text) {
                    Ok(_) => None,
                    Err(err) => {
                        let location = err.location();
                        Some(Validation {
                            message: err
                                .to_string()
                                .lines()
                                .next()
                                .unwrap_or("invalid YAML")
                                .into(),
                            line: location.as_ref().map(|l| l.line()).unwrap_or(1),
                            column: location.as_ref().map(|l| l.column()).unwrap_or(1),
                        })
                    }
                };
            }
            _ => {
                self.json = JsonDoc::default();
                self.validation = None;
            }
        }
        self.mdx = if self.kind == Kind::Mdx {
            crate::mdx::check(&self.text)
        } else {
            crate::mdx::Report::default()
        };
    }

    /// Called by the editor when it reports new text.
    pub fn on_edited(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.recompute();
        self.save = if self.is_dirty() {
            SaveState::Dirty
        } else {
            SaveState::Clean
        };
    }

    pub fn mark_saved(&mut self) {
        self.saved = self.text.clone();
        self.save = SaveState::Saved;
    }

    pub fn save_error(&mut self, message: impl Into<String>) {
        self.save = SaveState::SaveFailed(message.into());
    }
}
