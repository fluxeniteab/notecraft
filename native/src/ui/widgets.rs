//! Small widgets shared by the shell.
//!
//! These reproduce the control sizes and states of the archived UI: 24pt icon
//! buttons, 6pt radii, one-pixel separators, segmented view switches and keycaps.

use egui::{Align, Color32, CornerRadius, FontId, Layout, Response, Sense, Stroke, Ui, Vec2};

use crate::theme::{icons::Icon, radius, Theme};

/// An icon-only button with hover, active and tooltip states.
pub fn icon_button(
    ui: &mut Ui,
    theme: &Theme,
    icon: Icon,
    active: bool,
    tooltip: &str,
) -> Response {
    let size = Vec2::splat(24.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let p = &theme.palette;
    let hovered = response.hovered();

    if active {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius_sm()), p.accent_soft(0.16));
    } else if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius_sm()), p.hover);
    }

    let color = if active {
        p.accent
    } else if hovered {
        p.fg
    } else {
        p.fg_secondary
    };
    let icon_rect = rect.shrink(6.0);
    crate::theme::icons::paint(ui.painter(), icon_rect, icon, color, 1.5);

    if response.hovered() {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if !tooltip.is_empty() {
        response.clone().on_hover_text(tooltip);
    }
    response
}

fn radius_sm() -> u8 {
    crate::theme::radius::SM
}

/// A rounded label. `tone` selects accent, muted, success, warning or danger.
pub fn chip(ui: &mut Ui, theme: &Theme, text: &str, tone: Tone) {
    let p = &theme.palette;
    let (fg, bg) = match tone {
        Tone::Neutral => (p.fg_secondary, p.muted),
        Tone::Accent => (p.accent, p.accent_soft(0.14)),
        Tone::Success => (p.success, p.success.linear_multiply(0.16)),
        Tone::Warning => (p.warning, p.warning.linear_multiply(0.16)),
        Tone::Danger => (p.danger, p.danger.linear_multiply(0.16)),
    };
    let font = FontId::proportional(10.0);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, fg);
    let padding = Vec2::new(6.0, 2.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
    ui.painter().galley(rect.min + padding, galley, fg);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // full tone set for chips, not every tone is used yet
pub enum Tone {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

/// A keycap, e.g. `Ctrl K`.
pub fn kbd(ui: &mut Ui, theme: &Theme, text: &str) {
    let p = &theme.palette;
    let font = FontId::monospace(9.5);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, p.fg_muted);
    let padding = Vec2::new(5.0, 1.5);
    let (rect, _) = ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), p.muted);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, p.border),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(rect.min + padding, galley, p.fg_muted);
}

/// A segmented control. Returns the newly selected value.
pub fn segmented<T: Copy + PartialEq>(
    ui: &mut Ui,
    theme: &Theme,
    value: T,
    options: &[(T, &str)],
) -> Option<T> {
    let p = &theme.palette;
    let mut changed = None;
    let height = 22.0;
    let font = FontId::proportional(11.5);

    egui::Frame::new()
        .fill(p.muted)
        .corner_radius(CornerRadius::same(radius_sm()))
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (option, label) in options {
                    let galley = ui.painter().layout_no_wrap(
                        (*label).to_string(),
                        font.clone(),
                        p.fg_secondary,
                    );
                    let width = galley.size().x + 18.0;
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
                    let selected = *option == value;
                    if selected {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(radius::XS), p.surface);
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(radius::XS),
                            Stroke::new(1.0, p.border_strong),
                            egui::StrokeKind::Inside,
                        );
                    } else if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(radius::XS), p.hover);
                    }
                    let color = if selected { p.fg } else { p.fg_muted };
                    ui.painter()
                        .galley(rect.center() - galley.size() / 2.0, galley, color);
                    if response.clicked() && !selected {
                        changed = Some(*option);
                    }
                }
            });
        });
    changed
}

/// A small uppercase section heading.
pub fn section_label(ui: &mut Ui, theme: &Theme, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .font(FontId::proportional(9.5))
            .color(theme.palette.fg_faint),
    );
}

/// A one-pixel separator that respects the panel width.
pub fn divider(ui: &mut Ui, theme: &Theme) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().line_segment(
        [rect.left_center(), rect.right_center()],
        theme.palette.hairline(),
    );
}

/// Centred empty state with an icon, a title and a hint.
pub fn empty_state(ui: &mut Ui, theme: &Theme, icon: Icon, title: &str, hint: &str) {
    let p = &theme.palette;
    ui.add_space(28.0);
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
        crate::theme::icons::paint(ui.painter(), rect, icon, p.fg_faint, 1.4);
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(title)
                .font(FontId::proportional(12.5))
                .color(p.fg_secondary),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(hint)
                .font(FontId::proportional(11.0))
                .color(p.fg_faint),
        );
    });
}

/// A row in a menu: icon, label, optional shortcut. Returns true when clicked.
pub fn menu_row(
    ui: &mut Ui,
    theme: &Theme,
    icon: Option<Icon>,
    label: &str,
    shortcut: Option<&str>,
    enabled: bool,
) -> bool {
    let p = &theme.palette;
    let height = 24.0;
    let width = ui.available_width().max(160.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let hovered = response.hovered() && enabled;
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::XS), p.hover);
    }
    let mut x = rect.left() + 8.0;
    if let Some(icon) = icon {
        let icon_rect =
            egui::Rect::from_min_size(egui::pos2(x, rect.center().y - 7.0), Vec2::splat(14.0));
        crate::theme::icons::paint(
            ui.painter(),
            icon_rect,
            icon,
            if enabled { p.fg_secondary } else { p.fg_faint },
            1.4,
        );
        x += 20.0;
    }
    let color = if enabled { p.fg } else { p.fg_faint };
    ui.painter().text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        color,
    );
    if let Some(shortcut) = shortcut {
        ui.painter().text(
            egui::pos2(rect.right() - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            FontId::monospace(9.5),
            p.fg_faint,
        );
    }
    if hovered {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked() && enabled
}

/// A labelled settings row with the control on the right.
pub fn setting_row<R>(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    hint: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    let p = &theme.palette;
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new((ui.available_width() - 168.0).max(140.0), 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.label(
                    egui::RichText::new(title)
                        .font(FontId::proportional(12.5))
                        .color(p.fg),
                );
                if !hint.is_empty() {
                    ui.add_space(1.0);
                    ui.label(
                        egui::RichText::new(hint)
                            .font(FontId::proportional(10.5))
                            .color(p.fg_faint),
                    );
                }
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), control)
            .inner
    })
    .inner
}

/// A switch in the style of the archived settings pages.
pub fn toggle(ui: &mut Ui, theme: &Theme, value: &mut bool) -> Response {
    let p = &theme.palette;
    let size = Vec2::new(30.0, 17.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *value = !*value;
    }
    let on = *value;
    let bg = if on {
        p.accent
    } else if response.hovered() {
        p.border_strong
    } else {
        p.muted
    };
    let radius = (rect.height() * 0.5) as u8;
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius), bg);
    let knob_x = if on {
        rect.right() - rect.height() * 0.5
    } else {
        rect.left() + rect.height() * 0.5
    };
    ui.painter().circle_filled(
        egui::pos2(knob_x, rect.center().y),
        rect.height() * 0.36,
        Color32::WHITE,
    );
    if response.hovered() {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Standard push button.
pub fn button(ui: &mut Ui, theme: &Theme, label: &str, primary: bool) -> Response {
    let p = &theme.palette;
    let font = FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, p.fg);
    let padding = Vec2::new(12.0, 5.0);
    let (rect, response) = ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::click());
    let hovered = response.hovered();
    let (fill, stroke, text) = if primary {
        (
            if hovered { p.accent_hover } else { p.accent },
            Stroke::new(1.0, p.accent),
            p.accent_fg,
        )
    } else if hovered {
        (p.hover, Stroke::new(1.0, p.border_strong), p.fg)
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::new(1.0, p.border_strong),
            p.fg_secondary,
        )
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius_sm()), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius_sm()),
        stroke,
        egui::StrokeKind::Inside,
    );
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), FontId::proportional(12.0), text);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, text);
    if hovered {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// A full-width selectable row used by the explorer, palette and outline.
pub fn row(
    ui: &mut Ui,
    theme: &Theme,
    selected: bool,
    height: f32,
    add_contents: impl FnOnce(&mut Ui),
) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let p = &theme.palette;
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::XS), p.accent_soft(0.14));
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::XS), p.hover);
    }
    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(6.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    add_contents(&mut content);
    response
}
