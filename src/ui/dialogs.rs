//! Dialogs: settings, prompts, confirmations, the explorer context menu and the
//! informational windows.

use egui::{Align, FontId, Layout, Sense, Vec2};

use crate::app::{App, ConfirmAction, PromptAction};
use crate::theme::{icons, Icon};
use crate::ui::widgets;

pub fn show(app: &mut App, ctx: &egui::Context) {
    context_menu(app, ctx);
    vault_menu(app, ctx);
    new_document_menu(app, ctx);
    prompt(app, ctx);
    confirm(app, ctx);
    settings(app, ctx);
    about(app, ctx);
    plugins(app, ctx);
    shortcuts(app, ctx);
}

fn window_frame(theme: &crate::theme::Theme) -> egui::Frame {
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.elev)
        .stroke(egui::Stroke::new(1.0, p.border_strong))
        .corner_radius(egui::CornerRadius::same(crate::theme::radius::LG))
        .inner_margin(egui::Margin::same(14))
}

// ------------------------------------------------------------- context menu

fn context_menu(app: &mut App, ctx: &egui::Context) {
    let Some((pos, path, is_dir)) = app.context_menu.clone() else {
        return;
    };
    let theme = app.theme.clone();
    let mut close = false;

    egui::Area::new(crate::ui::id("nc-context-menu"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            window_frame(&theme)
                .inner_margin(egui::Margin::same(5))
                .show(ui, |ui| {
                    ui.set_min_width(220.0);
                    if !is_dir
                        && widgets::menu_row(ui, &theme, Some(Icon::FileText), "Open", None, true)
                    {
                        app.open_path(&path, false);
                        close = true;
                    }
                    let dir = if is_dir {
                        path.clone()
                    } else {
                        path.rsplit_once('/')
                            .map(|(d, _)| d.to_string())
                            .unwrap_or_default()
                    };
                    if widgets::menu_row(ui, &theme, Some(Icon::Plus), "New note here", None, true)
                    {
                        app.prompt = Some(crate::app::Prompt {
                            title: "New document".into(),
                            label: "File name".into(),
                            value: String::new(),
                            placeholder: "Untitled.md".into(),
                            confirm_label: "Create".into(),
                            action: PromptAction::NewDocument {
                                dir: dir.clone(),
                                kind: crate::documents::Kind::Markdown,
                            },
                            focus: false,
                        });
                        close = true;
                    }
                    if widgets::menu_row(
                        ui,
                        &theme,
                        Some(Icon::Folder),
                        "New folder here",
                        None,
                        true,
                    ) {
                        app.prompt = Some(crate::app::Prompt {
                            title: "New folder".into(),
                            label: "Folder name".into(),
                            value: String::new(),
                            placeholder: "Research".into(),
                            confirm_label: "Create".into(),
                            action: PromptAction::NewFolder { dir },
                            focus: false,
                        });
                        close = true;
                    }
                    widgets::divider(ui, &theme);
                    if widgets::menu_row(ui, &theme, Some(Icon::Pencil), "Rename…", None, true) {
                        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
                        app.prompt = Some(crate::app::Prompt {
                            title: "Rename".into(),
                            label: "New name".into(),
                            value: name,
                            placeholder: "name.md".into(),
                            confirm_label: "Rename".into(),
                            action: PromptAction::Rename { from: path.clone() },
                            focus: false,
                        });
                        close = true;
                    }
                    if widgets::menu_row(
                        ui,
                        &theme,
                        Some(Icon::Link),
                        "Copy vault path",
                        None,
                        true,
                    ) {
                        ui.ctx().copy_text(path.clone());
                        close = true;
                    }
                    widgets::divider(ui, &theme);
                    if widgets::menu_row(ui, &theme, Some(Icon::Trash), "Move to trash", None, true)
                    {
                        app.confirm = Some(crate::app::Confirm {
                            title: if is_dir {
                                "Delete folder?"
                            } else {
                                "Delete document?"
                            }
                            .into(),
                            message: format!("{path} will be moved to the vault trash."),
                            confirm_label: "Move to trash".into(),
                            danger: true,
                            action: ConfirmAction::Delete {
                                path: path.clone(),
                                is_dir,
                            },
                        });
                        close = true;
                    }
                });
        });

    // Clicking elsewhere or pressing Escape dismisses the menu.
    if ctx.input(|i| i.pointer.any_click() && i.pointer.interact_pos() != Some(pos))
        || ctx.input(|i| i.key_pressed(egui::Key::Escape))
    {
        close = true;
    }
    if close {
        app.context_menu = None;
    }
}

fn overlay(
    ctx: &egui::Context,
    id: &str,
    pos: egui::Pos2,
    theme: &crate::theme::Theme,
    width: f32,
    contents: impl FnOnce(&mut egui::Ui),
) -> bool {
    let mut closed = false;
    egui::Area::new(crate::ui::id(id))
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            window_frame(theme)
                .inner_margin(egui::Margin::same(5))
                .show(ui, |ui| {
                    ui.set_min_width(width);
                    contents(ui);
                });
        });
    if ctx.input(|i| i.pointer.any_click()) || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        closed = true;
    }
    closed
}

fn vault_menu(app: &mut App, ctx: &egui::Context) {
    let Some(pos) = app.vault_menu else {
        return;
    };
    let theme = app.theme.clone();
    let mut action: Option<usize> = None;
    let recents = app.session.recent_vaults.clone();

    let mut close = overlay(ctx, "nc-vault-overlay", pos, &theme, 230.0, |ui| {
        if widgets::menu_row(
            ui,
            &theme,
            Some(Icon::FolderOpen),
            "Open vault…",
            None,
            true,
        ) {
            action = Some(usize::MAX);
        }
        if widgets::menu_row(
            ui,
            &theme,
            Some(Icon::Refresh),
            "Rescan vault",
            None,
            app.vault.is_some(),
        ) {
            action = Some(usize::MAX - 1);
        }
        if !recents.is_empty() {
            widgets::divider(ui, &theme);
            widgets::section_label(ui, &theme, "Recent vaults");
            for (index, path) in recents.iter().take(6).enumerate() {
                let name = path
                    .replace('\\', "/")
                    .rsplit('/')
                    .next()
                    .unwrap_or(path)
                    .to_string();
                if widgets::menu_row(ui, &theme, Some(Icon::History), &name, None, true) {
                    action = Some(index);
                }
            }
        }
        if app.vault.is_some() {
            widgets::divider(ui, &theme);
            if widgets::menu_row(ui, &theme, Some(Icon::X), "Close vault", None, true) {
                action = Some(usize::MAX - 2);
            }
        }
    });

    if let Some(choice) = action {
        match choice {
            usize::MAX => app.pick_vault(),
            c if c == usize::MAX - 1 => app.refresh_vault(),
            c if c == usize::MAX - 2 => app.close_vault(),
            index => {
                if let Some(path) = recents.get(index) {
                    app.set_vault(std::path::PathBuf::from(path));
                }
            }
        }
        close = true;
    }
    if close {
        app.vault_menu = None;
    }
}

fn new_document_menu(app: &mut App, ctx: &egui::Context) {
    let Some(pos) = app.new_menu else {
        return;
    };
    let theme = app.theme.clone();
    let mut chosen: Option<crate::documents::Kind> = None;

    let mut close = overlay(ctx, "nc-new-overlay", pos, &theme, 210.0, |ui| {
        for caps in crate::documents::CREATABLE {
            let label = format!("New {}", caps.label);
            if widgets::menu_row(ui, &theme, Some(Icon::FileText), &label, None, true) {
                chosen = Some(caps.kind);
            }
        }
    });

    if let Some(kind) = chosen {
        let dir = app.active_tab().map(|tab| tab.folder()).unwrap_or_default();
        app.new_document(&dir, kind);
        close = true;
    }
    if close {
        app.new_menu = None;
    }
}

// -------------------------------------------------------------------- prompt

fn prompt(app: &mut App, ctx: &egui::Context) {
    let Some(mut prompt) = app.prompt.take() else {
        return;
    };
    let theme = app.theme.clone();
    let mut open = true;
    let mut submit = false;
    let mut cancel = false;

    egui::Window::new(prompt.title.clone())
        .id(crate::ui::id("nc-prompt"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(window_frame(&theme))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(
                egui::RichText::new(&prompt.label)
                    .font(FontId::proportional(11.0))
                    .color(theme.palette.fg_muted),
            );
            let response = ui.add(
                egui::TextEdit::singleline(&mut prompt.value)
                    .hint_text(prompt.placeholder.clone())
                    .desired_width(340.0)
                    .font(FontId::proportional(13.0)),
            );
            if !prompt.focus {
                response.request_focus();
                prompt.focus = true;
            }
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                submit = true;
            }
            ui.add_space(8.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::button(ui, &theme, &prompt.confirm_label, true).clicked() {
                    submit = true;
                }
                if widgets::button(ui, &theme, "Cancel", false).clicked() {
                    cancel = true;
                }
            });
        });

    if submit {
        app.prompt = Some(prompt);
        app.run_prompt();
    } else if !open || cancel {
        app.prompt = None;
    } else {
        app.prompt = Some(prompt);
    }
}

// ------------------------------------------------------------------- confirm

fn confirm(app: &mut App, ctx: &egui::Context) {
    let Some(dialog) = app.confirm.as_ref() else {
        return;
    };
    let theme = app.theme.clone();
    let mut run = false;
    let mut cancel = false;
    let title = dialog.title.clone();
    let message = dialog.message.clone();
    let label = dialog.confirm_label.clone();
    let danger = dialog.danger;

    egui::Window::new(title)
        .id(crate::ui::id("nc-confirm"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(window_frame(&theme))
        .show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(
                egui::RichText::new(message)
                    .font(FontId::proportional(12.0))
                    .color(theme.palette.fg_secondary),
            );
            ui.add_space(10.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::button(ui, &theme, &label, danger || true).clicked() {
                    run = true;
                }
                if widgets::button(ui, &theme, "Cancel", false).clicked() {
                    cancel = true;
                }
            });
        });

    if run {
        app.run_confirm();
    } else if cancel {
        app.confirm = None;
    }
}

// ------------------------------------------------------------------ settings

fn settings(app: &mut App, ctx: &egui::Context) {
    if !app.settings_open {
        return;
    }
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let mut open = true;
    let mut changed = false;

    egui::Window::new("Settings")
        .id(crate::ui::id("nc-settings"))
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(560.0, 560.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(window_frame(&theme))
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(520.0)
                .show(ui, |ui| {
                    widgets::section_label(ui, &theme, "Appearance");
                    ui.add_space(6.0);
                    let kind = app.settings.theme_kind();
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Theme",
                        "Dark charcoal or warm white.",
                        |ui| {
                            if let Some(next) = widgets::segmented(
                                ui,
                                &theme,
                                kind,
                                &[
                                    (crate::theme::ThemeKind::Dark, "Dark"),
                                    (crate::theme::ThemeKind::Light, "Light"),
                                ],
                            ) {
                                app.settings.theme = next.as_str().to_string();
                                changed = true;
                            }
                        },
                    );
                    ui.add_space(6.0);

                    let mut rgb = {
                        let c = crate::theme::parse_hex_color(&app.settings.accent, p.accent);
                        [c.r(), c.g(), c.b()]
                    };
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Accent",
                        "Used for selection and focus.",
                        |ui| {
                            if ui.color_edit_button_srgb(&mut rgb).changed() {
                                app.settings.accent = crate::theme::to_hex(
                                    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                                );
                                changed = true;
                            }
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(app.settings.accent.clone())
                                    .font(FontId::monospace(10.0))
                                    .color(p.fg_faint),
                            );
                        },
                    );

                    let mut reduced = app.settings.reduced_motion;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Reduced motion",
                        "Skip non-essential animation.",
                        |ui| {
                            if widgets::toggle(ui, &theme, &mut reduced).clicked() {
                                app.settings.reduced_motion = reduced;
                                changed = true;
                            }
                        },
                    );

                    ui.add_space(12.0);
                    widgets::section_label(ui, &theme, "Editor");
                    ui.add_space(6.0);

                    let mut size = app.settings.editor_font_size;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Font size",
                        "Editor and reading text.",
                        |ui| {
                            if ui
                                .add(egui::Slider::new(&mut size, 12..=22).suffix(" pt"))
                                .changed()
                            {
                                app.settings.editor_font_size = size;
                                changed = true;
                            }
                        },
                    );

                    let mut line_height = app.settings.editor_line_height;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Line height",
                        "Spacing between lines.",
                        |ui| {
                            if ui
                                .add(egui::Slider::new(&mut line_height, 1.3..=2.2).max_decimals(2))
                                .changed()
                            {
                                app.settings.editor_line_height = line_height;
                                changed = true;
                            }
                        },
                    );

                    let mut width = app.settings.reading_width as f32;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Reading width",
                        "Maximum prose column.",
                        |ui| {
                            if ui
                                .add(egui::Slider::new(&mut width, 520.0..=980.0).suffix(" px"))
                                .changed()
                            {
                                app.settings.reading_width = width as u32;
                                changed = true;
                            }
                        },
                    );

                    let mut numbers = app.settings.show_line_numbers;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Line numbers",
                        "Shown for code and data documents.",
                        |ui| {
                            if widgets::toggle(ui, &theme, &mut numbers).clicked() {
                                app.settings.show_line_numbers = numbers;
                                changed = true;
                            }
                        },
                    );

                    let mut wrap = app.settings.wrap_code;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Wrap code lines",
                        "Off keeps long lines intact.",
                        |ui| {
                            if widgets::toggle(ui, &theme, &mut wrap).clicked() {
                                app.settings.wrap_code = wrap;
                                changed = true;
                            }
                        },
                    );

                    ui.add_space(12.0);
                    widgets::section_label(ui, &theme, "Files");
                    ui.add_space(6.0);

                    let mut hidden = app.settings.hidden_files;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Show hidden files",
                        "Dotfiles in the explorer.",
                        |ui| {
                            if widgets::toggle(ui, &theme, &mut hidden).clicked() {
                                app.settings.hidden_files = hidden;
                                app.refresh_vault();
                                changed = true;
                            }
                        },
                    );

                    let mut sort = app.explorer.sort.clone();
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Explorer order",
                        "How files are listed.",
                        |ui| {
                            for option in ["name", "modified", "size"] {
                                let selected = sort == option;
                                if ui
                                    .selectable_label(
                                        selected,
                                        egui::RichText::new(option)
                                            .font(FontId::proportional(11.0)),
                                    )
                                    .clicked()
                                {
                                    sort = option.to_string();
                                }
                            }
                        },
                    );
                    if sort != app.explorer.sort {
                        app.explorer.sort = sort.clone();
                        app.settings.explorer_sort = sort;
                        app.refresh_vault();
                        changed = true;
                    }

                    let mut excluded = app.settings.excluded_folders.join(", ");
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Ignored folders",
                        "Comma separated names skipped by the explorer and search.",
                        |ui| {
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut excluded)
                                        .desired_width(220.0)
                                        .font(FontId::proportional(11.0)),
                                )
                                .changed()
                            {
                                app.settings.excluded_folders = excluded
                                    .split(',')
                                    .map(|part| part.trim().to_string())
                                    .filter(|part| !part.is_empty())
                                    .collect();
                                changed = true;
                            }
                        },
                    );

                    ui.add_space(12.0);
                    widgets::section_label(ui, &theme, "Saving");
                    ui.add_space(6.0);

                    let mut autosave = app.settings.autosave_enabled;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Auto-save",
                        "Write changes after a short pause.",
                        |ui| {
                            if widgets::toggle(ui, &theme, &mut autosave).clicked() {
                                app.settings.autosave_enabled = autosave;
                                changed = true;
                            }
                        },
                    );

                    let mut interval = app.settings.autosave_ms as f32;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Auto-save delay",
                        "Milliseconds of inactivity.",
                        |ui| {
                            if ui
                                .add(egui::Slider::new(&mut interval, 400.0..=4000.0).suffix(" ms"))
                                .changed()
                            {
                                app.settings.autosave_ms = interval as u32;
                                changed = true;
                            }
                        },
                    );

                    let mut retention = app.settings.snapshot_retention as f32;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Snapshots kept",
                        "Per document, in .notecraft/history.",
                        |ui| {
                            if ui
                                .add(egui::Slider::new(&mut retention, 5.0..=200.0))
                                .changed()
                            {
                                app.settings.snapshot_retention = retention as u32;
                                changed = true;
                            }
                        },
                    );

                    let mut reopen = app.settings.reopen_last_session;
                    widgets::setting_row(
                        ui,
                        &theme,
                        "Reopen last session",
                        "Restore open tabs and the vault on start.",
                        |ui| {
                            if widgets::toggle(ui, &theme, &mut reopen).clicked() {
                                app.settings.reopen_last_session = reopen;
                                changed = true;
                            }
                        },
                    );

                    ui.add_space(14.0);
                    widgets::divider(ui, &theme);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if widgets::button(ui, &theme, "Open data folder", false).clicked() {
                            let dir = crate::settings::data_dir();
                            let _ = std::fs::create_dir_all(&dir);
                            app.open_folder(&dir);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "NoteCraft {}",
                                    env!("CARGO_PKG_VERSION")
                                ))
                                .font(FontId::proportional(10.0))
                                .color(p.fg_faint),
                            );
                        });
                    });
                });
        });

    if changed {
        app.apply_theme(ctx);
        app.persist_settings();
    }
    if !open {
        app.settings_open = false;
        app.persist_settings();
    }
}

// --------------------------------------------------------------- info windows

fn info_window(
    app: &mut App,
    ctx: &egui::Context,
    id: &str,
    title: &str,
    open: &mut bool,
    contents: impl FnOnce(&mut egui::Ui, &crate::theme::Theme),
) {
    if !*open {
        return;
    }
    let theme = app.theme.clone();
    egui::Window::new(title)
        .id(crate::ui::id(id))
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(460.0, 320.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(window_frame(&theme))
        .open(open)
        .show(ctx, |ui| contents(ui, &theme));
}

fn about(app: &mut App, ctx: &egui::Context) {
    let mut open = app.about_open;
    info_window(
        app,
        ctx,
        "nc-about",
        "About NoteCraft",
        &mut open,
        |ui, theme| {
            let p = &theme.palette;
            ui.set_width(420.0);
            ui.label(
                egui::RichText::new("NoteCraft")
                    .font(FontId::proportional(18.0))
                    .color(p.fg),
            );
            ui.label(
                egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                    .font(FontId::proportional(11.0))
                    .color(p.fg_muted),
            );
            ui.add_space(10.0);
            for line in [
                "A local-first knowledge base for Markdown, MDX and JSON.",
                "Written in Rust with egui — no webview, no Node runtime.",
                "Documents stay on disk, in ordinary files you own.",
                "MDX is never executed: components and expressions are shown as source.",
            ] {
                ui.horizontal_top(|ui| {
                    icons::show(ui, Icon::Check, 10.0, p.success);
                    ui.label(
                        egui::RichText::new(line)
                            .font(FontId::proportional(11.5))
                            .color(p.fg_secondary),
                    );
                });
            }
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("GPL-3.0 · No telemetry · No account required")
                    .font(FontId::proportional(10.0))
                    .color(p.fg_faint),
            );
        },
    );
    app.about_open = open;
}

fn plugins(app: &mut App, ctx: &egui::Context) {
    let mut open = app.plugins_open;
    info_window(app, ctx, "nc-plugins", "Plugins", &mut open, |ui, theme| {
        let p = &theme.palette;
        ui.set_width(440.0);
        ui.label(
            egui::RichText::new("Plugin execution is not available in the native build")
                .font(FontId::proportional(12.5))
                .color(p.fg),
        );
        ui.add_space(6.0);
        for line in [
            "The Tauri build ran plugin modules inside the webview, which a native egui application does not have.",
            "ZIP packages are still validated as archives, but no JavaScript runtime is embedded, so plugin code is never executed and never gets filesystem or vault access.",
            "This is a deliberate limitation rather than a sandbox claim: a plugin system needs a real isolated runtime, not an iframe lookalike.",
        ] {
            ui.label(
                egui::RichText::new(line)
                    .font(FontId::proportional(11.0))
                    .color(p.fg_secondary),
            );
            ui.add_space(4.0);
        }
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Existing plugin folders remain untouched on disk.")
                .font(FontId::proportional(10.0))
                .color(p.fg_faint),
        );
    });
    app.plugins_open = open;
}

fn shortcuts(app: &mut App, ctx: &egui::Context) {
    let mut open = app.shortcuts_open;
    info_window(
        app,
        ctx,
        "nc-shortcuts",
        "Keyboard shortcuts",
        &mut open,
        |ui, theme| {
            let p = &theme.palette;
            ui.set_width(440.0);
            egui::ScrollArea::vertical()
                .max_height(380.0)
                .show(ui, |ui| {
                    for (keys, action) in [
                        ("Ctrl K", "Search files and vault content"),
                        ("Ctrl P", "Run a command"),
                        ("Ctrl N", "New Markdown note"),
                        ("Ctrl S", "Save document"),
                        ("Ctrl Shift S", "Save all documents"),
                        ("Ctrl W", "Close tab"),
                        ("Ctrl B", "Toggle file explorer"),
                        ("Ctrl \\", "Toggle context panel"),
                        ("Ctrl ,", "Settings"),
                        ("Ctrl E", "Cycle view mode"),
                        ("Ctrl 1 / 2 / 3", "Edit / split / read"),
                        ("Alt ← / →", "Previous / next tab"),
                        ("Ctrl Shift P", "Focus mode"),
                        ("Escape", "Close dialog or menu"),
                    ] {
                        ui.horizontal(|ui| {
                            widgets::kbd(ui, theme, keys);
                            ui.label(
                                egui::RichText::new(action)
                                    .font(FontId::proportional(11.0))
                                    .color(p.fg_secondary),
                            );
                        });
                        ui.add_space(3.0);
                    }
                });
        },
    );
    app.shortcuts_open = open;
}

#[allow(dead_code)]
fn inert(theme: &crate::theme::Theme) -> Sense {
    let _ = theme;
    Sense::hover()
}
