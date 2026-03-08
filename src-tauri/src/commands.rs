use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::State;

use crate::{
    models::{CommandResponse, DashboardSnapshot, SecureStoreConfig, ToolConnectResult, ToolManifest},
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

// ── Key injection helper ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_key_fields() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"field": "ANTHROPIC_API_KEY", "provider": "Anthropic", "group": "ai"}),
        serde_json::json!({"field": "OPENAI_API_KEY", "provider": "OpenAI", "group": "ai"}),
        serde_json::json!({"field": "GROK_API_KEY", "provider": "xAI Grok", "group": "ai"}),
        serde_json::json!({"field": "GEMINI_API_KEY", "provider": "Google Gemini", "group": "ai"}),
        serde_json::json!({"field": "GLM_API_KEY", "provider": "Zhipu GLM", "group": "ai"}),
        serde_json::json!({"field": "OLLAMA_API_KEY", "provider": "Ollama Local", "group": "local"}),
        serde_json::json!({"field": "OLLAMA_CLOUD_API_KEY", "provider": "Ollama Cloud", "group": "local"}),
        serde_json::json!({"field": "NOTION_API_KEY", "provider": "Notion", "group": "integrations"}),
        serde_json::json!({"field": "GITHUB_TOKEN", "provider": "GitHub", "group": "integrations"}),
        serde_json::json!({"field": "HF_TOKEN", "provider": "Hugging Face", "group": "integrations"}),
        serde_json::json!({"field": "PERPLEXICA_KEYS", "provider": "Perplexica", "group": "tools"}),
    ]
}

// ── Multi-integration "Connect & Test" ───────────────────────────────────────

#[tauri::command]
pub fn connect_and_test_tool(tool_id: String) -> ToolConnectResult {
    let mut notes = Vec::new();

    let ollama_ok = check_binary("ollama") && check_local_server("http://localhost:11434/api/tags");
    if ollama_ok {
        notes.push("Ollama running locally.".to_string());
    } else {
        notes.push("Ollama not running (run Ollama tab to start).".to_string());
    }

    let whisper_ok = check_python_import("faster_whisper") || check_binary("whisper");
    let piper_ok = check_binary("piper");
    let voice_ok = whisper_ok || piper_ok;
    if voice_ok {
        notes.push(format!(
            "Voice: STT={} TTS={}",
            if whisper_ok { "faster-whisper" } else { "none" },
            if piper_ok { "piper" } else { "none" }
        ));
    } else {
        notes.push("Voice not configured (open Voice tab to install).".to_string());
    }

    let mcp_ok = check_local_server("http://localhost:8931");
    if mcp_ok {
        notes.push("Playwright MCP server reachable.".to_string());
    } else {
        notes.push("MCP server not running (connect from Browser Agent tab).".to_string());
    }

    let cursor_ok = editor_candidates().iter().any(|c| {
        Path::new(c).exists() || Command::new(c).arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
    });
    if cursor_ok {
        notes.push("Cursor/VS Code detected.".to_string());
    } else {
        notes.push("No editor detected. Install Cursor or VS Code.".to_string());
    }

    ToolConnectResult {
        tool_id,
        ollama_ok,
        voice_ok,
        mcp_ok,
        cursor_ok,
        notes,
    }
}

#[tauri::command]
pub fn launch_in_cursor_desktop(workspace_path: String) -> Result<CommandResponse, String> {
    let workspace = PathBuf::from(&workspace_path);
    if !workspace.exists() {
        return Err(format!("Workspace not found: {workspace_path}"));
    }

    let candidates = editor_candidates();
    for candidate in &candidates {
        if candidate.to_lowercase().contains("cursor") {
            let status = Command::new(candidate)
                .args([&workspace_path, "--reuse-window"])
                .status();
            if status.map(|s| s.success()).unwrap_or(false) {
                return Ok(CommandResponse {
                    ok: true,
                    message: format!("Opened in Cursor Desktop: {workspace_path}"),
                });
            }
        }
    }

    Err("Cursor Desktop not found. Install from https://cursor.sh".to_string())
}

#[tauri::command]
pub fn launch_in_cursor_agent_web(workspace_path: String) -> Result<CommandResponse, String> {
    let url = format!(
        "https://cursor.sh/agents?workspace={}",
        urlencoding_simple(&workspace_path)
    );
    open_url(&url)
}

#[tauri::command]
pub fn launch_google_codex(prompt: Option<String>) -> Result<CommandResponse, String> {
    let base_prompt = prompt.unwrap_or_else(|| "Start a new coding task".to_string());
    let url = format!(
        "https://chatgpt.com/codex?prompt={}",
        urlencoding_simple(&base_prompt)
    );
    open_url(&url)
}

fn open_url(url: &str) -> Result<CommandResponse, String> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "start", url]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .or_else(|_| Command::new("sensible-browser").arg(url).spawn())
    };

    status
        .map(|_| CommandResponse {
            ok: true,
            message: format!("Opened: {url}"),
        })
        .map_err(|e| format!("Failed to open URL: {e}"))
}

fn check_binary(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_local_server(url: &str) -> bool {
    Command::new("curl")
        .args(["-sf", "--max-time", "2", url])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_python_import(module: &str) -> bool {
    Command::new("python")
        .args(["-c", &format!("import {module}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("python3")
            .args(["-c", &format!("import {module}")])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => '+'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}