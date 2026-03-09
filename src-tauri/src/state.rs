use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use crate::models::{BunStatus, DashboardSnapshot, WorkspacePaths};
use crate::tool_registry::all_tools;

#[derive(Debug, Clone)]
pub struct AppState {
    pub paths: RalphPaths,
}

#[derive(Debug, Clone)]
pub struct RalphPaths {
    pub app_data_dir: PathBuf,
    pub database_path: PathBuf,
    pub repos_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub workflows_dir: PathBuf,
    pub notebooks_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl AppState {
    pub fn init(app: &AppHandle) -> Result<Self> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data directory")?;

        let paths = RalphPaths {
            database_path: app_data_dir.join("amitos.db"),
            repos_dir: app_data_dir.join("repos"),
            logs_dir: app_data_dir.join("logs"),
            workflows_dir: app_data_dir.join("workflows"),
            notebooks_dir: app_data_dir.join("notebooks"),
            state_dir: app_data_dir.join("state"),
            app_data_dir,
        };

        paths.ensure_directories()?;
        initialize_database(&paths.database_path)?;

        Ok(Self { paths })
    }

    pub fn snapshot(&self) -> Result<DashboardSnapshot> {
        let connection = Connection::open(&self.paths.database_path)?;
        let managed_project_count = query_count(&connection, CountTable::ManagedProjects)?;
        let workflow_run_count = query_count(&connection, CountTable::WorkflowRuns)?;
        let overnight_loop_count = query_count(&connection, CountTable::OvernightLoops)?;

        Ok(DashboardSnapshot {
            bun: detect_bun_status(),
            paths: self.paths.as_payload(),
            tools: all_tools(),
            managed_project_count,
            workflow_run_count,
            overnight_loop_count,
        })
    }
}

impl RalphPaths {
    pub fn ensure_directories(&self) -> Result<()> {
        for dir in [
            &self.app_data_dir,
            &self.repos_dir,
            &self.logs_dir,
            &self.workflows_dir,
            &self.notebooks_dir,
            &self.state_dir,
        ] {
            fs::create_dir_all(dir)
                .with_context(|| format!("failed to create directory {}", dir.display()))?;
        }

        Ok(())
    }

    pub fn as_payload(&self) -> WorkspacePaths {
        WorkspacePaths {
            app_data_dir: self.app_data_dir.display().to_string(),
            database_path: self.database_path.display().to_string(),
            repos_dir: self.repos_dir.display().to_string(),
            logs_dir: self.logs_dir.display().to_string(),
            workflows_dir: self.workflows_dir.display().to_string(),
            notebooks_dir: self.notebooks_dir.display().to_string(),
            state_dir: self.state_dir.display().to_string(),
        }
    }
}

pub fn detect_bun_status() -> BunStatus {
    let output = Command::new("bun").arg("--version").output();

    match output {
        Ok(output) if output.status.success() => BunStatus {
            installed: true,
            version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
            installer_hint: bun_installer_hint(),
        },
        _ => BunStatus {
            installed: false,
            version: None,
            installer_hint: bun_installer_hint(),
        },
    }
}

pub fn bun_installer_hint() -> String {
    if cfg!(target_os = "windows") {
        "powershell -c \"irm bun.sh/install.ps1|iex\"".to_string()
    } else {
        "curl -fsSL https://bun.sh/install | bash".to_string()
    }
}

fn initialize_database(path: &Path) -> Result<()> {
    let connection = Connection::open(path)?;

    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS managed_projects (
            id TEXT PRIMARY KEY,
            slug TEXT NOT NULL,
            source_url TEXT NOT NULL,
            workspace_path TEXT NOT NULL,
            branch TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS workflow_runs (
            id TEXT PRIMARY KEY,
            workflow_name TEXT NOT NULL,
            status TEXT NOT NULL,
            config_path TEXT NOT NULL,
            state_path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS overnight_loops (
            id TEXT PRIMARY KEY,
            model_name TEXT NOT NULL,
            status TEXT NOT NULL,
            workspace_path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS milestones (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scope TEXT NOT NULL,
            summary TEXT NOT NULL,
            git_ref TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            base_url TEXT NOT NULL,
            auth_type TEXT NOT NULL DEFAULT 'bearer',
            api_key_env TEXT NOT NULL,
            models TEXT NOT NULL DEFAULT '[]',
            enabled INTEGER NOT NULL DEFAULT 1,
            is_local INTEGER NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT '',
            docs_url TEXT NOT NULL DEFAULT '',
            logo_emoji TEXT NOT NULL DEFAULT '🔌',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS api_usage_logs (
            id TEXT PRIMARY KEY,
            provider_id TEXT NOT NULL,
            provider_name TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL,
            tokens_in INTEGER NOT NULL DEFAULT 0,
            tokens_out INTEGER NOT NULL DEFAULT 0,
            cost_usd REAL NOT NULL DEFAULT 0.0,
            output_summary TEXT NOT NULL DEFAULT '',
            tool_id TEXT NOT NULL DEFAULT '',
            workflow_id TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kaizen_tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'todo',
            priority TEXT NOT NULL DEFAULT 'normal',
            source TEXT NOT NULL DEFAULT '',
            provider_id TEXT NOT NULL DEFAULT '',
            usage_log_id TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS memory_spine (
            id TEXT PRIMARY KEY,
            entry_type TEXT NOT NULL DEFAULT 'note',
            content TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            provider_id TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );
        ",
    )?;

    connection.execute(
        "INSERT INTO milestones (scope, summary, git_ref, created_at)
         SELECT ?1, ?2, ?3, ?4
         WHERE NOT EXISTS (
             SELECT 1 FROM milestones WHERE scope = ?1 AND summary = ?2
         )",
        params![
            "bootstrap",
            "Initialized RalphHub state database",
            Option::<String>::None,
            chrono::Utc::now().to_rfc3339()
        ],
    )?;

    Ok(())
}

enum CountTable {
    ManagedProjects,
    WorkflowRuns,
    OvernightLoops,
}

fn query_count(connection: &Connection, table: CountTable) -> Result<i64> {
    let sql = match table {
        CountTable::ManagedProjects => "SELECT COUNT(*) FROM managed_projects",
        CountTable::WorkflowRuns => "SELECT COUNT(*) FROM workflow_runs",
        CountTable::OvernightLoops => "SELECT COUNT(*) FROM overnight_loops",
    };
    let count = connection.query_row(sql, [], |row| row.get(0))?;
    Ok(count)
}