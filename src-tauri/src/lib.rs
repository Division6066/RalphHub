mod commands;
mod computer_agent;
mod kaizen;
mod models;
mod orchestrator;
mod process_manager;
mod provider_registry;
mod state;
mod tool_registry;
mod voice_assistant;
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

      let state = AppState::init(app.handle()).expect("failed to initialize AmitOS state");

      // Seed default providers after DB init
      {
        let conn = rusqlite::Connection::open(&state.paths.database_path)
          .expect("failed to open database for seeding");
        provider_registry::seed_default_providers(&conn)
          .expect("failed to seed default providers");

        // Run computer agent + voice assistant migrations
        computer_agent::run_migrations(&conn)
          .expect("failed to run computer_agent migrations");
        voice_assistant::run_migrations(&conn)
          .expect("failed to run voice_assistant migrations");
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
      // ── Milestone 1 & 2: Computer Agent + Parallel Execution ──
      computer_agent::start_agent_session,
      computer_agent::list_agent_sessions,
      computer_agent::stop_agent_session,
      computer_agent::execute_agent_action,
      computer_agent::list_session_actions,
      // ── Milestone 2: Parallel Tasks ──
      computer_agent::create_parallel_task,
      computer_agent::list_parallel_tasks,
      computer_agent::update_parallel_task_status,
      // ── Milestone 3: Android / Panda ──
      computer_agent::list_android_devices,
      computer_agent::execute_adb_command,
      computer_agent::install_panda_apk,
      // ── Milestone 5: Remote Permissions ──
      computer_agent::request_permission,
      computer_agent::resolve_permission,
      computer_agent::list_permission_requests,
      // ── Milestone 6: VPS / RPi Nodes ──
      computer_agent::deploy_remote_node,
      computer_agent::list_remote_nodes,
      // ── Milestone 4: Voice / Chat ──
      voice_assistant::send_chat_message,
      voice_assistant::list_chat_sessions,
      voice_assistant::list_chat_messages,
      // ── Milestone 5: Push Notifications ──
      voice_assistant::create_push_notification,
      voice_assistant::list_push_notifications,
      voice_assistant::mark_notification_read,
      // ── Rich Kaizen Commands (for Today Board and full task management) ──
      kaizen::list_kaizen_tasks,
      kaizen::create_kaizen_task,
      kaizen::update_kaizen_task,
      kaizen::delete_kaizen_task,
      kaizen::set_today_tasks,
      kaizen::list_kaizen_domains,
      kaizen::decompose_task,
      // ── New Tools: Background Process / Parallel Execution ──
      commands::launch_tool_background,
      commands::get_tool_process_status,
      commands::stop_tool_process,
      commands::get_tool_logs,
      commands::list_running_tools,
      commands::run_parallel_workflow,
      commands::list_parallel_workflows,
      commands::handle_voice_command,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
