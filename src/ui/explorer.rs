// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use crate::vault::operations::*;
use crate::vault::tree::{populate_children, FileNode};
use eframe::egui;
use std::path::PathBuf;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Files");
    ui.horizontal(|ui| {
        if ui.button("+ Note").clicked() {
            state.creating = Some(crate::app::CreateKind::Note);
            state.new_item_parent = state.vault_root.clone();
            state.new_name_input.clear();
        }
        if ui.button("+ Folder").clicked() {
            state.creating = Some(crate::app::CreateKind::Folder);
            state.new_item_parent = state.vault_root.clone();
            state.new_name_input.clear();
        }
        if ui.button("Refresh").clicked() {
            state.refresh_tree();
        }
    });
    ui.separator();
    if state.vault_root.is_none() {
        ui.label("Open a vault to see files.");
        return;
    }
    if let Some(tree) = &mut state.tree {
        render_node(ui, state, tree, 0);
    }
    // templates button
    ui.separator();
    if ui.button("Templates").clicked() {
        state.show_templates = true;
    }
}

fn render_node(ui: &mut egui::Ui, state: &mut AppState, node: &mut FileNode, depth: usize) {
    let indent = depth as f32 * 12.0;
    if node.is_dir {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let icon = if node.expanded { "▾" } else { "▸" };
            if ui.button(icon).clicked() {
                node.expanded = !node.expanded;
                if node.expanded && node.children.is_empty() {
                    let _ = populate_children(node);
                }
            }
            if ui.selectable_label(false, &node.name).clicked() {
                node.expanded = !node.expanded;
                if node.expanded && node.children.is_empty() {
                    let _ = populate_children(node);
                }
            }
            // context-ish buttons
            if ui.small_button("+").clicked() {
                state.creating = Some(crate::app::CreateKind::Note);
                state.new_item_parent = Some(node.path.clone());
                state.new_name_input.clear();
            }
            if ui.small_button("x").clicked() {
                state.confirm_delete = Some(node.path.clone());
            }
        });
        if node.expanded {
            // clone children indices to avoid borrow issues? We mutate state inside loop, but node borrow.
            // We'll iterate by index.
            let n = node.children.len();
            for i in 0..n {
                // need mutable access to child; do recursively with unsafe? Use index access.
                // We'll call a helper that takes &mut FileNode child.
                // Since node is &mut, we can get child mut.
                // But state also mut; okay separate.
                // Recursion:
                // We'll implement iterative via function with path? Simpler: recursive function takes &mut FileNode.
                // Already have node mutable; get child mut.
                // But loop borrow checker: node.children[i] borrow ends each iteration.
                // Call render_node on &mut node.children[i]
                // However node is already mutably borrowed for outer; nested mutable borrow of child is okay if not using node else.
                // We'll do:
            }
            // Because of borrow complexity, restructure: collect child paths first?
            // Let's just recurse by mutable index using a separate function that accepts &mut FileNode.
            for i in 0..node.children.len() {
                let child = &mut node.children[i];
                render_node(ui, state, child, depth + 1);
            }
        }
    } else {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let label = if state.active_tab_ref().and_then(|t| t.path.as_ref()) == Some(&node.path)
            {
                format!("● {}", node.name)
            } else {
                node.name.clone()
            };
            if ui.selectable_label(false, label).clicked() {
                state.open_note(node.path.clone());
            }
            if ui.small_button("rn").clicked() {
                state.renaming = Some(node.path.clone());
                state.rename_input = node.name.clone();
            }
            if ui.small_button("x").clicked() {
                state.confirm_delete = Some(node.path.clone());
            }
        });
    }
}

pub fn render_confirm_delete(ctx: &egui::Context, state: &mut AppState) {
    let path = state.confirm_delete.clone().unwrap();
    egui::Window::new("Confirm delete")
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(format!("Delete {}?", path.display()));
            ui.label("This cannot be undone.");
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    if let Some(root) = &state.vault_canonical {
                        let _ = delete_path(root, &path);
                    }
                    // close tabs referencing
                    let mut i = 0;
                    while i < state.tabs.len() {
                        if state.tabs[i].path.as_ref() == Some(&path) {
                            state.force_close_tab(i);
                        } else {
                            i += 1;
                        }
                    }
                    state.confirm_delete = None;
                    state.refresh_tree();
                    state.status_message = "Deleted".to_string();
                }
                if ui.button("Cancel").clicked() {
                    state.confirm_delete = None;
                }
            });
        });
}

pub fn render_create_dialog(ctx: &egui::Context, state: &mut AppState) {
    egui::Window::new("Create").show(ctx, |ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.new_name_input);
        if ui.button("Create").clicked() {
            let parent = state
                .new_item_parent
                .clone()
                .unwrap_or_else(|| state.vault_root.clone().unwrap_or_default());
            let root = state.vault_canonical.clone().unwrap_or_default();
            match state.creating {
                Some(crate::app::CreateKind::Note) => {
                    if let Ok(p) = create_note(&root, &parent, &state.new_name_input) {
                        state.refresh_tree();
                        state.open_note(p);
                    }
                }
                Some(crate::app::CreateKind::Folder) => {
                    let _ = create_dir(&root, &parent, &state.new_name_input);
                    state.refresh_tree();
                }
                None => {}
            }
            state.creating = None;
        }
        if ui.button("Cancel").clicked() {
            state.creating = None;
        }
    });
}

pub fn render_rename_dialog(ctx: &egui::Context, state: &mut AppState) {
    let path = state.renaming.clone().unwrap();
    egui::Window::new("Rename").show(ctx, |ui| {
        ui.label(path.display().to_string());
        ui.text_edit_singleline(&mut state.rename_input);
        if ui.button("Rename").clicked() {
            let root = state.vault_canonical.clone().unwrap_or_default();
            if let Ok(newp) = rename_path(&root, &path, &state.rename_input) {
                // update tabs
                for tab in &mut state.tabs {
                    if tab.path.as_ref() == Some(&path) {
                        tab.path = Some(newp.clone());
                        tab.title = newp.file_name().unwrap().to_string_lossy().to_string();
                    }
                }
                state.refresh_tree();
            }
            state.renaming = None;
        }
        if ui.button("Cancel").clicked() {
            state.renaming = None;
        }
    });
}
