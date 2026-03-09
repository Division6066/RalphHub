mod commands;
mod kaizen;
mod memory;
mod mobile_sync;
mod models;
mod orchestrator;
mod state;
mod tool_registry;
mod workflow;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve local app data path")
                .join("stronghold-salt.txt");

            app.handle()
                .plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;

            let state =
                AppState::init(app.handle()).expect("failed to initialize AmitOS state");
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ── Core
            commands::ensure_bun,
            commands::get_dashboard_snapshot,
            commands::get_editor_candidates,
            commands::get_secure_store_config,
            commands::list_builtin_tools,
            commands::list_api_providers,
            commands::open_in_code,
            // ── Orchestrator
            orchestrator::deploy_to_colab,
            orchestrator::deploy_to_pc,
            orchestrator::inject_keys,
            orchestrator::list_managed_projects,
            // ── Workflow
            workflow::create_workflow_run,
            workflow::list_workflow_runs,
            // ── Kaizen
            kaizen::list_kaizen_tasks,
            kaizen::create_kaizen_task,
            kaizen::update_kaizen_task,
            kaizen::delete_kaizen_task,
            kaizen::set_today_tasks,
            kaizen::list_kaizen_domains,
            kaizen::decompose_task,
            // ── Memory
            memory::list_memory_entries,
            memory::create_memory_entry,
            memory::search_memory,
            memory::delete_memory_entry,
            memory::update_memory_entry,
            // ── Mobile Sync
            mobile_sync::get_mobile_sync_status,
            mobile_sync::enable_mobile_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AmitOS");
}
