mod commands;
mod models;
mod orchestrator;
mod state;
mod tool_registry;

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

      let state = AppState::init(&app.handle()).expect("failed to initialize RalphHub state");
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
      orchestrator::deploy_to_colab,
      orchestrator::deploy_to_pc,
      orchestrator::inject_keys,
      orchestrator::list_managed_projects,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
