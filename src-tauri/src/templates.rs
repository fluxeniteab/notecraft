use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub path: String,
    pub name: String,
}

pub fn list(state: &State<AppState>) -> AppResult<Vec<TemplateInfo>> {
    let g = state.inner.lock();
    let root = g.vault_root.clone().ok_or(AppError::NoVault)?;
    drop(g);
    // templates in "templates" folder or .notecraft/templates
    let mut out = vec![];
    for folder in ["templates", ".notecraft/templates"] {
        let dir = root.join(folder);
        if dir.is_dir() {
            for entry in walkdir::WalkDir::new(&dir)
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file()
                    || !entry.path().extension().is_some_and(|ext| ext == "md")
                {
                    continue;
                }
                let Ok(relative) = entry.path().strip_prefix(&root) else {
                    continue;
                };
                let path = relative.to_string_lossy().replace('\\', "/");
                let name = entry
                    .path()
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_default();
                out.push(TemplateInfo { path, name });
            }
        }
    }
    out.sort_by(|a, b| a.path.to_lowercase().cmp(&b.path.to_lowercase()));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutorial_note_has_outline_headings() {
        let note = tutorial_note();
        assert!(note.contains("# Welcome to NoteCraft"));
        // The outline panel reads ATX headings, so the seed note needs some.
        assert!(note.lines().filter(|line| line.starts_with("## ")).count() >= 4);
        assert!(note.contains("[["));
    }
}

/// The note seeded into every new profile. Keeping it in Rust means first run
/// works offline and the tutorial matches the shipped feature set.
pub fn tutorial_note() -> String {
    format!(
        r#"# Welcome to NoteCraft

Your notes live in an encrypted vault inside this app's data folder and never leave your machine. Press `Ctrl+K` for any command, `Ctrl+P` to jump to a note.

## Write anything

Notes are plain Markdown. Type `[[` to link another note, `#` plus a word for a tag, and `- [ ]` for a task.

- [ ] Read this note
- [ ] Create your first note with `Ctrl+N`
- [ ] Open the command palette with `Ctrl+K`

## Link your thinking

A link like [[Tutorial]] shows up as a backlink on the other note, and the graph view draws the connection.

> Tip: open the right sidebar with `Ctrl+\` to see backlinks and the outline.

## Capture fast

`Ctrl+Shift+N` opens quick capture and appends to your inbox note. `Ctrl+Alt+N` opens today's daily note.

## Stay safe

- Notes are encrypted with a key derived from your password.
- Lock the app from the command palette when you step away.
- Snapshots are taken automatically so you can restore an older version.

## Where things are

| What | Shortcut |
| --- | --- |
| Command palette | `Ctrl+K` |
| Quick open | `Ctrl+P` |
| New note | `Ctrl+N` |
| Search everywhere | `Ctrl+Shift+F` |
| Toggle sidebar | `Ctrl+B` |
| Focus mode | `Ctrl+Shift+F` |

Delete this note whenever you like.
"#
    )
}

pub fn render(
    state: &State<AppState>,
    path: &str,
    vars: &std::collections::HashMap<String, String>,
) -> AppResult<String> {
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone().ok_or(AppError::NoVault)?
    };
    let content = crate::vault::read_file(&root, path)?;
    let now = chrono::Utc::now();
    let mut out = content;
    // built-in vars
    let title = vars.get("title").cloned().unwrap_or_default();
    let folder = vars.get("folder").cloned().unwrap_or_default();
    out = out.replace("{{title}}", &title);
    out = out.replace("{{folder}}", &folder);
    out = out.replace("{{date}}", &now.format("%Y-%m-%d").to_string());
    out = out.replace("{{time}}", &now.format("%H:%M").to_string());
    out = out.replace("{{datetime}}", &now.to_rfc3339());
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{}}}}}", k), v);
    }
    Ok(out)
}
