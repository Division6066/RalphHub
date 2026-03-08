use chrono::Utc;
use tauri::State;
use uuid::Uuid;
use rusqlite::{params, Connection};

use crate::{
    models::{NotionSyncRequest, NotionSyncResult},
    state::AppState,
};

/// Push AmitOS tasks to Notion (stub — requires HTTP client in future).
/// Writes a daily log entry and returns a sync result.
#[tauri::command]
pub fn sync_notion(
    request: NotionSyncRequest,
    state: State<'_, AppState>,
) -> Result<NotionSyncResult, String> {
    if request.api_key.trim().is_empty() {
        return Err("Notion API key is required. Add it in Settings.".to_string());
    }
    if request.database_id.trim().is_empty() {
        return Err("Notion database ID is required.".to_string());
    }

    let now = Utc::now().to_rfc3339();
    let log_date = &now[..10];

    // Log the sync attempt in daily log
    let connection = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO daily_log_entries (id, log_date, entry_type, title, content, created_at)
             VALUES (?1, ?2, 'notion_sync', 'Notion Sync', ?3, ?4)",
            params![
                Uuid::new_v4().to_string(),
                log_date,
                format!("Sync {} with Notion DB {}", request.direction, request.database_id),
                now
            ],
        )
        .map_err(|e| e.to_string())?;

    // TODO(M4): implement real HTTP calls to Notion API using reqwest or system curl.
    // For now, return a stub result showing the integration is wired.
    Ok(NotionSyncResult {
        pushed: 0,
        pulled: 0,
        errors: vec![
            "Notion HTTP integration is in stub mode. Full sync requires Notion API key configured in Settings and an HTTP client crate.".to_string()
        ],
        synced_at: now,
    })
}

/// Generate a "Open in Cursor Agent Web" URL with full Memory Spine context pack.
#[tauri::command]
pub fn open_in_cursor_agent_web(
    workflow_id: Option<String>,
    _memory_ids: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let connection = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    // Gather recent memory context
    let mut context_parts: Vec<String> = vec!["## AmitOS Memory Context Pack\n".to_string()];

    // Add recent working memory
    let mut wm_stmt = connection
        .prepare(
            "SELECT title, content FROM working_memory ORDER BY updated_at DESC LIMIT 5",
        )
        .map_err(|e| e.to_string())?;
    let wm_rows = wm_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    context_parts.push("### Working Memory\n".to_string());
    for row in wm_rows {
        if let Ok((title, content)) = row {
            context_parts.push(format!("**{title}**: {content}\n"));
        }
    }

    // Add recent raw events summary
    let mut ev_stmt = connection
        .prepare(
            "SELECT source_type, content FROM raw_events ORDER BY created_at DESC LIMIT 3",
        )
        .map_err(|e| e.to_string())?;
    let ev_rows = ev_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    context_parts.push("\n### Recent Raw Events\n".to_string());
    for row in ev_rows {
        if let Ok((src, content)) = row {
            let snippet: String = content.chars().take(120).collect();
            context_parts.push(format!("[{src}] {snippet}\n"));
        }
    }

    // Add workflow context
    if let Some(ref wid) = workflow_id {
        context_parts.push(format!("\n### Active Workflow: {wid}\n"));
    }

    let context_pack = context_parts.join("");
    let encoded = urlencoding_simple(&context_pack);

    // Build Cursor Agent Web URL (deep-link)
    let base_url = "https://cursor.com/agents/new";
    let url = format!("{base_url}?context={encoded}");

    // Log the action
    let now = Utc::now().to_rfc3339();
    let _ = connection.execute(
        "INSERT INTO daily_log_entries (id, log_date, entry_type, title, content, created_at)
         VALUES (?1, ?2, 'agent_run', 'Open in Cursor Agent Web', ?3, ?4)",
        params![
            Uuid::new_v4().to_string(),
            &now[..10],
            format!("Launched Cursor Agent Web with {} memory items", context_parts.len()),
            now
        ],
    );

    Ok(url)
}

/// Get AmitOS dashboard snapshot
#[tauri::command]
pub fn get_amitos_dashboard(
    state: State<'_, AppState>,
) -> Result<crate::models::AmitosDashboard, String> {
    state.amitos_dashboard().map_err(|e| e.to_string())
}

fn urlencoding_simple(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
