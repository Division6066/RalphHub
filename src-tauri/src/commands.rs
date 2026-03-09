use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::State;

use crate::{
    models::{ApiProvider, CommandResponse, DashboardSnapshot, SecureStoreConfig, ToolManifest},
    state::{bun_installer_hint, detect_bun_status, AppState},
    tool_registry::{all_providers, all_tools},
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
pub fn list_api_providers() -> Vec<ApiProvider> {
    all_providers()
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
            "Bun installation failed. Run {} manually and restart AmitOS.",
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
            "# AmitOS State\n\n- Status: initialized\n- Next step: update this file from the active workflow.\n",
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