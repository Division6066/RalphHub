use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use crate::models::{AmitosDashboard, BunStatus, DashboardSnapshot, InboxItem, KaizenTask, MemoryStats, WorkspacePaths};
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
            database_path: app_data_dir.join("ralphhub.db"),
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

    pub fn amitos_dashboard(&self) -> Result<AmitosDashboard> {
        let connection = Connection::open(&self.paths.database_path)?;
        let managed_project_count = query_count(&connection, CountTable::ManagedProjects)?;
        let workflow_run_count = query_count(&connection, CountTable::WorkflowRuns)?;
        let memory_stats = query_memory_stats(&connection)?;
        let today_tasks = query_today_tasks(&connection)?;
        let inbox_items = query_inbox_unprocessed(&connection, 5)?;
        let approval_queue = query_approval_queue(&connection)?;
        let recent_log = query_recent_daily_log(&connection, 5)?;

        Ok(AmitosDashboard {
            memory_stats,
            today_tasks,
            inbox_items,
            running_agents: vec![],
            approval_queue,
            recent_log,
            managed_project_count,
            workflow_run_count,
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

        -- ── AmitOS Memory Spine ──────────────────────────────────────────
        CREATE TABLE IF NOT EXISTS raw_events (
            id TEXT PRIMARY KEY,
            source_type TEXT NOT NULL,
            content TEXT NOT NULL,
            metadata TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS working_memory (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            expires_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS long_term_memory (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            source_ids TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS structured_summaries (
            id TEXT PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT NOT NULL,
            evidence TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        );

        -- ── AmitOS Kaizen Tasks ──────────────────────────────────────────
        CREATE TABLE IF NOT EXISTS kaizen_projects (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            domain TEXT NOT NULL DEFAULT 'work',
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kaizen_tasks (
            id TEXT PRIMARY KEY,
            project_id TEXT,
            parent_task_id TEXT,
            title TEXT NOT NULL,
            domain TEXT NOT NULL DEFAULT 'work',
            energy TEXT NOT NULL DEFAULT 'medium',
            estimate_minutes INTEGER NOT NULL DEFAULT 30,
            status TEXT NOT NULL DEFAULT 'inbox',
            do_date TEXT,
            deadline TEXT,
            agent_mode TEXT NOT NULL DEFAULT 'manual',
            approval_required INTEGER NOT NULL DEFAULT 0,
            evidence TEXT NOT NULL DEFAULT '{}',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- ── AmitOS Inbox + Daily Log ──────────────────────────────────────
        CREATE TABLE IF NOT EXISTS inbox_items (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'text',
            processed INTEGER NOT NULL DEFAULT 0,
            source TEXT NOT NULL DEFAULT 'manual',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS daily_log_entries (
            id TEXT PRIMARY KEY,
            log_date TEXT NOT NULL,
            entry_type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_daily_log_date ON daily_log_entries(log_date);
        CREATE INDEX IF NOT EXISTS idx_kaizen_tasks_do_date ON kaizen_tasks(do_date);
        CREATE INDEX IF NOT EXISTS idx_kaizen_tasks_status ON kaizen_tasks(status);
        CREATE INDEX IF NOT EXISTS idx_raw_events_created ON raw_events(created_at);
        CREATE INDEX IF NOT EXISTS idx_inbox_processed ON inbox_items(processed);
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
    let query = match table {
        CountTable::ManagedProjects => "SELECT COUNT(*) FROM managed_projects",
        CountTable::WorkflowRuns => "SELECT COUNT(*) FROM workflow_runs",
        CountTable::OvernightLoops => "SELECT COUNT(*) FROM overnight_loops",
    };
    let mut statement = connection.prepare(query)?;
    let count = statement.query_row([], |row| row.get(0))?;
    Ok(count)
}

fn count_table_simple(connection: &Connection, table: &str) -> Result<i64> {
    let q = format!("SELECT COUNT(*) FROM {table}");
    let mut stmt = connection.prepare(&q)?;
    let count = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub fn query_memory_stats(connection: &Connection) -> Result<MemoryStats> {
    Ok(MemoryStats {
        raw_events_count: count_table_simple(connection, "raw_events")?,
        working_memory_count: count_table_simple(connection, "working_memory")?,
        long_term_count: count_table_simple(connection, "long_term_memory")?,
        summaries_count: count_table_simple(connection, "structured_summaries")?,
        inbox_count: count_table_simple(connection, "inbox_items")?,
        daily_log_count: count_table_simple(connection, "daily_log_entries")?,
    })
}

pub fn query_today_tasks(connection: &Connection) -> Result<Vec<KaizenTask>> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut stmt = connection.prepare(
        "SELECT id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
                status, do_date, deadline, agent_mode, approval_required, evidence, notes,
                created_at, updated_at
         FROM kaizen_tasks
         WHERE do_date = ?1 AND status NOT IN ('done','cancelled')
         ORDER BY domain, energy DESC, estimate_minutes ASC
         LIMIT 50",
    )?;
    let rows = stmt.query_map(params![today], map_kaizen_task)?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn query_inbox_unprocessed(connection: &Connection, limit: i64) -> Result<Vec<InboxItem>> {
    let mut stmt = connection.prepare(
        "SELECT id, content, content_type, processed, source, created_at
         FROM inbox_items WHERE processed = 0 ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(InboxItem {
            id: row.get(0)?,
            content: row.get(1)?,
            content_type: row.get(2)?,
            processed: row.get::<_, i64>(3)? != 0,
            source: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

pub fn query_approval_queue(connection: &Connection) -> Result<Vec<KaizenTask>> {
    let mut stmt = connection.prepare(
        "SELECT id, project_id, parent_task_id, title, domain, energy, estimate_minutes,
                status, do_date, deadline, agent_mode, approval_required, evidence, notes,
                created_at, updated_at
         FROM kaizen_tasks
         WHERE approval_required = 1 AND status NOT IN ('done','cancelled')
         ORDER BY created_at DESC LIMIT 20",
    )?;
    let rows = stmt.query_map([], map_kaizen_task)?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn query_recent_daily_log(connection: &Connection, limit: i64) -> Result<Vec<crate::models::DailyLogEntry>> {
    let mut stmt = connection.prepare(
        "SELECT id, log_date, entry_type, title, content, created_at
         FROM daily_log_entries ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(crate::models::DailyLogEntry {
            id: row.get(0)?,
            log_date: row.get(1)?,
            entry_type: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

pub fn map_kaizen_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<KaizenTask> {
    let subtask_count = 0i64;
    let evidence_str: String = row.get(12)?;
    let evidence: serde_json::Value = serde_json::from_str(&evidence_str).unwrap_or(serde_json::Value::Object(Default::default()));
    Ok(KaizenTask {
        id: row.get(0)?,
        project_id: row.get(1)?,
        parent_task_id: row.get(2)?,
        title: row.get(3)?,
        domain: row.get(4)?,
        energy: row.get(5)?,
        estimate_minutes: row.get(6)?,
        status: row.get(7)?,
        do_date: row.get(8)?,
        deadline: row.get(9)?,
        agent_mode: row.get(10)?,
        approval_required: row.get::<_, i64>(11)? != 0,
        evidence,
        notes: row.get(13)?,
        subtask_count,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}