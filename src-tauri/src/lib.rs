mod automation;
mod commands;
mod error;
mod graph;
mod history;
mod import_export;
mod links;
mod markdown;
mod metadata;
mod plugins;
mod search;
mod security;
mod settings;
mod state;
mod sync;
mod tasks;
mod templates;
mod vault;
mod workspace;

use state::AppState;
use tauri::Manager;

pub fn run() {
    env_logger::init();
    let state = AppState::new();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::open_vault,
            commands::create_vault,
            commands::close_vault,
            commands::get_vault_root,
            commands::list_recent_vaults,
            commands::list_dir,
            commands::list_all_files,
            commands::read_file,
            commands::write_file,
            commands::create_file,
            commands::create_dir,
            commands::delete_path,
            commands::rename_path,
            commands::move_path,
            commands::parse_note,
            commands::render_html,
            commands::search,
            commands::rebuild_index,
            commands::index_status,
            commands::rebuild_links,
            commands::backlinks,
            commands::outgoing_links,
            commands::unresolved_links,
            commands::graph_data,
            commands::snapshot_now,
            commands::list_snapshots,
            commands::get_snapshot,
            commands::restore_snapshot,
            commands::diff_snapshots,
            commands::list_tasks,
            commands::set_task_done,
            commands::list_templates,
            commands::render_template,
            commands::get_settings,
            commands::save_settings,
            commands::list_workspaces,
            commands::save_workspace,
            commands::delete_workspace,
            commands::list_automations,
            commands::save_automation,
            commands::delete_automation,
            commands::run_automation_dry_run,
            commands::run_automation,
            commands::list_automation_runs,
            commands::list_plugins,
            commands::enable_plugin,
            commands::uninstall_plugin,
            commands::sync_status,
            commands::set_vault_password,
            commands::import_vault,
            commands::export_html,
            commands::backup_vault,
            commands::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
