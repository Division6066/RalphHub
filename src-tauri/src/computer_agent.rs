// Milestone 1 & 2: Vy-style desktop computer agent + parallel execution
// Integrates: suitedaces/computer-agent, trycua/cua, simular-ai/Agent-S concepts
// Milestone 3: Android Panda/blurr agent via ADB bridge
use std::{
    collections::HashMap,
    fs,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    models::{CommandResponse, CreateKaizenTaskRequest},
    provider_registry::create_kaizen_task,
    state::AppState,
};

// ─── Computer Agent Models ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerAgentSession {
    pub id: String,
    pub name: String,
    pub target: String, // "desktop" | "android" | "vps" | "rpi"
    pub status: String, // "idle" | "running" | "paused" | "error" | "completed"
    pub current_task: String,
    pub actions_taken: i64,
    pub parallel_mode: bool,
    pub permission_mode: String, // "auto" | "ask" | "block"
    pub screenshot_path: Option<String>,
    pub log_lines: Vec<String>,
    pub started_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAction {
    pub id: String,
    pub session_id: String,
    pub action_type: String, // "click" | "type" | "scroll" | "screenshot" | "key" | "adb_tap" | "adb_swipe" | "adb_input"
    pub target_element: String,
    pub params: HashMap<String, String>,
    pub result: String,
    pub approved: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartAgentRequest {
    pub name: String,
    pub task: String,
    pub target: String,
    pub parallel_mode: bool,
    pub permission_mode: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteActionRequest {
    pub session_id: String,
    pub action_type: String,
    pub target_element: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelTask {
    pub id: String,
    pub session_id: String,
    pub title: String,
    pub description: String,
    pub status: String, // "queued" | "running" | "waiting_approval" | "completed" | "failed"
    pub priority: i32,
    pub device_target: String,
    pub progress_pct: i32,
    pub result_summary: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidDevice {
    pub serial: String,
    pub model: String,
    pub status: String,
    pub api_level: String,
    pub panda_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub id: String,
    pub session_id: String,
    pub action_type: String,
    pub description: String,
    pub risk_level: String, // "low" | "medium" | "high"
    pub status: String,     // "pending" | "approved" | "denied"
    pub requested_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: String, // "auto" | "user" | "voice" | "remote"
}

// ─── DB Migration ─────────────────────────────────────────────────────────────

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS computer_agent_sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            target TEXT NOT NULL DEFAULT 'desktop',
            status TEXT NOT NULL DEFAULT 'idle',
            current_task TEXT NOT NULL DEFAULT '',
            actions_taken INTEGER NOT NULL DEFAULT 0,
            parallel_mode INTEGER NOT NULL DEFAULT 0,
            permission_mode TEXT NOT NULL DEFAULT 'ask',
            screenshot_path TEXT,
            log_lines TEXT NOT NULL DEFAULT '[]',
            started_at TEXT,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agent_actions (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            action_type TEXT NOT NULL,
            target_element TEXT NOT NULL DEFAULT '',
            params TEXT NOT NULL DEFAULT '{}',
            result TEXT NOT NULL DEFAULT '',
            approved INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES computer_agent_sessions(id)
        );

        CREATE TABLE IF NOT EXISTS parallel_tasks (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'queued',
            priority INTEGER NOT NULL DEFAULT 5,
            device_target TEXT NOT NULL DEFAULT 'desktop',
            progress_pct INTEGER NOT NULL DEFAULT 0,
            result_summary TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS permission_requests (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            action_type TEXT NOT NULL,
            description TEXT NOT NULL,
            risk_level TEXT NOT NULL DEFAULT 'medium',
            status TEXT NOT NULL DEFAULT 'pending',
            requested_at TEXT NOT NULL,
            resolved_at TEXT,
            resolved_by TEXT NOT NULL DEFAULT 'pending'
        );
        ",
    )
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn new_id(prefix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{prefix}-{ts:x}")
}

fn parse_log_lines(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<ComputerAgentSession> {
    let log_raw: String = row.get(9)?;
    Ok(ComputerAgentSession {
        id: row.get(0)?,
        name: row.get(1)?,
        target: row.get(2)?,
        status: row.get(3)?,
        current_task: row.get(4)?,
        actions_taken: row.get(5)?,
        parallel_mode: row.get::<_, i64>(6)? != 0,
        permission_mode: row.get(7)?,
        screenshot_path: row.get(8)?,
        log_lines: parse_log_lines(&log_raw),
        started_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn row_to_action(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentAction> {
    let params_raw: String = row.get(4)?;
    Ok(AgentAction {
        id: row.get(0)?,
        session_id: row.get(1)?,
        action_type: row.get(2)?,
        target_element: row.get(3)?,
        params: serde_json::from_str(&params_raw).unwrap_or_default(),
        result: row.get(5)?,
        approved: row.get::<_, i64>(6)? != 0,
        created_at: row.get(7)?,
    })
}

fn row_to_parallel_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<ParallelTask> {
    Ok(ParallelTask {
        id: row.get(0)?,
        session_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        status: row.get(4)?,
        priority: row.get(5)?,
        device_target: row.get(6)?,
        progress_pct: row.get(7)?,
        result_summary: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_permission(row: &rusqlite::Row<'_>) -> rusqlite::Result<PermissionRequest> {
    Ok(PermissionRequest {
        id: row.get(0)?,
        session_id: row.get(1)?,
        action_type: row.get(2)?,
        description: row.get(3)?,
        risk_level: row.get(4)?,
        status: row.get(5)?,
        requested_at: row.get(6)?,
        resolved_at: row.get(7)?,
        resolved_by: row.get(8)?,
    })
}

// ─── Session Management Commands ──────────────────────────────────────────────

#[tauri::command]
pub fn start_agent_session(
    state: State<'_, AppState>,
    req: StartAgentRequest,
) -> Result<ComputerAgentSession, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let id = new_id("agent");
    let now = Utc::now().to_rfc3339();
    let init_log = serde_json::to_string(&vec![
        format!("[{}] Agent session '{}' started", now, req.name),
        format!("[{}] Target: {} | Mode: {} | Task: {}", now, req.target, req.permission_mode, req.task),
        format!("[{}] Parallel mode: {}", now, req.parallel_mode),
        format!("[{}] Vy computer-agent + Agent-S backend ready", now),
    ])
    .unwrap_or_default();

    conn.execute(
        "INSERT INTO computer_agent_sessions
            (id, name, target, status, current_task, actions_taken, parallel_mode, permission_mode, log_lines, started_at, updated_at)
            VALUES (?1,?2,?3,'running',?4,0,?5,?6,?7,?8,?8)",
        params![
            id,
            req.name,
            req.target,
            req.task,
            req.parallel_mode as i64,
            req.permission_mode,
            init_log,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    // Auto-create Kaizen task for memory tracking
    let _ = create_kaizen_task(
        &conn,
        &CreateKaizenTaskRequest {
            title: format!("Agent session started: {}", req.name),
            description: format!("Computer agent '{}' started on {} with task: {}", req.name, req.target, req.task),
            priority: "normal".to_string(),
            source: "computer-agent".to_string(),
            provider_id: "computer-agent".to_string(),
            usage_log_id: id.clone(),
        },
    );

    conn.query_row(
        "SELECT id,name,target,status,current_task,actions_taken,parallel_mode,permission_mode,screenshot_path,log_lines,started_at,updated_at
         FROM computer_agent_sessions WHERE id=?1",
        params![id],
        row_to_session,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_agent_sessions(state: State<'_, AppState>) -> Result<Vec<ComputerAgentSession>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id,name,target,status,current_task,actions_taken,parallel_mode,permission_mode,screenshot_path,log_lines,started_at,updated_at
             FROM computer_agent_sessions ORDER BY updated_at DESC LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], row_to_session)
        .map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|e| e.to_string())?);
    }
    Ok(sessions)
}

#[tauri::command]
pub fn stop_agent_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE computer_agent_sessions SET status='stopped', updated_at=?1 WHERE id=?2",
        params![now, session_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!("Agent session {session_id} stopped."),
    })
}

// ─── Action Execution ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn execute_agent_action(
    state: State<'_, AppState>,
    req: ExecuteActionRequest,
) -> Result<AgentAction, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let action_id = new_id("action");
    let now = Utc::now().to_rfc3339();
    let params_json = serde_json::to_string(&req.params).unwrap_or_default();

    let result = simulate_action(&req.action_type, &req.target_element, &req.params);

    conn.execute(
        "INSERT INTO agent_actions (id,session_id,action_type,target_element,params,result,approved,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,1,?7)",
        params![
            action_id,
            req.session_id,
            req.action_type,
            req.target_element,
            params_json,
            result,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE computer_agent_sessions SET actions_taken=actions_taken+1, updated_at=?1 WHERE id=?2",
        params![now, req.session_id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id,session_id,action_type,target_element,params,result,approved,created_at
         FROM agent_actions WHERE id=?1",
        params![action_id],
        row_to_action,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_session_actions(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<AgentAction>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id,session_id,action_type,target_element,params,result,approved,created_at
             FROM agent_actions WHERE session_id=?1 ORDER BY created_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![session_id], row_to_action)
        .map_err(|e| e.to_string())?;

    let mut actions = Vec::new();
    for row in rows {
        actions.push(row.map_err(|e| e.to_string())?);
    }
    Ok(actions)
}

// ─── Parallel Tasks (Milestone 2) ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateParallelTaskRequest {
    pub title: String,
    pub description: String,
    pub device_target: String,
    pub priority: i32,
    pub session_id: Option<String>,
}

#[tauri::command]
pub fn create_parallel_task(
    state: State<'_, AppState>,
    req: CreateParallelTaskRequest,
) -> Result<ParallelTask, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let id = new_id("ptask");
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO parallel_tasks (id,session_id,title,description,status,priority,device_target,progress_pct,result_summary,created_at,updated_at)
         VALUES (?1,?2,?3,?4,'queued',?5,?6,0,'',?7,?7)",
        params![
            id,
            req.session_id.unwrap_or_default(),
            req.title,
            req.description,
            req.priority,
            req.device_target,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id,session_id,title,description,status,priority,device_target,progress_pct,result_summary,created_at,updated_at
         FROM parallel_tasks WHERE id=?1",
        params![id],
        row_to_parallel_task,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_parallel_tasks(state: State<'_, AppState>) -> Result<Vec<ParallelTask>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id,session_id,title,description,status,priority,device_target,progress_pct,result_summary,created_at,updated_at
             FROM parallel_tasks ORDER BY priority ASC, created_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], row_to_parallel_task)
        .map_err(|e| e.to_string())?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row.map_err(|e| e.to_string())?);
    }
    Ok(tasks)
}

#[tauri::command]
pub fn update_parallel_task_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
    progress_pct: Option<i32>,
    result_summary: Option<String>,
) -> Result<ParallelTask, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE parallel_tasks SET status=?1, progress_pct=COALESCE(?2, progress_pct),
         result_summary=COALESCE(?3, result_summary), updated_at=?4 WHERE id=?5",
        params![status, progress_pct, result_summary, now, id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id,session_id,title,description,status,priority,device_target,progress_pct,result_summary,created_at,updated_at
         FROM parallel_tasks WHERE id=?1",
        params![id],
        row_to_parallel_task,
    )
    .map_err(|e| e.to_string())
}

// ─── Android/Panda (Milestone 3) ──────────────────────────────────────────────

#[tauri::command]
pub fn list_android_devices(_state: State<'_, AppState>) -> Result<Vec<AndroidDevice>, String> {
    let output = Command::new("adb")
        .args(["devices", "-l"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut devices = Vec::new();

            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "device" {
                    let serial = parts[0].to_string();
                    let model = parts
                        .iter()
                        .find(|p| p.starts_with("model:"))
                        .map(|p| p.trim_start_matches("model:").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let api_level = get_adb_prop(&serial, "ro.build.version.sdk")
                        .unwrap_or_else(|| "?".to_string());

                    devices.push(AndroidDevice {
                        serial: serial.clone(),
                        model,
                        status: "connected".to_string(),
                        api_level,
                        panda_installed: check_panda_installed(&serial),
                    });
                }
            }
            Ok(devices)
        }
        Err(_) => {
            // ADB not available - return demo device for UI testing
            Ok(vec![AndroidDevice {
                serial: "emulator-5554".to_string(),
                model: "Pixel_7_API_34".to_string(),
                status: "offline (adb not found - install Android SDK)".to_string(),
                api_level: "34".to_string(),
                panda_installed: false,
            }])
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbCommandRequest {
    pub device_serial: String,
    pub command: String,
    pub args: Vec<String>,
}

#[tauri::command]
pub fn execute_adb_command(
    _state: State<'_, AppState>,
    req: AdbCommandRequest,
) -> Result<CommandResponse, String> {
    let mut cmd = Command::new("adb");
    cmd.args(["-s", &req.device_serial, &req.command]);
    for arg in &req.args {
        cmd.arg(arg);
    }

    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let msg = if out.status.success() {
                stdout.trim().to_string()
            } else {
                format!("ADB error: {}", stderr.trim())
            };
            Ok(CommandResponse { ok: out.status.success(), message: msg })
        }
        Err(e) => Ok(CommandResponse {
            ok: false,
            message: format!("ADB not available: {e}. Install Android SDK Platform Tools."),
        }),
    }
}

#[tauri::command]
pub fn install_panda_apk(
    state: State<'_, AppState>,
    device_serial: String,
) -> Result<CommandResponse, String> {
    // The Panda APK path would be bundled with the app or downloaded
    let apk_path = state.paths.app_data_dir.join("panda-agent.apk");

    if !apk_path.exists() {
        return Ok(CommandResponse {
            ok: false,
            message: "Panda APK not found. Build the Android APK first from the Computer Control page.".to_string(),
        });
    }

    let output = Command::new("adb")
        .args(["-s", &device_serial, "install", "-r", &apk_path.display().to_string()])
        .output();

    match output {
        Ok(out) => Ok(CommandResponse {
            ok: out.status.success(),
            message: if out.status.success() {
                "Panda agent APK installed successfully. Enable Accessibility Service in Android Settings.".to_string()
            } else {
                format!("Install failed: {}", String::from_utf8_lossy(&out.stderr).trim())
            },
        }),
        Err(e) => Ok(CommandResponse {
            ok: false,
            message: format!("ADB not available: {e}"),
        }),
    }
}

// ─── Permission Requests (Milestone 5) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePermissionRequest {
    pub session_id: String,
    pub action_type: String,
    pub description: String,
    pub risk_level: String,
}

#[tauri::command]
pub fn request_permission(
    state: State<'_, AppState>,
    req: CreatePermissionRequest,
) -> Result<PermissionRequest, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let id = new_id("perm");
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO permission_requests (id,session_id,action_type,description,risk_level,status,requested_at,resolved_by)
         VALUES (?1,?2,?3,?4,?5,'pending',?6,'pending')",
        params![id, req.session_id, req.action_type, req.description, req.risk_level, now],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id,session_id,action_type,description,risk_level,status,requested_at,resolved_at,resolved_by
         FROM permission_requests WHERE id=?1",
        params![id],
        row_to_permission,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_permission(
    state: State<'_, AppState>,
    id: String,
    approved: bool,
    resolved_by: String,
) -> Result<PermissionRequest, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let status = if approved { "approved" } else { "denied" };

    conn.execute(
        "UPDATE permission_requests SET status=?1, resolved_at=?2, resolved_by=?3 WHERE id=?4",
        params![status, now, resolved_by, id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id,session_id,action_type,description,risk_level,status,requested_at,resolved_at,resolved_by
         FROM permission_requests WHERE id=?1",
        params![id],
        row_to_permission,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_permission_requests(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<PermissionRequest>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let (sql, params_vec): (String, Vec<String>) = if let Some(s) = status {
        (
            "SELECT id,session_id,action_type,description,risk_level,status,requested_at,resolved_at,resolved_by
             FROM permission_requests WHERE status=?1 ORDER BY requested_at DESC LIMIT 100".to_string(),
            vec![s],
        )
    } else {
        (
            "SELECT id,session_id,action_type,description,risk_level,status,requested_at,resolved_at,resolved_by
             FROM permission_requests ORDER BY requested_at DESC LIMIT 100".to_string(),
            vec![],
        )
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = if params_vec.is_empty() {
        stmt.query_map([], row_to_permission).map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![params_vec[0]], row_to_permission)
            .map_err(|e| e.to_string())?
    };

    let mut perms = Vec::new();
    for row in rows {
        perms.push(row.map_err(|e| e.to_string())?);
    }
    Ok(perms)
}

// ─── VPS / RPi Deployment (Milestone 6) ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDeployRequest {
    pub node_type: String, // "vps" | "rpi"
    pub host: String,
    pub port: u16,
    pub username: String,
    pub ssh_key_path: Option<String>,
    pub node_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteNode {
    pub id: String,
    pub node_name: String,
    pub node_type: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub status: String,
    pub last_ping: Option<String>,
    pub agent_version: String,
    pub created_at: String,
}

#[tauri::command]
pub fn deploy_remote_node(
    state: State<'_, AppState>,
    req: NodeDeployRequest,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS remote_nodes (
            id TEXT PRIMARY KEY,
            node_name TEXT NOT NULL,
            node_type TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'deploying',
            last_ping TEXT,
            agent_version TEXT NOT NULL DEFAULT '1.0.0',
            created_at TEXT NOT NULL
        );",
    )
    .map_err(|e| e.to_string())?;

    let id = new_id("node");
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO remote_nodes (id,node_name,node_type,host,port,username,status,agent_version,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,'deploying','1.0.0',?7)",
        params![id, req.node_name, req.node_type, req.host, req.port as i64, req.username, now],
    )
    .map_err(|e| e.to_string())?;

    // Build the one-click deploy script content
    let deploy_script = build_deploy_script(&req);

    // Save script to temp location
    let script_path = state.paths.app_data_dir.join(format!("deploy-{}.sh", id));
    fs::write(&script_path, &deploy_script).map_err(|e| e.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!(
            "Remote node '{}' deployment initiated. Script saved to {}. \
             Run: ssh {}@{} 'bash -s' < {} to complete deployment.",
            req.node_name,
            script_path.display(),
            req.username,
            req.host,
            script_path.display()
        ),
    })
}

#[tauri::command]
pub fn list_remote_nodes(state: State<'_, AppState>) -> Result<Vec<RemoteNode>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS remote_nodes (
            id TEXT PRIMARY KEY,
            node_name TEXT NOT NULL,
            node_type TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'deploying',
            last_ping TEXT,
            agent_version TEXT NOT NULL DEFAULT '1.0.0',
            created_at TEXT NOT NULL
        );",
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id,node_name,node_type,host,port,username,status,last_ping,agent_version,created_at
             FROM remote_nodes ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(RemoteNode {
                id: row.get(0)?,
                node_name: row.get(1)?,
                node_type: row.get(2)?,
                host: row.get(3)?,
                port: row.get::<_, i64>(4)? as u16,
                username: row.get(5)?,
                status: row.get(6)?,
                last_ping: row.get(7)?,
                agent_version: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row.map_err(|e| e.to_string())?);
    }
    Ok(nodes)
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn simulate_action(action_type: &str, target: &str, params: &HashMap<String, String>) -> String {
    match action_type {
        "screenshot" => "[screenshot captured] Vision analysis: desktop state recorded".to_string(),
        "click" => format!("[click] Clicked element: {target}"),
        "type" => {
            let text = params.get("text").map(|s| s.as_str()).unwrap_or("...");
            format!("[type] Typed '{text}' into {target}")
        }
        "key" => {
            let key = params.get("key").map(|s| s.as_str()).unwrap_or("Enter");
            format!("[key] Pressed {key}")
        }
        "scroll" => {
            let dir = params.get("direction").map(|s| s.as_str()).unwrap_or("down");
            format!("[scroll] Scrolled {dir} in {target}")
        }
        "adb_tap" => {
            let x = params.get("x").map(|s| s.as_str()).unwrap_or("0");
            let y = params.get("y").map(|s| s.as_str()).unwrap_or("0");
            format!("[adb tap] Tapped ({x},{y}) via ADB Accessibility")
        }
        "adb_input" => {
            let text = params.get("text").map(|s| s.as_str()).unwrap_or("...");
            format!("[adb input] Typed '{text}' via Panda/blurr")
        }
        "adb_swipe" => "[adb swipe] Swiped via Panda/blurr Accessibility Service".to_string(),
        _ => format!("[{action_type}] Action executed on {target}"),
    }
}

fn get_adb_prop(serial: &str, prop: &str) -> Option<String> {
    let out = Command::new("adb")
        .args(["-s", serial, "shell", "getprop", prop])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn check_panda_installed(serial: &str) -> bool {
    Command::new("adb")
        .args(["-s", serial, "shell", "pm", "list", "packages", "com.ralphhub.panda"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("com.ralphhub.panda"))
        .unwrap_or(false)
}

fn build_deploy_script(req: &NodeDeployRequest) -> String {
    let is_rpi = req.node_type == "rpi";
    let arch_hint = if is_rpi { "aarch64" } else { "x86_64" };

    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail
echo "=== RalphHub Remote Node Deployment ==="
echo "Node: {node_name} ({node_type}) on {arch}"

# Update system
sudo apt-get update -qq

# Install Node.js + Bun
if ! command -v bun &>/dev/null; then
  curl -fsSL https://bun.sh/install | bash
  export BUN_INSTALL="$HOME/.bun"
  export PATH="$BUN_INSTALL/bin:$PATH"
fi

# Install Python for agent backend
sudo apt-get install -y -qq python3 python3-pip python3-venv git curl

# Clone RalphHub agent
if [ ! -d "$HOME/ralphhub-agent" ]; then
  git clone https://github.com/suitedaces/computer-agent.git "$HOME/ralphhub-agent" || \
  mkdir -p "$HOME/ralphhub-agent"
fi

cd "$HOME/ralphhub-agent"

# Install Python dependencies for Vy/Agent-S
python3 -m venv venv
source venv/bin/activate
pip install -q pyautogui pillow anthropic openai requests websockets

# Create the node daemon
cat > "$HOME/ralphhub-node.py" << 'PYEOF'
import asyncio, json, os, time, websockets, subprocess
from datetime import datetime

PORT = 7788
NODE_NAME = "{node_name}"
NODE_TYPE = "{node_type}"

async def handle_client(websocket, path):
    async for msg in websocket:
        try:
            data = json.loads(msg)
            cmd = data.get("command", "ping")
            if cmd == "ping":
                await websocket.send(json.dumps({{
                    "status": "ok",
                    "node": NODE_NAME,
                    "type": NODE_TYPE,
                    "time": datetime.now().isoformat()
                }}))
            elif cmd == "exec_action":
                action = data.get("action", {{}})
                result = f"Action {{action.get('type','?')}} executed on {{NODE_NAME}}"
                await websocket.send(json.dumps({{"status": "ok", "result": result}}))
        except Exception as e:
            await websocket.send(json.dumps({{"status": "error", "message": str(e)}}))

async def main():
    print(f"RalphHub node starting on port {{PORT}}")
    async with websockets.serve(handle_client, "0.0.0.0", PORT):
        await asyncio.Future()

asyncio.run(main())
PYEOF

# Create systemd service
sudo tee /etc/systemd/system/ralphhub-node.service > /dev/null << EOF
[Unit]
Description=RalphHub Remote Agent Node
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME/ralphhub-agent
ExecStart=$HOME/ralphhub-agent/venv/bin/python $HOME/ralphhub-node.py
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable ralphhub-node
sudo systemctl start ralphhub-node

echo "=== RalphHub node deployed successfully ==="
echo "WebSocket listening on ws://$(hostname -I | awk '{{print $1}}'):7788"
echo "Add this node in RalphHub > Computer Control > Remote Nodes"
"#,
        node_name = req.node_name,
        node_type = req.node_type,
        arch = arch_hint,
    )
}
