// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::{AppState, SaveStatus};
use eframe::egui;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    if state.show_find {
        ui.horizontal(|ui| {
            ui.label("Find:");
            ui.text_edit_singleline(&mut state.find_query);
            ui.label("Replace:");
            ui.text_edit_singleline(&mut state.find_replace);
            if ui.button("Replace").clicked() {
                if let Some(tab) = state.active_tab_mut() {
                    if !state.find_query.is_empty() {
                        tab.content = tab.content.replace(&state.find_query, &state.find_replace);
                        tab.modified = tab.content != tab.saved_content;
                        tab.save_status = if tab.modified { SaveStatus::Modified } else { SaveStatus::Saved };
                        tab.last_edit = Some(std::time::Instant::now());
                    }
                }
            }
            if ui.button("Close").clicked() {
                state.show_find = false;
            }
        });
        ui.separator();
    }

    let Some(idx) = state.active_tab else {
        ui.centered_and_justified(|ui| {
            ui.label("Open or create a note to start editing.");
        });
        return;
    };

    // ensure idx valid
    if idx >= state.tabs.len() {
        return;
    }

    // handle properties toggle
    // Render text edit
    let tab = &mut state.tabs[idx];
    let before = tab.content.clone();
    let font = egui::FontId::monospace(14.0);
    let mut edit = egui::TextEdit::multiline(&mut tab.content)
        .font(font)
        .desired_width(f32::INFINITY)
        .lock_focus(true);
    if state.settings.show_line_numbers {
        edit = edit.numbered_lines();
    }
    let resp = ui.add_sized(ui.available_size(), edit);
    if resp.changed() {
        tab.modified = tab.content != tab.saved_content;
        tab.save_status = if tab.modified { SaveStatus::Modified } else { SaveStatus::Saved };
        tab.last_edit = Some(std::time::Instant::now());
    }
    // update title if needed
    let _ = before;
    // properties panel below? We'll leave to separate.
}
