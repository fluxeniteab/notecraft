//! Application state and the eframe update loop.
//!
//! The shell is a thin orchestrator: panels are drawn by `ui::*`, documents live
//! in `document::Tab`, and every filesystem change goes through
//! `filesystem::*` so paths stay validated and writes stay atomic.

pub mod document;

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::Key;

pub use document::{SaveState, Tab, ViewMode};

use crate::documents::{self, Kind};
use crate::filesystem::{self, FileEntry};
use crate::search::{self, Hit, Scope};
use crate::session::Session;
use crate::settings::{self, Settings};
use crate::theme::{self, Theme, ThemeKind};

/// Which secondary view the right panel shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTab {
    Outline,
    Properties,
    Links,
    Data,
    History,
}

/// Command palette modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    Files,
    Commands,
}

/// One palette row.
#[derive(Debug, Clone)]
pub enum PaletteItem {
    OpenFile { path: String },
    RunCommand { id: CommandId },
    Search { line: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
    NewMarkdown,
    NewMdx,
    NewJson,
    NewYaml,
    NewText,
    NewFolder,
    Save,
    SaveAll,
    CloseTab,
    ExportHtml,
    ToggleSidebar,
    ToggleContext,
    ToggleTheme,
    CycleView,
    FormatJson,
    MinifyJson,
    OpenSettings,
    ToggleNumbers,
    ExpandAll,
    CollapseAll,
    Refresh,
}

#[derive(Clone)]
pub struct PaletteItemEntry {
    pub item: PaletteItem,
    pub label: String,
    pub hint: String,
}

pub struct PaletteState {
    pub open: bool,
    pub mode: PaletteMode,
    pub query: String,
    pub selected: usize,
}

impl Default for PaletteState {
    fn default() -> Self {
        Self {
            open: false,
            mode: PaletteMode::Files,
            query: String::new(),
            selected: 0,
        }
    }
}

/// File explorer state.
pub struct ExplorerState {
    pub expanded: HashSet<String>,
    pub filter: String,
    pub sort: String,
    pub selected: Option<String>,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            expanded: HashSet::new(),
            filter: String::new(),
            sort: "name".into(),
            selected: None,
        }
    }
}

/// Panel visibility and widths.
pub struct Panels {
    pub left: bool,
    pub right: bool,
    pub left_width: f32,
    pub right_width: f32,
}

/// What a confirmation dialog will do.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Delete { path: String, is_dir: bool },
    Discard { path: String, then_close: bool },
}

pub struct Confirm {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub danger: bool,
    pub action: ConfirmAction,
}

/// What a text prompt will do.
#[derive(Debug, Clone)]
pub enum PromptAction {
    NewDocument { dir: String, kind: Kind },
    NewFolder { dir: String },
    Rename { from: String },
}

pub struct Prompt {
    pub title: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub confirm_label: String,
    pub action: PromptAction,
    pub focus: bool,
}

pub struct Toast {
    pub text: String,
    pub detail: Option<String>,
    pub error: bool,
    pub born: Instant,
}

/// Full-text search state for the sidebar.
pub struct SearchState {
    pub query: String,
    pub scope: Scope,
    pub hits: Vec<Hit>,
    pub opened_folder: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            scope: Scope::All,
            hits: Vec::new(),
            opened_folder: false,
        }
    }
}

pub struct App {
    pub settings: Settings,
    pub session: Session,
    pub theme: Theme,
    pub vault: Option<PathBuf>,
    pub entries: Vec<FileEntry>,
    pub tabs: Vec<Tab>,
    pub active: Option<usize>,
    pub panels: Panels,
    pub explorer: ExplorerState,
    pub palette: PaletteState,
    pub palette_items: Vec<PaletteItemEntry>,
    pub search: SearchState,
    pub context: ContextTab,
    pub settings_open: bool,
    pub about_open: bool,
    pub plugins_open: bool,
    pub shortcuts_open: bool,
    pub prompt: Option<Prompt>,
    pub confirm: Option<Confirm>,
    pub context_menu: Option<(egui::Pos2, String, bool)>,
    /// Anchor of the vault switcher popup, when open.
    pub vault_menu: Option<egui::Pos2>,
    /// Anchor of the "new document" popup, when open.
    pub new_menu: Option<egui::Pos2>,
    pub toasts: Vec<Toast>,
    pub focus_mode: bool,
    /// A link waiting to be handed to the OS browser.
    pending_url: Option<String>,
    dirty_since: Option<Instant>,
    last_scan: Instant,
    /// Documents whose backlink list is being shown.
    pub backlinks: Vec<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
        let availability = theme::fonts::install(&cc.egui_ctx);

        let settings = settings::load();
        let session = Session::load();
        let mut app = Self {
            theme: Theme::build(settings.theme_kind(), &settings.accent, &settings),
            panels: Panels {
                left: session.left_visible,
                right: session.right_visible,
                left_width: session.left_width,
                right_width: session.right_width,
            },
            vault: None,
            entries: Vec::new(),
            tabs: Vec::new(),
            active: None,
            explorer: ExplorerState {
                sort: settings.explorer_sort.clone(),
                ..Default::default()
            },
            palette: PaletteState::default(),
            palette_items: Vec::new(),
            search: SearchState::default(),
            context: ContextTab::Outline,
            settings_open: false,
            about_open: false,
            plugins_open: false,
            shortcuts_open: false,
            prompt: None,
            confirm: None,
            context_menu: None,
            vault_menu: None,
            new_menu: None,
            toasts: Vec::new(),
            focus_mode: false,
            pending_url: None,
            dirty_since: None,
            last_scan: Instant::now(),
            backlinks: Vec::new(),
            session,
            settings,
        };
        app.theme.fonts = availability;

        if let Some(vault) = app.session.vault.clone() {
            if vault.is_dir() {
                app.set_vault(vault);
            }
        }
        if app.settings.reopen_last_session {
            let paths = app.session.tabs.clone();
            for path in paths {
                app.open_path(&path, true);
            }
            if let Some(active) = app.session.active.clone() {
                if let Some(index) = app.tabs.iter().position(|t| t.path == active) {
                    app.active = Some(index);
                }
            }
        }
        app.apply_theme(&cc.egui_ctx);
        app
    }

    // ---------------------------------------------------------------- theme

    pub fn apply_theme(&mut self, ctx: &egui::Context) {
        self.theme = Theme::build(
            self.settings.theme_kind(),
            &self.settings.accent,
            &self.settings,
        );
        theme::apply(ctx, &self.theme);
    }

    pub fn set_theme(&mut self, ctx: &egui::Context, kind: ThemeKind) {
        self.settings.theme = kind.as_str().to_string();
        self.apply_theme(ctx);
        self.persist_settings();
    }

    // --------------------------------------------------------------- vault

    pub fn set_vault(&mut self, path: PathBuf) {
        self.vault = Some(path.clone());
        self.session.vault = Some(path.clone());
        self.session.remember_vault(&path.to_string_lossy());
        self.refresh_vault();
        if let Some(root) = self.entries.iter().find(|e| e.is_dir) {
            self.explorer.expanded.insert(root.path.clone());
        }
        self.persist_session();
    }

    pub fn refresh_vault(&mut self) {
        let Some(root) = self.vault.clone() else {
            self.entries.clear();
            return;
        };
        let excluded: Vec<String> = self.settings.excluded_folders.clone();
        self.entries = filesystem::scan_vault(&root, &excluded, self.settings.hidden_files);
        self.entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                return b.is_dir.cmp(&a.is_dir);
            }
            match self.settings.explorer_sort.as_str() {
                "modified" => b.modified.cmp(&a.modified),
                "size" => b.size.cmp(&a.size),
                _ => a.path.to_lowercase().cmp(&b.path.to_lowercase()),
            }
        });
        self.last_scan = Instant::now();
    }

    pub fn pick_vault(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.set_vault(folder);
        }
    }

    pub fn close_vault(&mut self) {
        self.vault = None;
        self.entries.clear();
        self.tabs.clear();
        self.active = None;
        self.session.vault = None;
        self.session.tabs.clear();
        self.session.active = None;
        self.persist_session();
    }

    // ----------------------------------------------------------- documents

    pub fn active_tab(&self) -> Option<&Tab> {
        self.active.and_then(|index| self.tabs.get(index))
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active.and_then(|index| self.tabs.get_mut(index))
    }

    pub fn open_path(&mut self, rel: &str, quiet: bool) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.path == rel) {
            self.active = Some(index);
            return;
        }
        let Some(root) = self.vault.clone() else {
            return;
        };
        match filesystem::read_text(&root, rel) {
            Ok(text) => {
                let mtime = std::fs::metadata(root.join(rel))
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let mut tab = Tab::new(rel, text, mtime);
                tab.mode = tab.default_mode();
                self.tabs.push(tab);
                self.active = Some(self.tabs.len() - 1);
                self.context = if rel.ends_with(".json") {
                    ContextTab::Data
                } else {
                    ContextTab::Outline
                };
                self.persist_session();
            }
            Err(err) => {
                if !quiet {
                    self.toast_error("Could not open the document", err.to_string());
                }
            }
        }
    }

    pub fn new_document(&mut self, dir: &str, kind: Kind) {
        let Some(root) = self.vault.clone() else {
            self.toast(
                "Open a vault first",
                Some("File → Open folder…".into()),
                false,
            );
            return;
        };
        let mut index = 1;
        let name = loop {
            let candidate = if index == 1 {
                format!("Untitled.{}", kind.capabilities().extension)
            } else {
                format!("Untitled {index}.{}", kind.capabilities().extension)
            };
            let path = if dir.is_empty() {
                candidate.clone()
            } else {
                format!("{dir}/{candidate}")
            };
            if !root.join(&path).exists() {
                break candidate;
            }
            index += 1;
        };
        let rel = if dir.is_empty() {
            name.clone()
        } else {
            format!("{dir}/{name}")
        };
        match filesystem::create_file(&root, &rel, documents::template_for(kind)) {
            Ok(()) => {
                self.refresh_vault();
                self.open_path(&rel, false);
                self.persist_session();
                self.toast("Document created", Some(rel), false);
            }
            Err(err) => self.toast_error("Could not create the document", err.to_string()),
        }
    }

    pub fn new_folder(&mut self, dir: &str, name: &str) {
        let Some(root) = self.vault.clone() else {
            return;
        };
        let rel = if dir.is_empty() {
            name.to_string()
        } else {
            format!("{dir}/{name}")
        };
        match filesystem::create_dir(&root, &rel) {
            Ok(()) => {
                self.refresh_vault();
                self.explorer.expanded.insert(rel.clone());
                if !dir.is_empty() {
                    self.explorer.expanded.insert(dir.to_string());
                }
                self.toast("Folder created", Some(rel), false);
            }
            Err(err) => self.toast_error("Could not create the folder", err.to_string()),
        }
    }

    pub fn save_active(&mut self) {
        if let Some(index) = self.active {
            self.save_tab(index);
        }
    }

    pub fn save_tab(&mut self, index: usize) {
        let Some(root) = self.vault.clone() else {
            return;
        };
        let Some(tab) = self.tabs.get(index) else {
            return;
        };
        if tab.external_change {
            self.toast_error(
                "Not overwritten",
                "The file changed on disk. Reload it or save under a new name.",
            );
            return;
        }
        if !tab.is_dirty() {
            return;
        }
        let path = tab.path.clone();
        let text = tab.text.clone();
        let retention = self.settings.snapshot_retention as usize;

        if self.settings.snapshot_interval_min > 0 {
            let _ = filesystem::snapshot(&root, &path, retention);
        }
        match filesystem::write_text_atomic(&root, &path, &text) {
            Ok(()) => {
                let mtime = std::fs::metadata(root.join(&path))
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                if let Some(tab) = self.tabs.get_mut(index) {
                    tab.mark_saved();
                    tab.disk_mtime = mtime;
                }
                self.dirty_since = None;
                self.refresh_vault();
            }
            Err(err) => {
                if let Some(tab) = self.tabs.get_mut(index) {
                    tab.save_error(err.to_string());
                }
                self.toast_error("Could not save", err.to_string());
            }
        }
    }

    pub fn save_all(&mut self) {
        for index in 0..self.tabs.len() {
            if self.tabs[index].is_dirty() {
                self.save_tab(index);
            }
        }
    }

    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        if self.tabs[index].is_dirty() {
            let path = self.tabs[index].path.clone();
            self.confirm = Some(Confirm {
                title: "Discard unsaved changes?".into(),
                message: format!("{} has changes that were not saved.", path),
                confirm_label: "Discard".into(),
                danger: true,
                action: ConfirmAction::Discard {
                    path,
                    then_close: true,
                },
            });
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active = None;
        } else if let Some(active) = self.active {
            self.active = Some(active.min(self.tabs.len() - 1));
        }
        self.persist_session();
    }

    pub fn confirm_discard_and_close(&mut self, path: &str) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.path == path) {
            self.tabs.remove(index);
            if self.tabs.is_empty() {
                self.active = None;
            } else if let Some(active) = self.active {
                self.active = Some(active.min(self.tabs.len() - 1));
            }
        }
        self.persist_session();
    }

    pub fn rename_entry(&mut self, from: &str, to: &str) {
        let Some(root) = self.vault.clone() else {
            return;
        };
        match filesystem::rename(&root, from, to) {
            Ok(()) => {
                // Keep open tabs pointing at the new path.
                for tab in self.tabs.iter_mut() {
                    if tab.path == from {
                        tab.path = to.to_string();
                        tab.kind = Kind::from_path(to);
                    } else if tab.path.starts_with(&format!("{from}/")) {
                        tab.path = tab.path.replacen(from, to, 1);
                        tab.kind = Kind::from_path(&tab.path);
                    }
                }
                let updated = self.update_links(from, to);
                self.refresh_vault();
                self.persist_session();
                self.toast(
                    "Renamed",
                    Some(if updated > 0 {
                        format!("{to} · {updated} link(s) updated")
                    } else {
                        to.to_string()
                    }),
                    false,
                );
            }
            Err(err) => self.toast_error("Could not rename", err.to_string()),
        }
    }

    fn update_links(&mut self, from: &str, to: &str) -> usize {
        let Some(root) = self.vault.clone() else {
            return 0;
        };
        let mut updated = 0;
        let paths: Vec<String> = self
            .entries
            .iter()
            .filter(|entry| !entry.is_dir && documents::capabilities_for(&entry.path).metadata)
            .map(|entry| entry.path.clone())
            .collect();
        for path in paths {
            let Ok(text) = filesystem::read_text(&root, &path) else {
                continue;
            };
            if !text.contains("[[") {
                continue;
            }
            let renamed = crate::links::rename_targets(&text, from, to);
            if renamed != text && filesystem::write_text_atomic(&root, &path, &renamed).is_ok() {
                updated += 1;
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.path == path) {
                    if !tab.is_dirty() {
                        tab.text = renamed.clone();
                        tab.saved = renamed;
                        tab.recompute();
                    }
                }
            }
        }
        updated
    }

    pub fn delete_entry(&mut self, path: &str) {
        let Some(root) = self.vault.clone() else {
            return;
        };
        match filesystem::trash(&root, path) {
            Ok(moved) => {
                self.tabs
                    .retain(|tab| tab.path != path && !tab.path.starts_with(&format!("{path}/")));
                if self.tabs.is_empty() {
                    self.active = None;
                } else if let Some(active) = self.active {
                    self.active = Some(active.min(self.tabs.len() - 1));
                }
                self.refresh_vault();
                self.persist_session();
                self.toast("Moved to trash", Some(moved), false);
            }
            Err(err) => self.toast_error("Could not delete", err.to_string()),
        }
    }

    /// Toggle `- [ ]` / `- [x]` at a byte offset reported by the preview.
    pub fn toggle_task(&mut self, offset: usize) {
        let Some(tab) = self.active_tab_mut() else {
            return;
        };
        if offset >= tab.text.len() {
            return;
        }
        let rest = &tab.text[offset..];
        if rest.starts_with("- [ ]") || rest.starts_with("* [ ]") || rest.starts_with("[ ]") {
            let at = offset + rest.find("[ ]").unwrap_or(0);
            tab.text.replace_range(at..at + 3, "[x]");
            tab.on_edited();
        } else if rest.starts_with("- [x]") || rest.starts_with("* [x]") || rest.starts_with("[x]")
        {
            let at = offset + rest.find("[x]").unwrap_or(0);
            tab.text.replace_range(at..at + 3, "[ ]");
            tab.on_edited();
        }
    }

    pub fn format_json(&mut self, minify: bool) {
        let Some(tab) = self.active_tab_mut() else {
            return;
        };
        if tab.kind != Kind::Json {
            return;
        }
        let result = if minify {
            crate::json::minify(&tab.text)
        } else {
            crate::json::format(&tab.text, 2)
        };
        match result {
            Ok(formatted) => {
                tab.text = formatted;
                tab.on_edited();
                self.toast("JSON formatted", None, false);
            }
            Err(err) => {
                let message = err.to_string();
                self.toast_error("Invalid JSON — nothing was changed", message);
            }
        }
    }

    pub fn export_active(&mut self) {
        let Some(tab) = self.active_tab() else {
            return;
        };
        let title = tab.title();
        let path = tab.path.clone();
        let text = tab.text.clone();
        let kind = tab.kind;

        let suggested = format!(
            "{}.{}",
            title,
            if kind.capabilities().preview {
                "html"
            } else {
                documents::export_extension(kind)
            }
        );
        let Some(target) = rfd::FileDialog::new().set_file_name(suggested).save_file() else {
            return;
        };

        let body = if kind.capabilities().preview {
            crate::markdown::export::to_html(&text, &title)
        } else {
            text.clone()
        };
        match std::fs::write(&target, body) {
            Ok(()) => self.toast(
                "Exported",
                Some(target.to_string_lossy().to_string()),
                false,
            ),
            Err(err) => self.toast_error("Could not export", err.to_string()),
        }
        let _ = path;
    }

    pub fn copy_active_as_text(&mut self, ctx: &egui::Context) {
        if let Some(tab) = self.active_tab() {
            let body = if tab.kind.capabilities().preview {
                crate::markdown::export::to_plain_text(&tab.text)
            } else {
                tab.text.clone()
            };
            ctx.copy_text(body);
            self.toast("Copied as plain text", None, false);
        }
    }

    // ---------------------------------------------------------------- misc

    /// Queue an external link. Only explicit web/mail schemes are handed to the
    /// shell; anything else stays inside the application.
    pub fn open_external(&mut self, url: &str) {
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
            self.pending_url = Some(url.to_string());
        }
    }

    /// Open a folder in the platform file manager.
    pub fn open_folder(&self, path: &std::path::Path) {
        #[cfg(target_os = "windows")]
        let program = "explorer";
        #[cfg(target_os = "macos")]
        let program = "open";
        #[cfg(all(unix, not(target_os = "macos")))]
        let program = "xdg-open";
        if let Err(err) = std::process::Command::new(program).arg(path).spawn() {
            log::warn!("could not open {}: {err}", path.display());
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, detail: Option<String>, error: bool) {
        self.toasts.push(Toast {
            text: text.into(),
            detail,
            error,
            born: Instant::now(),
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub fn toast_error(&mut self, text: impl Into<String>, detail: impl Into<String>) {
        self.toast(text, Some(detail.into()), true);
    }

    pub fn persist_settings(&mut self) {
        self.settings.explorer_sort = self.explorer.sort.clone();
        if let Err(err) = settings::save(&self.settings) {
            log::warn!("could not save settings: {err}");
        }
    }

    pub fn persist_session(&mut self) {
        self.session.tabs = self.tabs.iter().map(|tab| tab.path.clone()).collect();
        self.session.active = self.active_tab().map(|tab| tab.path.clone());
        self.session.left_visible = self.panels.left;
        self.session.right_visible = self.panels.right;
        self.session.left_width = self.panels.left_width;
        self.session.right_width = self.panels.right_width;
        if let Err(err) = self.session.save() {
            log::warn!("could not save session: {err}");
        }
    }

    /// Run a palette command.
    pub fn run_command(&mut self, id: CommandId, ctx: &egui::Context) {
        match id {
            CommandId::NewMarkdown => self.new_document("", Kind::Markdown),
            CommandId::NewMdx => self.new_document("", Kind::Mdx),
            CommandId::NewJson => self.new_document("", Kind::Json),
            CommandId::NewYaml => self.new_document("", Kind::Yaml),
            CommandId::NewText => self.new_document("", Kind::Text),
            CommandId::NewFolder => self.prompt_new_folder(),
            CommandId::Save => self.save_active(),
            CommandId::SaveAll => self.save_all(),
            CommandId::CloseTab => {
                if let Some(index) = self.active {
                    self.close_tab(index);
                }
            }
            CommandId::ExportHtml => self.export_active(),
            CommandId::ToggleSidebar => {
                self.panels.left = !self.panels.left;
                self.persist_session();
            }
            CommandId::ToggleContext => {
                self.panels.right = !self.panels.right;
                self.persist_session();
            }
            CommandId::ToggleTheme => {
                let next = self.settings.theme_kind().toggled();
                self.set_theme(ctx, next);
            }
            CommandId::CycleView => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.mode = match tab.mode {
                        ViewMode::Edit => ViewMode::Split,
                        ViewMode::Split => ViewMode::Read,
                        ViewMode::Read => ViewMode::Edit,
                    };
                }
            }
            CommandId::FormatJson => self.format_json(false),
            CommandId::MinifyJson => self.format_json(true),
            CommandId::OpenSettings => self.settings_open = true,
            CommandId::ToggleNumbers => {
                self.settings.show_line_numbers = !self.settings.show_line_numbers;
                self.persist_settings();
            }
            CommandId::ExpandAll => {
                let dirs: Vec<String> = self
                    .entries
                    .iter()
                    .filter(|entry| entry.is_dir)
                    .map(|entry| entry.path.clone())
                    .collect();
                self.explorer.expanded.extend(dirs);
            }
            CommandId::CollapseAll => self.explorer.expanded.clear(),
            CommandId::Refresh => self.refresh_vault(),
        }
    }

    pub fn prompt_new_folder(&mut self) {
        self.prompt = Some(Prompt {
            title: "New folder".into(),
            label: "Folder name".into(),
            value: String::new(),
            placeholder: "Research".into(),
            confirm_label: "Create".into(),
            action: PromptAction::NewFolder { dir: String::new() },
            focus: false,
        });
    }

    pub fn run_prompt(&mut self) {
        let Some(prompt) = self.prompt.take() else {
            return;
        };
        let value = prompt.value.trim().to_string();
        if value.is_empty() {
            return;
        }
        match prompt.action {
            PromptAction::NewFolder { dir } => self.new_folder(&dir, &value),
            PromptAction::NewDocument { dir, kind } => {
                let name = documents::ensure_extension(&value);
                let rel = if dir.is_empty() {
                    name
                } else {
                    format!("{dir}/{name}")
                };
                if let Some(root) = self.vault.clone() {
                    match filesystem::create_file(&root, &rel, documents::template_for(kind)) {
                        Ok(()) => {
                            self.refresh_vault();
                            self.open_path(&rel, false);
                            self.persist_session();
                        }
                        Err(err) => self.toast_error("Could not create", err.to_string()),
                    }
                }
            }
            PromptAction::Rename { from } => {
                let dir = match from.rsplit_once('/') {
                    Some((dir, _)) => dir.to_string(),
                    None => String::new(),
                };
                let mut target = value;
                // Preserving the extension keeps the document type intact.
                if documents::extension_of(&target).is_none() {
                    if let Some(ext) = documents::extension_of(&from) {
                        target = format!("{target}.{ext}");
                    }
                }
                let to = if dir.is_empty() {
                    target
                } else {
                    format!("{dir}/{target}")
                };
                if to != from {
                    self.rename_entry(&from, &to);
                }
            }
        }
    }

    pub fn run_confirm(&mut self) {
        let Some(confirm) = self.confirm.take() else {
            return;
        };
        match confirm.action {
            ConfirmAction::Delete { path, is_dir } => {
                let _ = is_dir;
                self.delete_entry(&path);
            }
            ConfirmAction::Discard { path, then_close } => {
                if then_close {
                    self.confirm_discard_and_close(&path);
                }
            }
        }
    }

    pub fn reload_from_disk(&mut self, path: &str) {
        let Some(root) = self.vault.clone() else {
            return;
        };
        if let Ok(text) = filesystem::read_text(&root, path) {
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.path == path) {
                tab.text = text;
                tab.mark_saved();
                tab.external_change = false;
                tab.recompute();
                self.toast("Reloaded from disk", Some(path.to_string()), false);
            }
        }
    }

    /// Detect a file that changed underneath an open tab.
    fn check_external_changes(&mut self) {
        let Some(root) = self.vault.clone() else {
            return;
        };
        for tab in self.tabs.iter_mut() {
            let Ok(meta) = std::fs::metadata(root.join(&tab.path)) else {
                continue;
            };
            let mtime = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            if mtime > tab.disk_mtime {
                if tab.is_dirty() {
                    tab.external_change = true;
                } else if let Ok(text) = filesystem::read_text(&root, &tab.path) {
                    if text != tab.text {
                        tab.text = text;
                        tab.mark_saved();
                        tab.disk_mtime = mtime;
                        tab.recompute();
                    }
                }
            }
        }
    }

    fn autosave_tick(&mut self) {
        if !self.settings.autosave_enabled {
            return;
        }
        let dirty = self.tabs.iter().any(|tab| tab.is_dirty());
        if !dirty {
            self.dirty_since = None;
            return;
        }
        let since = *self.dirty_since.get_or_insert_with(Instant::now);
        let interval = Duration::from_millis(self.settings.autosave_ms.max(400) as u64);
        if since.elapsed() >= interval {
            for index in 0..self.tabs.len() {
                if self.tabs[index].is_dirty() {
                    self.save_tab(index);
                }
            }
            self.dirty_since = None;
        }
    }

    pub fn recompute_backlinks(&mut self) {
        let Some(tab) = self.active_tab() else {
            self.backlinks.clear();
            return;
        };
        let target = tab.path.clone();
        let Some(root) = self.vault.clone() else {
            return;
        };
        let documents: Vec<(String, String)> = self
            .entries
            .iter()
            .filter(|entry| !entry.is_dir && entry.path.ends_with(".md"))
            .filter_map(|entry| {
                filesystem::read_text(&root, &entry.path)
                    .ok()
                    .map(|text| (entry.path.clone(), text))
            })
            .collect();
        let refs: Vec<(&str, &str)> = documents
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str()))
            .collect();
        self.backlinks = crate::links::backlinks(&target, refs);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let modal_open = self.palette.open || self.prompt.is_some() || self.confirm.is_some();
        let command = |key: Key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, key));

        if command(Key::K) {
            self.palette.open = true;
            self.palette.query.clear();
            self.palette.mode = PaletteMode::Files;
            self.palette.selected = 0;
        }
        if command(Key::P) {
            self.palette.open = true;
            self.palette.query.clear();
            self.palette.mode = PaletteMode::Commands;
            self.palette.selected = 0;
        }
        if command(Key::S) && !modal_open {
            if ctx.input(|i| i.modifiers.shift) {
                self.save_all();
            } else {
                self.save_active();
            }
        }
        if command(Key::N) && !modal_open {
            self.new_document("", Kind::Markdown);
        }
        if command(Key::W) && !modal_open {
            if let Some(index) = self.active {
                self.close_tab(index);
            }
        }
        if command(Key::B) {
            self.panels.left = !self.panels.left;
            self.persist_session();
        }
        if command(Key::Backslash) {
            self.panels.right = !self.panels.right;
            self.persist_session();
        }
        if command(Key::Comma) {
            self.settings_open = true;
        }
        if command(Key::E) && !modal_open {
            if let Some(tab) = self.active_tab_mut() {
                tab.mode = match tab.mode {
                    ViewMode::Edit => ViewMode::Split,
                    ViewMode::Split => ViewMode::Read,
                    ViewMode::Read => ViewMode::Edit,
                };
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::Num1)) {
            if let Some(tab) = self.active_tab_mut() {
                tab.mode = ViewMode::Edit;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::Num2)) {
            if let Some(tab) = self.active_tab_mut() {
                tab.mode = ViewMode::Split;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::Num3)) {
            if let Some(tab) = self.active_tab_mut() {
                tab.mode = ViewMode::Read;
            }
        }
        if ctx
            .input_mut(|i| i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, Key::P))
        {
            self.focus_mode = !self.focus_mode;
            self.panels.left = !self.focus_mode;
            self.panels.right = !self.focus_mode && self.panels.right;
        }
        // Escape closes whatever is on top.
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.palette.open {
                self.palette.open = false;
            } else if self.prompt.is_some() {
                self.prompt = None;
            } else if self.confirm.is_some() {
                self.confirm = None;
            } else if self.context_menu.is_some() {
                self.context_menu = None;
            } else if self.settings_open {
                self.settings_open = false;
            }
        }
        // Alt+Arrow moves between tabs.
        let next_tab = ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, Key::ArrowRight));
        if next_tab && !self.tabs.is_empty() {
            let next = self.active.map_or(0, |index| (index + 1) % self.tabs.len());
            self.active = Some(next);
        }
        let previous_tab = ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, Key::ArrowLeft));
        if previous_tab && !self.tabs.is_empty() {
            let count = self.tabs.len();
            let previous = self.active.map_or(0, |index| (index + count - 1) % count);
            self.active = Some(previous);
        }
    }

    pub fn palette_candidates(&self, query: &str) -> Vec<PaletteItemEntry> {
        let needle = query.trim().to_lowercase();
        let mut out: Vec<PaletteItemEntry> = Vec::new();

        match self.palette.mode {
            PaletteMode::Files => {
                let mut scored: Vec<(usize, PaletteItemEntry)> = Vec::new();
                for entry in self.entries.iter().filter(|e| !e.is_dir) {
                    let Some(score) = search::fuzzy_path(&needle, &entry.path) else {
                        continue;
                    };
                    scored.push((
                        score,
                        PaletteItemEntry {
                            item: PaletteItem::OpenFile {
                                path: entry.path.clone(),
                            },
                            label: entry.name.clone(),
                            hint: entry.path.clone(),
                        },
                    ));
                }
                scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
                out = scored.into_iter().map(|(_, item)| item).collect();
            }
            PaletteMode::Commands => {
                for (id, label, hint) in self.command_list() {
                    if !needle.is_empty() && !label.to_lowercase().contains(&needle) {
                        continue;
                    }
                    out.push(PaletteItemEntry {
                        item: PaletteItem::RunCommand { id },
                        label,
                        hint,
                    });
                }
            }
        }

        if !needle.is_empty() {
            self.search
                .hits
                .iter()
                .filter(|hit| {
                    hit.snippet.to_lowercase().contains(&needle)
                        || hit.path.to_lowercase().contains(&needle)
                })
                .take(8)
                .for_each(|hit| {
                    out.push(PaletteItemEntry {
                        item: PaletteItem::Search { line: hit.line },
                        label: format!("{}:{}", hit.path, hit.line),
                        hint: hit.snippet.clone(),
                    });
                });
        }
        out.truncate(60);
        out
    }

    pub fn command_list(&self) -> Vec<(CommandId, String, String)> {
        vec![
            (
                CommandId::NewMarkdown,
                "New Markdown note".into(),
                "Ctrl N".into(),
            ),
            (CommandId::NewMdx, "New MDX document".into(), String::new()),
            (
                CommandId::NewJson,
                "New JSON document".into(),
                String::new(),
            ),
            (
                CommandId::NewYaml,
                "New YAML document".into(),
                String::new(),
            ),
            (
                CommandId::NewText,
                "New plain text file".into(),
                String::new(),
            ),
            (CommandId::NewFolder, "New folder".into(), String::new()),
            (CommandId::Save, "Save document".into(), "Ctrl S".into()),
            (
                CommandId::SaveAll,
                "Save all documents".into(),
                "Ctrl Shift S".into(),
            ),
            (CommandId::CloseTab, "Close tab".into(), "Ctrl W".into()),
            (
                CommandId::ExportHtml,
                "Export document…".into(),
                String::new(),
            ),
            (
                CommandId::ToggleSidebar,
                "Toggle file explorer".into(),
                "Ctrl B".into(),
            ),
            (
                CommandId::ToggleContext,
                "Toggle context panel".into(),
                "Ctrl \\".into(),
            ),
            (CommandId::ToggleTheme, "Switch theme".into(), String::new()),
            (
                CommandId::CycleView,
                "Cycle view mode".into(),
                "Ctrl E".into(),
            ),
            (CommandId::FormatJson, "Format JSON".into(), String::new()),
            (CommandId::MinifyJson, "Minify JSON".into(), String::new()),
            (
                CommandId::ToggleNumbers,
                "Toggle line numbers".into(),
                String::new(),
            ),
            (
                CommandId::ExpandAll,
                "Expand all folders".into(),
                String::new(),
            ),
            (
                CommandId::CollapseAll,
                "Collapse all folders".into(),
                String::new(),
            ),
            (CommandId::Refresh, "Rescan vault".into(), String::new()),
            (
                CommandId::OpenSettings,
                "Open settings".into(),
                "Ctrl ,".into(),
            ),
        ]
    }

    pub fn run_palette_item(&mut self, item: PaletteItem, ctx: &egui::Context) {
        match item {
            PaletteItem::OpenFile { path } => self.open_path(&path, false),
            PaletteItem::RunCommand { id } => self.run_command(id, ctx),
            PaletteItem::Search { line } => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.goto_line = Some(line);
                    tab.mode = ViewMode::Edit;
                }
            }
        }
    }

    pub fn search_vault(&mut self) {
        let Some(root) = self.vault.clone() else {
            self.search.hits.clear();
            return;
        };
        let filter = search::Filter {
            folder: "",
            scope: self.search.scope,
        };
        self.search.hits = search::search(&root, &self.entries, &self.search.query, &filter);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;
        self.autosave_tick();
        if self.last_scan.elapsed() > Duration::from_secs(3) {
            self.last_scan = Instant::now();
            self.check_external_changes();
        }
        self.handle_shortcuts(ctx);

        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::F)) {
            self.recompute_backlinks();
        }

        if let Some(url) = self.pending_url.take() {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }

        crate::ui::titlebar::show(self, ui);
        crate::ui::menubar::show(self, ui);
        crate::ui::status::show(self, ui);

        if self.panels.left {
            crate::ui::sidebar::show(self, ui);
        }
        if self.panels.right {
            crate::ui::context_panel::show(self, ui);
        }
        crate::ui::workspace::show(self, ui);

        crate::ui::palette::show(self, ctx);
        crate::ui::dialogs::show(self, ctx);
        crate::ui::toasts::show(self, ctx);

        // Keep the caret animation smooth without repainting the world.
        if self.tabs.iter().any(|tab| tab.is_dirty()) {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist_session();
        self.persist_settings();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_titles_strip_the_extension() {
        let tab = Tab::new("notes/Hello World.md", String::new(), 0);
        assert_eq!(tab.title(), "Hello World");
        assert_eq!(tab.folder(), "notes");
        assert_eq!(tab.file_name(), "Hello World.md");
    }

    #[test]
    fn editing_marks_a_tab_dirty_and_saving_clears_it() {
        let mut tab = Tab::new("a.md", "# a\n".into(), 0);
        assert!(!tab.is_dirty());
        tab.text.push_str("more");
        tab.on_edited();
        assert!(tab.is_dirty());
        assert_eq!(tab.save, SaveState::Dirty);
        tab.mark_saved();
        assert!(!tab.is_dirty());
        assert_eq!(tab.save, SaveState::Saved);
    }

    #[test]
    fn json_tabs_expose_a_validator_and_tree() {
        let mut tab = Tab::new("d.json", r#"{"a":1}"#.into(), 0);
        assert!(tab.json.is_valid());
        assert!(tab.validation.is_none());
        tab.text = "{\"a\": }".into();
        tab.on_edited();
        assert!(!tab.json.is_valid());
        assert_eq!(tab.validation.as_ref().unwrap().line, 1);
    }

    #[test]
    fn mdx_tabs_report_diagnostics() {
        let mut tab = Tab::new("x.mdx", "<Card>\n".into(), 0);
        tab.on_edited();
        assert!(tab.mdx.has_errors());
    }

    #[test]
    fn yaml_validation_uses_serde_yaml() {
        let mut tab = Tab::new("x.yaml", "a: 1\nb: 2\n".into(), 0);
        assert!(tab.validation.is_none());
        tab.text = "a: [1,\n".into();
        tab.on_edited();
        assert!(tab.validation.is_some());
    }

    #[test]
    fn default_mode_follows_the_document_capabilities() {
        let md = Tab::new("a.md", String::new(), 0);
        assert_eq!(md.default_mode(), ViewMode::Split);
        let json = Tab::new("a.json", "{}".into(), 0);
        assert_eq!(json.default_mode(), ViewMode::Edit);
    }
}
