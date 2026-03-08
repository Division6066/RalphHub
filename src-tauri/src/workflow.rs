use std::fs;

use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;

use uuid::Uuid;

use crate::{
    memory::write_daily_log_entry,
    models::{WorkflowRequest, WorkflowRun},
    state::AppState,
    tool_registry::all_tools,
};

#[tauri::command]
pub fn create_workflow_run(
    request: WorkflowRequest,
    state: State<'_, AppState>,
) -> Result<WorkflowRun, String> {
    if request.tool_ids.is_empty() {
        return Err("Select at least one tool for the workflow.".to_string());
    }

    let run_id = format!(
        "{}-{}",
        slugify(&request.name),
        Utc::now().format("%Y%m%d%H%M%S")
    );
    let timestamp = Utc::now().to_rfc3339();
    let config_path = state
        .paths
        .workflows_dir
        .join(format!("{run_id}.json"));
    let state_path = state.paths.workflows_dir.join(format!("{run_id}.md"));

    let known_tools = all_tools();
    let selected_tools = known_tools
        .into_iter()
        .filter(|tool| request.tool_ids.contains(&tool.id))
        .map(|tool| {
            serde_json::json!({
                "id": tool.id,
                "name": tool.name,
                "repoUrl": tool.repo_url,
                "launchCommand": tool.launch_command
            })
        })
        .collect::<Vec<_>>();

    let workflow_config = serde_json::json!({
        "id": run_id,
        "name": request.name,
        "modelName": request.model_name,
        "toolIds": request.tool_ids,
        "steps": selected_tools,
        "createdAt": timestamp,
        "strategy": "overnight-chain"
    });

    fs::write(
        &config_path,
        serde_json::to_string_pretty(&workflow_config).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    fs::write(
        &state_path,
        format!(
            "# Workflow State\n\n- Name: {}\n- Model: {}\n- Status: prepared\n- Tools: {}\n- Created: {}\n",
            request.name,
            request.model_name,
            request.tool_ids.join(", "),
            timestamp
        ),
    )
    .map_err(|error| error.to_string())?;

    let run = WorkflowRun {
        id: run_id,
        workflow_name: request.name,
        tool_ids: request.tool_ids,
        model_name: request.model_name,
        status: "prepared".to_string(),
        config_path: config_path.display().to_string(),
        state_path: state_path.display().to_string(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };

    insert_workflow_run(&state, &run)?;

    // Auto-write workflow creation to Memory Spine + Daily Log
    if let Ok(conn) = Connection::open(&state.paths.database_path) {
        let ev_id = Uuid::new_v4().to_string();
        let now_str = run.created_at.clone();
        let summary = format!("Workflow '{}' created with tools: {}", run.workflow_name, run.tool_ids.join(", "));
        let _ = conn.execute(
            "INSERT INTO raw_events (id, source_type, content, metadata, created_at) VALUES (?1, 'workflow', ?2, '{}', ?3)",
            params![ev_id, summary, now_str],
        );
        let _ = write_daily_log_entry(&conn, &run.created_at[..10], "agent_run", &format!("Workflow created: {}", run.workflow_name), &summary);
    }

    Ok(run)
}

#[tauri::command]
pub fn list_workflow_runs(state: State<'_, AppState>) -> Result<Vec<WorkflowRun>, String> {
    let connection = Connection::open(&state.paths.database_path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, workflow_name, status, config_path, state_path, created_at, updated_at
            FROM workflow_runs
            ORDER BY updated_at DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(WorkflowRun {
                id,
                workflow_name: row.get(1)?,
                tool_ids: Vec::new(),
                model_name: "custom".to_string(),
                status: row.get(2)?,
                config_path: row.get(3)?,
                state_path: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut runs = Vec::new();
    for row in rows {
        runs.push(row.map_err(|error| error.to_string())?);
    }

    Ok(runs)
}

fn insert_workflow_run(state: &AppState, run: &WorkflowRun) -> Result<(), String> {
    let connection = Connection::open(&state.paths.database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "
            INSERT INTO workflow_runs (id, workflow_name, status, config_path, state_path, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                config_path = excluded.config_path,
                state_path = excluded.state_path,
                updated_at = excluded.updated_at
            ",
            params![
                run.id,
                run.workflow_name,
                run.status,
                run.config_path,
                run.state_path,
                run.created_at,
                run.updated_at
            ],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn slugify(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|char| if char.is_ascii_alphanumeric() { char } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
