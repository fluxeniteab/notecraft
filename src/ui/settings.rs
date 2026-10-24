// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use crate::config::save_settings;
use eframe::egui;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::Window::new("Settings")
        .open(&mut state.show_settings)
        .show(ctx, |ui| {
            ui.label("Autosave debounce (ms):");
            ui.add(egui::Slider::new(&mut state.settings.autosave_debounce_ms, 300..=5000));
            ui.checkbox(&mut state.settings.show_line_numbers, "Show line numbers");
            ui.label(format!("Templates dir: {}", state.templates_dir.display()));
            if ui.button("Save settings").clicked() {
                let _ = save_settings(&state.settings);
                state.status_message = "Settings saved".to_string();
            }
            if ui.button("Close").clicked() {
                state.show_settings = false;
            }
        });
}
