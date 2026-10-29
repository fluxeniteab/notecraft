//! The reading view.
//!
//! Wraps the Markdown renderer and resolves what a click means: external URL,
//! heading anchor, or a link to another document in the vault.

use egui::Ui;

use crate::app::{App, ViewMode};
use crate::markdown::{self, PreviewAction};
use crate::theme::Theme;

pub fn render(ui: &mut Ui, theme: &Theme, text: &str) -> Option<PreviewAction> {
    markdown::render(ui, theme, text)
}

pub fn handle_action(app: &mut App, index: usize, action: PreviewAction) {
    match action {
        PreviewAction::ToggleTask(offset) => {
            app.active = Some(index);
            app.toggle_task(offset);
        }
        PreviewAction::Open(target) => open_target(app, index, &target),
    }
}

fn open_target(app: &mut App, index: usize, target: &str) {
    let target = target.trim();
    if target.is_empty() {
        return;
    }

    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
    {
        app.open_external(target);
        return;
    }

    if let Some(anchor) = target.strip_prefix('#') {
        jump_to_anchor(app, index, anchor);
        return;
    }

    // Split `file.md#heading`.
    let (path_part, anchor) = match target.split_once('#') {
        Some((path, anchor)) => (path, Some(anchor)),
        None => (target, None),
    };

    let folder = app.tabs[index].folder();
    let candidate = if path_part.contains('/') || folder.is_empty() {
        path_part.to_string()
    } else {
        format!("{folder}/{path_part}")
    };

    let resolved = if app.entries.iter().any(|entry| entry.path == candidate) {
        Some(candidate)
    } else {
        // Allow extension-less wiki links: `Note` → `Note.md`.
        app.entries
            .iter()
            .find(|entry| {
                !entry.is_dir
                    && entry.path.rsplit('/').next().is_some_and(|name| {
                        name.eq_ignore_ascii_case(path_part)
                            || name
                                .rsplit_once('.')
                                .is_some_and(|(stem, _)| stem.eq_ignore_ascii_case(path_part))
                    })
            })
            .map(|entry| entry.path.clone())
    };

    match resolved {
        Some(path) => {
            app.open_path(&path, false);
            if let Some(anchor) = anchor {
                if let Some(new_index) = app.active {
                    jump_to_anchor(app, new_index, anchor);
                }
            }
        }
        None => app.toast(
            "Link target not found",
            Some(format!("{path_part} is not in this vault yet")),
            true,
        ),
    }
}

fn jump_to_anchor(app: &mut App, index: usize, anchor: &str) {
    let Some(tab) = app.tabs.get(index) else {
        return;
    };
    let wanted = anchor.trim_start_matches('#');
    let line = tab
        .outline
        .iter()
        .find(|heading| heading.anchor == wanted)
        .map(|heading| heading.line)
        .or_else(|| {
            tab.outline
                .iter()
                .find(|heading| heading.anchor.eq_ignore_ascii_case(wanted))
                .map(|heading| heading.line)
        });
    match line {
        Some(line) => {
            if let Some(tab) = app.tabs.get_mut(index) {
                tab.goto_line = Some(line);
                tab.mode = ViewMode::Edit;
            }
        }
        None => app.toast(
            "Heading not found",
            Some(format!("No heading matches “{wanted}”")),
            true,
        ),
    }
}
