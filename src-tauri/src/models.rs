use serde::{Deserialize, Serialize};

// ─── Provider Registry ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub category: String,
    pub base_url: String,
    pub auth_type: String,
    pub api_key_env: String,
    pub models: Vec<String>,
    pub enabled: bool,
    pub is_local: bool,
    pub description: String,
    pub docs_url: String,
    pub logo_emoji: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProviderRequest {
    pub name: String,
    pub category: String,
    pub base_url: String,
    pub auth_type: String,
    pub api_key_env: String,
    pub models: Vec<String>,
    pub is_local: bool,
    pub description: String,
    pub docs_url: String,
    pub logo_emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderRequest {
    pub id: String,
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub models: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

// ─── API Usage Logging (Memory Spine) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsageLog {
    pub id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
    pub output_summary: String,
    pub tool_id: String,
    pub workflow_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogApiUsageRequest {
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
    pub output_summary: String,
    pub tool_id: String,
    pub workflow_id: String,
}

// ─── Kaizen Tasks ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub source: String,
    pub provider_id: String,
    pub usage_log_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKaizenTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub source: String,
    pub provider_id: String,
    pub usage_log_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySpineEntry {
    pub id: String,
    pub entry_type: String,
    pub content: String,
    pub tags: Vec<String>,
    pub provider_id: String,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySpineStats {
    pub total_entries: i64,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub providers_used: Vec<String>,
    pub recent_logs: Vec<ApiUsageLog>,
    pub active_tasks: Vec<KaizenTask>,
}

// ─── Tool Manifest ───────────────────────────────────────────────────────────

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
