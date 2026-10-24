// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::commands::Command;
use crate::config::{load_settings, save_settings, Settings};
use crate::error::Result;
use crate::search::index::{SearchIndex, SearchResult};
use crate::vault::operations::*;
use crate::vault::safe_write::{atomic_write, file_meta, FileMeta};
use crate::vault::tree::{build_tree, is_markdown, FileNode};
use crate::vault::watcher::VaultWatcher;
use eframe::egui;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Edit,
    Split,
    Read,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: Uuid,
    pub path: Option<PathBuf>,
    pub title: String,
    pub content: String,
    pub saved_content: String,
    pub modified: bool,
    pub save_status: SaveStatus,
    pub last_meta: Option<FileMeta>,
    pub cursor: usize,
    pub scroll: f32,
    pub last_edit: Option<Instant>,
    pub pending_save: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Saved,
    Modified,
    Saving,
    Error,
}

impl Tab {
    pub fn new_empty() -> Self {
        Self {
            id: Uuid::new_v4(),
            path: None,
            title: "Untitled".to_string(),
            content: String::new(),
            saved_content: String::new(),
            modified: false,
            save_status: SaveStatus::Saved,
            last_meta: None,
            cursor: 0,
            scroll: 0.0,
            last_edit: None,
            pending_save: false,
        }
    }

    pub fn new_from_file(path: PathBuf, content: String, meta: Option<FileMeta>) -> Self {
        let title = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        Self {
            id: Uuid::new_v4(),
            path: Some(path),
            title,
            content: content.clone(),
            saved_content: content,
            modified: false,
            save_status: SaveStatus::Saved,
            last_meta: meta,
            cursor: 0,
            scroll: 0.0,
            last_edit: None,
            pending_save: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.content != self.saved_content
    }
}

pub struct AppState {
    pub settings: Settings,
    pub vault_root: Option<PathBuf>,
    pub vault_canonical: Option<PathBuf>,
    pub tree: Option<FileNode>,
    pub tabs: Vec<Tab>,
    pub active_tab: Option<usize>,
    pub mode: EditorMode,
    pub sidebar_open: bool,
    pub sidebar_width: f32,
    pub search_index: SearchIndex,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub show_search: bool,
    pub show_command_palette: bool,
    pub command_filter: String,
    pub show_settings: bool,
    pub status_message: String,
    pub confirm_delete: Option<PathBuf>,
    pub new_name_input: String,
    pub new_item_parent: Option<PathBuf>,
    pub creating: Option<CreateKind>,
    pub renaming: Option<PathBuf>,
    pub rename_input: String,
    pub find_query: String,
    pub find_replace: String,
    pub show_find: bool,
    pub templates_dir: PathBuf,
    pub templates: Vec<PathBuf>,
    pub show_templates: bool,
    pub recent_notes: VecDeque<PathBuf>,
    pub watcher: Option<VaultWatcher>,
    pub watcher_rx: Option<mpsc::Receiver<notify::Result<notify::Event>>>,
    pub external_change: Option<PathBuf>,
    pub last_save_error: Option<String>,
    pub doc_properties_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateKind {
    Note,
    Folder,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let settings = load_settings();
        let templates_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("NoteCraft")
            .join("templates");
        let _ = std::fs::create_dir_all(&templates_dir);
        Self {
            sidebar_width: settings.sidebar_width,
            settings,
            vault_root: None,
            vault_canonical: None,
            tree: None,
            tabs: Vec::new(),
            active_tab: None,
            mode: EditorMode::Edit,
            sidebar_open: true,
            search_index: SearchIndex::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            show_search: false,
            show_command_palette: false,
            command_filter: String::new(),
            show_settings: false,
            status_message: "Ready".to_string(),
            confirm_delete: None,
            new_name_input: String::new(),
            new_item_parent: None,
            creating: None,
            renaming: None,
            rename_input: String::new(),
            find_query: String::new(),
            find_replace: String::new(),
            show_find: false,
            templates_dir,
            templates: Vec::new(),
            show_templates: false,
            recent_notes: VecDeque::new(),
            watcher: None,
            watcher_rx: None,
            external_change: None,
            last_save_error: None,
            doc_properties_visible: true,
        }
    }

    pub fn open_vault(&mut self, path: PathBuf) -> Result<()> {
        let canon = canonicalize_vault(&path)?;
        self.vault_root = Some(path.clone());
        self.vault_canonical = Some(canon.clone());
        self.tree = Some(build_tree(&canon)?);
        self.search_index.rebuild(&canon);
        self.refresh_templates();
        // watcher
        if let Ok(mut w) = VaultWatcher::new(&canon) {
            self.watcher_rx = w.rx.take();
            self.watcher = Some(w);
        }
        self.settings.last_vault = Some(path.clone());
        let _ = save_settings(&self.settings);
        self.status_message = format!("Opened vault: {}", path.display());
        Ok(())
    }

    pub fn close_vault(&mut self) {
        self.vault_root = None;
        self.vault_canonical = None;
        self.tree = None;
        self.search_index = SearchIndex::new();
        self.watcher = None;
        self.watcher_rx = None;
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|i| self.tabs.get_mut(i))
    }

    pub fn active_tab_ref(&self) -> Option<&Tab> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    pub fn open_note(&mut self, path: PathBuf) {
        // if already open, focus
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path.as_ref() == Some(&path))
        {
            self.active_tab = Some(idx);
            return;
        }
        // read file
        let content = read_file_strict(&path).unwrap_or_default();
        let meta = file_meta(&path).ok();
        let tab = Tab::new_from_file(path.clone(), content, meta);
        self.tabs.push(tab);
        self.active_tab = Some(self.tabs.len() - 1);
        // recent
        self.recent_notes.push_front(path.clone());
        if self.recent_notes.len() > 20 {
            self.recent_notes.pop_back();
        }
    }

    pub fn save_current(&mut self) {
        if let Some(idx) = self.active_tab {
            self.save_tab(idx);
        }
    }

    pub fn save_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        let tab = &mut self.tabs[idx];
        if tab.path.is_none() {
            // unsaved new doc; cannot save without path; mark modified
            tab.save_status = SaveStatus::Error;
            self.status_message = "No file path; use Save As (not implemented)".to_string();
            return;
        }
        let path = tab.path.clone().unwrap();
        let content = tab.content.clone();
        if content == tab.saved_content {
            tab.modified = false;
            tab.save_status = SaveStatus::Saved;
            return;
        }
        tab.save_status = SaveStatus::Saving;
        tab.pending_save = true;
        let expected = tab.last_meta;
        match atomic_write(&path, &content, expected) {
            Ok(()) => {
                tab.saved_content = content;
                tab.modified = false;
                tab.save_status = SaveStatus::Saved;
                tab.last_meta = file_meta(&path).ok();
                tab.pending_save = false;
                self.status_message = format!("Saved {}", path.display());
                if let Some(root) = &self.vault_canonical {
                    self.search_index
                        .update_file(&path, &tab.saved_content.clone());
                    let _ = root;
                }
            }
            Err(e) => {
                tab.save_status = SaveStatus::Error;
                tab.pending_save = false;
                self.last_save_error = Some(e.to_string());
                self.status_message = format!("Save failed: {}", e);
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        if self.tabs[idx].is_dirty() {
            // mark; actual confirmation handled by UI setting a flag. For simplicity, save prompt not modal; we just keep tab.
            // We'll set a status message and not close. UI can call force_close.
            self.status_message = "Unsaved changes; close again to discard".to_string();
            // allow second close by clearing dirty? Instead track. Simpler: close anyway after warning? Requirement warns. We'll not auto-close.
            return;
        }
        self.tabs.remove(idx);
        if self.tabs.is_empty() {
            self.active_tab = None;
        } else {
            if let Some(a) = self.active_tab {
                if a >= self.tabs.len() {
                    self.active_tab = Some(self.tabs.len() - 1);
                } else if a > idx {
                    self.active_tab = Some(a - 1);
                }
            } else {
                self.active_tab = Some(0);
            }
        }
    }

    pub fn force_close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        self.tabs.remove(idx);
        if self.tabs.is_empty() {
            self.active_tab = None;
        } else {
            if let Some(a) = self.active_tab {
                if a >= self.tabs.len() {
                    self.active_tab = Some(self.tabs.len() - 1);
                } else if a > idx {
                    self.active_tab = Some(a - 1);
                }
            } else {
                self.active_tab = Some(0);
            }
        }
    }

    pub fn new_note_in(&mut self, dir: PathBuf) {
        let path = create_note(
            self.vault_canonical.as_deref().unwrap_or(Path::new(".")),
            &dir,
            "Untitled",
        )
        .ok();
        if let Some(p) = path {
            self.refresh_tree();
            self.open_note(p);
        }
    }

    pub fn new_folder_in(&mut self, parent: PathBuf) {
        let _ = create_dir(
            self.vault_canonical.as_deref().unwrap_or(Path::new(".")),
            &parent,
            "New Folder",
        );
        self.refresh_tree();
    }

    pub fn refresh_tree(&mut self) {
        if let Some(root) = &self.vault_canonical {
            self.tree = build_tree(root).ok();
            self.search_index.rebuild(root);
        }
    }

    pub fn refresh_templates(&mut self) {
        self.templates.clear();
        if self.templates_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.templates_dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_file() && is_markdown(&p) {
                        self.templates.push(p);
                    }
                }
            }
        }
        // also vault templates dir
        if let Some(root) = &self.vault_root {
            let vt = root.join("templates");
            if vt.exists() {
                if let Ok(entries) = std::fs::read_dir(&vt) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.is_file() && is_markdown(&p) {
                            self.templates.push(p);
                        }
                    }
                }
            }
        }
    }

    pub fn apply_template_to_new_note(&mut self, template_path: &Path) {
        let content = read_file_strict(template_path).unwrap_or_default();
        let title = "Untitled".to_string();
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let rendered = content
            .replace("{{date}}", &now)
            .replace("{{title}}", &title);
        // create note in vault root or active dir
        let dir = self
            .vault_root
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));
        if let Ok(path) = create_note(
            self.vault_canonical.as_deref().unwrap_or(Path::new(".")),
            &dir,
            "New from template",
        ) {
            let _ = atomic_write(&path, &rendered, None);
            self.refresh_tree();
            self.open_note(path);
        }
    }

    pub fn insert_template_into_current(&mut self, template_path: &Path) {
        let content = read_file_strict(template_path).unwrap_or_default();
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let rendered = content.replace("{{date}}", &now);
        if let Some(tab) = self.active_tab_mut() {
            tab.content.push_str(&rendered);
            tab.modified = true;
            tab.save_status = SaveStatus::Modified;
        }
    }

    pub fn run_command(&mut self, cmd: Command) {
        match cmd {
            Command::NewNote => {
                let dir = self
                    .vault_root
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("."));
                self.new_note_in(dir);
            }
            Command::NewFolder => {
                let parent = self
                    .vault_root
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("."));
                self.new_folder_in(parent);
            }
            Command::Save => self.save_current(),
            Command::CloseTab => {
                if let Some(i) = self.active_tab {
                    self.close_tab(i);
                }
            }
            Command::SwitchModeEdit => self.mode = EditorMode::Edit,
            Command::SwitchModeSplit => self.mode = EditorMode::Split,
            Command::SwitchModeRead => self.mode = EditorMode::Read,
            Command::TogglePreview => {
                self.mode = match self.mode {
                    EditorMode::Edit => EditorMode::Split,
                    _ => EditorMode::Edit,
                };
            }
            Command::SearchVault => {
                self.show_search = true;
                if let Some(root) = &self.vault_canonical {
                    self.search_index.rebuild(root);
                }
            }
            Command::SearchDocument => self.show_find = true,
            Command::OpenSettings => self.show_settings = true,
            Command::RefreshExplorer => self.refresh_tree(),
            Command::OpenCommandPalette => self.show_command_palette = true,
        }
    }

    pub fn autosave_tick(&mut self) {
        let now = Instant::now();
        for idx in 0..self.tabs.len() {
            let should = {
                let t = &self.tabs[idx];
                t.is_dirty()
                    && t.last_edit
                        .map(|le| {
                            now.duration_since(le).as_millis() as u64
                                >= self.settings.autosave_debounce_ms
                        })
                        .unwrap_or(false)
                    && !t.pending_save
            };
            if should {
                self.save_tab(idx);
            }
        }
    }

    pub fn poll_watcher(&mut self) {
        if let Some(rx) = &self.watcher_rx {
            while let Ok(res) = rx.try_recv() {
                if let Ok(ev) = res {
                    for p in ev.paths {
                        if is_markdown(&p) {
                            self.external_change = Some(p);
                        }
                    }
                }
            }
        }
    }

    pub fn reload_external(&mut self, path: &Path) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path.as_deref() == Some(path))
        {
            let content = read_file_strict(path).unwrap_or_default();
            let meta = file_meta(path).ok();
            let tab = &mut self.tabs[idx];
            if tab.is_dirty() {
                // keep modified? set conflict status
                tab.save_status = SaveStatus::Error;
                self.status_message = "External change detected; reload manually".to_string();
            } else {
                tab.content = content;
                tab.saved_content = tab.content.clone();
                tab.last_meta = meta;
                tab.modified = false;
                tab.save_status = SaveStatus::Saved;
            }
        }
        self.refresh_tree();
    }
}

pub struct NoteCraftApp {
    pub state: AppState,
}

impl NoteCraftApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();
        // restore last vault
        let last = state.settings.last_vault.clone();
        if let Some(p) = last {
            if p.exists() {
                let _ = state.open_vault(p);
            }
        }
        Self { state }
    }
}

impl eframe::App for NoteCraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = &mut self.state;
        // keyboard shortcuts
        handle_shortcuts(ctx, state);
        // autosave tick
        state.autosave_tick();
        // watcher
        state.poll_watcher();
        if let Some(p) = state.external_change.take() {
            state.reload_external(&p);
        }
        // apply theme
        apply_theme(ctx);

        // top-level layout
        crate::ui::layout::render(ctx, state);
        // request repaint for animations?
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::P) && i.modifiers.ctrl && !i.modifiers.shift {
            state.show_command_palette = true;
        }
        if i.key_pressed(egui::Key::S) && i.modifiers.ctrl {
            state.save_current();
        }
        if i.key_pressed(egui::Key::N) && i.modifiers.ctrl {
            state.run_command(Command::NewNote);
        }
        if i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift {
            state.run_command(Command::SearchVault);
        }
        if i.key_pressed(egui::Key::F) && i.modifiers.ctrl && !i.modifiers.shift {
            state.run_command(Command::SearchDocument);
        }
    });
}

fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_fill = egui::Color32::from_rgb(24, 26, 32);
    style.visuals.panel_fill = egui::Color32::from_rgb(24, 26, 32);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(30, 32, 40);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(18, 20, 26);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(120, 90, 200);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 32, 40);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(36, 38, 46);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(48, 50, 60);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 62, 74);
    style.spacing.item_spacing = egui::vec2(6.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

pub fn main_entry() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("NoteCraft"),
        ..Default::default()
    };
    eframe::run_native(
        "NoteCraft",
        options,
        Box::new(|cc| Ok(Box::new(NoteCraftApp::new(cc)))),
    )
}
