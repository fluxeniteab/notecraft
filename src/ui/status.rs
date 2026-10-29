//! The status bar.
//!
//! Only information that applies to the active document is shown: prose counts
//! words, data documents report lines and validation, code reports lines.

use egui::{Align, FontId, Layout};

use crate::app::{App, SaveState};
use crate::documents::{Stats, Validator};
use crate::theme::{icons, space, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();

    egui::Panel::bottom("nc-statusbar")
        .exact_size(space::STATUS)
        .frame(
            egui::Frame::new()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .inner_margin(egui::Margin::symmetric(10, 0)),
        )
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                item(
                    ui,
                    &p,
                    if app.vault.is_some() {
                        p.accent
                    } else {
                        p.fg_faint
                    },
                );

                let vault_name = app
                    .vault
                    .as_ref()
                    .and_then(|path| path.file_name())
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "No vault".into());
                ui.label(text(&vault_name, p.fg_muted));

                let documents = app
                    .entries
                    .iter()
                    .filter(|entry| {
                        !entry.is_dir && crate::documents::is_supported_document(&entry.name)
                    })
                    .count();
                if app.vault.is_some() {
                    icons::show(ui, Icon::FileText, 9.0, p.fg_faint);
                    ui.label(text(&format!("{documents} documents"), p.fg_faint));
                }

                if let Some(tab) = app.active_tab() {
                    ui.label(text("·", p.fg_faint));
                    ui.label(text(&tab.path, p.fg_faint));
                    let (label, color) = match &tab.save {
                        SaveState::Clean => ("Saved", p.fg_faint),
                        SaveState::Dirty => ("Unsaved changes", p.warning),
                        SaveState::Saved => ("Saved", p.fg_faint),
                        SaveState::SaveFailed(_) => ("Save failed", p.danger),
                    };
                    ui.label(text(&format!("· {label}"), color));
                    if tab.external_change {
                        ui.label(text("· changed on disk", p.warning));
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.label(text("Local only", p.fg_faint));
                    if let Some(tab) = app.active_tab() {
                        let caps = tab.capabilities();
                        ui.label(text(&format!("UTF-8 · {}", caps.label), p.fg_faint));

                        match caps.stats {
                            Stats::Prose if tab.kind.capabilities().metadata => {
                                ui.label(text(
                                    &format!(
                                        "{} words · {} chars",
                                        tab.stats.words, tab.stats.chars
                                    ),
                                    p.fg_faint,
                                ));
                            }
                            Stats::Data => {
                                ui.label(text(
                                    &format!(
                                        "{} lines · {} chars",
                                        tab.stats.lines, tab.stats.chars
                                    ),
                                    p.fg_faint,
                                ));
                            }
                            Stats::Code => {
                                ui.label(text(&format!("{} lines", tab.stats.lines), p.fg_faint));
                            }
                            _ => {}
                        }

                        if caps.validator == Validator::Json {
                            match &tab.validation {
                                None => {
                                    icons::show(ui, Icon::Check, 9.0, p.success);
                                    ui.label(text("Valid JSON", p.fg_faint));
                                }
                                Some(problem) => {
                                    icons::show(ui, Icon::Alert, 9.0, p.danger);
                                    ui.label(
                                        egui::RichText::new("Invalid JSON")
                                            .font(FontId::proportional(10.0))
                                            .color(p.danger),
                                    )
                                    .on_hover_text(format!(
                                        "line {}, column {}: {}",
                                        problem.line, problem.column, problem.message
                                    ));
                                }
                            }
                        }
                        if caps.validator == Validator::Yaml {
                            match &tab.validation {
                                None => {
                                    icons::show(ui, Icon::Check, 9.0, p.success);
                                    ui.label(text("Valid YAML", p.fg_faint));
                                }
                                Some(problem) => {
                                    icons::show(ui, Icon::Alert, 9.0, p.danger);
                                    ui.label(
                                        egui::RichText::new("Invalid YAML")
                                            .font(FontId::proportional(10.0))
                                            .color(p.danger),
                                    )
                                    .on_hover_text(format!(
                                        "line {}, column {}: {}",
                                        problem.line, problem.column, problem.message
                                    ));
                                }
                            }
                        }
                        if tab.kind == crate::documents::Kind::Mdx {
                            if tab.mdx.has_errors() {
                                icons::show(ui, Icon::Alert, 9.0, p.danger);
                                ui.label(
                                    egui::RichText::new("MDX error")
                                        .font(FontId::proportional(10.0))
                                        .color(p.danger),
                                );
                            } else {
                                ui.label(text("MDX ok · not executed", p.fg_faint));
                            }
                        }
                        // Caret position is meaningful for text editing only.
                        if tab.mode != crate::app::ViewMode::Read {
                            ui.label(text(
                                &format!("Ln {}, Col {}", tab.cursor.0, tab.cursor.1),
                                p.fg_secondary,
                            ));
                        }
                    }
                });
            });
        });
}

fn text(value: &str, color: egui::Color32) -> egui::RichText {
    egui::RichText::new(value)
        .font(FontId::proportional(10.0))
        .color(color)
}

fn item(ui: &mut egui::Ui, p: &crate::theme::Palette, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(5.0, 5.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 2.5, color);
    let _ = p;
}
