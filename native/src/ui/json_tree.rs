//! The structured JSON viewer.
//!
//! Rows come from `json::flatten`, which honours each document's collapse state,
//! so opening a large file costs one traversal rather than one per visible row.
//! Values are only ever rendered as text.

use egui::{Align, FontId, Layout, Sense, Vec2};

use crate::app::App;
use crate::json::{self, ColorRole, NodeKind};
use crate::theme::{icons, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();

    if app.tabs[index].kind != crate::documents::Kind::Json {
        return;
    }

    if let Some(problem) = app.tabs[index].validation.clone() {
        egui::Frame::new()
            .fill(p.danger.linear_multiply(0.12))
            .corner_radius(egui::CornerRadius::same(crate::theme::radius::SM))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    icons::show(ui, Icon::Alert, 12.0, p.danger);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Invalid JSON at line {}, column {}",
                                problem.line, problem.column
                            ))
                            .font(FontId::proportional(11.5))
                            .color(p.danger),
                        );
                        ui.label(
                            egui::RichText::new(problem.message)
                                .font(FontId::monospace(10.0))
                                .color(p.fg_muted),
                        );
                        ui.label(
                            egui::RichText::new("The source is unchanged — fix it in Edit view.")
                                .font(FontId::proportional(10.0))
                                .color(p.fg_faint),
                        );
                    });
                });
            });
        ui.add_space(8.0);
        return;
    }

    let Some(root) = app.tabs[index].json.root.clone() else {
        ui.label(
            egui::RichText::new("Nothing to inspect yet.")
                .font(FontId::proportional(11.0))
                .color(p.fg_faint),
        );
        return;
    };

    // Toolbar.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "{} · {} root {}",
                "Structured view",
                root.as_object()
                    .map(|map| map.len())
                    .or_else(|| root.as_array().map(|arr| arr.len()))
                    .unwrap_or(0),
                app.tabs[index].json.shape.unwrap_or("value")
            ))
            .font(FontId::proportional(10.5))
            .color(p.fg_faint),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if crate::ui::widgets::button(ui, &theme, "Collapse", false).clicked() {
                let all: std::collections::HashSet<String> =
                    json::all_containers(&root).into_iter().collect();
                app.tabs[index].json_collapsed = all;
            }
            if crate::ui::widgets::button(ui, &theme, "Expand", false).clicked() {
                app.tabs[index].json_collapsed.clear();
            }
            let prefix = app.tabs[index].path.clone();
            let pointer_target = crate::ui::id(("nc-json-filter", prefix));
            let mut filter = ui
                .ctx()
                .memory(|m| m.data.get_temp::<String>(pointer_target))
                .unwrap_or_default();
            let response = ui.add(
                egui::TextEdit::singleline(&mut filter)
                    .hint_text("Filter keys…")
                    .desired_width(120.0)
                    .font(FontId::proportional(10.5)),
            );
            if response.changed() {
                ui.ctx()
                    .memory_mut(|m| m.data.insert_temp(pointer_target, filter.clone()));
            }
        });
    });
    ui.add_space(6.0);

    let filter = ui
        .ctx()
        .memory(|m| {
            m.data.get_temp::<String>(crate::ui::id((
                "nc-json-filter",
                app.tabs[index].path.clone(),
            )))
        })
        .unwrap_or_default()
        .to_lowercase();

    let collapsed = app.tabs[index].json_collapsed.clone();
    let rows = json::flatten(&root, &collapsed);

    let mut toggle: Option<String> = None;
    let mut copy_pointer: Option<String> = None;

    egui::ScrollArea::both()
        .id_salt(("nc-json-tree", index))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            for row in &rows {
                if !filter.is_empty()
                    && !row.key.to_lowercase().contains(&filter)
                    && !row.preview.to_lowercase().contains(&filter)
                {
                    continue;
                }
                let height = 20.0;
                let width = ui.available_width().max(240.0);
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
                if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(3), p.hover);
                }

                let mut x = rect.left() + 4.0 + row.depth as f32 * 14.0;
                if row.kind.is_container() {
                    let is_collapsed = collapsed.contains(&row.pointer);
                    icons::paint(
                        ui.painter(),
                        egui::Rect::from_center_size(
                            egui::pos2(x + 5.0, rect.center().y),
                            Vec2::splat(10.0),
                        ),
                        if is_collapsed {
                            Icon::ChevronRight
                        } else {
                            Icon::ChevronDown
                        },
                        p.fg_faint,
                        1.3,
                    );
                    if response.clicked() {
                        toggle = Some(row.pointer.clone());
                    }
                }
                x += 14.0;

                let key_color = match row.kind {
                    NodeKind::Object | NodeKind::Array => p.fg,
                    _ => p.fg_secondary,
                };
                ui.painter().text(
                    egui::pos2(x, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &row.key,
                    FontId::monospace(10.5),
                    key_color,
                );
                let key_width = ui
                    .painter()
                    .layout_no_wrap(row.key.clone(), FontId::monospace(10.5), key_color)
                    .size()
                    .x;
                x += key_width + 6.0;

                if row.kind.is_container() {
                    let summary = match row.kind {
                        NodeKind::Object => format!("{{…}} {}", row.children),
                        _ => format!("[…] {}", row.children),
                    };
                    ui.painter().text(
                        egui::pos2(x, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        summary,
                        FontId::monospace(10.0),
                        p.fg_faint,
                    );
                } else {
                    let color = match row.kind.color_role() {
                        ColorRole::Structure => p.fg_muted,
                        ColorRole::String => p.success,
                        ColorRole::Number => p.warning,
                        ColorRole::Bool => p.accent,
                        ColorRole::Null => p.fg_faint,
                    };
                    ui.painter().text(
                        egui::pos2(x, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &row.preview,
                        FontId::monospace(10.0),
                        color,
                    );
                }

                if response.secondary_clicked() {
                    copy_pointer = Some(row.pointer.clone());
                }
                let pointer = row.pointer.clone();
                response.on_hover_text(format!(
                    "{}\n{} · right-click to copy JSON Pointer",
                    pointer,
                    row.kind.label()
                ));
            }
        });

    if let Some(pointer) = toggle {
        if !app.tabs[index].json_collapsed.remove(&pointer) {
            app.tabs[index].json_collapsed.insert(pointer);
        }
    }
    if let Some(pointer) = copy_pointer {
        ui.ctx().copy_text(pointer.clone());
        app.toast("JSON Pointer copied", Some(pointer), false);
    }
}
