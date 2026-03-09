use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::models::{
    CommandResponse, CreateKaizenTaskRequest, KaizenDomain, KaizenTask, UpdateKaizenTaskRequest,
};
use crate::state::AppState;

// ─── Tasks ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_kaizen_tasks(
    state: State<'_, AppState>,
    domain: Option<String>,
    today_only: Option<bool>,
    status: Option<String>,
) -> Result<Vec<KaizenTask>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let mut conditions = vec!["1=1".to_string()];
    if let Some(d) = &domain {
        conditions.push(format!("domain = '{}'", d.replace('\'', "''")));
    }
    if today_only.unwrap_or(false) {
        conditions.push("is_today = 1".to_string());
    }
    if let Some(s) = &status {
        conditions.push(format!("status = '{}'", s.replace('\'', "''")));
    }

    let sql = format!(
        "SELECT id, title, description, domain, status, is_today, is_minimum_version,
                priority, parent_id, subtasks, energy, estimated_minutes, tags, due_date,
                created_at, updated_at
         FROM kaizen_tasks
         WHERE {}
         ORDER BY is_today DESC, priority ASC, created_at DESC",
        conditions.join(" AND ")
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let tasks = stmt
        .query_map([], |row| {
            let subtasks_json: String = row.get(9)?;
            let tags_json: String = row.get(12)?;
            Ok(KaizenTask {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                domain: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "general".to_string()),
                status: row.get(4)?,
                is_today: row.get::<_, i32>(5)? != 0,
                is_minimum_version: row.get::<_, i32>(6)? != 0,
                priority: row.get(7)?,
                parent_id: row.get(8)?,
                subtasks: serde_json::from_str(&subtasks_json).unwrap_or_default(),
                energy: row.get::<_, Option<String>>(10)?.unwrap_or_else(|| "medium".to_string()),
                estimated_minutes: row.get(11)?,
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                due_date: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                source: None,
                provider_id: None,
                usage_log_id: None,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tasks)
}

#[tauri::command]
pub fn create_kaizen_task(
    state: State<'_, AppState>,
    request: CreateKaizenTaskRequest,
) -> Result<KaizenTask, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let tags = request.tags.unwrap_or_default();
    let energy = request.energy.unwrap_or_else(|| "medium".to_string());
    let tags_json = serde_json::to_string(&tags).unwrap_or_default();

    conn.execute(
        "INSERT INTO kaizen_tasks
         (id, title, description, domain, status, is_today, is_minimum_version, priority,
          parent_id, subtasks, energy, estimated_minutes, tags, due_date, created_at, updated_at)
         VALUES (?1,?2,?3,?4,'todo',?5,?6,?7,?8,'[]',?9,?10,?11,?12,?13,?13)",
        params![
            id,
            request.title,
            request.description,
            request.domain.as_deref().unwrap_or("general"),
            if request.is_today.unwrap_or(false) { 1 } else { 0 },
            if request.is_minimum_version.unwrap_or(false) { 1 } else { 0 },
            request.priority.unwrap_or(3),
            request.parent_id,
            energy,
            request.estimated_minutes,
            tags_json,
            request.due_date,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(KaizenTask {
        id,
        title: request.title,
        description: request.description,
        domain: request.domain.unwrap_or_else(|| "general".to_string()),
        status: "todo".to_string(),
        is_today: request.is_today.unwrap_or(false),
        is_minimum_version: request.is_minimum_version.unwrap_or(false),
        priority: request.priority.unwrap_or(3),
        parent_id: request.parent_id,
        subtasks: vec![],
        energy: energy.clone(),
        estimated_minutes: request.estimated_minutes,
        tags,
        due_date: request.due_date,
        created_at: now.clone(),
        updated_at: now,
        source: None,
        provider_id: None,
        usage_log_id: None,
    })
}

#[tauri::command]
pub fn update_kaizen_task(
    state: State<'_, AppState>,
    request: UpdateKaizenTaskRequest,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    let mut updates = vec!["updated_at = ?1".to_string()];
    let mut values: Vec<String> = vec![now.clone()];
    let mut idx = 2usize;

    macro_rules! maybe_update {
        ($field:expr, $val:expr) => {
            if let Some(v) = $val {
                updates.push(format!("{} = ?{}", $field, idx));
                values.push(v.to_string());
                idx += 1;
            }
        };
    }

    maybe_update!("title", request.title.as_deref());
    maybe_update!("description", request.description.as_deref());
    maybe_update!("domain", request.domain.as_deref());
    maybe_update!("status", request.status.as_deref());
    maybe_update!("energy", request.energy.as_deref());

    if let Some(t) = request.is_today {
        updates.push(format!("is_today = ?{idx}"));
        values.push(if t { "1" } else { "0" }.to_string());
        idx += 1;
    }
    if let Some(m) = request.is_minimum_version {
        updates.push(format!("is_minimum_version = ?{idx}"));
        values.push(if m { "1" } else { "0" }.to_string());
        idx += 1;
    }
    if let Some(p) = request.priority {
        updates.push(format!("priority = ?{idx}"));
        values.push(p.to_string());
        idx += 1;
    }
    if let Some(em) = request.estimated_minutes {
        updates.push(format!("estimated_minutes = ?{idx}"));
        values.push(em.to_string());
        idx += 1;
    }
    if let Some(tags) = request.tags {
        updates.push(format!("tags = ?{idx}"));
        values.push(serde_json::to_string(&tags).unwrap_or_default());
        idx += 1;
    }
    if let Some(dd) = request.due_date {
        updates.push(format!("due_date = ?{idx}"));
        values.push(dd);
        idx += 1;
    }

    values.push(request.id.clone());
    let sql = format!(
        "UPDATE kaizen_tasks SET {} WHERE id = ?{}",
        updates.join(", "),
        idx
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> =
        values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
    conn.execute(&sql, params_refs.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!("Task {} updated.", request.id),
    })
}

#[tauri::command]
pub fn delete_kaizen_task(
    state: State<'_, AppState>,
    id: String,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM kaizen_tasks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(CommandResponse {
        ok: true,
        message: format!("Task {id} deleted."),
    })
}

#[tauri::command]
pub fn set_today_tasks(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<CommandResponse, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    // Clear all today flags
    conn.execute("UPDATE kaizen_tasks SET is_today = 0", [])
        .map_err(|e| e.to_string())?;
    // Set selected ones
    for id in &ids {
        conn.execute(
            "UPDATE kaizen_tasks SET is_today = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(CommandResponse {
        ok: true,
        message: format!("{} tasks set as Today.", ids.len()),
    })
}

// ─── Domains ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_kaizen_domains(state: State<'_, AppState>) -> Result<Vec<KaizenDomain>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.name, d.color, d.icon, d.description,
                    (SELECT COUNT(*) FROM kaizen_tasks t WHERE t.domain = d.id) as task_count,
                    (SELECT COUNT(*) FROM kaizen_tasks t WHERE t.domain = d.id AND t.is_today = 1 AND t.status != 'done') as today_count
             FROM kaizen_domains d
             ORDER BY d.name",
        )
        .map_err(|e| e.to_string())?;

    let domains = stmt
        .query_map([], |row| {
            Ok(KaizenDomain {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
                description: row.get(4)?,
                task_count: row.get(5)?,
                today_count: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(domains)
}

#[tauri::command]
pub fn decompose_task(
    state: State<'_, AppState>,
    parent_id: String,
    subtask_titles: Vec<String>,
) -> Result<Vec<KaizenTask>, String> {
    let conn = Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    // Get parent domain
    let parent_domain: String = conn
        .query_row(
            "SELECT domain FROM kaizen_tasks WHERE id = ?1",
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut created = vec![];
    let mut child_ids = vec![];

    for title in &subtask_titles {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO kaizen_tasks
             (id, title, domain, status, is_today, is_minimum_version, priority,
              parent_id, subtasks, energy, tags, created_at, updated_at)
             VALUES (?1,?2,?3,'todo',0,0,3,?4,'[]','medium','[]',?5,?5)",
            params![id, title, parent_domain, parent_id, now],
        )
        .map_err(|e| e.to_string())?;

        child_ids.push(id.clone());
        created.push(KaizenTask {
            id,
            title: title.clone(),
            description: None,
            domain: parent_domain.clone(),
            status: "todo".to_string(),
            is_today: false,
            is_minimum_version: false,
            priority: 3,
            parent_id: Some(parent_id.clone()),
            subtasks: vec![],
            energy: "medium".to_string(),
            estimated_minutes: None,
            tags: vec![],
            due_date: None,
            created_at: now.clone(),
            updated_at: now.clone(),
            source: None,
            provider_id: None,
            usage_log_id: None,
        });
    }

    // Update parent's subtasks list
    let subtasks_json = serde_json::to_string(&child_ids).unwrap_or_default();
    conn.execute(
        "UPDATE kaizen_tasks SET subtasks = ?1 WHERE id = ?2",
        params![subtasks_json, parent_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(created)
}
