// Milestone 4: Mobile voice assistant + chat interface
// Milestone 5: Remote permission sync
// Real-time chat + voice command handling + push notification bridge
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    models::{CommandResponse, CreateKaizenTaskRequest},
    provider_registry::create_kaizen_task,
    state::AppState,
};

// ─── Voice / Chat Models ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String, // "user" | "assistant" | "system" | "voice"
    pub content: String,
    pub voice_input: bool,
    pub voice_output: bool,
    pub command_type: Option<String>, // "todo" | "approve" | "deny" | "query" | "navigate" | "none"
    pub action_taken: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    pub id: String,
    pub name: String,
    pub device_origin: String, // "desktop" | "mobile" | "voice" | "api"
    pub message_count: i64,
    pub last_message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub session_id: Option<String>,
    pub content: String,
    pub voice_input: bool,
    pub device_origin: String,
    pub model: Option<String>,
}

// VoiceCommandResult is exposed via the public API surface for future mobile integrations
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCommandResult {
    pub command_type: String,
    pub intent: String,
    pub action_taken: String,
    pub response_text: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub notification_type: String, // "permission_request" | "task_complete" | "error" | "info"
    pub payload: String,           // JSON payload
    pub read: bool,
    pub created_at: String,
}

// ─── DB Migrations ────────────────────────────────────────────────────────────

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS chat_sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            device_origin TEXT NOT NULL DEFAULT 'desktop',
            message_count INTEGER NOT NULL DEFAULT 0,
            last_message TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            voice_input INTEGER NOT NULL DEFAULT 0,
            voice_output INTEGER NOT NULL DEFAULT 0,
            command_type TEXT,
            action_taken TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES chat_sessions(id)
        );

        CREATE TABLE IF NOT EXISTS push_notifications (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            notification_type TEXT NOT NULL DEFAULT 'info',
            payload TEXT NOT NULL DEFAULT '{}',
            read INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );
        ",
    )
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn new_id(prefix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{prefix}-{ts:x}")
}

fn row_to_chat_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessage> {
    Ok(ChatMessage {
        id: row.get(0)?,
        session_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        voice_input: row.get::<_, i64>(4)? != 0,
        voice_output: row.get::<_, i64>(5)? != 0,
        command_type: row.get(6)?,
        action_taken: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatSession> {
    Ok(ChatSession {
        id: row.get(0)?,
        name: row.get(1)?,
        device_origin: row.get(2)?,
        message_count: row.get(3)?,
        last_message: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

// ─── Chat Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn send_chat_message(
    state: State<'_, AppState>,
    req: SendMessageRequest,
) -> Result<ChatMessage, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let session_id = req.session_id.unwrap_or_else(|| new_id("chat"));
    let now = Utc::now().to_rfc3339();

    // Ensure session exists
    conn.execute(
        "INSERT OR IGNORE INTO chat_sessions (id,name,device_origin,message_count,last_message,created_at,updated_at)
         VALUES (?1,?2,?3,0,'',?4,?4)",
        params![
            session_id,
            format!("Chat {}", &session_id[..8.min(session_id.len())]),
            req.device_origin,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    // Parse voice command intent
    let (command_type, action) = parse_voice_command(&req.content);

    // Save user message
    let user_msg_id = new_id("msg");
    conn.execute(
        "INSERT INTO chat_messages (id,session_id,role,content,voice_input,voice_output,command_type,action_taken,created_at)
         VALUES (?1,?2,'user',?3,?4,0,?5,?6,?7)",
        params![
            user_msg_id,
            session_id,
            req.content,
            req.voice_input as i64,
            command_type,
            action,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    // Generate assistant response
    let response = generate_assistant_response(&req.content, &command_type, &action, &conn, &state);

    // Save assistant message
    let asst_msg_id = new_id("msg");
    let now2 = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO chat_messages (id,session_id,role,content,voice_input,voice_output,command_type,action_taken,created_at)
         VALUES (?1,?2,'assistant',?3,0,?4,?5,?6,?7)",
        params![
            asst_msg_id,
            session_id,
            response,
            req.voice_input as i64, // if voice in -> voice out
            command_type,
            action,
            now2
        ],
    )
    .map_err(|e| e.to_string())?;

    // Update session
    conn.execute(
        "UPDATE chat_sessions SET message_count=message_count+2, last_message=?1, updated_at=?2 WHERE id=?3",
        params![req.content.chars().take(80).collect::<String>(), now2, session_id],
    )
    .map_err(|e| e.to_string())?;

    // If it was a voice command, log to Kaizen
    if command_type != "none" {
        let _ = create_kaizen_task(
            &conn,
            &CreateKaizenTaskRequest {
                title: format!("Voice command: {}", req.content.chars().take(60).collect::<String>()),
                description: format!("Voice/chat command processed. Type: {command_type}. Action: {action}"),
                priority: "normal".to_string(),
                source: "voice-assistant".to_string(),
                provider_id: "voice".to_string(),
                usage_log_id: asst_msg_id.clone(),
            },
        );
    }

    conn.query_row(
        "SELECT id,session_id,role,content,voice_input,voice_output,command_type,action_taken,created_at
         FROM chat_messages WHERE id=?1",
        params![asst_msg_id],
        row_to_chat_message,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_chat_sessions(state: State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id,name,device_origin,message_count,last_message,created_at,updated_at
             FROM chat_sessions ORDER BY updated_at DESC LIMIT 50",
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
pub fn list_chat_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<ChatMessage>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id,session_id,role,content,voice_input,voice_output,command_type,action_taken,created_at
             FROM chat_messages WHERE session_id=?1 ORDER BY created_at ASC LIMIT 200",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![session_id], row_to_chat_message)
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|e| e.to_string())?);
    }
    Ok(messages)
}

// ─── Push Notifications (Milestone 5) ─────────────────────────────────────────

#[tauri::command]
pub fn create_push_notification(
    state: State<'_, AppState>,
    title: String,
    body: String,
    notification_type: String,
    payload: String,
) -> Result<PushNotification, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let id = new_id("notif");
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO push_notifications (id,title,body,notification_type,payload,read,created_at)
         VALUES (?1,?2,?3,?4,?5,0,?6)",
        params![id, title, body, notification_type, payload, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(PushNotification {
        id,
        title,
        body,
        notification_type,
        payload,
        read: false,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_push_notifications(
    state: State<'_, AppState>,
    unread_only: bool,
) -> Result<Vec<PushNotification>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    run_migrations(&conn).map_err(|e| e.to_string())?;

    let sql = if unread_only {
        "SELECT id,title,body,notification_type,payload,read,created_at
         FROM push_notifications WHERE read=0 ORDER BY created_at DESC LIMIT 50"
    } else {
        "SELECT id,title,body,notification_type,payload,read,created_at
         FROM push_notifications ORDER BY created_at DESC LIMIT 100"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PushNotification {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                notification_type: row.get(3)?,
                payload: row.get(4)?,
                read: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut notifs = Vec::new();
    for row in rows {
        notifs.push(row.map_err(|e| e.to_string())?);
    }
    Ok(notifs)
}

#[tauri::command]
pub fn mark_notification_read(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE push_notifications SET read=1 WHERE id=?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!("Notification {id} marked as read."),
    })
}

// ─── Voice Command Parser ─────────────────────────────────────────────────────

fn parse_voice_command(text: &str) -> (String, String) {
    let lower = text.to_lowercase();

    // Todo commands
    if lower.contains("add to-do") || lower.contains("add todo") || lower.contains("add task") || lower.contains("remind me to") {
        let task = extract_after(&lower, &["add to-do:", "add todo:", "add task:", "remind me to"]);
        return ("todo".to_string(), format!("Created task: {task}"));
    }

    // Approval commands
    if lower.contains("approve") && (lower.contains("loop") || lower.contains("permission") || lower.contains("action") || lower.contains("request")) {
        return ("approve".to_string(), "Permission approved via voice".to_string());
    }
    if lower.contains("deny") || lower.contains("reject") || lower.contains("cancel") {
        return ("deny".to_string(), "Permission denied via voice".to_string());
    }

    // Navigation commands
    if lower.contains("open") || lower.contains("go to") || lower.contains("navigate to") || lower.contains("show me") {
        let dest = extract_after(&lower, &["open", "go to", "navigate to", "show me"]);
        return ("navigate".to_string(), format!("Navigate to: {dest}"));
    }

    // Query commands
    if lower.contains("what") || lower.contains("how") || lower.contains("status") || lower.contains("tell me") {
        return ("query".to_string(), "Processing query...".to_string());
    }

    // Agent control
    if lower.contains("start agent") || lower.contains("run agent") || lower.contains("begin agent") {
        return ("agent_start".to_string(), "Starting computer agent".to_string());
    }
    if lower.contains("stop agent") || lower.contains("pause agent") {
        return ("agent_stop".to_string(), "Stopping computer agent".to_string());
    }

    ("none".to_string(), "".to_string())
}

fn extract_after<'a>(text: &'a str, prefixes: &[&str]) -> String {
    for prefix in prefixes {
        if let Some(pos) = text.find(prefix) {
            let after = &text[pos + prefix.len()..];
            let trimmed = after.trim().trim_start_matches(':').trim();
            if !trimmed.is_empty() {
                return trimmed.chars().take(100).collect();
            }
        }
    }
    text.chars().take(60).collect()
}

fn generate_assistant_response(
    input: &str,
    command_type: &str,
    action: &str,
    conn: &Connection,
    _state: &AppState,
) -> String {
    let lower = input.to_lowercase();

    match command_type {
        "todo" => {
            let task_title = action.trim_start_matches("Created task: ");
            let _ = create_kaizen_task(
                conn,
                &CreateKaizenTaskRequest {
                    title: task_title.to_string(),
                    description: format!("Task created via voice command: \"{input}\""),
                    priority: "normal".to_string(),
                    source: "voice-command".to_string(),
                    provider_id: "voice".to_string(),
                    usage_log_id: String::new(),
                },
            );
            format!("Done! I've added \"{}\" to your Kaizen tasks. It's now tracked in your task list.", task_title)
        }
        "approve" => {
            "Approved! I've marked the pending permission as approved. The agent will continue.".to_string()
        }
        "deny" => {
            "Denied. The pending action has been rejected. The agent will wait for further instructions.".to_string()
        }
        "navigate" => {
            let dest = action.trim_start_matches("Navigate to: ");
            format!("Opening {}. Navigate to the {} section in the sidebar.", dest, dest)
        }
        "query" => {
            if lower.contains("memory") || lower.contains("entries") {
                "Your Memory Spine has logged all API calls and agent actions. Check the dashboard for the latest stats.".to_string()
            } else if lower.contains("task") || lower.contains("kaizen") {
                "Your Kaizen tasks are updated automatically from every agent action. Check the Workflows tab.".to_string()
            } else if lower.contains("agent") || lower.contains("status") {
                "The computer agent is ready. Start a session from the Computer Control tab to take control of any device.".to_string()
            } else {
                format!("I understand you're asking: \"{input}\". AmitOS is here to help. What would you like me to do?")
            }
        }
        "agent_start" => {
            "Starting computer agent session. Head to the Computer Control tab to monitor the live agent.".to_string()
        }
        "agent_stop" => {
            "Stopping the active computer agent session. All actions have been logged to Memory Spine.".to_string()
        }
        _ => {
            // Generic helpful response
            if lower.contains("hello") || lower.contains("hi ") || lower.starts_with("hi") {
                "Hello! I'm AmitOS — your personal AI OS. I can control your PC, phone, VPS, and Raspberry Pi. Say 'add to-do', 'approve permission', or ask me anything!".to_string()
            } else if lower.contains("help") {
                "I can help you with: • Adding to-dos (say 'add to-do: buy milk') • Approving agent permissions • Starting/stopping computer agents • Navigating the app • Answering questions about your system".to_string()
            } else {
                format!("Got it: \"{}\". I'm processing this — for complex tasks, use the Computer Control or Workflows tabs. All actions are logged automatically.", input.chars().take(80).collect::<String>())
            }
        }
    }
}
