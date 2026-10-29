pub mod ai;
pub mod automation;
pub mod commands;
pub mod doc_types;
pub mod engine;
pub mod error;
pub mod graph;
pub mod history;
pub mod import_export;
pub mod json_doc;
pub mod links;
pub mod markdown;
pub mod metadata;
pub mod plugins;
pub mod search;
pub mod secure;
pub mod settings;
pub mod state;
pub mod sync;
pub mod tasks;
pub mod templates;
pub mod vault;
pub mod workspace;

use state::AppState;

pub fn run() {
    env_logger::init();
    let state = AppState::new();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::open_vault,
            commands::create_vault,
            commands::close_vault,
            commands::get_vault_root,
            commands::list_recent_vaults,
            commands::list_profiles,
            commands::current_profile,
            commands::create_profile,
            commands::unlock_profile,
            commands::lock_profile,
            commands::change_profile_password,
            commands::delete_profile,
            commands::list_dir,
            commands::list_all_files,
            commands::read_file,
            commands::write_file,
            commands::write_file_if_unchanged,
            commands::create_file,
            commands::create_dir,
            commands::delete_path,
            commands::rename_path,
            commands::move_path,
            commands::parse_note,
            commands::render_html,
            commands::render_document,
            commands::document_capabilities,
            commands::document_template,
            commands::create_document,
            commands::json_analyze,
            commands::json_format,
            commands::json_minify,
            commands::export_note,
            commands::export_print_html,
            commands::save_text_file,
            commands::ai_providers,
            commands::ai_load_config,
            commands::ai_save_config,
            commands::ai_chat,
            commands::import_plugin_zip,
            commands::plugin_package,
            commands::search,
            commands::save_search,
            commands::list_saved_searches,
            commands::delete_saved_search,
            commands::search_history,
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
            commands::create_daily_note,
            commands::append_capture,
            commands::sync_push,
            commands::sync_pull,
            commands::sync_status_folder,
            commands::rename_tag,
            commands::unlinked_mentions,
            commands::propagate_link_rename,
            commands::list_collections,
            commands::save_collection,
            commands::delete_collection,
            commands::collection_notes,
            commands::plugin_catalog,
            commands::install_catalog_plugin,
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
