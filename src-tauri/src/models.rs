use serde::{Deserialize, Serialize};

// ─── AmitOS Memory Spine ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEvent {
    pub id: String,
    pub source_type: String, // text | url | file | browser_agent | notion | workflow | task
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingMemoryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongTermMemoryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub source_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredSummary {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub summary: String,
    pub evidence: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestRequest {
    pub source_type: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub auto_summarize: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    pub raw_events_count: i64,
    pub working_memory_count: i64,
    pub long_term_count: i64,
    pub summaries_count: i64,
    pub inbox_count: i64,
    pub daily_log_count: i64,
}

// ─── AmitOS Kaizen Tasks ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenProject {
    pub id: String,
    pub title: String,
    pub domain: String,
    pub description: String,
    pub status: String, // active | paused | complete | archived
    pub task_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenTask {
    pub id: String,
    pub project_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub title: String,          // verb-first: "Write X", "Build Y"
    pub domain: String,         // work | health | learning | personal | system
    pub energy: String,         // low | medium | high
    pub estimate_minutes: i64,
    pub status: String,         // inbox | todo | doing | blocked | done | cancelled
    pub do_date: Option<String>,
    pub deadline: Option<String>,
    pub agent_mode: String,     // manual | auto | approval_required
    pub approval_required: bool,
    pub evidence: serde_json::Value,
    pub notes: String,
    pub subtask_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKaizenTaskRequest {
    pub project_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub title: String,
    pub domain: String,
    pub energy: String,
    pub estimate_minutes: i64,
    pub do_date: Option<String>,
    pub deadline: Option<String>,
    pub agent_mode: Option<String>,
    pub approval_required: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayBoardGroup {
    pub domain: String,
    pub tasks: Vec<KaizenTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanColumn {
    pub status: String,
    pub label: String,
    pub tasks: Vec<KaizenTask>,
}

// ─── AmitOS Inbox + Daily Log ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxItem {
    pub id: String,
    pub content: String,
    pub content_type: String, // text | url | screenshot | voice | file
    pub processed: bool,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddInboxRequest {
    pub content: String,
    pub content_type: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyLogEntry {
    pub id: String,
    pub log_date: String,
    pub entry_type: String, // agent_run | task_complete | browser_action | notion_sync | morning_digest | nightly_wrap | memory_write
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDailyLogRequest {
    pub log_date: Option<String>,
    pub entry_type: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MorningDigest {
    pub date: String,
    pub today_tasks: Vec<KaizenTask>,
    pub inbox_count: i64,
    pub memory_updates: i64,
    pub yesterday_summary: String,
}

// ─── AmitOS Notion Sync ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotionSyncRequest {
    pub database_id: String,
    pub api_key: String,
    pub direction: String, // push | pull | both
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotionSyncResult {
    pub pushed: i64,
    pub pulled: i64,
    pub errors: Vec<String>,
    pub synced_at: String,
}

// ─── AmitOS Dashboard ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmitosDashboard {
    pub memory_stats: MemoryStats,
    pub today_tasks: Vec<KaizenTask>,
    pub inbox_items: Vec<InboxItem>,
    pub running_agents: Vec<String>,
    pub approval_queue: Vec<KaizenTask>,
    pub recent_log: Vec<DailyLogEntry>,
    pub managed_project_count: i64,
    pub workflow_run_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub description: String,
    pub launch_command: String,
    pub status: String,
    pub open_in_code: bool,
    pub needs_sandbox: bool,
    pub required_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePaths {
    pub app_data_dir: String,
    pub database_path: String,
    pub repos_dir: String,
    pub logs_dir: String,
    pub workflows_dir: String,
    pub notebooks_dir: String,
    pub state_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BunStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub installer_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSnapshot {
    pub bun: BunStatus,
    pub paths: WorkspacePaths,
    pub tools: Vec<ToolManifest>,
    pub managed_project_count: i64,
    pub workflow_run_count: i64,
    pub overnight_loop_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecureStoreConfig {
    pub vault_path: String,
    pub client_name: String,
    pub vault_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvInjectionRequest {
    pub workspace_path: String,
    pub entries: Vec<EnvEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployResult {
    pub workspace_path: String,
    pub normalized_url: String,
    pub branch: String,
    pub message: String,
    pub state_path: String,
    pub env_path: String,
    pub notebook_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedProject {
    pub id: String,
    pub slug: String,
    pub source_url: String,
    pub workspace_path: String,
    pub branch: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRequest {
    pub name: String,
    pub tool_ids: Vec<String>,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_name: String,
    pub tool_ids: Vec<String>,
    pub model_name: String,
    pub status: String,
    pub config_path: String,
    pub state_path: String,
    pub created_at: String,
    pub updated_at: String,
}
