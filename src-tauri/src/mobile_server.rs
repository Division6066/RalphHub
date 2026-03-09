/**
 * Mobile companion HTTP server — runs on desktop alongside Tauri.
 * Mobile devices connect via local LAN (discovered via QR code).
 *
 * Endpoints:
 *   GET  /api/ping                      Health check
 *   POST /api/sync/events               Receive event from mobile
 *   GET  /api/sync/events?since=&device Pull events for mobile
 *   POST /api/memory/write              Write to memory spine
 *   GET  /api/memory/read               Read memory spine
 *   GET  /api/tasks                     Get task list
 *   GET  /api/approvals                 Get pending approvals
 *   POST /api/approvals/:id/resolve     Resolve approval
 *   GET  /api/agents                    Get agent run list
 *   GET  /api/digest                    Get daily digest
 */
use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::memory::{
    query_raw_events, write_raw_event, get_kaizen_tasks, MemoryWriteRequest, RawEvent,
};

#[derive(Clone)]
pub struct MobileServerState {
    pub db_path: PathBuf,
    pub sync_events: Arc<Mutex<Vec<SyncEventRow>>>,
    pub approvals: Arc<Mutex<Vec<ApprovalRow>>>,
    pub tasks: Arc<Mutex<Vec<TaskRow>>>,
    pub agents: Arc<Mutex<Vec<AgentRow>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventRow {
    pub id: String,
    pub event_type: String,
    pub payload: Value,
    pub device_id: String,
    pub created_at: String,
    pub synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRow {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub action: String,
    pub context: String,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub priority: String,
    pub status: String,
    pub due_date: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRow {
    pub id: String,
    pub name: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub memory_ref: Option<String>,
    pub notion_task_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SyncQueryParams {
    pub since: Option<String>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ResolvePayload {
    pub decision: String,
    pub resolved_by: Option<String>,
}

pub async fn start_mobile_server(db_path: PathBuf, port: u16) -> anyhow::Result<()> {
    let state = MobileServerState {
        db_path,
        sync_events: Arc::new(Mutex::new(Vec::new())),
        approvals: Arc::new(Mutex::new(seed_approvals())),
        tasks: Arc::new(Mutex::new(seed_tasks())),
        agents: Arc::new(Mutex::new(seed_agents())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/ping", get(ping))
        .route("/api/sync/events", post(receive_sync_event))
        .route("/api/sync/events", get(pull_sync_events))
        .route("/api/memory/write", post(write_memory))
        .route("/api/memory/read", get(read_memory))
        .route("/api/tasks", get(get_tasks))
        .route("/api/approvals", get(get_approvals))
        .route("/api/approvals/{id}/resolve", post(resolve_approval))
        .route("/api/agents", get(get_agents))
        .route("/api/digest", get(get_digest))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Mobile sync server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ping() -> Json<Value> {
    Json(json!({ "ok": true, "version": "1.0.0", "name": "RalphHub Desktop Sync" }))
}

async fn receive_sync_event(
    State(state): State<MobileServerState>,
    Json(event): Json<SyncEventRow>,
) -> impl IntoResponse {
    let mut events = state.sync_events.lock().unwrap();
    events.push(event.clone());

    // Write to memory spine
    let req = MemoryWriteRequest {
        source: "mobile".to_string(),
        event_type: event.event_type.clone(),
        payload: event.payload,
        device_id: Some(event.device_id),
        session_id: None,
        kaizen_hint: None,
    };
    let _ = write_raw_event(&state.db_path, &req);

    Json(json!({ "ok": true, "id": event.id }))
}

async fn pull_sync_events(
    State(state): State<MobileServerState>,
    Query(params): Query<SyncQueryParams>,
) -> Json<Value> {
    let since = params.since.as_deref();
    let device_id = params.device_id.as_deref();
    let source = if device_id.is_some() { None } else { None };

    match query_raw_events(&state.db_path, since, source, 200) {
        Ok(events) => Json(json!(events)),
        Err(e) => Json(json!({ "error": e.to_string(), "events": [] })),
    }
}

async fn write_memory(
    State(state): State<MobileServerState>,
    Json(req): Json<MemoryWriteRequest>,
) -> impl IntoResponse {
    match write_raw_event(&state.db_path, &req) {
        Ok(event) => (StatusCode::CREATED, Json(json!({ "ok": true, "id": event.id }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn read_memory(
    State(state): State<MobileServerState>,
    Query(params): Query<SyncQueryParams>,
) -> Json<Value> {
    let since = params.since.as_deref();
    let raw = query_raw_events(&state.db_path, since, None, 100).unwrap_or_default();
    let kaizen = get_kaizen_tasks(&state.db_path, None).unwrap_or_default();
    Json(json!({
        "rawEvents": raw,
        "kaizenTasks": kaizen,
        "totalCount": raw.len(),
    }))
}

async fn get_tasks(State(state): State<MobileServerState>) -> Json<Value> {
    let tasks = state.tasks.lock().unwrap().clone();
    Json(json!(tasks))
}

async fn get_approvals(State(state): State<MobileServerState>) -> Json<Value> {
    let approvals = state.approvals.lock().unwrap().clone();
    Json(json!(approvals))
}

async fn resolve_approval(
    State(state): State<MobileServerState>,
    Path(id): Path<String>,
    Json(payload): Json<ResolvePayload>,
) -> impl IntoResponse {
    let mut approvals = state.approvals.lock().unwrap();
    let now = Utc::now().to_rfc3339();

    if let Some(approval) = approvals.iter_mut().find(|a| a.id == id) {
        approval.status = payload.decision.clone();
        approval.resolved_at = Some(now.clone());
        approval.resolved_by = payload.resolved_by.clone().or(Some("mobile".to_string()));

        // Write to memory
        let req = MemoryWriteRequest {
            source: "mobile".to_string(),
            event_type: "approval.resolve".to_string(),
            payload: serde_json::json!({
                "approvalId": id,
                "decision": payload.decision,
                "resolvedAt": now,
            }),
            device_id: Some("mobile".to_string()),
            session_id: None,
            kaizen_hint: if payload.decision == "rejected" {
                Some(format!("Follow up on rejected approval: {}", approval.action))
            } else {
                None
            },
        };
        let _ = write_raw_event(&state.db_path, &req);

        (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Approval not found" }))).into_response()
    }
}

async fn get_agents(State(state): State<MobileServerState>) -> Json<Value> {
    let agents = state.agents.lock().unwrap().clone();
    Json(json!(agents))
}

async fn get_digest(State(state): State<MobileServerState>) -> Json<Value> {
    let tasks = state.tasks.lock().unwrap();
    let agents = state.agents.lock().unwrap();

    let completed_tasks = tasks.iter().filter(|t| t.status == "done").count();
    let agents_done = agents.iter().filter(|a| a.status == "success").count();

    Json(json!({
        "date": Utc::now().format("%Y-%m-%d").to_string(),
        "tasksCompleted": completed_tasks,
        "habitsCompleted": 0,
        "agentsRun": agents_done,
        "capturesProcessed": 0,
        "highlights": [
            "Memory Spine is active and recording all events",
            format!("{} agents completed successfully", agents_done),
        ],
        "generatedAt": Utc::now().to_rfc3339(),
    }))
}

fn seed_tasks() -> Vec<TaskRow> {
    let now = Utc::now().to_rfc3339();
    vec![
        TaskRow { id: Uuid::new_v4().to_string(), title: "Review memory spine schema".to_string(), priority: "high".to_string(), status: "todo".to_string(), due_date: None, source: "desktop".to_string(), created_at: now.clone(), updated_at: now.clone() },
        TaskRow { id: Uuid::new_v4().to_string(), title: "Test Notion sync integration".to_string(), priority: "normal".to_string(), status: "in_progress".to_string(), due_date: None, source: "desktop".to_string(), created_at: now.clone(), updated_at: now.clone() },
    ]
}

fn seed_approvals() -> Vec<ApprovalRow> {
    let now = Utc::now().to_rfc3339();
    vec![
        ApprovalRow {
            id: Uuid::new_v4().to_string(),
            agent_id: "browser-agent".to_string(),
            agent_name: "Browser Agent".to_string(),
            action: "POST /api/notion/create-task".to_string(),
            context: "Agent wants to create: 'Review memory schema in Notion'".to_string(),
            status: "pending".to_string(),
            priority: "normal".to_string(),
            created_at: now.clone(),
            resolved_at: None,
            resolved_by: None,
        },
    ]
}

fn seed_agents() -> Vec<AgentRow> {
    let now = Utc::now().to_rfc3339();
    vec![
        AgentRow {
            id: Uuid::new_v4().to_string(),
            name: "Browser Agent #7".to_string(),
            status: "running".to_string(),
            started_at: Some(now.clone()),
            completed_at: None,
            memory_ref: Some("mem_007".to_string()),
            notion_task_id: None,
        },
        AgentRow {
            id: Uuid::new_v4().to_string(),
            name: "Ralph Overnight Loop".to_string(),
            status: "success".to_string(),
            started_at: Some(now.clone()),
            completed_at: Some(now.clone()),
            memory_ref: Some("mem_008".to_string()),
            notion_task_id: Some("notion_xyz".to_string()),
        },
    ]
}
