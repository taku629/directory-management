//! Sift core library.
//!
//! Wires the Tauri app: plugins, application state, and the IPC command
//! surface exposed to the React frontend.

mod ai;
mod cloud;
mod commands;
mod db;
mod duplicates;
mod error;
mod fs;
mod license;
mod rules;
mod search;
mod state;
mod telemetry;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let state = AppState::init(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ---- Phase 1: files ----
            commands::files::list_dir,
            commands::files::get_file,
            commands::files::index_directory,
            commands::files::update_file_metadata,
            commands::files::open_in_explorer,
            commands::files::move_file,
            commands::files::rename_file,
            commands::files::delete_file,
            // ---- Phase 1: roots / favorites / smart folders ----
            commands::files::add_watched_root,
            commands::files::list_watched_roots,
            commands::files::remove_watched_root,
            commands::files::add_favorite,
            commands::files::list_favorites,
            commands::files::remove_favorite,
            commands::files::create_smart_folder,
            commands::files::list_smart_folders,
            commands::files::delete_smart_folder,
            // ---- Phase 1: tags ----
            commands::tags::create_tag,
            commands::tags::list_tags,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::tag_file,
            commands::tags::untag_file,
            commands::tags::get_file_tags,
            commands::tags::list_files_by_tag,
            // ---- Phase 1: search ----
            commands::search::search_files,
            // ---- Phase 1: settings ----
            commands::settings::get_setting,
            commands::settings::set_setting,
            // ---- Phase 2: rules ----
            commands::rules::create_rule,
            commands::rules::list_rules,
            commands::rules::update_rule,
            commands::rules::delete_rule,
            commands::rules::run_rule_now,
            // ---- Phase 2: watcher ----
            commands::watcher::start_watcher,
            commands::watcher::stop_watcher,
            commands::watcher::watcher_status,
            // ---- Phase 2: duplicates ----
            commands::duplicates::find_duplicates,
            commands::duplicates::find_similar_images,
            // ---- Phase 2: disk usage ----
            commands::disk::compute_disk_usage,
            // ---- Phase 2: history / undo ----
            commands::history::list_operations,
            commands::history::undo_operation,
            // ---- Phase 3: AI ----
            commands::ai::ai_tag_file,
            commands::ai::ai_classify_file,
            commands::ai::ai_search,
            commands::ai::ai_summarize,
            commands::ai::ocr_file,
            // ---- Phase 4: license ----
            commands::license::get_license,
            commands::license::activate_license,
            commands::license::deactivate_license,
            commands::license::get_entitlement,
            // ---- Phase 4: cloud ----
            commands::cloud::list_cloud_providers,
            commands::cloud::configure_cloud,
            commands::cloud::sync_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
