use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::{
    models::{
        IngestRequest, LongTermMemoryItem, MemoryStats, RawEvent, StructuredSummary,
        WorkingMemoryItem,
    },
    state::{query_memory_stats, AppState},
};

// ─── Ingest ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn ingest_memory(
    request: IngestRequest,
    state: State<'_, AppState>,
) -> Result<RawEvent, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let metadata = request
        .metadata
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let metadata_str = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;

    connection
        .execute(
            "INSERT INTO raw_events (id, source_type, content, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, request.source_type, request.content, metadata_str, now],
        )
        .map_err(|e| e.to_string())?;

    // If auto_summarize, create a working memory entry
    if request.auto_summarize.unwrap_or(false) {
        let title = first_line(&request.content);
        let wm_id = Uuid::new_v4().to_string();
        connection
            .execute(
                "INSERT INTO working_memory (id, title, content, tags, expires_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '[]', NULL, ?4, ?4)",
                params![wm_id, title, request.content, now],
            )
            .map_err(|e| e.to_string())?;

        // Also write to daily log
        let _ = write_daily_log_entry(
            &connection,
            &now[..10],
            "memory_write",
            &format!("Memory ingested: {title}"),
            &request.content[..request.content.len().min(200)],
        );
    }

    Ok(RawEvent {
        id,
        source_type: request.source_type,
        content: request.content,
        metadata,
        created_at: now,
    })
}

// ─── List Raw Events ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_raw_events(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<RawEvent>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let lim = limit.unwrap_or(50);
    let mut stmt = connection
        .prepare(
            "SELECT id, source_type, content, metadata, created_at
             FROM raw_events ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![lim], |row| {
            let meta_str: String = row.get(3)?;
            let metadata: serde_json::Value =
                serde_json::from_str(&meta_str).unwrap_or_default();
            Ok(RawEvent {
                id: row.get(0)?,
                source_type: row.get(1)?,
                content: row.get(2)?,
                metadata,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| e.to_string())?);
    }
    Ok(events)
}

// ─── Working Memory ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_working_memory(
    state: State<'_, AppState>,
) -> Result<Vec<WorkingMemoryItem>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let mut stmt = connection
        .prepare(
            "SELECT id, title, content, tags, expires_at, created_at, updated_at
             FROM working_memory ORDER BY updated_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let tags_str: String = row.get(3)?;
            let tags: Vec<String> =
                serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(WorkingMemoryItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags,
                expires_at: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn save_working_memory(
    title: String,
    content: String,
    tags: Vec<String>,
    state: State<'_, AppState>,
) -> Result<WorkingMemoryItem, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let tags_str = serde_json::to_string(&tags).map_err(|e| e.to_string())?;

    connection
        .execute(
            "INSERT INTO working_memory (id, title, content, tags, expires_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5)",
            params![id, title, content, tags_str, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(WorkingMemoryItem {
        id,
        title,
        content,
        tags,
        expires_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

// ─── Long-Term Memory ────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_long_term_memory(
    state: State<'_, AppState>,
) -> Result<Vec<LongTermMemoryItem>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let mut stmt = connection
        .prepare(
            "SELECT id, title, content, tags, source_ids, created_at, updated_at
             FROM long_term_memory ORDER BY updated_at DESC LIMIT 200",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let tags_str: String = row.get(3)?;
            let src_str: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            let source_ids: Vec<String> = serde_json::from_str(&src_str).unwrap_or_default();
            Ok(LongTermMemoryItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags,
                source_ids,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[tauri::command]
pub fn promote_to_long_term(
    working_memory_id: String,
    state: State<'_, AppState>,
) -> Result<LongTermMemoryItem, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let wm: WorkingMemoryItem = connection
        .query_row(
            "SELECT id, title, content, tags, expires_at, created_at, updated_at
             FROM working_memory WHERE id = ?1",
            params![working_memory_id],
            |row| {
                let tags_str: String = row.get(3)?;
                let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
                Ok(WorkingMemoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    tags,
                    expires_at: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let tags_str = serde_json::to_string(&wm.tags).map_err(|e| e.to_string())?;
    let src_str = serde_json::to_string(&[&wm.id]).map_err(|e| e.to_string())?;

    connection
        .execute(
            "INSERT INTO long_term_memory (id, title, content, tags, source_ids, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, wm.title, wm.content, tags_str, src_str, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(LongTermMemoryItem {
        id,
        title: wm.title,
        content: wm.content,
        tags: wm.tags,
        source_ids: vec![wm.id],
        created_at: now.clone(),
        updated_at: now,
    })
}

// ─── Structured Summaries ────────────────────────────────────────────────────

#[tauri::command]
pub fn list_structured_summaries(
    state: State<'_, AppState>,
) -> Result<Vec<StructuredSummary>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let mut stmt = connection
        .prepare(
            "SELECT id, source_type, source_id, title, summary, evidence, created_at
             FROM structured_summaries ORDER BY created_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let ev_str: String = row.get(5)?;
            let evidence: serde_json::Value =
                serde_json::from_str(&ev_str).unwrap_or_default();
            Ok(StructuredSummary {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_id: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                evidence,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut summaries = Vec::new();
    for row in rows {
        summaries.push(row.map_err(|e| e.to_string())?);
    }
    Ok(summaries)
}

// ─── Memory Stats ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_memory_stats(state: State<'_, AppState>) -> Result<MemoryStats, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    query_memory_stats(&connection).map_err(|e| e.to_string())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or("Untitled")
        .chars()
        .take(80)
        .collect()
}

pub fn write_daily_log_entry(
    connection: &Connection,
    log_date: &str,
    entry_type: &str,
    title: &str,
    content: &str,
) -> rusqlite::Result<()> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO daily_log_entries (id, log_date, entry_type, title, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, log_date, entry_type, title, content, now],
    )?;
    Ok(())
}
