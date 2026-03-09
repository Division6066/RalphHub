use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::models::{CommandResponse, CreateMemoryRequest, MemoryEntry, MemorySearchRequest};
use crate::state::AppState;

#[tauri::command]
pub fn list_memory_entries(
    state: State<'_, AppState>,
    domain: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<MemoryEntry>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let lim = limit.unwrap_or(50);

    let sql = if let Some(d) = domain {
        format!(
            "SELECT id, title, content, tags, domain, source, created_at, updated_at
             FROM memory_entries WHERE domain = '{}'
             ORDER BY updated_at DESC LIMIT {}",
            d.replace('\'', "''"),
            lim
        )
    } else {
        format!(
            "SELECT id, title, content, tags, domain, source, created_at, updated_at
             FROM memory_entries
             ORDER BY updated_at DESC LIMIT {lim}"
        )
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map([], |row| {
            let tags_json: String = row.get(3)?;
            Ok(MemoryEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                domain: row.get(4)?,
                source: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

#[tauri::command]
pub fn create_memory_entry(
    state: State<'_, AppState>,
    request: CreateMemoryRequest,
) -> Result<MemoryEntry, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let tags = request.tags.unwrap_or_default();
    let tags_json = serde_json::to_string(&tags).unwrap_or_default();
    let domain = request.domain.unwrap_or_else(|| "general".to_string());
    let source = request.source.unwrap_or_else(|| "manual".to_string());

    conn.execute(
        "INSERT INTO memory_entries (id, title, content, tags, domain, source, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?7)",
        params![id, request.title, request.content, tags_json, domain, source, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(MemoryEntry {
        id,
        title: request.title,
        content: request.content,
        tags,
        domain,
        source,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn search_memory(
    state: State<'_, AppState>,
    request: MemorySearchRequest,
) -> Result<Vec<MemoryEntry>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let lim = request.limit.unwrap_or(20);
    let q = format!("%{}%", request.query.to_lowercase());

    let sql = if let Some(d) = &request.domain {
        format!(
            "SELECT id, title, content, tags, domain, source, created_at, updated_at
             FROM memory_entries
             WHERE domain = '{}' AND (LOWER(title) LIKE ?1 OR LOWER(content) LIKE ?1 OR LOWER(tags) LIKE ?1)
             ORDER BY updated_at DESC LIMIT {}",
            d.replace('\'', "''"),
            lim
        )
    } else {
        format!(
            "SELECT id, title, content, tags, domain, source, created_at, updated_at
             FROM memory_entries
             WHERE LOWER(title) LIKE ?1 OR LOWER(content) LIKE ?1 OR LOWER(tags) LIKE ?1
             ORDER BY updated_at DESC LIMIT {lim}"
        )
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map(params![q], |row| {
            let tags_json: String = row.get(3)?;
            Ok(MemoryEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                domain: row.get(4)?,
                source: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

#[tauri::command]
pub fn delete_memory_entry(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memory_entries WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(CommandResponse {
        ok: true,
        message: format!("Memory entry {id} deleted."),
    })
}

#[tauri::command]
pub fn update_memory_entry(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    domain: Option<String>,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut vals: Vec<String> = vec![now];
    let mut idx = 2usize;

    if let Some(t) = title {
        sets.push(format!("title = ?{idx}"));
        vals.push(t);
        idx += 1;
    }
    if let Some(c) = content {
        sets.push(format!("content = ?{idx}"));
        vals.push(c);
        idx += 1;
    }
    if let Some(tg) = tags {
        sets.push(format!("tags = ?{idx}"));
        vals.push(serde_json::to_string(&tg).unwrap_or_default());
        idx += 1;
    }
    if let Some(d) = domain {
        sets.push(format!("domain = ?{idx}"));
        vals.push(d);
        idx += 1;
    }

    vals.push(id.clone());
    let sql = format!("UPDATE memory_entries SET {} WHERE id = ?{}", sets.join(", "), idx);
    let params_refs: Vec<&dyn rusqlite::ToSql> =
        vals.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
    conn.execute(&sql, params_refs.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!("Memory entry {id} updated."),
    })
}
