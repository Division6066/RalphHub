mod browser_agent;
mod commands;
mod kaizen;
mod memory_spine;
mod models;
mod ollama;
mod orchestrator;
mod state;
mod tool_registry;
mod voice;
mod workflow;
mod workflow_runner;

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
      commands::get_key_fields,
      commands::connect_and_test_tool,
      commands::launch_in_cursor_desktop,
      commands::launch_in_cursor_agent_web,
      commands::launch_google_codex,
      // Deploy / Orchestrator
      orchestrator::deploy_to_colab,
      orchestrator::deploy_to_pc,
      orchestrator::deploy_tool_by_id,
      orchestrator::inject_keys,
      orchestrator::list_managed_projects,
      // Workflows
      workflow::create_workflow_run,
      workflow::list_workflow_runs,
      // Ollama
      ollama::get_ollama_status,
      ollama::ensure_ollama,
      ollama::pull_ollama_model,
      ollama::pull_recommended_models,
      ollama::list_ollama_models,
      ollama::start_ollama_server,
      // Voice
      voice::get_voice_config,
      voice::ensure_voice,
      voice::check_voice_system,
      voice::list_piper_voices,
      // Memory Spine
      memory_spine::write_memory_entry,
      memory_spine::list_memory_entries,
      memory_spine::delete_memory_entry,
      memory_spine::write_run_report,
      // Kaizen Tasks
      kaizen::create_kaizen_task,
      kaizen::list_kaizen_tasks,
      kaizen::update_kaizen_task_status,
      kaizen::delete_kaizen_task,
      // Browser Agent
      browser_agent::get_browser_sessions,
      browser_agent::connect_browser_mcp,
      browser_agent::disconnect_browser_mcp,
      browser_agent::ensure_playwright,
      browser_agent::launch_browser_with_profile,
      browser_agent::get_mcp_connection_instructions,
      browser_agent::check_mcp_server_status,
      // Workflow runner
      workflow_runner::run_voice_full_stack,
      workflow_runner::transcribe_audio_file,
      workflow_runner::speak_text,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
