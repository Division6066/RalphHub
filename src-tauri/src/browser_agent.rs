use std::process::Command;
use chrono::Utc;
use tauri::State;

use crate::{
    models::{BrowserSession, CommandResponse},
    state::AppState,
};

const MCP_PORT: u16 = 8931;

#[tauri::command]
pub fn get_browser_sessions(state: State<'_, AppState>) -> Vec<BrowserSession> {
    let db_path = state.paths.database_path.clone();
    let connection = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match connection.prepare(
        "SELECT id, url, status, backend, created_at FROM browser_sessions ORDER BY created_at DESC LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok(BrowserSession {
            id: row.get(0)?,
            url: row.get(1)?,
            status: row.get(2)?,
            backend: row.get(3)?,
            created_at: row.get(4)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
pub fn connect_browser_mcp(state: State<'_, AppState>) -> Result<BrowserSession, String> {
    let backend = if detect_playwright_mcp_installed() {
        start_playwright_mcp_server()?;
        "playwright-mcp"
    } else if detect_playwright_installed() {
        "playwright"
    } else {
        install_playwright_mcp()?;
        start_playwright_mcp_server()?;
        "playwright-mcp"
    };

    let session = BrowserSession {
        id: format!("browser-{}", Utc::now().format("%Y%m%d%H%M%S")),
        url: format!("http://localhost:{MCP_PORT}"),
        status: "connecting".to_string(),
        backend: backend.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    let db_path = state.paths.database_path.clone();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let _ = conn.execute(
            "INSERT INTO browser_sessions (id, url, status, backend, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![session.id, session.url, session.status, session.backend, session.created_at],
        );
    }

    Ok(session)
}

#[tauri::command]
pub fn disconnect_browser_mcp(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let db_path = state.paths.database_path.clone();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let now = Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE browser_sessions SET status = 'disconnected', created_at = ?1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        );
    }

    Ok(CommandResponse {
        ok: true,
        message: format!("Browser session {session_id} disconnected."),
    })
}

#[tauri::command]
pub fn ensure_playwright() -> Result<CommandResponse, String> {
    if detect_playwright_mcp_installed() {
        return Ok(CommandResponse {
            ok: true,
            message: "Playwright MCP already installed.".to_string(),
        });
    }

    install_playwright_mcp()?;

    Ok(CommandResponse {
        ok: true,
        message: "Playwright MCP installed successfully.".to_string(),
    })
}

#[tauri::command]
pub fn launch_browser_with_profile(
    url: String,
    profile_dir: Option<String>,
) -> Result<CommandResponse, String> {
    let profile = profile_dir.unwrap_or_else(|| {
        if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(|d| format!("{d}\\Microsoft\\Edge\\User Data"))
                .unwrap_or_default()
        } else if cfg!(target_os = "macos") {
            dirs_home().map(|h| format!("{h}/Library/Application Support/Microsoft Edge")).unwrap_or_default()
        } else {
            dirs_home().map(|h| format!("{h}/.config/microsoft-edge")).unwrap_or_default()
        }
    });

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "start", "msedge", &format!("--user-data-dir={profile}"), &url])
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .args(["-a", "Microsoft Edge", "--args", &format!("--user-data-dir={profile}"), &url])
            .spawn()
    } else {
        Command::new("microsoft-edge")
            .args([&format!("--user-data-dir={profile}"), &url])
            .spawn()
            .or_else(|_| {
                Command::new("google-chrome")
                    .args([&format!("--user-data-dir={profile}"), &url])
                    .spawn()
            })
    };

    status
        .map(|_| CommandResponse {
            ok: true,
            message: format!("Browser launched with profile: {url}"),
        })
        .map_err(|e| format!("Failed to launch browser: {e}"))
}

fn detect_playwright_mcp_installed() -> bool {
    Command::new("npx")
        .args(["--yes", "@playwright/mcp", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn detect_playwright_installed() -> bool {
    Command::new("npx")
        .args(["playwright", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn install_playwright_mcp() -> Result<(), String> {
    let status = Command::new("bun")
        .args(["add", "-g", "@playwright/mcp"])
        .status()
        .or_else(|_| {
            Command::new("npm")
                .args(["install", "-g", "@playwright/mcp"])
                .status()
        })
        .map_err(|e| format!("Failed to install Playwright MCP: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("Failed to install @playwright/mcp".to_string())
    }
}

fn start_playwright_mcp_server() -> Result<(), String> {
    Command::new("sh")
        .args(["-c", &format!("npx @playwright/mcp --port {MCP_PORT} &")])
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to start Playwright MCP server: {e}"))
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME").ok()
}

#[tauri::command]
pub fn get_mcp_connection_instructions() -> serde_json::Value {
    serde_json::json!({
        "playwright_mcp": {
            "install": "bun add -g @playwright/mcp",
            "start": "npx @playwright/mcp --port 8931",
            "connect": "http://localhost:8931",
            "description": "Playwright MCP server for browser automation via MCP protocol"
        },
        "capture_mcp": {
            "extension_url": "https://chromewebstore.google.com/detail/capture-mcp/...",
            "description": "Capture MCP Browser extension — gives full click/type/scrape access",
            "steps": [
                "Install the Capture MCP Browser extension from Chrome Web Store",
                "Open Edge and activate the extension",
                "Click 'Connect Browser via MCP' in RalphHub Browser Agent tab",
                "Extension will auto-connect to RalphHub on port 8931"
            ]
        },
        "edge_profile": {
            "windows": "%APPDATA%\\Microsoft\\Edge\\User Data",
            "macos": "~/Library/Application Support/Microsoft Edge",
            "linux": "~/.config/microsoft-edge"
        }
    })
}

#[tauri::command]
pub fn check_mcp_server_status() -> serde_json::Value {
    let running = check_local_server_reachable(&format!("http://localhost:{MCP_PORT}/health"))
        || check_local_server_reachable(&format!("http://localhost:{MCP_PORT}/api/tags"))
        || check_local_server_reachable(&format!("http://localhost:{MCP_PORT}"));

    serde_json::json!({
        "running": running,
        "port": MCP_PORT,
        "endpoint": format!("http://localhost:{MCP_PORT}"),
        "playwright_installed": detect_playwright_mcp_installed(),
        "playwright_fallback": detect_playwright_installed()
    })
}

fn check_local_server_reachable(url: &str) -> bool {
    Command::new("curl")
        .args(["-sf", "--max-time", "2", url])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
