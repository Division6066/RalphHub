use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::State;
use uuid::Uuid;

use crate::{
    models::{
        ApiUsageLog, CommandResponse, CreateKaizenTaskRequest, CreateProviderRequest,
        DashboardSnapshot, KaizenTask, LaunchBackgroundRequest, LogApiUsageRequest,
        MemorySpineEntry, MemorySpineStats, ParallelWorkflowRequest,
        ParallelWorkflowResult, Provider, SecureStoreConfig, ToolLogsResult, ToolManifest,
        ToolProcessStatus, UpdateProviderRequest, VoiceCommandRequest, VoiceCommandResult,
    },
    process_manager,
    provider_registry::{
        create_kaizen_task, create_provider, delete_provider, get_memory_spine_stats,
        list_kaizen_tasks, list_memory_entries, list_providers, list_usage_logs, log_api_usage,
        search_providers, update_kaizen_task_status, update_provider,
    },
    state::{bun_installer_hint, detect_bun_status, AppState},
    tool_registry::{all_tools, get_tool},
};

#[tauri::command]
pub fn get_dashboard_snapshot(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
    state.snapshot().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_builtin_tools() -> Vec<ToolManifest> {
    all_tools()
}

#[tauri::command]
pub fn ensure_bun() -> Result<CommandResponse, String> {
    let bun = detect_bun_status();
    if bun.installed {
        return Ok(CommandResponse {
            ok: true,
            message: format!(
                "Bun is already available{}",
                bun.version
                    .map(|version| format!(" ({version})"))
                    .unwrap_or_default()
            ),
        });
    }

    let status = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-c", "irm bun.sh/install.ps1|iex"])
            .status()
    } else {
        Command::new("sh")
            .args(["-c", "curl -fsSL https://bun.sh/install | bash"])
            .status()
    }
    .map_err(|error| error.to_string())?;

    if status.success() {
        Ok(CommandResponse {
            ok: true,
            message: "Bun installed successfully. Relaunch managed operations to pick up the new PATH."
                .to_string(),
        })
    } else {
        Err(format!(
            "Bun installation failed. Run {} manually and restart RalphHub.",
            bun_installer_hint()
        ))
    }
}

#[tauri::command]
pub fn open_in_code(workspace_path: String, branch: Option<String>) -> Result<CommandResponse, String> {
    let workspace = PathBuf::from(&workspace_path);
    if !workspace.exists() {
        return Err(format!("Workspace does not exist: {workspace_path}"));
    }

    // `branch` is informational only; the repo is already on the correct branch
    // after deploy_to_pc. Free-form git checkout is not performed here because
    // it creates an unnecessary Git invocation surface and is redundant in the
    // current call flow.
    let _ = branch;

    let state_file = ensure_state_file(&workspace).map_err(|error| error.to_string())?;
    let launched = launch_editor(&workspace, &state_file).map_err(|error| error.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!("Opened workspace in {launched}."),
    })
}

#[tauri::command]
pub fn get_editor_candidates() -> Vec<String> {
    editor_candidates()
}

#[tauri::command]
pub fn get_secure_store_config(state: State<'_, AppState>) -> Result<SecureStoreConfig, String> {
    let username = env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "unknown-user".to_string());
    let machine = env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-machine".to_string());

    Ok(SecureStoreConfig {
        vault_path: state
            .paths
            .app_data_dir
            .join("ralphhub.vault.hold")
            .display()
            .to_string(),
        client_name: "ralphhub-keys".to_string(),
        vault_password: format!("ralphhub::{machine}::{username}::stronghold"),
    })
}

fn ensure_state_file(workspace: &Path) -> Result<PathBuf, String> {
    let path = workspace.join("STATE.md");
    if !path.exists() {
        fs::write(
            &path,
            "# RalphHub State\n\n- Status: initialized\n- Next step: update this file from the active workflow.\n",
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(path)
}

fn launch_editor(workspace: &Path, state_file: &Path) -> Result<String, String> {
    let candidates = default_editor_commands(workspace, state_file);

    for (label, program, args) in candidates {
        let mut command = Command::new(program);
        command.args(args.clone());
        command.current_dir(workspace);

        if command.status().map(|status| status.success()).unwrap_or(false) {
            return Ok(label.to_string());
        }
    }

    Err(format!(
        "Unable to open an editor automatically for {} (expected Cursor or VS Code).",
        workspace.display()
    ))
}

fn default_editor_commands(workspace: &Path, state_file: &Path) -> Vec<(&'static str, String, Vec<String>)> {
    let candidates = detect_editor_paths();
    let mut commands = Vec::new();
    let workspace_arg = workspace.display().to_string();
    let state_target = format!("{}:1", state_file.display());

    if let Some(cursor) = candidates.cursor {
        commands.push((
            "Cursor",
            cursor,
            vec![
                workspace_arg.clone(),
                "--reuse-window".to_string(),
                "-g".to_string(),
                state_target.clone(),
            ],
        ));
    }

    if let Some(code) = candidates.code {
        commands.push((
            "Visual Studio Code",
            code,
            vec![
                workspace_arg.clone(),
                "--reuse-window".to_string(),
                "-g".to_string(),
                state_target.clone(),
            ],
        ));
    }

    if cfg!(target_os = "macos") && commands.is_empty() {
        commands.push(("Open", "open".to_string(), vec!["-a".to_string(), "Cursor".to_string(), workspace_arg]));
    }

    commands
}

struct EditorPaths {
    cursor: Option<String>,
    code: Option<String>,
}

fn detect_editor_paths() -> EditorPaths {
    EditorPaths {
        cursor: first_existing(&[
            env::var("CURSOR").ok(),
            Some("cursor".to_string()),
            windows_program("Cursor", "Cursor.exe"),
            windows_program("Cursor", "cursor.exe"),
        ]),
        code: first_existing(&[
            env::var("VSCODE").ok(),
            Some("code".to_string()),
            windows_program("Microsoft VS Code", "Code.exe"),
            windows_program("Microsoft VS Code", "code.exe"),
        ]),
    }
}

fn windows_program(folder: &str, executable: &str) -> Option<String> {
    let local_app_data = env::var("LOCALAPPDATA").ok()?;
    let path = Path::new(&local_app_data)
        .join("Programs")
        .join(folder)
        .join(executable);
    path.exists().then(|| path.display().to_string())
}

fn first_existing(options: &[Option<String>]) -> Option<String> {
    options.iter().flatten().find_map(|candidate| {
        if candidate.contains(std::path::MAIN_SEPARATOR) {
            Path::new(candidate).exists().then(|| candidate.clone())
        } else {
            Some(candidate.clone())
        }
    })
}

fn editor_candidates() -> Vec<String> {
    let paths = detect_editor_paths();
    let mut candidates = Vec::new();

    if let Some(cursor) = paths.cursor {
        candidates.push(cursor);
    }

    if let Some(code) = paths.code {
        candidates.push(code);
    }

    candidates
}

// ─── Provider Registry Commands ───────────────────────────────────────────────

#[tauri::command]
pub fn list_providers_cmd(state: State<'_, AppState>, category: Option<String>) -> Result<Vec<Provider>, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    list_providers(&conn, category.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_provider_cmd(state: State<'_, AppState>, req: CreateProviderRequest) -> Result<Provider, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    create_provider(&conn, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_provider_cmd(state: State<'_, AppState>, req: UpdateProviderRequest) -> Result<Provider, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    update_provider(&conn, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_provider_cmd(state: State<'_, AppState>, id: String) -> Result<CommandResponse, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    delete_provider(&conn, &id).map_err(|e| e.to_string())?;
    Ok(CommandResponse { ok: true, message: format!("Provider {id} deleted.") })
}

#[tauri::command]
pub fn search_providers_cmd(state: State<'_, AppState>, query: String) -> Result<Vec<Provider>, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    search_providers(&conn, &query).map_err(|e| e.to_string())
}

// ─── API Usage Logging Commands ───────────────────────────────────────────────

#[tauri::command]
pub fn log_api_usage_cmd(state: State<'_, AppState>, req: LogApiUsageRequest) -> Result<ApiUsageLog, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    log_api_usage(&conn, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_usage_logs_cmd(state: State<'_, AppState>, limit: Option<i64>) -> Result<Vec<ApiUsageLog>, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    list_usage_logs(&conn, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_memory_spine_stats_cmd(state: State<'_, AppState>) -> Result<MemorySpineStats, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    get_memory_spine_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_memory_entries_cmd(state: State<'_, AppState>, limit: Option<i64>) -> Result<Vec<MemorySpineEntry>, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    list_memory_entries(&conn, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

// ─── Kaizen Task Commands ─────────────────────────────────────────────────────

#[tauri::command]
pub fn create_kaizen_task_cmd(state: State<'_, AppState>, req: CreateKaizenTaskRequest) -> Result<KaizenTask, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    create_kaizen_task(&conn, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_kaizen_tasks_cmd(state: State<'_, AppState>, status: Option<String>) -> Result<Vec<KaizenTask>, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    list_kaizen_tasks(&conn, status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_kaizen_task_status_cmd(state: State<'_, AppState>, id: String, status: String) -> Result<KaizenTask, String> {
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    update_kaizen_task_status(&conn, &id, &status).map_err(|e| e.to_string())
}

// ─── Background Process / Parallel Execution Commands ────────────────────────

#[tauri::command]
pub fn launch_tool_background(
    state: State<'_, AppState>,
    request: LaunchBackgroundRequest,
) -> Result<ToolProcessStatus, String> {
    let tool = get_tool(&request.tool_id)
        .ok_or_else(|| format!("Unknown tool: {}", request.tool_id))?;

    if tool.repo_url.starts_with("internal://") {
        return Err(format!("{} is an internal RalphHub capability and cannot be launched as a background process.", tool.name));
    }

    let status = process_manager::launch_background(
        &state.process_registry,
        &request.tool_id,
        &tool.name,
        &request.workspace_path,
        &tool.launch_command,
        &request.env_entries,
        &state.paths.logs_dir,
    )?;

    // Write to Memory Spine
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let spine_req = crate::models::LogApiUsageRequest {
        provider_id: "background-process".to_string(),
        provider_name: "Background Process".to_string(),
        model: "process".to_string(),
        tokens_in: 0,
        tokens_out: 0,
        cost_usd: 0.0,
        output_summary: format!(
            "Background launch: {} (pid:{}) at {}",
            tool.name,
            status.pid.unwrap_or(0),
            request.workspace_path
        ),
        tool_id: request.tool_id.clone(),
        workflow_id: String::new(),
    };
    let _ = log_api_usage(&conn, &spine_req);

    Ok(status)
}

#[tauri::command]
pub fn get_tool_process_status(
    state: State<'_, AppState>,
    tool_id: String,
) -> ToolProcessStatus {
    process_manager::get_status(&state.process_registry, &tool_id)
}

#[tauri::command]
pub fn stop_tool_process(
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<CommandResponse, String> {
    process_manager::stop_tool(&state.process_registry, &tool_id)?;

    // Log to Memory Spine
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let _ = log_api_usage(
        &conn,
        &crate::models::LogApiUsageRequest {
            provider_id: "background-process".to_string(),
            provider_name: "Background Process".to_string(),
            model: "process".to_string(),
            tokens_in: 0,
            tokens_out: 0,
            cost_usd: 0.0,
            output_summary: format!("Stopped background tool: {tool_id}"),
            tool_id: tool_id.clone(),
            workflow_id: String::new(),
        },
    );

    Ok(CommandResponse {
        ok: true,
        message: format!("Tool {tool_id} stopped."),
    })
}

#[tauri::command]
pub fn get_tool_logs(
    state: State<'_, AppState>,
    tool_id: String,
    tail_lines: Option<usize>,
) -> ToolLogsResult {
    let lines = process_manager::read_logs(
        &state.process_registry,
        &tool_id,
        tail_lines.unwrap_or(50),
    );

    let log_path = {
        let reg = state.process_registry.lock().unwrap_or_else(|e| e.into_inner());
        reg.get(&tool_id)
            .map(|t| t.log_path.clone())
            .unwrap_or_else(|| state.paths.logs_dir.join(format!("{tool_id}.log")).display().to_string())
    };

    ToolLogsResult { tool_id, log_path, lines }
}

#[tauri::command]
pub fn list_running_tools(state: State<'_, AppState>) -> Vec<ToolProcessStatus> {
    process_manager::list_all(&state.process_registry)
}

#[tauri::command]
pub fn run_parallel_workflow(
    state: State<'_, AppState>,
    request: ParallelWorkflowRequest,
) -> Result<ParallelWorkflowResult, String> {
    let workflow_id = Uuid::new_v4().to_string();
    let mut statuses = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for config in &request.tool_configs {
        let tool = match get_tool(&config.tool_id) {
            Some(t) => t,
            None => {
                errors.push(format!("Unknown tool: {}", config.tool_id));
                continue;
            }
        };

        match process_manager::launch_background(
            &state.process_registry,
            &config.tool_id,
            &tool.name,
            &config.workspace_path,
            &tool.launch_command,
            &config.env_entries,
            &state.paths.logs_dir,
        ) {
            Ok(status) => statuses.push(status),
            Err(e) => {
                errors.push(format!("{}: {e}", config.tool_id));
                statuses.push(ToolProcessStatus {
                    tool_id: config.tool_id.clone(),
                    name: tool.name.clone(),
                    status: format!("error: {e}"),
                    pid: None,
                    started_at: None,
                    log_path: None,
                });
            }
        }
    }

    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let tool_names: Vec<String> = request
        .tool_configs
        .iter()
        .filter_map(|c| get_tool(&c.tool_id).map(|t| t.name))
        .collect();

    let summary = format!(
        "Parallel workflow '{}' launched tools: {} | workflow_id: {}{}",
        request.workflow_name,
        tool_names.join(", "),
        workflow_id,
        if errors.is_empty() {
            String::new()
        } else {
            format!(" | errors: {}", errors.join("; "))
        }
    );

    let usage_log = log_api_usage(
        &conn,
        &crate::models::LogApiUsageRequest {
            provider_id: "parallel-executor".to_string(),
            provider_name: "Parallel Executor".to_string(),
            model: "parallel".to_string(),
            tokens_in: 0,
            tokens_out: 0,
            cost_usd: 0.0,
            output_summary: summary.clone(),
            tool_id: "parallel-workflow".to_string(),
            workflow_id: workflow_id.clone(),
        },
    )
    .map_err(|e| e.to_string())?;

    let kaizen_task = create_kaizen_task(
        &conn,
        &crate::models::CreateKaizenTaskRequest {
            title: format!("Parallel workflow: {}", request.workflow_name),
            description: format!(
                "Tools: {} | Status: {} launched, {} errors | {}",
                tool_names.join(", "),
                statuses.iter().filter(|s| s.status == "running").count(),
                errors.len(),
                summary
            ),
            priority: "high".to_string(),
            source: "parallel-executor".to_string(),
            provider_id: "parallel-executor".to_string(),
            usage_log_id: usage_log.id.clone(),
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(ParallelWorkflowResult {
        workflow_id,
        workflow_name: request.workflow_name,
        statuses,
        memory_spine_id: usage_log.id,
        kaizen_task_id: kaizen_task.id,
    })
}

// ─── Voice Command Handler ────────────────────────────────────────────────────

#[tauri::command]
pub fn handle_voice_command(
    state: State<'_, AppState>,
    request: VoiceCommandRequest,
) -> Result<VoiceCommandResult, String> {
    let transcript = request.transcript.to_lowercase();

    // Parse voice command intent
    let result = if transcript.contains("superpowers") || transcript.contains("super powers") {
        VoiceCommandResult {
            action: "launch_tool".to_string(),
            tool_id: Some("superpowers".to_string()),
            message: "Superpowers skill framework queued for launch. Navigate to Tools to start.".to_string(),
            success: true,
        }
    } else if transcript.contains("video") || transcript.contains("diffusion") || transcript.contains("edit") {
        VoiceCommandResult {
            action: "launch_tool".to_string(),
            tool_id: Some("diffusionstudio-agent".to_string()),
            message: "Diffusionstudio video agent queued for launch in background.".to_string(),
            success: true,
        }
    } else if transcript.contains("parallel") || transcript.contains("both") {
        VoiceCommandResult {
            action: "launch_parallel".to_string(),
            tool_id: None,
            message: "Parallel workflow queued: Superpowers + Diffusionstudio Agent. Navigate to Parallel page.".to_string(),
            success: true,
        }
    } else if transcript.contains("stop") || transcript.contains("pause") || transcript.contains("halt") {
        let running = process_manager::list_all(&state.process_registry);
        let running_count = running.iter().filter(|s| s.status == "running").count();
        VoiceCommandResult {
            action: "stop_all".to_string(),
            tool_id: None,
            message: format!("Stopping {running_count} running background processes."),
            success: true,
        }
    } else if transcript.contains("status") || transcript.contains("report") {
        let running = process_manager::list_all(&state.process_registry);
        let running_names: Vec<&str> = running
            .iter()
            .filter(|s| s.status == "running")
            .map(|s| s.name.as_str())
            .collect();
        VoiceCommandResult {
            action: "status_report".to_string(),
            tool_id: None,
            message: if running_names.is_empty() {
                "No tools currently running in background.".to_string()
            } else {
                format!("Running tools: {}", running_names.join(", "))
            },
            success: true,
        }
    } else {
        VoiceCommandResult {
            action: "unknown".to_string(),
            tool_id: None,
            message: format!(
                "Voice command not recognized: '{}'. Try: 'launch superpowers', 'start video edit', 'run parallel', 'stop all', 'status report'.",
                request.transcript
            ),
            success: false,
        }
    };

    // Log to Memory Spine
    let conn = rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let _ = log_api_usage(
        &conn,
        &crate::models::LogApiUsageRequest {
            provider_id: "voice-assistant".to_string(),
            provider_name: "Voice Assistant".to_string(),
            model: "speech-to-intent".to_string(),
            tokens_in: 0,
            tokens_out: 0,
            cost_usd: 0.0,
            output_summary: format!(
                "Voice command: '{}' → action: {} | {}",
                request.transcript, result.action, result.message
            ),
            tool_id: result.tool_id.clone().unwrap_or_else(|| "voice-command".to_string()),
            workflow_id: String::new(),
        },
    );

    Ok(result)
}