use serde::{Deserialize, Serialize};

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
