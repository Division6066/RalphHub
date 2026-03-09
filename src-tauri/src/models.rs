use serde::{Deserialize, Serialize};

// ─── Tool / Provider ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub description: String,
    pub launch_command: String,
    pub status: String,
    pub category: String,
    pub open_in_code: bool,
    pub needs_sandbox: bool,
    pub required_keys: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProvider {
    pub id: String,
    pub name: String,
    pub category: String,
    pub key_name: String,
    pub url: String,
    pub description: String,
    pub color: String,
}

// ─── Paths / Status ───────────────────────────────────────────────────────────

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
    pub memory_dir: String,
    pub kaizen_dir: String,
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
    pub memory_entry_count: i64,
    pub kaizen_task_count: i64,
    pub today_task_count: i64,
    pub api_key_count: i64,
}

// ─── API Keys ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyEntry {
    pub provider_id: String,
    pub key_name: String,
    pub masked_value: String,
    pub saved_at: String,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

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

// ─── Deploy ───────────────────────────────────────────────────────────────────

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

// ─── Workflow ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRequest {
    pub name: String,
    pub tool_ids: Vec<String>,
    pub model_name: String,
    pub description: Option<String>,
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

// ─── Kaizen / Today Board ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub domain: String,
    pub status: String,
    pub is_today: bool,
    pub is_minimum_version: bool,
    pub priority: i32,
    pub parent_id: Option<String>,
    pub subtasks: Vec<String>,
    pub energy: String,
    pub estimated_minutes: Option<i32>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKaizenTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub domain: String,
    pub is_today: bool,
    pub is_minimum_version: bool,
    pub priority: Option<i32>,
    pub parent_id: Option<String>,
    pub energy: Option<String>,
    pub estimated_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKaizenTaskRequest {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub domain: Option<String>,
    pub status: Option<String>,
    pub is_today: Option<bool>,
    pub is_minimum_version: Option<bool>,
    pub priority: Option<i32>,
    pub energy: Option<String>,
    pub estimated_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenDomain {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub description: String,
    pub task_count: i64,
    pub today_count: i64,
}

// ─── Memory ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub domain: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemoryRequest {
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub domain: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchRequest {
    pub query: String,
    pub domain: Option<String>,
    pub limit: Option<i32>,
}

// ─── Voice ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCommand {
    pub transcript: String,
    pub confidence: f32,
    pub action: String,
    pub params: serde_json::Value,
}

// ─── Mobile Sync ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileSyncStatus {
    pub enabled: bool,
    pub port: u16,
    pub qr_data: String,
    pub local_ip: String,
    pub connected_devices: i32,
    pub last_sync: Option<String>,
}
