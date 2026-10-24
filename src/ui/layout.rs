// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use crate::ui;
use eframe::egui;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    // top bar
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("NoteCraft");
            ui.separator();
            if ui.button("Open Vault").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let _ = state.open_vault(path);
                }
            }
            if let Some(root) = &state.vault_root {
                ui.label(root.display().to_string());
            } else {
                ui.label("No vault open");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    state.show_settings = true;
                }
                if ui.button("Search").clicked() {
                    state.show_search = true;
                }
                if ui.button("Palette").clicked() {
                    state.show_command_palette = true;
                }
            });
        });
    });

    // status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tab_status = state
                .active_tab_ref()
                .map(|t| match t.save_status {
                    crate::app::SaveStatus::Saved => "Saved",
                    crate::app::SaveStatus::Modified => "Modified",
                    crate::app::SaveStatus::Saving => "Saving...",
                    crate::app::SaveStatus::Error => "Save error",
                })
                .unwrap_or("No note");
            ui.label(tab_status);
            ui.separator();
            ui.label(&state.status_message);
        });
    });

    // sidebar
    if state.sidebar_open {
        egui::SidePanel::left("sidebar")
            .default_width(state.sidebar_width)
            .resizable(true)
            .show(ctx, |ui| {
                ui::explorer::render(ui, state);
            });
    }

    // central
    egui::CentralPanel::default().show(ctx, |ui| {
        ui::tabs::render(ui, state);
        // editor area
        match state.mode {
            crate::app::EditorMode::Edit => {
                ui::editor::render(ui, state);
            }
            crate::app::EditorMode::Split => {
                ui.columns(2, |cols| {
                    ui::editor::render(&mut cols[0], state);
                    ui::preview::render(&mut cols[1], state);
                });
            }
            crate::app::EditorMode::Read => {
                ui::preview::render(ui, state);
            }
        }
    });

    // floating windows
    if state.show_search {
        ui::search::render(ctx, state);
    }
    if state.show_command_palette {
        ui::command_palette::render(ctx, state);
    }
    if state.show_settings {
        ui::settings::render(ctx, state);
    }
    // find bar is rendered inside editor
    // templates window
    if state.show_templates {
        ui::templates::render(ctx, state);
    }
    // properties panel? doc properties in side? We'll render in preview/editor side maybe.
    // confirm delete dialog
    if state.confirm_delete.is_some() {
        ui::explorer::render_confirm_delete(ctx, state);
    }
    // creating dialog
    if state.creating.is_some() {
        ui::explorer::render_create_dialog(ctx, state);
    }
    if state.renaming.is_some() {
        ui::explorer::render_rename_dialog(ctx, state);
    }
}
