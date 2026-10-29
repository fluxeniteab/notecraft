//! The application menu bar.
//!
//! Every entry performs a real action. Menus that would need a capability the
//! native build does not have (plugin execution via a JavaScript runtime) open
//! an honest explanation instead of pretending to work.

use egui::{FontId, RichText};

use crate::app::{App, CommandId, PaletteMode, Prompt, PromptAction, ViewMode};
use crate::documents::Kind;
use crate::theme::{space, Icon};
use crate::ui::widgets;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let p = theme.palette.clone();
    let ctx = ui.ctx().clone();
    let ctx = &ctx;

    egui::Panel::top("nc-menubar")
        .exact_size(space::MENUBAR)
        .frame(
            egui::Frame::new()
                .fill(p.surface)
                .inner_margin(egui::Margin::symmetric(8, 0))
                .stroke(egui::Stroke::new(1.0, p.border)),
        )
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                file_menu(app, ui, ctx);
                edit_menu(app, ui, ctx);
                format_menu(app, ui, ctx);
                view_menu(app, ui, ctx);
                insert_menu(app, ui, ctx);
                navigate_menu(app, ui, ctx);
                tools_menu(app, ui, ctx);
                plugins_menu(app, ui);
                help_menu(app, ui);
            });
        });
}

fn menu_label(text: &str) -> RichText {
    RichText::new(text).font(FontId::proportional(12.0))
}

fn row(
    ui: &mut egui::Ui,
    app: &App,
    icon: Option<Icon>,
    label: &str,
    shortcut: Option<&str>,
    enabled: bool,
) -> bool {
    widgets::menu_row(ui, &app.theme, icon, label, shortcut, enabled)
}

fn file_menu(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    ui.menu_button(menu_label("File"), |ui| {
        ui.set_min_width(230.0);
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "New Markdown note",
            Some("Ctrl N"),
            true,
        ) {
            app.run_command(CommandId::NewMarkdown, ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "New MDX document",
            None,
            true,
        ) {
            app.run_command(CommandId::NewMdx, ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "New JSON document",
            None,
            true,
        ) {
            app.run_command(CommandId::NewJson, ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "New YAML document",
            None,
            true,
        ) {
            app.run_command(CommandId::NewYaml, ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "New plain text file",
            None,
            true,
        ) {
            app.run_command(CommandId::NewText, ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Folder),
            "New folder…",
            None,
            app.vault.is_some(),
        ) {
            app.prompt_new_folder();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(ui, app, Some(Icon::FolderOpen), "Open vault…", None, true) {
            app.pick_vault();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Refresh),
            "Rescan vault",
            None,
            app.vault.is_some(),
        ) {
            app.refresh_vault();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        let has_tab = app.active.is_some();
        if row(ui, app, Some(Icon::Save), "Save", Some("Ctrl S"), has_tab) {
            app.save_active();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Save),
            "Save all",
            Some("Ctrl Shift S"),
            has_tab,
        ) {
            app.save_all();
            ui.close();
        }
        if row(ui, app, Some(Icon::Download), "Export…", None, has_tab) {
            app.export_active();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "Copy as plain text",
            None,
            has_tab,
        ) {
            app.copy_active_as_text(ctx);
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(ui, app, Some(Icon::X), "Close tab", Some("Ctrl W"), has_tab) {
            app.run_command(CommandId::CloseTab, ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::X),
            "Close vault",
            None,
            app.vault.is_some(),
        ) {
            app.close_vault();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(ui, app, Some(Icon::X), "Quit", None, true) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn edit_menu(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    ui.menu_button(menu_label("Edit"), |ui| {
        ui.set_min_width(230.0);
        if row(
            ui,
            app,
            Some(Icon::Command),
            "Command palette",
            Some("Ctrl K"),
            true,
        ) {
            app.palette.open = true;
            app.palette.mode = PaletteMode::Files;
            app.palette.query.clear();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::List),
            "Command list",
            Some("Ctrl P"),
            true,
        ) {
            app.palette.open = true;
            app.palette.mode = PaletteMode::Commands;
            app.palette.query.clear();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(
            ui,
            app,
            Some(Icon::FileText),
            "Copy document as text",
            None,
            app.active.is_some(),
        ) {
            app.copy_active_as_text(ctx);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Pencil),
            "Rename current file…",
            None,
            app.active.is_some(),
        ) {
            if let Some(tab) = app.active_tab() {
                let path = tab.path.clone();
                let name = tab.file_name().to_string();
                app.prompt = Some(Prompt {
                    title: "Rename".into(),
                    label: "New name".into(),
                    value: name,
                    placeholder: "name.md".into(),
                    confirm_label: "Rename".into(),
                    action: PromptAction::Rename { from: path },
                    focus: false,
                });
            }
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        let json = app.active_tab().is_some_and(|tab| tab.kind == Kind::Json);
        if row(ui, app, Some(Icon::Code), "Format JSON", None, json) {
            app.format_json(false);
            ui.close();
        }
        if row(ui, app, Some(Icon::Code), "Minify JSON", None, json) {
            app.format_json(true);
            ui.close();
        }
    });
}

fn format_menu(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    let enabled = app
        .active_tab()
        .is_some_and(|tab| tab.capabilities().metadata);
    ui.menu_button(menu_label("Format"), |ui| {
        ui.set_min_width(230.0);
        if row(ui, app, Some(Icon::Bold), "Bold", None, enabled) {
            apply_wrap(app, ctx, "**", "**");
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Italic),
            "Italic",
            Some("Ctrl I"),
            enabled,
        ) {
            apply_wrap(app, ctx, "*", "*");
            ui.close();
        }
        if row(ui, app, Some(Icon::Code), "Inline code", None, enabled) {
            apply_wrap(app, ctx, "`", "`");
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        for level in 1..=3u8 {
            let label = format!("Heading {level}");
            if row(ui, app, Some(Icon::Heading), &label, None, enabled) {
                apply_line_prefix(app, ctx, &format!("{} ", "#".repeat(level as usize)));
                ui.close();
            }
        }
        widgets::divider(ui, &app.theme);
        if row(ui, app, Some(Icon::List), "Bullet list", None, enabled) {
            apply_line_prefix(app, ctx, "- ");
            ui.close();
        }
        if row(ui, app, Some(Icon::Check), "Task list", None, enabled) {
            apply_line_prefix(app, ctx, "- [ ] ");
            ui.close();
        }
        if row(ui, app, Some(Icon::Quote), "Quote", None, enabled) {
            apply_line_prefix(app, ctx, "> ");
            ui.close();
        }
        if row(ui, app, Some(Icon::Code), "Code block", None, enabled) {
            apply_wrap(app, ctx, "\n```\n", "\n```\n");
            ui.close();
        }
    });
}

fn view_menu(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    ui.menu_button(menu_label("View"), |ui| {
        ui.set_min_width(240.0);
        if row(
            ui,
            app,
            Some(Icon::PanelLeft),
            "File explorer",
            Some("Ctrl B"),
            true,
        ) {
            app.panels.left = !app.panels.left;
            app.persist_session();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::PanelRight),
            "Context panel",
            Some("Ctrl \\"),
            true,
        ) {
            app.panels.right = !app.panels.right;
            app.persist_session();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        let mode = app.active_tab().map(|tab| tab.mode);
        for (label, value) in [
            ("Edit only", ViewMode::Edit),
            ("Split view", ViewMode::Split),
            ("Reading view", ViewMode::Read),
        ] {
            let marker = if mode == Some(value) { "✓ " } else { "   " };
            if row(
                ui,
                app,
                None,
                &format!("{marker}{label}"),
                None,
                mode.is_some(),
            ) {
                if let Some(tab) = app.active_tab_mut() {
                    tab.mode = value;
                }
                ui.close();
            }
        }
        if row(
            ui,
            app,
            Some(Icon::Refresh),
            "Cycle view mode",
            Some("Ctrl E"),
            mode.is_some(),
        ) {
            app.run_command(CommandId::CycleView, ctx);
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        let numbers = app.settings.show_line_numbers;
        if row(
            ui,
            app,
            Some(Icon::List),
            &format!("{} Line numbers", if numbers { "✓" } else { "  " }),
            None,
            true,
        ) {
            app.run_command(CommandId::ToggleNumbers, ctx);
            ui.close();
        }
        let wrap = app.settings.wrap_code;
        if row(
            ui,
            app,
            Some(Icon::FileText),
            &format!("{} Wrap code lines", if wrap { "✓" } else { "  " }),
            None,
            true,
        ) {
            app.settings.wrap_code = !app.settings.wrap_code;
            app.persist_settings();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        let kind = app.settings.theme_kind();
        if row(
            ui,
            app,
            Some(if kind == crate::theme::ThemeKind::Dark {
                Icon::Sun
            } else {
                Icon::Moon
            }),
            if kind == crate::theme::ThemeKind::Dark {
                "Switch to light theme"
            } else {
                "Switch to dark theme"
            },
            None,
            true,
        ) {
            app.set_theme(ctx, kind.toggled());
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Settings),
            "Settings…",
            Some("Ctrl ,"),
            true,
        ) {
            app.settings_open = true;
            ui.close();
        }
    });
}

fn insert_menu(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    let enabled = app.active.is_some();
    ui.menu_button(menu_label("Insert"), |ui| {
        ui.set_min_width(230.0);
        if row(ui, app, Some(Icon::History), "Current date", None, enabled) {
            insert_text(
                app,
                ctx,
                &chrono::Local::now().format("%Y-%m-%d").to_string(),
            );
            ui.close();
        }
        if row(ui, app, Some(Icon::History), "Current time", None, enabled) {
            insert_text(app, ctx, &chrono::Local::now().format("%H:%M").to_string());
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(ui, app, Some(Icon::Link), "Wiki link", None, enabled) {
            insert_text(app, ctx, "[[]]");
            ui.close();
        }
        if row(ui, app, Some(Icon::Table), "Table", None, enabled) {
            insert_text(app, ctx, "\n| Column | Column |\n| --- | --- |\n|  |  |\n");
            ui.close();
        }
        if row(ui, app, Some(Icon::Code), "Code block", None, enabled) {
            insert_text(app, ctx, "\n```\n\n```\n");
            ui.close();
        }
        if row(ui, app, Some(Icon::Quote), "Callout", None, enabled) {
            insert_text(app, ctx, "\n> [!note]\n> \n");
            ui.close();
        }
    });
}

fn navigate_menu(app: &mut App, ui: &mut egui::Ui, _ctx: &egui::Context) {
    ui.menu_button(menu_label("Navigate"), |ui| {
        ui.set_min_width(240.0);
        if row(
            ui,
            app,
            Some(Icon::Search),
            "Go to file…",
            Some("Ctrl K"),
            true,
        ) {
            app.palette.open = true;
            app.palette.mode = PaletteMode::Files;
            app.palette.query.clear();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Command),
            "Command list…",
            Some("Ctrl P"),
            true,
        ) {
            app.palette.open = true;
            app.palette.mode = PaletteMode::Commands;
            app.palette.query.clear();
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(
            ui,
            app,
            Some(Icon::ArrowLeft),
            "Previous tab",
            Some("Alt ←"),
            !app.tabs.is_empty(),
        ) {
            let count = app.tabs.len();
            let previous = app.active.map_or(0, |index| (index + count - 1) % count);
            app.active = Some(previous);
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::ArrowRight),
            "Next tab",
            Some("Alt →"),
            !app.tabs.is_empty(),
        ) {
            let next = app.active.map_or(0, |index| (index + 1) % app.tabs.len());
            app.active = Some(next);
            ui.close();
        }
        if row(ui, app, Some(Icon::List), "Outline", None, true) {
            app.context = crate::app::ContextTab::Outline;
            app.panels.right = true;
            ui.close();
        }
        if row(ui, app, Some(Icon::Link), "Backlinks", None, true) {
            app.context = crate::app::ContextTab::Links;
            app.panels.right = true;
            app.recompute_backlinks();
            ui.close();
        }
    });
}

fn tools_menu(app: &mut App, ui: &mut egui::Ui, _ctx: &egui::Context) {
    ui.menu_button(menu_label("Tools"), |ui| {
        ui.set_min_width(240.0);
        if row(
            ui,
            app,
            Some(Icon::Refresh),
            "Rescan vault",
            None,
            app.vault.is_some(),
        ) {
            app.refresh_vault();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Search),
            "Search vault",
            None,
            app.vault.is_some(),
        ) {
            if let Some(tab) = app.active_tab() {
                let _ = tab;
            }
            app.search.opened_folder = true;
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(
            ui,
            app,
            Some(Icon::Download),
            "Export document…",
            None,
            app.active.is_some(),
        ) {
            app.export_active();
            ui.close();
        }
        if row(
            ui,
            app,
            Some(Icon::Settings),
            "Settings…",
            Some("Ctrl ,"),
            true,
        ) {
            app.settings_open = true;
            ui.close();
        }
        widgets::divider(ui, &app.theme);
        if row(
            ui,
            app,
            Some(Icon::FolderOpen),
            "Open data folder",
            None,
            true,
        ) {
            let dir = crate::settings::data_dir();
            let _ = std::fs::create_dir_all(&dir);
            app.open_folder(&dir);
            ui.close();
        }
    });
}

fn plugins_menu(app: &mut App, ui: &mut egui::Ui) {
    ui.menu_button(menu_label("Plugins"), |ui| {
        ui.set_min_width(260.0);
        if row(ui, app, Some(Icon::List), "Plugin status…", None, true) {
            app.plugins_open = true;
            ui.close();
        }
    });
}

fn help_menu(app: &mut App, ui: &mut egui::Ui) {
    ui.menu_button(menu_label("Help"), |ui| {
        ui.set_min_width(240.0);
        if row(
            ui,
            app,
            Some(Icon::Command),
            "Keyboard shortcuts",
            None,
            true,
        ) {
            app.shortcuts_open = true;
            ui.close();
        }
        if row(ui, app, Some(Icon::FileText), "About NoteCraft", None, true) {
            app.about_open = true;
            ui.close();
        }
    });
}

// ---------------------------------------------------------------- editing

fn active_editor_id(app: &App) -> Option<egui::Id> {
    app.active_tab().map(|tab| crate::ui::editor_id(&tab.path))
}

/// Byte range of the current selection (or the caret position twice).
fn selection(app: &App, ctx: &egui::Context) -> Option<(usize, usize)> {
    let id = active_editor_id(app)?;
    let tab = app.active_tab()?;
    let state = egui::text_edit::TextEditState::load(ctx, id)?;
    let range = state.cursor.char_range()?;
    let to_byte = |index: usize| {
        tab.text
            .char_indices()
            .nth(index)
            .map(|(byte, _)| byte)
            .unwrap_or(tab.text.len())
    };
    let a = to_byte(range.primary.index.0);
    let b = to_byte(range.secondary.index.0);
    Some((a.min(b), a.max(b)))
}

fn apply_wrap(app: &mut App, ctx: &egui::Context, prefix: &str, suffix: &str) {
    let Some((start, end)) = selection(app, ctx) else {
        return;
    };
    let Some(tab) = app.active_tab_mut() else {
        return;
    };
    let selected = tab.text[start..end].to_string();
    tab.text
        .replace_range(start..end, &format!("{prefix}{selected}{suffix}"));
    tab.on_edited();
}

fn apply_line_prefix(app: &mut App, ctx: &egui::Context, prefix: &str) {
    let Some((start, end)) = selection(app, ctx) else {
        return;
    };
    let Some(tab) = app.active_tab_mut() else {
        return;
    };
    let line_start = tab.text[..start]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    tab.text.insert_str(line_start, prefix);
    // Apply to every line in the selection.
    let mut cursor = line_start + prefix.len() + (end - start);
    while let Some(offset) = tab.text[cursor..].find('\n') {
        let next_line = cursor + offset + 1;
        if next_line > end + prefix.len() {
            break;
        }
        if next_line >= tab.text.len() {
            break;
        }
        tab.text.insert_str(next_line, prefix);
        cursor = next_line + prefix.len();
    }
    tab.on_edited();
}

fn insert_text(app: &mut App, ctx: &egui::Context, text: &str) {
    let at = selection(app, ctx).map(|(_, end)| end);
    let Some(tab) = app.active_tab_mut() else {
        return;
    };
    let position = at.unwrap_or(tab.text.len());
    tab.text.insert_str(position, text);
    tab.on_edited();
}
