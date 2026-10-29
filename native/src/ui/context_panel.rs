//! The right context panel.
//!
//! Tabs mirror the archived application: Outline, Properties, Links, Data and
//! History. Only the tabs that make sense for the active document are offered.

use egui::{Align, FontId, Layout, Sense, Vec2};

use crate::app::{App, ContextTab, ViewMode};
use crate::documents::Kind;
use crate::theme::{icons, space, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let width = app.panels.right_width;
    let ctx = ui.ctx().clone();
    let ctx = &ctx;

    egui::Panel::right("nc-context")
        .resizable(true)
        .default_size(width)
        .size_range(240.0..=460.0)
        .frame(
            egui::Frame::new()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(ui, |ui| {
            tabs(app, ui);
            ui.add_space(6.0);
            crate::ui::widgets::divider(ui, &theme);
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match app.context {
                    ContextTab::Outline => outline(app, ui),
                    ContextTab::Properties => properties(app, ui),
                    ContextTab::Links => links(app, ui),
                    ContextTab::Data => data(app, ui),
                    ContextTab::History => history(app, ui),
                });
        });

    let _ = space::CONTEXT_DEFAULT;
    if let Some(width) = ctx.memory(|m| m.data.get_temp::<f32>(crate::ui::id("nc-context-width"))) {
        app.panels.right_width = width;
    }
}

fn tabs(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let kind = app.active_tab().map(|tab| tab.kind);
    let available: Vec<(ContextTab, &str)> = match kind {
        Some(Kind::Json) | Some(Kind::Yaml) => vec![
            (ContextTab::Data, "Data"),
            (ContextTab::Properties, "Meta"),
            (ContextTab::History, "History"),
        ],
        Some(_) => vec![
            (ContextTab::Outline, "Outline"),
            (ContextTab::Properties, "Props"),
            (ContextTab::Links, "Links"),
            (ContextTab::History, "History"),
        ],
        None => vec![(ContextTab::History, "History")],
    };
    if !available.iter().any(|(tab, _)| *tab == app.context) {
        app.context = available[0].0;
    }

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (tab, label) in available {
            let selected = app.context == tab;
            let font = FontId::proportional(11.0);
            let galley =
                ui.painter()
                    .layout_no_wrap(label.to_string(), font.clone(), p.fg_secondary);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(galley.size().x + 16.0, 22.0), Sense::click());
            if selected {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(crate::theme::radius::XS),
                    p.hover,
                );
            } else if response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(crate::theme::radius::XS),
                    p.raised,
                );
            }
            ui.painter().galley(
                rect.center() - galley.size() / 2.0,
                galley,
                if selected { p.fg } else { p.fg_muted },
            );
            if response.clicked() {
                app.context = tab;
                if tab == ContextTab::Links {
                    app.recompute_backlinks();
                }
            }
        }
    });
}

fn outline(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let Some(index) = app.active else {
        crate::ui::widgets::empty_state(
            ui,
            &theme,
            Icon::List,
            "No document",
            "Open a note to see its outline.",
        );
        return;
    };
    if app.tabs[index].outline.is_empty() {
        crate::ui::widgets::empty_state(
            ui,
            &theme,
            Icon::Heading,
            "No headings",
            "Add `#` headings to build an outline.",
        );
        return;
    }
    let headings = app.tabs[index].outline.clone();
    let mut goto: Option<usize> = None;
    for heading in &headings {
        let indent = (heading.level.saturating_sub(1)) as f32 * 10.0;
        let response = crate::ui::widgets::row(ui, &theme, false, 22.0, |ui| {
            ui.add_space(indent);
            ui.label(
                egui::RichText::new(&heading.text)
                    .font(FontId::proportional(if heading.level <= 2 {
                        11.5
                    } else {
                        11.0
                    }))
                    .color(if heading.level <= 2 {
                        p.fg_secondary
                    } else {
                        p.fg_muted
                    }),
            );
        });
        if response.clicked() {
            goto = Some(heading.line);
        }
    }
    if let Some(line) = goto {
        if let Some(tab) = app.tabs.get_mut(index) {
            tab.goto_line = Some(line);
            tab.mode = ViewMode::Edit;
        }
    }
}

fn properties(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let Some(tab) = app.active_tab() else {
        return;
    };
    let props = crate::links::properties(&tab.text);
    let tags = crate::links::tags(&tab.text);

    if props.is_empty() && tags.is_empty() {
        crate::ui::widgets::empty_state(
            ui,
            &theme,
            Icon::Tag,
            "No metadata",
            "Add a YAML frontmatter block for properties and tags.",
        );
        return;
    }

    if !props.is_empty() {
        crate::ui::widgets::section_label(ui, &theme, "Properties");
        ui.add_space(4.0);
        for (key, value) in &props {
            ui.horizontal_top(|ui| {
                ui.label(
                    egui::RichText::new(key)
                        .font(FontId::proportional(10.5))
                        .color(p.fg_faint),
                );
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(value)
                            .font(FontId::proportional(11.0))
                            .color(p.fg_secondary),
                    );
                });
            });
        }
        ui.add_space(8.0);
    }
    if !tags.is_empty() {
        crate::ui::widgets::section_label(ui, &theme, "Tags");
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for tag in tags {
                crate::ui::widgets::chip(
                    ui,
                    &theme,
                    &format!("#{tag}"),
                    crate::ui::widgets::Tone::Neutral,
                );
            }
        });
    }
}

fn links(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    if app.backlinks.is_empty() {
        app.recompute_backlinks();
    }
    crate::ui::widgets::section_label(ui, &theme, "Backlinks");
    ui.add_space(4.0);
    if app.backlinks.is_empty() {
        ui.label(
            egui::RichText::new("Nothing links here yet.")
                .font(FontId::proportional(11.0))
                .color(p.fg_faint),
        );
    }
    let mut open: Option<String> = None;
    for path in app.backlinks.clone() {
        let response = crate::ui::widgets::row(ui, &theme, false, 22.0, |ui| {
            icons::show(ui, Icon::Link, 10.0, p.fg_faint);
            ui.label(
                egui::RichText::new(path.clone())
                    .font(FontId::proportional(11.0))
                    .color(p.fg_secondary),
            );
        });
        if response.clicked() {
            open = Some(path);
        }
    }
    if let Some(path) = open {
        app.open_path(&path, false);
    }

    ui.add_space(10.0);
    crate::ui::widgets::section_label(ui, &theme, "Outgoing links");
    ui.add_space(4.0);
    if let Some(tab) = app.active_tab() {
        let outgoing = crate::links::wiki_links(&tab.text);
        if outgoing.is_empty() {
            ui.label(
                egui::RichText::new("No wiki links in this note.")
                    .font(FontId::proportional(11.0))
                    .color(p.fg_faint),
            );
        } else {
            for link in outgoing {
                ui.label(
                    egui::RichText::new(format!("[[{}]]", link.target))
                        .font(FontId::monospace(10.0))
                        .color(p.fg_muted),
                );
            }
        }
    }
}

fn data(app: &mut App, ui: &mut egui::Ui) {
    let Some(index) = app.active else {
        return;
    };
    match app.tabs[index].kind {
        Kind::Json => crate::ui::json_tree::show(app, ui, index),
        _ => {
            let theme = app.theme.clone();
            let text = app.tabs[index].text.clone();
            let kind = app.tabs[index].kind;
            let path = app.tabs[index].path.clone();
            crate::ui::editor::code_view(
                ui,
                &theme,
                crate::ui::id(("nc-context-code", path)),
                &text,
                kind,
            );
        }
    }
}

fn history(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let Some(root) = app.vault.clone() else {
        return;
    };
    let Some(tab) = app.active_tab() else {
        return;
    };
    let prefix = tab.path.replace('/', "__");
    let dir = root.join(".notecraft").join("history");
    let mut snaps: Vec<(String, std::path::PathBuf)> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&prefix) && name.ends_with(".snap") {
                        Some((name, entry.path()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    snaps.sort_by(|a, b| b.0.cmp(&a.0));

    if snaps.is_empty() {
        crate::ui::widgets::empty_state(
            ui,
            &theme,
            Icon::History,
            "No snapshots yet",
            "A snapshot is written each time you save.",
        );
        return;
    }

    let mut restore: Option<std::path::PathBuf> = None;
    for (name, path) in &snaps {
        let stamp = name
            .rsplit_once('.')
            .map(|(_, stamp)| stamp.trim_end_matches(".snap").to_string())
            .unwrap_or_else(|| name.clone());
        let response = crate::ui::widgets::row(ui, &theme, false, 22.0, |ui| {
            icons::show(ui, Icon::History, 10.0, p.fg_faint);
            ui.label(
                egui::RichText::new(stamp.replace('T', " "))
                    .font(FontId::monospace(10.0))
                    .color(p.fg_secondary),
            );
        });
        if response.clicked() {
            restore = Some(path.clone());
        }
    }

    if let Some(path) = restore {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                if let Some(tab) = app.active_tab_mut() {
                    tab.text = text;
                    tab.on_edited();
                    tab.mode = ViewMode::Edit;
                }
                app.toast(
                    "Snapshot restored into the editor",
                    Some("Save to keep it".into()),
                    false,
                );
            }
            Err(err) => app.toast_error("Could not read the snapshot", err.to_string()),
        }
    }
}
