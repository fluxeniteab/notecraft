// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::{AppState, SaveStatus};
use eframe::egui;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal_wrapped(|ui| {
        // mode switcher
        for (label, mode) in [
            ("Edit", crate::app::EditorMode::Edit),
            ("Split", crate::app::EditorMode::Split),
            ("Read", crate::app::EditorMode::Read),
        ] {
            if ui.selectable_label(state.mode == mode, label).clicked() {
                state.mode = mode;
            }
        }
        ui.separator();
        for i in 0..state.tabs.len() {
            let title = state.tabs[i].title.clone();
            let dirty = state.tabs[i].is_dirty();
            let active = state.active_tab == Some(i);
            let label = if dirty { format!("{}*", title) } else { title };
            if ui.selectable_label(active, label).clicked() {
                state.active_tab = Some(i);
            }
            if ui.small_button("x").clicked() {
                state.close_tab(i);
                break;
            }
        }
        if state.tabs.is_empty() {
            ui.label("No open notes");
        }
    });
    ui.separator();
}
