mod commands;
mod memory;
mod mobile_api;
mod models;
mod notion;
mod orchestrator;
mod state;
mod tasks;
mod tasks_inbox;
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

      let state = AppState::init(&app.handle()).expect("failed to initialize RalphHub state");
      app.manage(state);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // ── Core RalphHub ────────────────────────────────────────────────────
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
      // ── AmitOS Memory Spine ───────────────────────────────────────────────
      memory::ingest_memory,
      memory::list_raw_events,
      memory::list_working_memory,
      memory::save_working_memory,
      memory::list_long_term_memory,
      memory::promote_to_long_term,
      memory::list_structured_summaries,
      memory::get_memory_stats,
      // ── AmitOS Kaizen Tasks ───────────────────────────────────────────────
      tasks::create_kaizen_project,
      tasks::list_kaizen_projects,
      tasks::create_kaizen_task,
      tasks::list_kaizen_tasks,
      tasks::update_kaizen_task_status,
      tasks::update_kaizen_task_do_date,
      tasks::get_today_board,
      tasks::get_kanban_board,
      tasks::decompose_task,
      // ── AmitOS Inbox + Daily Log ──────────────────────────────────────────
      tasks_inbox::add_inbox_item,
      tasks_inbox::list_inbox_items,
      tasks_inbox::mark_inbox_processed,
      tasks_inbox::inbox_to_task,
      tasks_inbox::add_daily_log_entry,
      tasks_inbox::list_daily_log,
      tasks_inbox::get_morning_digest,
      // ── AmitOS Notion + Cursor Agent Web ─────────────────────────────────
      notion::sync_notion,
      notion::open_in_cursor_agent_web,
      notion::get_amitos_dashboard,
      // ── AmitOS Mobile Stage-2 API stubs ──────────────────────────────────
      mobile_api::mobile_get_dashboard,
      mobile_api::mobile_get_today_tasks,
      mobile_api::mobile_get_morning_digest,
      mobile_api::mobile_add_inbox,
      mobile_api::mobile_approve_task,
      mobile_api::mobile_get_inbox,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
