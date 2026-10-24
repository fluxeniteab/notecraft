// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use eframe::egui;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    let mut show = state.show_templates;
    egui::Window::new("Templates")
        .open(&mut show)
        .show(ctx, |ui| {
            state.refresh_templates();
            for t in state.templates.clone() {
                if ui.button(t.display().to_string()).clicked() {
                    state.apply_template_to_new_note(&t);
                    state.show_templates = false;
                }
            }
            ui.separator();
            if ui.button("Insert into current note").clicked() {
                // pick first template
                if let Some(t) = state.templates.first().cloned() {
                    state.insert_template_into_current(&t);
                }
            }
            if ui.button("Create template (current content)").clicked() {
                if let Some(tab) = state.active_tab_ref() {
                    let name = format!("template-{}.md", state.templates.len() + 1);
                    let path = state.templates_dir.join(name);
                    let _ = std::fs::write(path, &tab.content);
                    state.refresh_templates();
                }
            }
        });
    state.show_templates = show;
}
