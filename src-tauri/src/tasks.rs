use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::{
    models::{
        CreateKaizenTaskRequest, KaizenProject, KaizenTask, KanbanColumn, TodayBoardGroup,
    },
    state::{map_kaizen_task, query_today_tasks, AppState},
};

// ─── Projects ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_kaizen_project(
    title: String,
    domain: String,
    description: String,
    state: State<'_, AppState>,
) -> Result<KaizenProject, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    connection
        .execute(
            "INSERT INTO kaizen_projects (id, title, domain, description, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
            params![id, title, domain, description, now],
        )
        .map_err(|e| e.to_string())?;

    Ok(KaizenProject {
        id,
        title,
        domain,
        description,
        status: "active".to_string(),
        task_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn list_kaizen_projects(
    state: State<'_, AppState>,
) -> Result<Vec<KaizenProject>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let mut stmt = connection
        .prepare(
            "SELECT p.id, p.title, p.domain, p.description, p.status,
                    COUNT(t.id) as task_count,
                    p.created_at, p.updated_at
             FROM kaizen_projects p
             LEFT JOIN kaizen_tasks t ON t.project_id = p.id AND t.status NOT IN ('done','cancelled')
             WHERE p.status != 'archived'
             GROUP BY p.id
             ORDER BY p.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(KaizenProject {
                id: row.get(0)?,
                title: row.get(1)?,
                domain: row.get(2)?,
                description: row.get(3)?,
                status: row.get(4)?,
                task_count: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row.map_err(|e| e.to_string())?);
    }
    Ok(projects)
}

// ─── Tasks ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_kaizen_task(
    request: CreateKaizenTaskRequest,
    state: State<'_, AppState>,
) -> Result<KaizenTask, String> {
    if request.title.trim().is_empty() {
        return Err("Task title is required.".to_string());
    }

    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let agent_mode = request.agent_mode.unwrap_or_else(|| "manual".to_string());
    let approval_required = request.approval_required.unwrap_or(false) as i64;
    let notes = request.notes.unwrap_or_default();

    connection
        .execute(
            "INSERT INTO kaizen_tasks
             (id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
              status, do_date, deadline, agent_mode, approval_required, evidence, notes,
              created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'inbox', ?8, ?9, ?10, ?11, '{}', ?12, ?13, ?13)",
            params![
                id,
                request.project_id,
                request.parent_task_id,
                request.title,
                request.domain,
                request.energy,
                request.estimate_minutes,
                request.do_date,
                request.deadline,
                agent_mode,
                approval_required,
                notes,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

    Ok(KaizenTask {
        id,
        project_id: request.project_id,
        parent_task_id: request.parent_task_id,
        title: request.title,
        domain: request.domain,
        energy: request.energy,
        estimate_minutes: request.estimate_minutes,
        status: "inbox".to_string(),
        do_date: request.do_date,
        deadline: request.deadline,
        agent_mode,
        approval_required: request.approval_required.unwrap_or(false),
        evidence: serde_json::Value::Object(Default::default()),
        notes,
        subtask_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn list_kaizen_tasks(
    project_id: Option<String>,
    status_filter: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<KaizenTask>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let (where_clause, bind_count) = match (&project_id, &status_filter) {
        (Some(_), Some(_)) => ("WHERE t.project_id = ?1 AND t.status = ?2", 2),
        (Some(_), None) => ("WHERE t.project_id = ?1", 1),
        (None, Some(_)) => ("WHERE t.status = ?1", 1),
        (None, None) => ("", 0),
    };

    let sql = format!(
        "SELECT t.id, t.project_id, t.parent_task_id, t.title, t.domain, t.energy,
                t.estimate_minutes, t.status, t.do_date, t.deadline, t.agent_mode,
                t.approval_required, t.evidence, t.notes, t.created_at, t.updated_at
         FROM kaizen_tasks t
         {where_clause}
         ORDER BY t.do_date ASC NULLS LAST, t.updated_at DESC LIMIT 200"
    );

    let mut stmt = connection.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = match bind_count {
        2 => stmt
            .query_map(
                params![project_id.as_deref().unwrap(), status_filter.as_deref().unwrap()],
                map_kaizen_task,
            )
            .map_err(|e| e.to_string())?,
        1 => {
            let val = project_id
                .as_deref()
                .or(status_filter.as_deref())
                .unwrap_or("");
            stmt.query_map(params![val], map_kaizen_task)
                .map_err(|e| e.to_string())?
        }
        _ => stmt
            .query_map([], map_kaizen_task)
            .map_err(|e| e.to_string())?,
    };

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
) -> Result<(), String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    connection
        .execute(
            "UPDATE kaizen_tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )
        .map_err(|e| e.to_string())?;

    // Write to daily log on task completion
    if status == "done" {
        let title: Option<String> = connection
            .query_row("SELECT title FROM kaizen_tasks WHERE id = ?1", params![id], |row| row.get(0))
            .ok();
        let task_title = title.unwrap_or_else(|| "Task".to_string());
        let log_date = &now[..10];
        let _ = connection.execute(
            "INSERT INTO daily_log_entries (id, log_date, entry_type, title, content, created_at)
             VALUES (?1, ?2, 'task_complete', ?3, '', ?4)",
            params![Uuid::new_v4().to_string(), log_date, format!("Completed: {task_title}"), now],
        );
    }

    Ok(())
}

#[tauri::command]
pub fn update_kaizen_task_do_date(
    id: String,
    do_date: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    connection
        .execute(
            "UPDATE kaizen_tasks SET do_date = ?1, updated_at = ?2 WHERE id = ?3",
            params![do_date, now, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Today Board ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_today_board(state: State<'_, AppState>) -> Result<Vec<TodayBoardGroup>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;
    let tasks = query_today_tasks(&connection).map_err(|e| e.to_string())?;

    let mut groups: std::collections::HashMap<String, Vec<KaizenTask>> =
        std::collections::HashMap::new();

    for task in tasks {
        groups.entry(task.domain.clone()).or_default().push(task);
    }

    let domain_order = ["work", "health", "learning", "personal", "system"];
    let mut board: Vec<TodayBoardGroup> = Vec::new();

    // Ordered domains first
    for domain in domain_order {
        if let Some(task_list) = groups.remove(domain) {
            board.push(TodayBoardGroup {
                domain: domain.to_string(),
                tasks: task_list,
            });
        }
    }
    // Any remaining custom domains
    for (domain, task_list) in groups {
        board.push(TodayBoardGroup {
            domain,
            tasks: task_list,
        });
    }

    Ok(board)
}

// ─── Kanban ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_kanban_board(
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<KanbanColumn>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let where_clause = if project_id.is_some() {
        "WHERE t.project_id = ?1"
    } else {
        "WHERE 1=1"
    };

    let sql = format!(
        "SELECT t.id, t.project_id, t.parent_task_id, t.title, t.domain, t.energy,
                t.estimate_minutes, t.status, t.do_date, t.deadline, t.agent_mode,
                t.approval_required, t.evidence, t.notes, t.created_at, t.updated_at
         FROM kaizen_tasks t
         {where_clause}
         ORDER BY t.updated_at DESC LIMIT 500"
    );

    let mut stmt = connection.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = if let Some(ref pid) = project_id {
        stmt.query_map(params![pid], map_kaizen_task)
            .map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], map_kaizen_task)
            .map_err(|e| e.to_string())?
    };

    let mut all_tasks: Vec<KaizenTask> = Vec::new();
    for row in rows {
        all_tasks.push(row.map_err(|e| e.to_string())?);
    }

    let statuses = [
        ("inbox", "Inbox"),
        ("todo", "To Do"),
        ("doing", "Doing"),
        ("blocked", "Blocked"),
        ("done", "Done"),
        ("cancelled", "Cancelled"),
    ];

    let columns = statuses
        .iter()
        .map(|(status, label)| {
            let tasks = all_tasks
                .iter()
                .filter(|t| t.status == *status)
                .cloned()
                .collect();
            KanbanColumn {
                status: status.to_string(),
                label: label.to_string(),
                tasks,
            }
        })
        .collect();

    Ok(columns)
}

// ─── Decompose task (>90min) ──────────────────────────────────────────────────

#[tauri::command]
pub fn decompose_task(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<KaizenTask>, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let parent: KaizenTask = connection
        .query_row(
            "SELECT id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
                    status, do_date, deadline, agent_mode, approval_required, evidence, notes,
                    created_at, updated_at
             FROM kaizen_tasks WHERE id = ?1",
            params![task_id],
            map_kaizen_task,
        )
        .map_err(|e| e.to_string())?;

    if parent.estimate_minutes <= 90 {
        return Err("Task is already ≤90 minutes, no decomposition needed.".to_string());
    }

    let chunks = (parent.estimate_minutes + 89) / 90;
    let per_chunk = parent.estimate_minutes / chunks;
    let now = Utc::now().to_rfc3339();
    let mut subtasks = Vec::new();

    for i in 1..=chunks {
        let id = Uuid::new_v4().to_string();
        let subtask_title = format!("{} – part {}/{}", parent.title, i, chunks);
        connection
            .execute(
                "INSERT INTO kaizen_tasks
                 (id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
                  status, do_date, deadline, agent_mode, approval_required, evidence, notes,
                  created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'inbox', ?8, ?9, ?10, ?11, '{}', '', ?12, ?12)",
                params![
                    id,
                    parent.project_id,
                    task_id,
                    subtask_title,
                    parent.domain,
                    parent.energy,
                    per_chunk,
                    parent.do_date,
                    parent.deadline,
                    parent.agent_mode,
                    parent.approval_required as i64,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;
        subtasks.push(KaizenTask {
            id,
            project_id: parent.project_id.clone(),
            parent_task_id: Some(task_id.clone()),
            title: subtask_title,
            domain: parent.domain.clone(),
            energy: parent.energy.clone(),
            estimate_minutes: per_chunk,
            status: "inbox".to_string(),
            do_date: parent.do_date.clone(),
            deadline: parent.deadline.clone(),
            agent_mode: parent.agent_mode.clone(),
            approval_required: parent.approval_required,
            evidence: serde_json::Value::Object(Default::default()),
            notes: String::new(),
            subtask_count: 0,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
    }

    Ok(subtasks)
}

// ─── Minimum Version ─────────────────────────────────────────────────────────

/// Generate a "Minimum Version" subtask — fastest path to 80% of the value.
/// The subtask is titled "MVP: <parent title>" with estimate capped at 25 min.
#[tauri::command]
pub fn generate_minimum_version(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<KaizenTask, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let parent: KaizenTask = connection
        .query_row(
            "SELECT id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
                    status, do_date, deadline, agent_mode, approval_required, evidence, notes,
                    created_at, updated_at
             FROM kaizen_tasks WHERE id = ?1",
            params![task_id],
            map_kaizen_task,
        )
        .map_err(|e| e.to_string())?;

    let mvp_estimate = (parent.estimate_minutes / 4).max(5).min(25);
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let mvp_title = format!("MVP: {}", parent.title);
    let notes = format!(
        "Minimum Version of '{}'\nFocus: ship 80% value in {} minutes. Cut scope aggressively.",
        parent.title, mvp_estimate
    );

    connection
        .execute(
            "INSERT INTO kaizen_tasks
             (id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
              status, do_date, deadline, agent_mode, approval_required, evidence, notes,
              created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'low', ?6, 'inbox', ?7, NULL, 'manual', 0, '{}', ?8, ?9, ?9)",
            params![
                id,
                parent.project_id,
                task_id,
                mvp_title,
                parent.domain,
                mvp_estimate,
                parent.do_date,
                notes,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

    Ok(KaizenTask {
        id,
        project_id: parent.project_id,
        parent_task_id: Some(task_id),
        title: mvp_title,
        domain: parent.domain,
        energy: "low".to_string(),
        estimate_minutes: mvp_estimate,
        status: "inbox".to_string(),
        do_date: parent.do_date,
        deadline: None,
        agent_mode: "manual".to_string(),
        approval_required: false,
        evidence: serde_json::Value::Object(Default::default()),
        notes,
        subtask_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Generate a "Low-Energy Version" — same task but re-scoped for low-energy state.
/// Useful for ADHD/dyslexia: do something when brain fog hits.
#[tauri::command]
pub fn generate_low_energy_version(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<KaizenTask, String> {
    let connection =
        Connection::open(&state.paths.database_path).map_err(|e| e.to_string())?;

    let parent: KaizenTask = connection
        .query_row(
            "SELECT id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
                    status, do_date, deadline, agent_mode, approval_required, evidence, notes,
                    created_at, updated_at
             FROM kaizen_tasks WHERE id = ?1",
            params![task_id],
            map_kaizen_task,
        )
        .map_err(|e| e.to_string())?;

    let low_estimate = (parent.estimate_minutes / 3).max(5).min(20);
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let low_title = format!("🟢 Low-energy: {}", parent.title);
    let notes = format!(
        "Low-Energy Version of '{}'\nBrain fog mode: just do the simplest possible action in {} min.",
        parent.title, low_estimate
    );

    connection
        .execute(
            "INSERT INTO kaizen_tasks
             (id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
              status, do_date, deadline, agent_mode, approval_required, evidence, notes,
              created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'low', ?6, 'inbox', ?7, NULL, 'manual', 0, '{}', ?8, ?9, ?9)",
            params![
                id,
                parent.project_id,
                task_id,
                low_title,
                parent.domain,
                low_estimate,
                parent.do_date,
                notes,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

    Ok(KaizenTask {
        id,
        project_id: parent.project_id,
        parent_task_id: Some(task_id),
        title: low_title,
        domain: parent.domain,
        energy: "low".to_string(),
        estimate_minutes: low_estimate,
        status: "inbox".to_string(),
        do_date: parent.do_date,
        deadline: None,
        agent_mode: "manual".to_string(),
        approval_required: false,
        evidence: serde_json::Value::Object(Default::default()),
        notes,
        subtask_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}
