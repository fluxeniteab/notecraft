// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use eframe::egui;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    let mut show = state.show_search;
    egui::Window::new("Search vault")
        .open(&mut show)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.search_query);
                if ui.button("Search").clicked() {
                    if let Some(root) = &state.vault_canonical {
                        state.search_index.rebuild(root);
                    }
                    state.search_results = state.search_index.search(&state.search_query);
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for res in state.search_results.clone() {
                    if ui
                        .selectable_label(false, format!("{}", res.path.display()))
                        .clicked()
                    {
                        state.open_note(res.path.clone());
                    }
                    ui.label(res.snippet.clone());
                    ui.separator();
                }
            });
        });
    state.show_search = show;
}
