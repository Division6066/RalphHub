use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;

use crate::{
    models::{CommandResponse, KaizenTask},
    state::AppState,
};

#[tauri::command]
pub fn create_kaizen_task(
    title: String,
    description: String,
    priority: String,
    tool_id: String,
    state: State<'_, AppState>,
) -> Result<KaizenTask, String> {
    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    let id = format!(
        "kaizen-{}",
        Utc::now().format("%Y%m%d%H%M%S%3f")
    );
    let now = Utc::now().to_rfc3339();

    connection
        .execute(
            "INSERT INTO kaizen_tasks (id, title, description, status, priority, tool_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, title, description, "pending", priority, tool_id, now, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(KaizenTask {
        id,
        title,
        description,
        status: "pending".to_string(),
        priority,
        tool_id,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn list_kaizen_tasks(
    status_filter: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<KaizenTask>, String> {
    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    let (sql, filter): (&str, Option<String>) = if let Some(ref s) = status_filter {
        (
            "SELECT id, title, description, status, priority, tool_id, created_at, updated_at
             FROM kaizen_tasks WHERE status = ?1
             ORDER BY priority DESC, created_at DESC LIMIT 200",
            Some(s.clone()),
        )
    } else {
        (
            "SELECT id, title, description, status, priority, tool_id, created_at, updated_at
             FROM kaizen_tasks
             ORDER BY priority DESC, created_at DESC LIMIT 200",
            None,
        )
    };

    let mut stmt = connection.prepare(sql).map_err(|e| e.to_string())?;

    let rows = if let Some(ref s) = filter {
        stmt.query_map(params![s], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(|e| e.to_string())?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row.map_err(|e| e.to_string())?);
    }

    Ok(tasks)
}

#[tauri::command]
pub fn update_kaizen_task_status(
    id: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let allowed = ["pending", "in_progress", "done", "cancelled"];
    if !allowed.contains(&status.as_str()) {
        return Err(format!(
            "Invalid status '{status}'. Allowed: {}",
            allowed.join(", ")
        ));
    }

    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    let rows_affected = connection
        .execute(
            "UPDATE kaizen_tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )
        .map_err(|e| e.to_string())?;

    if rows_affected == 0 {
        return Err(format!("Task {id} not found."));
    }

    Ok(CommandResponse {
        ok: true,
        message: format!("Task {id} updated to '{status}'."),
    })
}

#[tauri::command]
pub fn delete_kaizen_task(id: String, state: State<'_, AppState>) -> Result<CommandResponse, String> {
    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    connection
        .execute("DELETE FROM kaizen_tasks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(CommandResponse { ok: true, message: format!("Task {id} deleted.") })
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<KaizenTask> {
    Ok(KaizenTask {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: row.get(3)?,
        priority: row.get(4)?,
        tool_id: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}
