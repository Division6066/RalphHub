//! Example Workflow: Perplexica research → llm-council → task creation → memory write
//!
//! This module demonstrates the AmitOS agent automation pipeline described in Milestone 5:
//! 1. Perplexica research query runs in deployed workspace
//! 2. Results feed through llm-council for multi-model synthesis
//! 3. A follow-up Kaizen task is automatically created
//! 4. Summary and evidence is written to Memory Spine
//! 5. Daily Log entry is recorded
//!
//! In production, this is triggered by a workflow run completing.
//! The functions here are called from workflow.rs when a workflow status changes to "done".

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::{memory::write_daily_log_entry, state::AppState};

/// Run after a Perplexica + llm-council workflow completes.
/// Automatically creates a follow-up task and writes to memory.
pub fn on_workflow_complete(state: &AppState, workflow_name: &str, output_summary: &str) {
    let Ok(conn) = Connection::open(&state.paths.database_path) else {
        return;
    };

    let now = Utc::now().to_rfc3339();
    let today = &now[..10];

    // 1. Write raw event to Memory Spine
    let ev_id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO raw_events (id, source_type, content, metadata, created_at)
         VALUES (?1, 'workflow', ?2, '{}', ?3)",
        params![ev_id, output_summary, now],
    );

    // 2. Write structured summary
    let sum_id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO structured_summaries (id, source_type, source_id, title, summary, evidence, created_at)
         VALUES (?1, 'workflow', ?2, ?3, ?4, '{}', ?5)",
        params![
            sum_id,
            ev_id,
            format!("Workflow: {workflow_name}"),
            output_summary,
            now
        ],
    );

    // 3. Auto-create follow-up Kaizen task if workflow produced blockers
    //    (In real usage, the AI output would be parsed for action items)
    let task_id = Uuid::new_v4().to_string();
    let task_title = format!("Review results: {workflow_name}");
    let _ = conn.execute(
        "INSERT INTO kaizen_tasks
         (id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
          status, do_date, deadline, agent_mode, approval_required, evidence, notes,
          created_at, updated_at)
         VALUES (?1, NULL, NULL, ?2, 'work', 'medium', 30, 'inbox', ?3, NULL, 'manual', 0,
                 '{}', ?4, ?5, ?5)",
        params![
            task_id,
            task_title,
            today,
            format!("Auto-created from workflow completion: {workflow_name}"),
            now
        ],
    );

    // 4. Write daily log entry
    let _ = write_daily_log_entry(
        &conn,
        today,
        "agent_run",
        &format!("Workflow completed: {workflow_name}"),
        output_summary,
    );
}

/// Called when a Browser Agent action completes (e.g., Edge automation).
/// Writes evidence and screenshot description to Memory Spine.
pub fn on_browser_agent_action(state: &AppState, action_type: &str, url: &str, screenshot_desc: &str) {
    let Ok(conn) = Connection::open(&state.paths.database_path) else {
        return;
    };

    let now = Utc::now().to_rfc3339();
    let today = &now[..10];

    let content = format!("[{action_type}] {url}\n{screenshot_desc}");
    let ev_id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO raw_events (id, source_type, content, metadata, created_at)
         VALUES (?1, 'browser_agent', ?2, '{}', ?3)",
        params![ev_id, content, now],
    );

    let _ = write_daily_log_entry(
        &conn,
        today,
        "browser_action",
        &format!("Browser: {action_type} → {url}"),
        screenshot_desc,
    );
}
