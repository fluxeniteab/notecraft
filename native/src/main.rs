//! NoteCraft - native entry point.
//!
//! A Rust + egui desktop application: no webview, no Node.js, no Tauri runtime.

mod app;
mod documents;
mod filesystem;
mod json;
mod links;
mod markdown;
mod mdx;
mod search;
mod session;
mod settings;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("NoteCraft")
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([880.0, 560.0])
            .with_app_id("notecraft"),
        ..Default::default()
    };

    eframe::run_native(
        "NoteCraft",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
