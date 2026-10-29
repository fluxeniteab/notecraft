//! The document tab strip.

use egui::{Align, CornerRadius, FontId, Layout, Sense, Vec2};

use crate::app::App;
use crate::theme::{icons, space, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let mut close: Option<usize> = None;

    egui::Frame::new()
        .fill(p.surface)
        .inner_margin(egui::Margin::symmetric(6, 0))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                egui::ScrollArea::horizontal()
                    .auto_shrink([false, true])
                    .id_salt("nc-tabstrip")
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            let rows: Vec<(usize, String, String, bool)> = app
                                .tabs
                                .iter()
                                .enumerate()
                                .map(|(index, tab)| {
                                    (index, tab.title(), tab.path.clone(), tab.is_dirty())
                                })
                                .collect();
                            for (index, title, full, dirty) in rows {
                                let active = app.active == Some(index);

                                let size = Vec2::new(
                                    (title.chars().count() as f32 * 6.4 + 62.0).clamp(96.0, 210.0),
                                    26.0,
                                );
                                let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                                let hovered = response.hovered();

                                if active {
                                    ui.painter().rect_filled(
                                        rect,
                                        CornerRadius::same(crate::theme::radius::SM),
                                        p.raised,
                                    );
                                    ui.painter().rect_stroke(
                                        rect,
                                        CornerRadius::same(crate::theme::radius::SM),
                                        egui::Stroke::new(1.0, p.border_strong),
                                        egui::StrokeKind::Inside,
                                    );
                                } else if hovered {
                                    ui.painter().rect_filled(
                                        rect,
                                        CornerRadius::same(crate::theme::radius::SM),
                                        p.hover,
                                    );
                                }

                                icons::paint(
                                    ui.painter(),
                                    egui::Rect::from_center_size(
                                        egui::pos2(rect.left() + 13.0, rect.center().y),
                                        Vec2::splat(11.0),
                                    ),
                                    Icon::FileText,
                                    if active { p.accent } else { p.fg_faint },
                                    1.3,
                                );
                                ui.painter().text(
                                    egui::pos2(rect.left() + 24.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    truncate(&title, 22),
                                    FontId::proportional(11.5),
                                    if active { p.fg } else { p.fg_secondary },
                                );

                                let close_rect = egui::Rect::from_center_size(
                                    egui::pos2(rect.right() - 12.0, rect.center().y),
                                    Vec2::splat(14.0),
                                );
                                let close_hover = ui
                                    .ctx()
                                    .input(|i| i.pointer.hover_pos())
                                    .is_some_and(|pos| close_rect.contains(pos));
                                if dirty && !close_hover {
                                    ui.painter()
                                        .circle_filled(close_rect.center(), 2.6, p.accent);
                                } else {
                                    icons::paint(
                                        ui.painter(),
                                        close_rect.shrink(2.0),
                                        Icon::X,
                                        if close_hover { p.fg } else { p.fg_faint },
                                        1.3,
                                    );
                                }

                                if response.clicked() {
                                    if close_hover {
                                        close = Some(index);
                                    } else {
                                        app.active = Some(index);
                                        app.persist_session();
                                    }
                                }
                                if response.middle_clicked() {
                                    close = Some(index);
                                }
                                response.on_hover_text(full);
                            }
                        });
                    });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if crate::ui::widgets::icon_button(
                        ui,
                        &theme,
                        Icon::Plus,
                        false,
                        "New document  Ctrl N",
                    )
                    .clicked()
                    {
                        app.new_document("", crate::documents::Kind::Markdown);
                    }
                    if app.tabs.len() > 1
                        && crate::ui::widgets::icon_button(
                            ui,
                            &theme,
                            Icon::List,
                            false,
                            "Close all tabs",
                        )
                        .clicked()
                    {
                        app.tabs.clear();
                        app.active = None;
                        app.persist_session();
                    }
                });
            });
        });

    let _ = space::TABSTRIP;

    if let Some(index) = close {
        app.close_tab(index);
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
