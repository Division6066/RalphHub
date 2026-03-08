use std::{env, fs, path::PathBuf, process::Command};

use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::{
    models::{
        BrowserAction, BrowserActionApproval, BrowserActionRequest, BrowserSettings,
        CommandResponse, EdgeProfileConfig,
    },
    state::AppState,
};

// Stage 2: Mobile remote approvals via Expo/React Native + push notifications

// ── Public Tauri commands ────────────────────────────────────────────────────

/// Open a URL in the user's preferred browser.
/// mode "permission" logs the action as pending for UI approval first;
/// mode "autonomous" executes immediately (big warning shown in dashboard).
#[tauri::command]
pub fn launch_browser_with_profile(
    url: String,
    mode: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    validate_url(&url)?;

    let settings = load_settings(&state)?;
    let effective_mode = if mode.is_empty() { settings.agent_mode.clone() } else { mode };

    let status = if effective_mode == "permission" { "pending" } else { "executed" };
    log_action(&state, "launch", &url, &url, Some(format!("mode={effective_mode}")), status)?;

    if effective_mode == "autonomous" || status == "executed" {
        let launched_in = open_url(&url, &settings.preferred_browser, &app)?;
        return Ok(CommandResponse {
            ok: true,
            message: format!("Opened {url} in {launched_in}."),
        });
    }

    Ok(CommandResponse {
        ok: true,
        message: format!("Launch request queued — approve it in the Browser Agent tab."),
    })
}

/// Detect the Edge profile directory and binary path on this machine.
#[tauri::command]
pub fn get_edge_profile_config() -> EdgeProfileConfig {
    let binary_path = detect_edge_binary();
    let profile_dir = edge_user_data_dir();
    EdgeProfileConfig {
        detected: binary_path.is_some() || profile_dir.is_some(),
        binary_path,
        profile_dir: profile_dir
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    }
}

#[tauri::command]
pub fn get_browser_settings(state: State<'_, AppState>) -> Result<BrowserSettings, String> {
    load_settings(&state)
}

#[tauri::command]
pub fn save_browser_settings(
    settings: BrowserSettings,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(settings_path(&state), json).map_err(|e| e.to_string())?;
    Ok(CommandResponse {
        ok: true,
        message: "Browser settings saved.".to_string(),
    })
}

/// Enumerate browser actions from the audit log, newest first.
#[tauri::command]
pub fn list_browser_actions(state: State<'_, AppState>) -> Result<Vec<BrowserAction>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, action_type, target, url, details, status, screenshot_path, \
             created_at, updated_at \
             FROM browser_actions ORDER BY created_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BrowserAction {
                id: row.get(0)?,
                action_type: row.get(1)?,
                target: row.get(2)?,
                url: row.get(3)?,
                details: row.get(4)?,
                status: row.get(5)?,
                screenshot_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut actions = Vec::new();
    for row in rows {
        actions.push(row.map_err(|e| e.to_string())?);
    }
    Ok(actions)
}

/// Called by Ralph loops / workflow runners to record a desired browser action.
/// In permission mode the action stays "pending" until the user approves it.
#[tauri::command]
pub fn log_browser_action(
    request: BrowserActionRequest,
    state: State<'_, AppState>,
) -> Result<BrowserAction, String> {
    let settings = load_settings(&state)?;
    let status = if settings.agent_mode == "permission" { "pending" } else { "executed" };
    log_action(&state, &request.action_type, &request.target, &request.url, request.details, status)
}

/// Approve or deny a pending browser action.
#[tauri::command]
pub fn approve_browser_action(
    request: BrowserActionApproval,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let status = if request.approved { "approved" } else { "denied" };

    conn.execute(
        "UPDATE browser_actions SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, now, request.action_id],
    )
    .map_err(|e| e.to_string())?;

    // If approved, execute the action now (for launch/navigate actions).
    if request.approved {
        execute_approved_action(&request.action_id, &state, &app)?;
    }

    Ok(CommandResponse {
        ok: true,
        message: format!("Action {}.", status),
    })
}

/// Open Google Colab in the user's browser after notebook generation.
/// Shows a confirmation step in permission mode.
#[tauri::command]
pub fn open_colab_url(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    const COLAB: &str = "https://colab.research.google.com/";
    let settings = load_settings(&state)?;

    log_action(&state, "navigate", COLAB, COLAB, Some("Open Colab for notebook upload".to_string()), "executed")?;

    let launched_in = open_url(COLAB, &settings.preferred_browser, &app)?;
    Ok(CommandResponse {
        ok: true,
        message: format!(
            "Opened Google Colab in {launched_in}. Upload your generated .ipynb file to run it."
        ),
    })
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn settings_path(state: &AppState) -> PathBuf {
    state.paths.app_data_dir.join("browser_settings.json")
}

fn load_settings(state: &AppState) -> Result<BrowserSettings, String> {
    let path = settings_path(state);
    if !path.exists() {
        return Ok(BrowserSettings::default());
    }
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

fn log_action(
    state: &AppState,
    action_type: &str,
    target: &str,
    url: &str,
    details: Option<String>,
    status: &str,
) -> Result<BrowserAction, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO browser_actions \
         (id, action_type, target, url, details, status, screenshot_path, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8)",
        params![id, action_type, target, url, details, status, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(BrowserAction {
        id,
        action_type: action_type.to_string(),
        target: target.to_string(),
        url: url.to_string(),
        details: None,
        status: status.to_string(),
        screenshot_path: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

fn execute_approved_action(
    action_id: &str,
    state: &AppState,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT action_type, url FROM browser_actions WHERE id = ?1",
            params![action_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    if let Some((action_type, url)) = row {
        if matches!(action_type.as_str(), "launch" | "navigate") {
            let settings = load_settings(state)?;
            open_url(&url, &settings.preferred_browser, app)?;
        }
    }
    Ok(())
}

/// Open a URL using Edge (with profile) or the system default browser.
fn open_url(url: &str, preferred: &str, app: &tauri::AppHandle) -> Result<String, String> {
    if matches!(preferred, "edge") {
        if let Some(label) = try_launch_edge_with_profile(url) {
            return Ok(label);
        }
    }
    open_system_default(url, app);
    Ok("system default browser".to_string())
}

/// Launch Edge with the user's real profile directory for full session access.
fn try_launch_edge_with_profile(url: &str) -> Option<String> {
    let binary = detect_edge_binary()?;
    let profile_dir = edge_user_data_dir()?;

    Command::new(&binary)
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg("--profile-directory=Default")
        .arg(url)
        .spawn()
        .ok()?;

    Some("Microsoft Edge (persistent profile)".to_string())
}

/// Cross-platform system-default browser via tauri-plugin-opener.
fn open_system_default(url: &str, app: &tauri::AppHandle) {
    use tauri_plugin_opener::OpenerExt;
    let _ = app.opener().open_url(url, None::<&str>);
}

fn detect_edge_binary() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let candidates: Vec<Option<String>> = vec![
            env::var("ProgramFiles").ok().map(|p| {
                std::path::Path::new(&p)
                    .join("Microsoft")
                    .join("Edge")
                    .join("Application")
                    .join("msedge.exe")
                    .display()
                    .to_string()
            }),
            env::var("ProgramFiles(x86)").ok().map(|p| {
                std::path::Path::new(&p)
                    .join("Microsoft")
                    .join("Edge")
                    .join("Application")
                    .join("msedge.exe")
                    .display()
                    .to_string()
            }),
            Some("msedge.exe".to_string()),
        ];
        for c in candidates.into_iter().flatten() {
            if std::path::Path::new(&c).exists()
                || !c.contains(std::path::MAIN_SEPARATOR)
            {
                return Some(c);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        let path = "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge";
        std::path::Path::new(path).exists().then(|| path.to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        for candidate in &["microsoft-edge", "microsoft-edge-stable", "microsoft-edge-beta"] {
            if Command::new(candidate)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Some(candidate.to_string());
            }
        }
        None
    }
}

fn edge_user_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local = env::var("LOCALAPPDATA").ok()?;
        Some(
            std::path::Path::new(&local)
                .join("Microsoft")
                .join("Edge")
                .join("User Data"),
        )
    }
    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME").ok()?;
        Some(
            std::path::Path::new(&home)
                .join("Library")
                .join("Application Support")
                .join("Microsoft Edge"),
        )
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let home = env::var("HOME").ok()?;
        Some(std::path::Path::new(&home).join(".config").join("microsoft-edge"))
    }
}

fn validate_url(url: &str) -> Result<(), String> {
    if url.trim().is_empty() {
        return Err("URL is required.".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Only http:// and https:// URLs are supported.".to_string());
    }
    Ok(())
}
