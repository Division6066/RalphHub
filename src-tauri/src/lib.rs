mod commands;
mod memory;
mod mobile_commands;
mod mobile_server;
mod models;
mod orchestrator;
mod state;
mod tool_registry;
mod workflow;

use memory::init_memory_tables;
use mobile_commands::{MobileServerPort, MobileServerInfo};
use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

const MOBILE_SERVER_PORT: u16 = 7842;

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

            let app_state = AppState::init(app.handle()).expect("failed to initialize RalphHub state");

            // Initialize memory spine tables
            init_memory_tables(&app_state.paths.database_path)
                .expect("failed to initialize memory tables");

            // Start mobile sync HTTP server in background
            let db_path = app_state.paths.database_path.clone();
            let port_holder: Arc<Mutex<Option<u16>>> = Arc::new(Mutex::new(None));
            let port_for_task = port_holder.clone();

            tokio::spawn(async move {
                match mobile_server::start_mobile_server(db_path, MOBILE_SERVER_PORT).await {
                    Ok(_) => {},
                    Err(e) => log::error!("Mobile server error: {}", e),
                }
            });

            *port_holder.lock().unwrap() = Some(MOBILE_SERVER_PORT);

            app.manage(app_state);
            app.manage(MobileServerPort(port_holder));

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
            workflow::create_workflow_run,
            workflow::list_workflow_runs,
            // Memory Spine commands
            mobile_commands::write_to_memory,
            mobile_commands::read_memory,
            mobile_commands::list_kaizen_tasks,
            mobile_commands::get_mobile_server_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
