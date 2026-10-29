//! Toast notifications, anchored above the status bar on the right.
//!
//! They never block interaction and expire on their own; errors stay longer.

use egui::{Align, FontId, Layout, Vec2};

use crate::app::App;
use crate::theme::{icons, Icon};

const LIFETIME: f32 = 4.0;
const ERROR_LIFETIME: f32 = 7.0;

pub fn show(app: &mut App, ctx: &egui::Context) {
    let now = std::time::Instant::now();
    app.toasts.retain(|toast| {
        let limit = if toast.error {
            ERROR_LIFETIME
        } else {
            LIFETIME
        };
        now.duration_since(toast.born).as_secs_f32() < limit
    });
    if app.toasts.is_empty() {
        return;
    }
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let screen = ctx.input(|i| i.viewport_rect());
    let mut dismiss: Option<usize> = None;

    egui::Area::new(crate::ui::id("nc-toasts"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            screen.right() - 332.0,
            screen.bottom() - 44.0 - app.toasts.len() as f32 * 54.0,
        ))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for (index, toast) in app.toasts.iter().enumerate() {
                    let accent = if toast.error { p.danger } else { p.success };
                    let response = egui::Frame::new()
                        .fill(p.elev)
                        .stroke(egui::Stroke::new(1.0, p.border_strong))
                        .corner_radius(egui::CornerRadius::same(crate::theme::radius::MD))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 10],
                            blur: 26,
                            spread: 0,
                            color: egui::Color32::from_black_alpha(90),
                        })
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_width(300.0);
                            ui.horizontal_top(|ui| {
                                icons::show(
                                    ui,
                                    if toast.error {
                                        Icon::Alert
                                    } else {
                                        Icon::Check
                                    },
                                    12.0,
                                    accent,
                                );
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&toast.text)
                                            .font(FontId::proportional(11.5))
                                            .color(p.fg),
                                    );
                                    if let Some(detail) = &toast.detail {
                                        ui.label(
                                            egui::RichText::new(detail)
                                                .font(FontId::proportional(10.0))
                                                .color(p.fg_faint),
                                        );
                                    }
                                });
                                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                    if crate::ui::widgets::icon_button(
                                        ui,
                                        &theme,
                                        Icon::X,
                                        false,
                                        "Dismiss",
                                    )
                                    .clicked()
                                    {
                                        dismiss = Some(index);
                                    }
                                });
                            });
                        })
                        .response;
                    let _ = response;
                    ui.add_space(6.0);
                    let _ = Vec2::ZERO;
                }
            });
        });

    if let Some(index) = dismiss {
        app.toasts.remove(index);
    }
}
