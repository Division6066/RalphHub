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
    pub paths: AmitPaths,
}

#[derive(Debug, Clone)]
pub struct AmitPaths {
    pub app_data_dir: PathBuf,
    pub database_path: PathBuf,
    pub repos_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub workflows_dir: PathBuf,
    pub notebooks_dir: PathBuf,
    pub state_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub kaizen_dir: PathBuf,
}

impl AppState {
    pub fn init(app: &AppHandle) -> Result<Self> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data directory")?;

        let paths = AmitPaths {
            database_path: app_data_dir.join("amitos.db"),
            repos_dir: app_data_dir.join("repos"),
            logs_dir: app_data_dir.join("logs"),
            workflows_dir: app_data_dir.join("workflows"),
            notebooks_dir: app_data_dir.join("notebooks"),
            state_dir: app_data_dir.join("state"),
            memory_dir: app_data_dir.join("memory"),
            kaizen_dir: app_data_dir.join("kaizen"),
            app_data_dir,
        };

        paths.ensure_directories()?;
        initialize_database(&paths.database_path)?;

        Ok(Self { paths })
    }

    pub fn snapshot(&self) -> Result<DashboardSnapshot> {
        let connection = Connection::open(&self.paths.database_path)?;
        let managed_project_count = query_count(&connection, "managed_projects")?;
        let workflow_run_count = query_count(&connection, "workflow_runs")?;
        let overnight_loop_count = query_count(&connection, "overnight_loops")?;
        let memory_entry_count = query_count(&connection, "memory_entries")?;
        let kaizen_task_count = query_count(&connection, "kaizen_tasks")?;
        let today_task_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM kaizen_tasks WHERE is_today = 1 AND status != 'done'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let api_key_count = query_count(&connection, "api_keys")?;

        Ok(DashboardSnapshot {
            bun: detect_bun_status(),
            paths: self.paths.as_payload(),
            tools: all_tools(),
            managed_project_count,
            workflow_run_count,
            overnight_loop_count,
            memory_entry_count,
            kaizen_task_count,
            today_task_count,
            api_key_count,
        })
    }
}

impl AmitPaths {
    pub fn ensure_directories(&self) -> Result<()> {
        for dir in [
            &self.app_data_dir,
            &self.repos_dir,
            &self.logs_dir,
            &self.workflows_dir,
            &self.notebooks_dir,
            &self.state_dir,
            &self.memory_dir,
            &self.kaizen_dir,
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
            memory_dir: self.memory_dir.display().to_string(),
            kaizen_dir: self.kaizen_dir.display().to_string(),
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
            tool_ids TEXT NOT NULL DEFAULT '[]',
            model_name TEXT NOT NULL DEFAULT '',
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

        CREATE TABLE IF NOT EXISTS memory_entries (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            domain TEXT NOT NULL DEFAULT 'general',
            source TEXT NOT NULL DEFAULT 'manual',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kaizen_tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            domain TEXT NOT NULL DEFAULT 'general',
            status TEXT NOT NULL DEFAULT 'todo',
            is_today INTEGER NOT NULL DEFAULT 0,
            is_minimum_version INTEGER NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 3,
            parent_id TEXT,
            subtasks TEXT NOT NULL DEFAULT '[]',
            energy TEXT NOT NULL DEFAULT 'medium',
            estimated_minutes INTEGER,
            tags TEXT NOT NULL DEFAULT '[]',
            due_date TEXT,
            source TEXT NOT NULL DEFAULT '',
            provider_id TEXT NOT NULL DEFAULT '',
            usage_log_id TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kaizen_domains (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#6366f1',
            icon TEXT NOT NULL DEFAULT '🎯',
            description TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS api_keys (
            provider_id TEXT PRIMARY KEY,
            key_name TEXT NOT NULL,
            masked_value TEXT NOT NULL DEFAULT '',
            saved_at TEXT NOT NULL
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

    // Seed default Kaizen domains
    let domains = [
        ("health", "Health & Fitness", "#10b981", "🏃"),
        ("work", "Work & Career", "#6366f1", "💼"),
        ("learning", "Learning & Growth", "#f59e0b", "📚"),
        ("creative", "Creative Projects", "#ec4899", "🎨"),
        ("relationships", "Relationships", "#ef4444", "❤️"),
        ("finance", "Finance", "#14b8a6", "💰"),
        ("home", "Home & Life", "#8b5cf6", "🏠"),
        ("general", "General", "#64748b", "⭐"),
    ];

    for (id, name, color, icon) in &domains {
        connection.execute(
            "INSERT OR IGNORE INTO kaizen_domains (id, name, color, icon, description, created_at) VALUES (?1, ?2, ?3, ?4, '', ?5)",
            params![id, name, color, icon, chrono::Utc::now().to_rfc3339()],
        )?;
    }

    connection.execute(
        "INSERT INTO milestones (scope, summary, git_ref, created_at)
         SELECT ?1, ?2, ?3, ?4
         WHERE NOT EXISTS (
             SELECT 1 FROM milestones WHERE scope = ?1 AND summary = ?2
         )",
        params![
            "bootstrap",
            "Initialized AmitOS state database",
            Option::<String>::None,
            chrono::Utc::now().to_rfc3339()
        ],
    )?;

    Ok(())
}

fn query_count(connection: &Connection, table: &str) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let count = connection.query_row(&sql, [], |row| row.get(0))?;
    Ok(count)
}
