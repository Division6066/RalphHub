use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::{
    models::{AddDailyLogRequest, AddInboxRequest, DailyLogEntry, InboxItem, KaizenTask, MorningDigest},
    state::{query_today_tasks, AppState},
};

// ─── Inbox ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_inbox_item(
    request: AddInboxRequest,
    state: State<'_, AppState>,
) -> Result<InboxItem, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let source = request.source.unwrap_or_else(|| "manual".to_string());

    connection
        .execute(
            "INSERT INTO inbox_items (id, content, content_type, processed, source, created_at)
             VALUES (?1, ?2, ?3, 0, ?4, ?5)",
            params![id, request.content, request.content_type, source, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(InboxItem {
        id,
        content: request.content,
        content_type: request.content_type,
        processed: false,
        source,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_inbox_items(
    unprocessed_only: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<InboxItem>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let sql = if unprocessed_only.unwrap_or(false) {
        "SELECT id, content, content_type, processed, source, created_at
         FROM inbox_items WHERE processed = 0 ORDER BY created_at DESC LIMIT 100"
    } else {
        "SELECT id, content, content_type, processed, source, created_at
         FROM inbox_items ORDER BY created_at DESC LIMIT 100"
    };

    let mut stmt = connection.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(InboxItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                processed: row.get::<_, i64>(3)? != 0,
                source: row.get(4)?,
                created_at: row.get(5)?,
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
pub fn mark_inbox_processed(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    connection
        .execute(
            "UPDATE inbox_items SET processed = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn inbox_to_task(
    inbox_id: String,
    domain: String,
    energy: String,
    estimate_minutes: i64,
    do_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<KaizenTask, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let content: String = connection
        .query_row(
            "SELECT content FROM inbox_items WHERE id = ?1",
            params![inbox_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let title: String = content.lines().next().unwrap_or("Task from inbox").chars().take(80).collect();
    let task_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    connection
        .execute(
            "INSERT INTO kaizen_tasks
             (id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
              status, do_date, deadline, agent_mode, approval_required, evidence, notes,
              created_at, updated_at)
             VALUES (?1, NULL, NULL, ?2, ?3, ?4, ?5, 'inbox', ?6, NULL, 'manual', 0, '{}', ?7, ?8, ?8)",
            params![
                task_id, title, domain, energy, estimate_minutes,
                do_date, content, now
            ],
        )
        .map_err(|e| e.to_string())?;

    connection
        .execute(
            "UPDATE inbox_items SET processed = 1 WHERE id = ?1",
            params![inbox_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(KaizenTask {
        id: task_id,
        project_id: None,
        parent_task_id: None,
        title,
        domain,
        energy,
        estimate_minutes,
        status: "inbox".to_string(),
        do_date,
        deadline: None,
        agent_mode: "manual".to_string(),
        approval_required: false,
        evidence: serde_json::Value::Object(Default::default()),
        notes: content,
        subtask_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

// ─── Daily Log ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_daily_log_entry(
    request: AddDailyLogRequest,
    state: State<'_, AppState>,
) -> Result<DailyLogEntry, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let log_date = request
        .log_date
        .unwrap_or_else(|| now[..10].to_string());

    connection
        .execute(
            "INSERT INTO daily_log_entries (id, log_date, entry_type, title, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, log_date, request.entry_type, request.title, request.content, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(DailyLogEntry {
        id,
        log_date,
        entry_type: request.entry_type,
        title: request.title,
        content: request.content,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_daily_log(
    date: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DailyLogEntry>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let sql = if date.is_some() {
        "SELECT id, log_date, entry_type, title, content, created_at
         FROM daily_log_entries WHERE log_date = ?1 ORDER BY created_at ASC"
    } else {
        "SELECT id, log_date, entry_type, title, content, created_at
         FROM daily_log_entries ORDER BY created_at DESC LIMIT 100"
    };

    let mut stmt = connection.prepare(sql).map_err(|e| e.to_string())?;

    let rows = if let Some(ref d) = date {
        stmt.query_map(params![d], map_daily_log_row)
            .map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], map_daily_log_row)
            .map_err(|e| e.to_string())?
    };

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }
    Ok(entries)
}

#[tauri::command]
pub fn get_morning_digest(state: State<'_, AppState>) -> Result<MorningDigest, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let today_tasks = query_today_tasks(&connection).map_err(|e| e.to_string())?;

    let inbox_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM inbox_items WHERE processed = 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let memory_updates: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM raw_events WHERE created_at LIKE ?1",
            params![format!("{yesterday}%")],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let yesterday_entries = list_daily_log_for_date(&connection, &yesterday);
    let yesterday_summary = if yesterday_entries.is_empty() {
        "No activity logged yesterday.".to_string()
    } else {
        yesterday_entries
            .iter()
            .map(|e| format!("• {}", e.title))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Auto-write morning digest to log
    let _ = connection.execute(
        "INSERT INTO daily_log_entries (id, log_date, entry_type, title, content, created_at)
         VALUES (?1, ?2, 'morning_digest', 'Morning Digest', ?3, ?4)",
        params![
            Uuid::new_v4().to_string(),
            today,
            format!("{} tasks today, {} inbox items", today_tasks.len(), inbox_count),
            Utc::now().to_rfc3339()
        ],
    );

    Ok(MorningDigest {
        date: today,
        today_tasks,
        inbox_count,
        memory_updates,
        yesterday_summary,
    })
}

fn map_daily_log_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DailyLogEntry> {
    Ok(DailyLogEntry {
        id: row.get(0)?,
        log_date: row.get(1)?,
        entry_type: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn list_daily_log_for_date(connection: &Connection, date: &str) -> Vec<DailyLogEntry> {
    let mut stmt = match connection.prepare(
        "SELECT id, log_date, entry_type, title, content, created_at
         FROM daily_log_entries WHERE log_date = ?1 ORDER BY created_at ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![date], map_daily_log_row)
        .unwrap_or_else(|_| panic!())
        .filter_map(|r| r.ok())
        .collect()
}
