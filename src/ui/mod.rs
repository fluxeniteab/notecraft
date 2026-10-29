//! The native interface, split by region of the shell.

pub mod context_panel;
pub mod dialogs;
pub mod editor;
pub mod json_tree;
pub mod menubar;
pub mod palette;
pub mod preview;
pub mod sidebar;
pub mod status;
pub mod tabs;
pub mod titlebar;
pub mod toasts;
pub mod widgets;
pub mod workspace;

use crate::app::App;

/// A stable widget id from a hashable salt.
///
/// egui 0.37 deprecates `Id::new` in favour of salted ids; this keeps the call
/// in one place so the rest of the UI doesn't repeat a deprecated constructor.
#[allow(deprecated)]
pub fn id(salt: impl std::hash::Hash + std::fmt::Debug) -> egui::Id {
    egui::Id::new(salt)
}

/// Stable widget id for a document's editor surface.
pub fn editor_id(path: &str) -> egui::Id {
    id(("nc-editor", path))
}

/// A separator between the editor and the reading view.
pub fn split_handle(ui: &mut egui::Ui, app: &mut App) {
    let theme = &app.theme;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(6.0, ui.available_height()), egui::Sense::drag());
    let hovered = response.hovered() || response.dragged();
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(0),
        if hovered {
            theme.palette.accent_soft(0.35)
        } else {
            theme.palette.border
        },
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
}
