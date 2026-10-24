// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use crate::commands::command_label;
use crate::commands::Command;
use eframe::egui;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::Window::new("Command Palette")
        .open(&mut state.show_command_palette)
        .show(ctx, |ui| {
            ui.text_edit_singleline(&mut state.command_filter);
            let filter = state.command_filter.to_lowercase();
            let commands = [
                Command::NewNote,
                Command::NewFolder,
                Command::Save,
                Command::CloseTab,
                Command::SwitchModeEdit,
                Command::SwitchModeSplit,
                Command::SwitchModeRead,
                Command::TogglePreview,
                Command::SearchVault,
                Command::SearchDocument,
                Command::OpenSettings,
                Command::RefreshExplorer,
            ];
            for cmd in commands {
                let label = command_label(cmd);
                if !filter.is_empty() && !label.to_lowercase().contains(&filter) {
                    continue;
                }
                if ui.button(label).clicked() {
                    state.run_command(cmd);
                    state.show_command_palette = false;
                }
            }
        });
}
