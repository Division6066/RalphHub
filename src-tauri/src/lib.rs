mod browser;
mod commands;
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
    .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      let salt_path = app
        .path()
        .app_local_data_dir()
        .expect("could not resolve local app data path")
        .join("stronghold-salt.txt");

      app.handle()
        .plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;

      let state = AppState::init(&app.handle()).expect("failed to initialize RalphHub state");
      app.manage(state);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Core
      commands::ensure_bun,
      commands::get_dashboard_snapshot,
      commands::get_editor_candidates,
      commands::get_secure_store_config,
      commands::list_builtin_tools,
      commands::open_in_code,
      // Deploy / orchestration
      orchestrator::deploy_to_colab,
      orchestrator::deploy_to_pc,
      orchestrator::inject_keys,
      orchestrator::list_managed_projects,
      // Workflows
      workflow::create_workflow_run,
      workflow::list_workflow_runs,
      // Browser agent
      browser::approve_browser_action,
      browser::get_browser_settings,
      browser::get_edge_profile_config,
      browser::launch_browser_with_profile,
      browser::list_browser_actions,
      browser::log_browser_action,
      browser::open_colab_url,
      browser::save_browser_settings,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
