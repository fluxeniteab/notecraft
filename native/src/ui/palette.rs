//! The command palette (Ctrl K) and command list (Ctrl P).
//!
//! Also lists vault search hits, so one keystroke reaches a file, a command or a
//! match inside a document.

use egui::{Align, FontId, Layout, Vec2};

use crate::app::{App, PaletteMode};
use crate::theme::{icons, Icon};
use crate::ui::widgets;

pub fn show(app: &mut App, ctx: &egui::Context) {
    if !app.palette.open {
        return;
    }
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let screen = ctx.input(|i| i.viewport_rect());
    let width = 560.0_f32.min(screen.width() - 60.0);

    let items = app.palette_candidates(&app.palette.query);
    app.palette_items = items;
    let count = app.palette_items.len();
    if app.palette.selected >= count {
        app.palette.selected = count.saturating_sub(1);
    }

    // Keyboard navigation is handled before drawing so the list reflects it.
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        app.palette.selected = (app.palette.selected + 1).min(count.saturating_sub(1));
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        app.palette.selected = app.palette.selected.saturating_sub(1);
    }
    let submit = ctx.input(|i| i.key_pressed(egui::Key::Enter));

    let mut chosen: Option<usize> = None;
    let mut close = false;

    egui::Area::new(crate::ui::id("nc-palette"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            screen.center().x - width / 2.0,
            screen.top() + 72.0,
        ))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.elev)
                .stroke(egui::Stroke::new(1.0, p.border_strong))
                .corner_radius(egui::CornerRadius::same(crate::theme::radius::LG))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 16],
                    blur: 40,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(110),
                })
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(width);

                    // Mode switch.
                    ui.horizontal(|ui| {
                        if widgets::segmented(
                            ui,
                            &theme,
                            app.palette.mode,
                            &[
                                (PaletteMode::Files, "Files"),
                                (PaletteMode::Commands, "Commands"),
                            ],
                        )
                        .is_some()
                        {
                            // `segmented` reports the clicked value via return.
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            widgets::kbd(ui, &theme, "Esc");
                        });
                    });
                    if let Some(mode) = widgets::segmented(
                        ui,
                        &theme,
                        app.palette.mode,
                        &[
                            (PaletteMode::Files, "Files"),
                            (PaletteMode::Commands, "Commands"),
                        ],
                    ) {
                        app.palette.mode = mode;
                        app.palette.selected = 0;
                    }

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut app.palette.query)
                            .hint_text(match app.palette.mode {
                                PaletteMode::Files => "Search files and vault content…",
                                PaletteMode::Commands => "Run a command…",
                            })
                            .desired_width(width - 24.0)
                            .font(FontId::proportional(13.0))
                            .frame(egui::Frame::NONE)
                            .margin(egui::Margin::symmetric(4, 8)),
                    );
                    if !response.has_focus() {
                        response.request_focus();
                    }
                    if response.changed() {
                        app.palette.selected = 0;
                        if !app.palette.query.trim().is_empty() && !app.search.opened_folder {
                            app.search.query = app.palette.query.clone();
                            app.search_vault();
                        }
                    }

                    ui.add_space(4.0);
                    widgets::divider(ui, &theme);
                    ui.add_space(4.0);

                    let mut list = ui
                        .allocate_ui_with_layout(
                            Vec2::new(width - 16.0, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                let mut height = 0.0;
                                if count == 0 {
                                    ui.label(
                                        egui::RichText::new("Nothing matches")
                                            .font(FontId::proportional(11.0))
                                            .color(p.fg_faint),
                                    );
                                }
                                for (index, entry) in app.palette_items.iter().take(12).enumerate()
                                {
                                    let selected = index == app.palette.selected;
                                    let response = widgets::row(ui, &theme, selected, 30.0, |ui| {
                                        let icon = match &entry.item {
                                            crate::app::PaletteItem::OpenFile { .. } => {
                                                Icon::FileText
                                            }
                                            crate::app::PaletteItem::RunCommand { .. } => {
                                                Icon::Command
                                            }
                                            crate::app::PaletteItem::Search { .. } => Icon::Search,
                                        };
                                        icons::show(
                                            ui,
                                            icon,
                                            11.0,
                                            if selected { p.accent } else { p.fg_faint },
                                        );
                                        ui.label(
                                            egui::RichText::new(&entry.label)
                                                .font(FontId::proportional(12.0))
                                                .color(if selected {
                                                    p.fg
                                                } else {
                                                    p.fg_secondary
                                                }),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(&entry.hint)
                                                        .font(FontId::proportional(9.5))
                                                        .color(p.fg_faint),
                                                );
                                            },
                                        );
                                    });
                                    if response.clicked() {
                                        chosen = Some(index);
                                    }
                                    height += 30.0;
                                }
                                height
                            },
                        )
                        .inner;
                    let _ = &mut list;
                });
        });

    if submit && count > 0 {
        chosen = Some(app.palette.selected.min(count - 1));
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close = true;
    }

    if let Some(index) = chosen {
        if let Some(entry) = app.palette_items.get(index).cloned() {
            app.run_palette_item(entry.item, ctx);
            close = true;
        }
    }
    if close {
        app.palette.open = false;
        app.palette.query.clear();
        app.palette.selected = 0;
    }
}
