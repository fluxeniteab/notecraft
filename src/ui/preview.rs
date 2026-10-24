// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::app::AppState;
use crate::markdown::frontmatter::parse_frontmatter;
use crate::markdown::renderer::render_markdown;
use crate::markdown::tags::extract_inline_tags;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let Some(idx) = state.active_tab else {
        ui.label("No note selected.");
        return;
    };
    if idx >= state.tabs.len() {
        return;
    }
    let content = state.tabs[idx].content.clone();
    let parsed = parse_frontmatter(&content);
    // properties
    if state.doc_properties_visible {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label("Properties");
            if let Some(fm) = &parsed.frontmatter {
                if let Some(t) = &fm.title {
                    ui.label(format!("Title: {}", t));
                }
                if !fm.tags.is_empty() {
                    ui.label(format!("Tags: {}", fm.tags.join(", ")));
                }
                if !fm.aliases.is_empty() {
                    ui.label(format!("Aliases: {}", fm.aliases.join(", ")));
                }
                if let Some(c) = &fm.created {
                    ui.label(format!("Created: {}", c));
                }
                for (k, v) in &fm.extra {
                    ui.label(format!("{}: {:?}", k, v));
                }
            }
            let inline = extract_inline_tags(&parsed.body);
            if !inline.is_empty() {
                ui.label(format!("Inline tags: {}", inline.join(", ")));
            }
        });
        ui.separator();
    }
    // render markdown body
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_max_width(720.0);
        render_markdown(ui, &parsed.body);
    });
}
