//! The left sidebar: vault explorer and vault search.
//!
//! The tree is built from the cached vault scan, so expanding a folder is a
//! memory operation and only the scan itself touches the disk.

use std::collections::HashMap;

use egui::{Align, FontId, Layout, Sense, Vec2};

use crate::app::{App, PaletteMode, Prompt, PromptAction};
use crate::documents::{self, Kind};
use crate::theme::{icons, space, Icon};
use crate::ui::widgets;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let width = app.panels.left_width;
    let ctx = ui.ctx().clone();
    let ctx = &ctx;

    egui::Panel::left("nc-sidebar")
        .resizable(true)
        .default_size(width)
        .size_range(space::SIDEBAR_MIN..=space::SIDEBAR_MAX)
        .frame(
            egui::Frame::new()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(ui, |ui| {
            header(app, ui);
            ui.add_space(6.0);
            filter_field(app, ui);
            ui.add_space(6.0);
            widgets::divider(ui, &theme);
            ui.add_space(4.0);

            let height = (ui.available_height() - 26.0).max(80.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(height)
                .show(ui, |ui| {
                    if !app.search.query.trim().is_empty() {
                        search_results(app, ui);
                    } else {
                        tree(app, ui);
                    }
                });

            ui.add_space(2.0);
            footer(app, ui);
        });

    app.panels.left_width = ctx
        .memory(|m| m.data.get_temp::<f32>(crate::ui::id("nc-sidebar-width")))
        .unwrap_or(width);
}

fn header(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Explorer")
                .font(FontId::proportional(11.0))
                .color(p.fg_muted),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::icon_button(ui, &theme, Icon::Refresh, false, "Rescan vault").clicked() {
                app.refresh_vault();
            }
            if widgets::icon_button(ui, &theme, Icon::Folder, false, "New folder").clicked() {
                app.prompt = Some(Prompt {
                    title: "New folder".into(),
                    label: "Folder name".into(),
                    value: String::new(),
                    placeholder: "Research".into(),
                    confirm_label: "Create".into(),
                    action: PromptAction::NewFolder { dir: String::new() },
                    focus: false,
                });
            }
            if widgets::icon_button(ui, &theme, Icon::Plus, false, "New document").clicked() {
                app.new_menu = Some(egui::pos2(
                    ui.cursor().left() - 150.0,
                    ui.cursor().top() + 24.0,
                ));
            }
        });
    });
}

fn filter_field(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    // The single field filters the tree, and doubles as vault search when the
    // search toggle is on.
    let mut text = if app.search.opened_folder {
        app.search.query.clone()
    } else {
        app.explorer.filter.clone()
    };
    egui::Frame::new()
        .fill(p.raised)
        .corner_radius(egui::CornerRadius::same(crate::theme::radius::SM))
        .stroke(egui::Stroke::new(1.0, p.border))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::show(ui, Icon::Search, 11.0, p.fg_faint);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .hint_text(if app.search.opened_folder {
                            "Search the vault…"
                        } else {
                            "Filter files…"
                        })
                        .frame(egui::Frame::NONE)
                        .desired_width(ui.available_width() - 54.0)
                        .font(FontId::proportional(11.5)),
                );
                if response.changed() {
                    if app.search.opened_folder {
                        app.search.query = text.clone();
                        app.search_vault();
                    } else {
                        app.explorer.filter = text.clone();
                    }
                }
                let search_active = app.search.opened_folder;
                if widgets::icon_button(ui, &theme, Icon::Search, search_active, "Search contents")
                    .clicked()
                {
                    app.search.opened_folder = !app.search.opened_folder;
                    if !search_active {
                        app.search.query = text.clone();
                        if !text.trim().is_empty() {
                            app.search_vault();
                        }
                    }
                }
                let scope_label = app.search.scope.label();
                if widgets::icon_button(ui, &theme, Icon::List, false, "Filter by type").clicked() {
                    app.search.scope = match app.search.scope {
                        crate::search::Scope::All => crate::search::Scope::Prose,
                        crate::search::Scope::Prose => crate::search::Scope::Data,
                        crate::search::Scope::Data => crate::search::Scope::Code,
                        crate::search::Scope::Code => crate::search::Scope::All,
                    };
                    if !app.search.query.trim().is_empty() {
                        app.search_vault();
                    }
                }
                ui.label(
                    egui::RichText::new(scope_label)
                        .font(FontId::proportional(9.0))
                        .color(p.fg_faint),
                );
            });
        });
}

fn search_results(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    if app.search.hits.is_empty() {
        widgets::empty_state(
            ui,
            &theme,
            Icon::Search,
            "No matches",
            "Try another word or widen the filter.",
        );
        return;
    }
    let hits = app.search.hits.clone();
    for hit in hits.iter().take(200) {
        let selected = app.active_tab().is_some_and(|tab| tab.path == hit.path);
        let response = widgets::row(ui, &theme, false, 40.0, |ui| {
            ui.vertical(|ui| {
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(format!("{}:{}", hit.path, hit.line))
                        .font(FontId::proportional(10.5))
                        .color(if selected { p.accent } else { p.fg_secondary }),
                );
                ui.label(
                    egui::RichText::new(hit.snippet.trim())
                        .font(FontId::proportional(10.0))
                        .color(p.fg_faint),
                );
            });
        });
        if response.clicked() {
            app.open_path(&hit.path, false);
            if let Some(tab) = app.active_tab_mut() {
                tab.goto_line = Some(hit.line);
                tab.mode = crate::app::ViewMode::Edit;
            }
        }
    }
}

fn tree(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    if app.vault.is_none() {
        widgets::empty_state(
            ui,
            &theme,
            Icon::FolderOpen,
            "No vault open",
            "Open a folder to start writing.",
        );
        if widgets::button(ui, &theme, "Open folder…", true).clicked() {
            app.pick_vault();
        }
        return;
    }

    let filter = app.explorer.filter.trim().to_lowercase();
    if !filter.is_empty() {
        let entries: Vec<_> = app
            .entries
            .iter()
            .filter(|entry| !entry.is_dir && entry.path.to_lowercase().contains(&filter))
            .cloned()
            .collect();
        if entries.is_empty() {
            widgets::empty_state(
                ui,
                &theme,
                Icon::Search,
                "Nothing matches",
                "Clear the filter.",
            );
            return;
        }
        for entry in entries {
            if file_row(app, ui, &entry.path, &entry.name, 0).clicked() {
                app.open_path(&entry.path, false);
            }
        }
        return;
    }

    // Build parent → children map once per frame.
    // Clone the scan so the rows can call back into `app` while drawing.
    let entries: Vec<crate::filesystem::FileEntry> = app.entries.clone();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    let mut roots: Vec<String> = Vec::new();
    for entry in &entries {
        match entry.path.rsplit_once('/') {
            Some((parent, _)) => children
                .entry(parent.to_string())
                .or_default()
                .push(entry.path.clone()),
            None => roots.push(entry.path.clone()),
        }
    }
    let by_path: HashMap<String, crate::filesystem::FileEntry> =
        entries.into_iter().map(|e| (e.path.clone(), e)).collect();

    let visible = app.explorer.expanded.clone();
    let mut to_open: Option<String> = None;
    let mut to_toggle: Option<String> = None;

    {
        let mut stack: Vec<(String, usize)> = roots
            .iter()
            .rev()
            .map(|path| (path.clone(), 0usize))
            .collect();
        while let Some((path, depth)) = stack.pop() {
            let Some(entry) = by_path.get(&path) else {
                continue;
            };
            let expanded = visible.contains(&path);
            let name = entry.name.clone();
            let is_dir = entry.is_dir;

            let response = if is_dir {
                folder_row(app, ui, &theme, &path, &name, depth, expanded)
            } else {
                file_row(app, ui, &path, &name, depth)
            };
            if response.clicked() {
                if is_dir {
                    to_toggle = Some(path.clone());
                } else {
                    to_open = Some(path.clone());
                }
            }
            if response.secondary_clicked() {
                app.context_menu = Some((response.rect.left_bottom(), path.clone(), is_dir));
            }
            if is_dir && expanded {
                if let Some(kids) = children.get(&path) {
                    for kid in kids.iter().rev() {
                        stack.push((kid.clone(), depth + 1));
                    }
                }
            }
        }
        let _ = p;
    }

    if let Some(path) = to_toggle {
        if !app.explorer.expanded.remove(&path) {
            app.explorer.expanded.insert(path);
        }
    }
    if let Some(path) = to_open {
        app.explorer.selected = Some(path.clone());
        app.open_path(&path, false);
    }
}

fn folder_row(
    _app: &mut App,
    ui: &mut egui::Ui,
    theme: &crate::theme::Theme,
    path: &str,
    name: &str,
    depth: usize,
    expanded: bool,
) -> egui::Response {
    let p = theme.palette.clone();
    widgets::row(ui, theme, false, 24.0, |ui| {
        ui.add_space(depth as f32 * 12.0);
        icons::show(
            ui,
            if expanded {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            },
            11.0,
            p.fg_faint,
        );
        icons::show(
            ui,
            if expanded {
                Icon::FolderOpen
            } else {
                Icon::Folder
            },
            13.0,
            p.fg_muted,
        );
        ui.label(
            egui::RichText::new(name)
                .font(FontId::proportional(12.0))
                .color(p.fg_secondary),
        );
    })
    .on_hover_text(path)
}

fn file_row(
    app: &mut App,
    ui: &mut egui::Ui,
    path: &str,
    name: &str,
    depth: usize,
) -> egui::Response {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let is_active = app.active_tab().is_some_and(|tab| tab.path == path);
    let is_dirty = app
        .tabs
        .iter()
        .any(|tab| tab.path == path && tab.is_dirty());
    let kind = Kind::from_path(path);
    let muted = !documents::is_supported_document(path);

    let response = widgets::row(ui, &theme, is_active, 24.0, |ui| {
        ui.add_space(depth as f32 * 12.0 + 14.0);
        icons::show(
            ui,
            Icon::FileText,
            if muted { 10.0 } else { 11.0 },
            if is_active { p.accent } else { p.fg_faint },
        );
        ui.label(
            egui::RichText::new(name)
                .font(FontId::proportional(12.0))
                .color(if is_active {
                    p.fg
                } else if muted {
                    p.fg_faint
                } else {
                    p.fg_secondary
                }),
        );
        if is_dirty {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 2.5, p.accent);
        }
        let _ = kind;
    });
    response.on_hover_text(path)
}

fn footer(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let count = app
        .entries
        .iter()
        .filter(|entry| !entry.is_dir && documents::is_supported_document(&entry.name))
        .count();
    ui.horizontal(|ui| {
        icons::show(ui, Icon::FileText, 9.0, p.fg_faint);
        ui.label(
            egui::RichText::new(format!(
                "{count} {}",
                if count == 1 { "document" } else { "documents" }
            ))
            .font(FontId::proportional(9.5))
            .color(p.fg_faint),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.vault.is_some()
                && widgets::icon_button(ui, &theme, Icon::Command, false, "Command palette")
                    .clicked()
            {
                app.palette.open = true;
                app.palette.mode = PaletteMode::Commands;
                app.palette.query.clear();
            }
        });
    });
}
