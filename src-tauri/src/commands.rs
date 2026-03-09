use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::State;

use crate::{
    models::{
        ApiUsageLog, CommandResponse, CreateKaizenTaskRequest, CreateProviderRequest,
        DashboardSnapshot, KaizenTask, LogApiUsageRequest, MemorySpineEntry, MemorySpineStats,
        Provider, SecureStoreConfig, ToolManifest, UpdateProviderRequest,
    },
    provider_registry::{
        create_kaizen_task, create_provider, delete_provider, get_memory_spine_stats,
        list_kaizen_tasks, list_memory_entries, list_providers, list_usage_logs, log_api_usage,
        search_providers, update_kaizen_task_status, update_provider,
    },
    state::{bun_installer_hint, detect_bun_status, AppState},
    tool_registry::all_tools,
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
            .join("amitos.vault.hold")
            .display()
            .to_string(),
        client_name: "amitos-keys".to_string(),
        vault_password: format!("amitos::{machine}::{username}::stronghold"),
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