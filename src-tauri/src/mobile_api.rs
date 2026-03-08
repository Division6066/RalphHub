//! Mobile Stage-2 API foundations.
//!
//! TODO(M7-Mobile): These commands expose the same data as the desktop UI but
//! over a serializable interface suitable for a future Android APK wrapper.
//! Each endpoint corresponds to a planned REST/WebSocket endpoint for mobile.
//!
//! Implementation path:
//! 1. Add `tauri-plugin-http` to enable an HTTP server inside Tauri
//! 2. Wire these commands to `/api/v1/*` routes served on localhost:4242
//! 3. Android APK connects to the desktop over USB/LAN via the local server
//!
//! Commands here are thin wrappers that return the same structs as the desktop
//! commands. They are deliberately stub/passthrough for now.

use tauri::State;

use crate::{
    models::{AddInboxRequest, AmitosDashboard, InboxItem, KaizenTask, MorningDigest},
    state::AppState,
};

/// GET /api/v1/dashboard — AmitOS dashboard for mobile home screen
/// TODO(M7-Mobile): expose via HTTP server plugin on localhost:4242/api/v1/dashboard
#[tauri::command]
pub fn mobile_get_dashboard(state: State<'_, AppState>) -> Result<AmitosDashboard, String> {
    state.amitos_dashboard().map_err(|e| e.to_string())
}

/// GET /api/v1/today — Today board tasks for mobile task list
/// TODO(M7-Mobile): expose via GET /api/v1/today
#[tauri::command]
pub fn mobile_get_today_tasks(state: State<'_, AppState>) -> Result<Vec<KaizenTask>, String> {
    let connection =
        rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    crate::state::query_today_tasks(&connection).map_err(|e| e.to_string())
}

/// GET /api/v1/morning-digest — Morning digest for mobile daily digest screen
/// TODO(M7-Mobile): expose via GET /api/v1/morning-digest
#[tauri::command]
pub fn mobile_get_morning_digest(state: State<'_, AppState>) -> Result<MorningDigest, String> {
    // Delegate to the same logic as the desktop command
    let connection =
        rusqlite::Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let today_tasks = crate::state::query_today_tasks(&connection).map_err(|e| e.to_string())?;

    let inbox_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM inbox_items WHERE processed = 0", [], |row| row.get(0))
        .unwrap_or(0);

    let memory_updates: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM raw_events WHERE created_at LIKE ?1",
            rusqlite::params![format!("{yesterday}%")],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(MorningDigest {
        date: today,
        today_tasks,
        inbox_count,
        memory_updates,
        yesterday_summary: format!("Yesterday: {memory_updates} memory events"),
    })
}

/// POST /api/v1/inbox — Add inbox item from mobile capture
/// TODO(M7-Mobile): expose via POST /api/v1/inbox with JSON body
#[tauri::command]
pub fn mobile_add_inbox(
    request: AddInboxRequest,
    state: State<'_, AppState>,
) -> Result<InboxItem, String> {
    // Reuse the existing inbox command
    crate::tasks_inbox::add_inbox_item(request, state)
}

/// POST /api/v1/approvals/:id/approve — Approve a task from mobile approval screen
/// TODO(M7-Mobile): expose via POST /api/v1/approvals/:id/approve
#[tauri::command]
pub fn mobile_approve_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    crate::tasks::update_kaizen_task_status(id, "todo".to_string(), state)
}

/// GET /api/v1/inbox/unprocessed — Unprocessed inbox for mobile badge count
/// TODO(M7-Mobile): expose via GET /api/v1/inbox/unprocessed
#[tauri::command]
pub fn mobile_get_inbox(state: State<'_, AppState>) -> Result<Vec<InboxItem>, String> {
    crate::tasks_inbox::list_inbox_items(Some(true), state)
}
