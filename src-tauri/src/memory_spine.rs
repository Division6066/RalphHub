use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;

use crate::{
    models::{CommandResponse, MemoryEntry},
    state::AppState,
};

#[tauri::command]
pub fn write_memory_entry(
    tool_id: String,
    entry_type: String,
    content: String,
    tags: String,
    state: State<'_, AppState>,
) -> Result<MemoryEntry, String> {
    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    let id = format!(
        "{}-{}",
        tool_id,
        Utc::now().format("%Y%m%d%H%M%S%3f")
    );
    let now = Utc::now().to_rfc3339();

    connection
        .execute(
            "INSERT INTO memory_entries (id, tool_id, entry_type, content, tags, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, tool_id, entry_type, content, tags, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(MemoryEntry {
        id,
        tool_id,
        entry_type,
        content,
        tags,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_memory_entries(
    tool_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<MemoryEntry>, String> {
    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    let (sql, filter): (&str, Option<String>) = if let Some(ref id) = tool_id {
        (
            "SELECT id, tool_id, entry_type, content, tags, created_at
             FROM memory_entries
             WHERE tool_id = ?1
             ORDER BY created_at DESC LIMIT 100",
            Some(id.clone()),
        )
    } else {
        (
            "SELECT id, tool_id, entry_type, content, tags, created_at
             FROM memory_entries
             ORDER BY created_at DESC LIMIT 100",
            None,
        )
    };

    let mut stmt = connection.prepare(sql).map_err(|e| e.to_string())?;

    let rows = if let Some(ref id) = filter {
        stmt.query_map(params![id], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }

    Ok(entries)
}

#[tauri::command]
pub fn delete_memory_entry(id: String, state: State<'_, AppState>) -> Result<CommandResponse, String> {
    let connection = Connection::open(&state.paths.database_path)
        .map_err(|e| e.to_string())?;

    connection
        .execute("DELETE FROM memory_entries WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(CommandResponse { ok: true, message: format!("Memory entry {id} deleted.") })
}

#[tauri::command]
pub fn write_run_report(
    tool_id: String,
    summary: String,
    details: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let content = format!("## Run Report\n\n**Summary:** {summary}\n\n### Details\n\n{details}");
    write_memory_entry(
        tool_id,
        "report".to_string(),
        content,
        "run,report,auto".to_string(),
        state,
    )
    .map(|entry| CommandResponse {
        ok: true,
        message: format!("Report written to Memory Spine: {}", entry.id),
    })
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryEntry> {
    Ok(MemoryEntry {
        id: row.get(0)?,
        tool_id: row.get(1)?,
        entry_type: row.get(2)?,
        content: row.get(3)?,
        tags: row.get(4)?,
        created_at: row.get(5)?,
    })
}
