//! The central workspace: tab strip, document toolbar and the editor/reading
//! split. This is where document type decides what is shown — a JSON file never
//! gets Markdown statistics or a Markdown preview.

use egui::{Align, FontId, Layout, Sense, Vec2};

use crate::app::{App, ViewMode};
use crate::theme::{icons, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(p.bg))
        .show(ui, |ui| {
            crate::ui::tabs::show(app, ui);
            if app.active.is_none() {
                empty(app, ui);
                return;
            }
            toolbar(app, ui);
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
            ui.painter()
                .line_segment([rect.left_center(), rect.right_center()], p.hairline());
            conflict_banner(app, ui);
            content(app, ui);
        });
}

fn empty(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    crate::ui::widgets::empty_state(
        ui,
        &theme,
        Icon::FileText,
        "No document open",
        "Pick a file in the explorer, or press Ctrl N for a new note.",
    );
    if app.vault.is_some() {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            if crate::ui::widgets::button(ui, &theme, "New Markdown note", true).clicked() {
                app.new_document("", crate::documents::Kind::Markdown);
            }
        });
    }
}

fn toolbar(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();

    egui::Frame::new()
        .fill(p.surface)
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (kind, folder, dirty, path) = match app.active_tab() {
                    Some(tab) => (
                        tab.capabilities().label,
                        tab.folder(),
                        tab.is_dirty(),
                        tab.path.clone(),
                    ),
                    None => return,
                };

                crate::ui::widgets::chip(ui, &theme, kind, crate::ui::widgets::Tone::Neutral);
                if !folder.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("{folder}/"))
                            .font(FontId::proportional(10.5))
                            .color(p.fg_faint),
                    );
                }
                let name = path.rsplit('/').next().unwrap_or(&path).to_string();
                ui.label(
                    egui::RichText::new(name)
                        .font(FontId::proportional(11.5))
                        .color(p.fg_secondary),
                )
                .on_hover_text(path.clone());
                if dirty {
                    ui.label(
                        egui::RichText::new("•")
                            .font(FontId::proportional(12.0))
                            .color(p.accent),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if crate::ui::widgets::icon_button(
                        ui,
                        &theme,
                        Icon::Save,
                        false,
                        "Save  Ctrl S",
                    )
                    .clicked()
                    {
                        app.save_active();
                    }
                    let more = crate::ui::widgets::icon_button(
                        ui,
                        &theme,
                        Icon::GripVertical,
                        false,
                        "More actions",
                    );
                    if more.clicked() {
                        app.panels.right = !app.panels.right;
                        app.persist_session();
                    }
                    ui.add_space(4.0);

                    let mode = app
                        .active_tab()
                        .map(|tab| tab.mode)
                        .unwrap_or(ViewMode::Edit);
                    let preview = app
                        .active_tab()
                        .is_some_and(|tab| tab.capabilities().preview);
                    let options: Vec<(ViewMode, &str)> = if preview {
                        vec![
                            (ViewMode::Edit, "Edit"),
                            (ViewMode::Split, "Split"),
                            (ViewMode::Read, "Read"),
                        ]
                    } else {
                        vec![(ViewMode::Edit, "Edit"), (ViewMode::Read, "Inspect")]
                    };
                    if let Some(next) = crate::ui::widgets::segmented(ui, &theme, mode, &options) {
                        if let Some(tab) = app.active_tab_mut() {
                            tab.mode = next;
                        }
                    }

                    if let Some(tab) = app.active_tab() {
                        if tab.capabilities().stats == crate::documents::Stats::Prose {
                            ui.label(
                                egui::RichText::new(format!("{} words", tab.stats.words))
                                    .font(FontId::proportional(10.0))
                                    .color(p.fg_faint),
                            );
                        }
                    }
                });
            });
        });
}

fn conflict_banner(app: &mut App, ui: &mut egui::Ui) {
    let Some(tab) = app.active_tab() else {
        return;
    };
    if !tab.external_change {
        return;
    }
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let path = tab.path.clone();

    egui::Frame::new()
        .fill(p.warning.linear_multiply(0.14))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::show(ui, Icon::Alert, 12.0, p.warning);
                ui.label(
                    egui::RichText::new("This file changed on disk while you had unsaved edits.")
                        .font(FontId::proportional(11.0))
                        .color(p.fg),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if crate::ui::widgets::button(ui, &theme, "Keep mine", false).clicked() {
                        if let Some(tab) = app.active_tab_mut() {
                            tab.external_change = false;
                        }
                    }
                    if crate::ui::widgets::button(ui, &theme, "Reload from disk", true).clicked() {
                        app.reload_from_disk(&path);
                    }
                });
            });
        });
}

fn content(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let Some(index) = app.active else {
        return;
    };
    let mode = app.tabs[index].mode;
    let preview_capable = app.tabs[index].capabilities().preview;
    let height = ui.available_height();

    match mode {
        ViewMode::Edit => editor_pane(app, ui, index, height),
        ViewMode::Read => {
            if preview_capable {
                preview_pane(app, ui, index, height)
            } else {
                inspect_pane(app, ui, index, height)
            }
        }
        ViewMode::Split => {
            if !preview_capable {
                // Data and code documents split into source and structure.
                let total = ui.available_width();
                let left = (total * 0.56).max(220.0);
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(left, height),
                        Layout::top_down(Align::Min),
                        |ui| editor_pane(app, ui, index, height),
                    );
                    let (divider, _) =
                        ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
                    ui.painter()
                        .line_segment([divider.left_top(), divider.left_bottom()], p.hairline());
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), height),
                        Layout::top_down(Align::Min),
                        |ui| inspect_pane(app, ui, index, height),
                    );
                });
            } else {
                let total = ui.available_width();
                let left = (total * 0.55).max(240.0);
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(left, height),
                        Layout::top_down(Align::Min),
                        |ui| editor_pane(app, ui, index, height),
                    );
                    crate::ui::split_handle(ui, app);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), height),
                        Layout::top_down(Align::Min),
                        |ui| preview_pane(app, ui, index, height),
                    );
                });
            }
        }
    }
}

fn editor_pane(app: &mut App, ui: &mut egui::Ui, index: usize, height: f32) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), height),
        Layout::top_down(Align::Min),
        |ui| {
            ui.set_min_height(height);
            egui::Frame::new()
                .fill(p.bg)
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    let path = app.tabs[index].path.clone();
                    let id = crate::ui::editor_id(&path);
                    let show_numbers = app.settings.show_line_numbers;

                    // "Go to line" moves the caret before the widget is built.
                    if let Some(line) = app.tabs[index].goto_line.take() {
                        jump_to_line(ui.ctx(), &app.tabs[index], id, line);
                    }

                    let kind = app.tabs[index].kind;
                    let (changed, cursor) = {
                        let tab = &mut app.tabs[index];
                        let output = crate::ui::editor::text_editor(
                            ui,
                            &theme,
                            id,
                            &mut tab.text,
                            kind,
                            show_numbers,
                        );
                        (output.changed, (output.line, output.column))
                    };
                    if changed {
                        app.tabs[index].on_edited();
                    }
                    app.tabs[index].cursor = cursor;
                });
        },
    );
}

fn jump_to_line(ctx: &egui::Context, tab: &crate::app::Tab, id: egui::Id, line: usize) {
    let byte = crate::ui::editor::line_start(&tab.text, line);
    let char_index = tab.text[..byte].chars().count();
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(char_index),
        )));
    state.store(ctx, id);
    ctx.memory_mut(|memory| memory.request_focus(id));
}

fn preview_pane(app: &mut App, ui: &mut egui::Ui, index: usize, height: f32) {
    let theme = app.theme.clone();
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), height),
        Layout::top_down(Align::Min),
        |ui| {
            let revision = app.tabs[index].revision;
            let cache_id = crate::ui::id(("nc-preview-scroll", index));
            if revision == 0 {
                let _ = cache_id;
            }
            if app.tabs[index].kind == crate::documents::Kind::Mdx {
                mdx_banner(app, ui, index);
            }
            egui::ScrollArea::vertical()
                .id_salt(("nc-preview", index))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let text = app.tabs[index].text.clone();
                    if let Some(action) = crate::ui::preview::render(ui, &theme, &text) {
                        crate::ui::preview::handle_action(app, index, action);
                    }
                });
        },
    );
}

fn mdx_banner(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let report = &app.tabs[index].mdx;
    let summary = format!(
        "{} component(s) · {} expression(s) · not executed",
        report.components.len(),
        report.expressions
    );
    let (color, message) = match report.first_error() {
        Some(error) => (p.danger, error.describe()),
        None => (p.fg_faint, format!("MDX is rendered read-only: {summary}.")),
    };
    egui::Frame::new()
        .fill(color.linear_multiply(0.1))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::show(ui, Icon::Alert, 11.0, color);
                ui.label(
                    egui::RichText::new(message)
                        .font(FontId::proportional(10.5))
                        .color(if report.has_errors() {
                            p.danger
                        } else {
                            p.fg_muted
                        }),
                );
            });
        });
}

fn inspect_pane(app: &mut App, ui: &mut egui::Ui, index: usize, height: f32) {
    let theme = app.theme.clone();
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), height),
        Layout::top_down(Align::Min),
        |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| match app.tabs[index].kind {
                    crate::documents::Kind::Json => {
                        crate::ui::json_tree::show(app, ui, index);
                    }
                    _ => {
                        let text = app.tabs[index].text.clone();
                        let kind = app.tabs[index].kind;
                        let path = app.tabs[index].path.clone();
                        crate::ui::editor::code_view(
                            ui,
                            &theme,
                            crate::ui::id(("nc-code-view", path)),
                            &text,
                            kind,
                        );
                    }
                });
        },
    );
}
