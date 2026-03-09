mod commands;
mod models;
mod orchestrator;
mod process_manager;
mod provider_registry;
mod state;
mod tool_registry;
mod workflow;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
    .setup(|app| {
      let salt_path = app
        .path()
        .app_local_data_dir()
        .expect("could not resolve local app data path")
        .join("stronghold-salt.txt");

      app.handle()
        .plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;

      let state = AppState::init(app.handle()).expect("failed to initialize RalphHub state");

      // Seed default providers after DB init
      {
        let conn = rusqlite::Connection::open(&state.paths.database_path)
          .expect("failed to open database for seeding");
        provider_registry::seed_default_providers(&conn)
          .expect("failed to seed default providers");
      }

      app.manage(state);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::ensure_bun,
      commands::get_dashboard_snapshot,
      commands::get_editor_candidates,
      commands::get_secure_store_config,
      commands::list_builtin_tools,
      commands::open_in_code,
      // Provider registry
      commands::list_providers_cmd,
      commands::create_provider_cmd,
      commands::update_provider_cmd,
      commands::delete_provider_cmd,
      commands::search_providers_cmd,
      // API usage / memory spine
      commands::log_api_usage_cmd,
      commands::list_usage_logs_cmd,
      commands::get_memory_spine_stats_cmd,
      commands::list_memory_entries_cmd,
      // Kaizen tasks
      commands::create_kaizen_task_cmd,
      commands::list_kaizen_tasks_cmd,
      commands::update_kaizen_task_status_cmd,
      // Orchestration
      orchestrator::deploy_to_colab,
      orchestrator::deploy_to_pc,
      orchestrator::inject_keys,
      orchestrator::list_managed_projects,
      workflow::create_workflow_run,
      workflow::list_workflow_runs,
      // Background process / parallel execution
      commands::launch_tool_background,
      commands::get_tool_process_status,
      commands::stop_tool_process,
      commands::get_tool_logs,
      commands::list_running_tools,
      commands::run_parallel_workflow,
      // Voice command handler
      commands::handle_voice_command,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
