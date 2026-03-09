/**
 * Memory Spine — Rust backend.
 * Handles raw events, working/long-term memory, and Kaizen task generation.
 * Exposes Tauri commands AND is queried by the mobile sync HTTP server.
 */
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEvent {
    pub id: String,
    pub source: String,         // mobile | desktop | agent | browser_agent | api
    pub event_type: String,
    pub payload: serde_json::Value,
    pub device_id: String,
    pub timestamp: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingMemoryEntry {
    pub id: String,
    pub topic: String,
    pub content: String,
    pub confidence: f64,
    pub raw_event_ids: Vec<String>,
    pub created_at: String,
    pub expires_at: String,
    pub accessed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongTermMemoryEntry {
    pub id: String,
    pub category: String,
    pub summary: String,
    pub detail: String,
    pub source_event_ids: Vec<String>,
    pub notion_page_id: Option<String>,
    pub kaizen_task_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaizenTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_type: String,
    pub source_event_id: String,
    pub priority: String,
    pub status: String,
    pub notion_task_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryWriteRequest {
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub device_id: Option<String>,
    pub session_id: Option<String>,
    pub kaizen_hint: Option<String>,
}

pub fn init_memory_tables(db_path: &PathBuf) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS memory_raw (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            device_id TEXT NOT NULL DEFAULT 'desktop',
            timestamp TEXT NOT NULL,
            session_id TEXT
        );

        CREATE TABLE IF NOT EXISTS memory_working (
            id TEXT PRIMARY KEY,
            topic TEXT NOT NULL,
            content TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.8,
            raw_event_ids TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            accessed INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS memory_long_term (
            id TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            summary TEXT NOT NULL,
            detail TEXT NOT NULL,
            source_event_ids TEXT NOT NULL DEFAULT '[]',
            notion_page_id TEXT,
            kaizen_task_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kaizen_tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_event_id TEXT NOT NULL,
            priority TEXT NOT NULL DEFAULT 'normal',
            status TEXT NOT NULL DEFAULT 'backlog',
            notion_task_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_raw_timestamp ON memory_raw(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_raw_source ON memory_raw(source);
        CREATE INDEX IF NOT EXISTS idx_working_expires ON memory_working(expires_at);
        CREATE INDEX IF NOT EXISTS idx_kaizen_status ON kaizen_tasks(status);
    ")?;
    Ok(())
}

pub fn write_raw_event(db_path: &PathBuf, req: &MemoryWriteRequest) -> Result<RawEvent> {
    let conn = Connection::open(db_path)?;
    let now = Utc::now().to_rfc3339();
    let event = RawEvent {
        id: Uuid::new_v4().to_string(),
        source: req.source.clone(),
        event_type: req.event_type.clone(),
        payload: req.payload.clone(),
        device_id: req.device_id.clone().unwrap_or_else(|| "desktop".to_string()),
        timestamp: now.clone(),
        session_id: req.session_id.clone(),
    };

    conn.execute(
        "INSERT INTO memory_raw (id, source, event_type, payload, device_id, timestamp, session_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            event.id,
            event.source,
            event.event_type,
            serde_json::to_string(&event.payload)?,
            event.device_id,
            event.timestamp,
            event.session_id,
        ],
    )?;

    // Auto-promote to working memory
    let expires_at = (Utc::now() + chrono::Duration::days(7)).to_rfc3339();
    conn.execute(
        "INSERT INTO memory_working (id, topic, content, confidence, raw_event_ids, created_at, expires_at, accessed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            Uuid::new_v4().to_string(),
            event.event_type,
            serde_json::to_string(&event.payload)?,
            0.8f64,
            serde_json::to_string(&[&event.id])?,
            now.clone(),
            expires_at,
            0i64,
        ],
    )?;

    // Auto-create Kaizen if hinted
    if let Some(hint) = &req.kaizen_hint {
        let desc = format!(
            "Auto-created from {}: {}",
            req.event_type,
            serde_json::to_string(&req.payload)
                .unwrap_or_default()
                .chars()
                .take(200)
                .collect::<String>()
        );
        conn.execute(
            "INSERT INTO kaizen_tasks (id, title, description, source_type, source_event_id, priority, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                Uuid::new_v4().to_string(),
                hint,
                desc,
                "memory_gap",
                event.id,
                "normal",
                "backlog",
                now.clone(),
                now,
            ],
        )?;
    }

    Ok(event)
}

pub fn query_raw_events(
    db_path: &PathBuf,
    since: Option<&str>,
    source: Option<&str>,
    limit: i64,
) -> Result<Vec<RawEvent>> {
    let conn = Connection::open(db_path)?;
    let mut events = Vec::new();

    let since_filter = since.unwrap_or("1970-01-01T00:00:00Z");
    let source_filter = source.unwrap_or("%");

    let mut stmt = conn.prepare(
        "SELECT id, source, event_type, payload, device_id, timestamp, session_id
         FROM memory_raw
         WHERE timestamp >= ?1 AND source LIKE ?2
         ORDER BY timestamp DESC
         LIMIT ?3",
    )?;

    let rows = stmt.query_map(params![since_filter, source_filter, limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    })?;

    for row in rows {
        let (id, source, event_type, payload_str, device_id, timestamp, session_id) = row?;
        events.push(RawEvent {
            id,
            source,
            event_type,
            payload: serde_json::from_str(&payload_str).unwrap_or(serde_json::Value::Null),
            device_id,
            timestamp,
            session_id,
        });
    }

    Ok(events)
}

pub fn get_kaizen_tasks(db_path: &PathBuf, status: Option<&str>) -> Result<Vec<KaizenTask>> {
    let conn = Connection::open(db_path)?;
    let mut tasks = Vec::new();

    let status_filter = status.unwrap_or("%");
    let mut stmt = conn.prepare(
        "SELECT id, title, description, source_type, source_event_id, priority, status, notion_task_id, created_at, updated_at
         FROM kaizen_tasks
         WHERE status LIKE ?1
         ORDER BY created_at DESC
         LIMIT 100",
    )?;

    let rows = stmt.query_map(params![status_filter], |row| {
        Ok(KaizenTask {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            source_type: row.get(3)?,
            source_event_id: row.get(4)?,
            priority: row.get(5)?,
            status: row.get(6)?,
            notion_task_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}
