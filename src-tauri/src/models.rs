use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub description: String,
    pub launch_command: String,
    pub install_method: String,
    pub status: String,
    pub open_in_code: bool,
    pub needs_sandbox: bool,
    pub required_keys: Vec<String>,
    pub categories: Vec<String>,
    pub supports_ollama: bool,
    pub supports_voice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    pub name: String,
    pub display_name: String,
    pub size_hint: String,
    pub status: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub running: bool,
    pub endpoint: String,
    pub models: Vec<OllamaModel>,
    pub installer_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConfig {
    pub enabled: bool,
    pub stt_provider: String,
    pub tts_provider: String,
    pub stt_model: String,
    pub tts_voice: String,
    pub whisper_installed: bool,
    pub piper_installed: bool,
    pub offline_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub id: String,
    pub tool_id: String,
    pub entry_type: String,
    pub content: String,
    pub created_at: String,
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub tool_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSession {
    pub id: String,
    pub url: String,
    pub status: String,
    pub backend: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConnectResult {
    pub tool_id: String,
    pub ollama_ok: bool,
    pub voice_ok: bool,
    pub mcp_ok: bool,
    pub cursor_ok: bool,
    pub notes: Vec<String>,
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
