//! The application header.
//!
//! Mirrors the archived title bar: brand mark, vault switcher chip, a centred
//! search pill that opens the palette, and the panel/theme/settings controls.
//! Window buttons are intentionally absent — the native window keeps its own
//! decorations (documented limitation).

use egui::{Align, Color32, CornerRadius, FontId, Layout, Sense, Stroke, Vec2};

use crate::app::App;
use crate::theme::{icons, space, Icon};
use crate::ui::widgets;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let ctx = ui.ctx().clone();
    let ctx = &ctx;

    egui::Panel::top("nc-titlebar")
        .exact_size(space::TITLEBAR)
        .frame(
            egui::Frame::new()
                .fill(p.surface)
                .inner_margin(egui::Margin::symmetric(10, 0)),
        )
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                // Brand mark.
                let (mark_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
                ui.painter()
                    .rect_filled(mark_rect, CornerRadius::same(5), p.accent);
                ui.painter().text(
                    mark_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "N",
                    FontId::proportional(12.0),
                    p.accent_fg,
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("NoteCraft")
                        .font(FontId::proportional(12.5))
                        .color(p.fg),
                );
                ui.add_space(10.0);

                vault_chip(app, ui);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::icon_button(ui, &theme, Icon::Settings, false, "Settings  Ctrl ,")
                        .clicked()
                    {
                        app.settings_open = true;
                    }
                    let theme_icon = match theme.kind {
                        crate::theme::ThemeKind::Dark => Icon::Sun,
                        crate::theme::ThemeKind::Light => Icon::Moon,
                    };
                    if widgets::icon_button(ui, &theme, theme_icon, false, "Switch theme").clicked()
                    {
                        let next = app.settings.theme_kind().toggled();
                        app.set_theme(ctx, next);
                    }
                    if widgets::icon_button(
                        ui,
                        &theme,
                        Icon::PanelRight,
                        app.panels.right,
                        "Toggle context panel  Ctrl \\",
                    )
                    .clicked()
                    {
                        app.panels.right = !app.panels.right;
                        app.persist_session();
                    }
                    if widgets::icon_button(
                        ui,
                        &theme,
                        Icon::PanelLeft,
                        app.panels.left,
                        "Toggle file explorer  Ctrl B",
                    )
                    .clicked()
                    {
                        app.panels.left = !app.panels.left;
                        app.persist_session();
                    }
                    ui.add_space(6.0);
                    search_pill(app, ui);
                });
            });
        });
}

fn vault_chip(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let name = app
        .vault
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "No vault open".into());

    let label = name.clone();
    let font = FontId::proportional(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.clone(), font, p.fg_secondary);
    let size = Vec2::new(galley.size().x + 34.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(crate::theme::radius::SM),
        if hovered {
            p.hover
        } else {
            Color32::TRANSPARENT
        },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(crate::theme::radius::SM),
        Stroke::new(1.0, p.border),
        egui::StrokeKind::Inside,
    );
    ui.painter().circle_filled(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        2.5,
        if app.vault.is_some() {
            p.accent
        } else {
            p.fg_faint
        },
    );
    ui.painter().text(
        egui::pos2(rect.left() + 18.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.5),
        p.fg_secondary,
    );
    icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(rect.right() - 10.0, rect.center().y),
            Vec2::splat(10.0),
        ),
        Icon::ChevronDown,
        p.fg_muted,
        1.3,
    );
    if hovered {
        ctx_pointer(ui);
    }

    if response.clicked() {
        app.vault_menu = Some(egui::pos2(rect.left(), rect.bottom() + 4.0));
    }
}

fn ctx_pointer(ui: &egui::Ui) {
    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
}

fn search_pill(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let text = "Search notes and commands…";
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), FontId::proportional(11.5), p.fg_faint);
    let width = (galley.size().x + 86.0).min(340.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::click());
    let hovered = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(crate::theme::radius::SM),
        if hovered { p.hover } else { p.raised },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(crate::theme::radius::SM),
        Stroke::new(1.0, if hovered { p.border_strong } else { p.border }),
        egui::StrokeKind::Inside,
    );
    icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 12.0, rect.center().y),
            Vec2::splat(11.0),
        ),
        Icon::Search,
        p.fg_faint,
        1.3,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 23.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        FontId::proportional(11.5),
        p.fg_faint,
    );
    ui.painter().text(
        egui::pos2(rect.right() - 10.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        "Ctrl K",
        FontId::monospace(9.0),
        p.fg_faint,
    );
    if hovered {
        ctx_pointer(ui);
    }
    if response.clicked() {
        app.palette.open = true;
        app.palette.query.clear();
        app.palette.mode = crate::app::PaletteMode::Files;
        app.palette.selected = 0;
    }
}
