// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NewNote,
    NewFolder,
    Save,
    CloseTab,
    SwitchModeEdit,
    SwitchModeSplit,
    SwitchModeRead,
    TogglePreview,
    SearchVault,
    SearchDocument,
    OpenSettings,
    RefreshExplorer,
    OpenCommandPalette,
}

pub fn command_label(cmd: Command) -> &'static str {
    match cmd {
        Command::NewNote => "Create new note",
        Command::NewFolder => "Create new folder",
        Command::Save => "Save current note",
        Command::CloseTab => "Close current tab",
        Command::SwitchModeEdit => "Switch to edit mode",
        Command::SwitchModeSplit => "Switch to split mode",
        Command::SwitchModeRead => "Switch to reading mode",
        Command::TogglePreview => "Toggle preview",
        Command::SearchVault => "Search vault",
        Command::SearchDocument => "Find in document",
        Command::OpenSettings => "Open settings",
        Command::RefreshExplorer => "Refresh file explorer",
        Command::OpenCommandPalette => "Open command palette",
    }
}
