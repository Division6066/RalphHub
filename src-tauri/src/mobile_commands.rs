/**
 * Tauri commands for memory spine and mobile server management.
 */
use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::net::TcpListener;

use crate::state::AppState;
use crate::memory::{
    write_raw_event, query_raw_events, get_kaizen_tasks,
    MemoryWriteRequest, RawEvent, KaizenTask,
};

/// Shared mobile server port state
pub struct MobileServerPort(pub Arc<Mutex<Option<u16>>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileServerInfo {
    pub running: bool,
    pub port: Option<u16>,
    pub local_ip: Option<String>,
    pub qr_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    pub raw_event_count: usize,
    pub kaizen_task_count: usize,
}

#[tauri::command]
pub fn write_to_memory(
    state: State<'_, AppState>,
    source: String,
    event_type: String,
    payload: serde_json::Value,
    device_id: Option<String>,
    session_id: Option<String>,
    kaizen_hint: Option<String>,
) -> Result<String, String> {
    let req = MemoryWriteRequest {
        source,
        event_type,
        payload,
        device_id,
        session_id,
        kaizen_hint,
    };
    write_raw_event(&state.paths.database_path, &req)
        .map(|e| e.id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_memory(
    state: State<'_, AppState>,
    since: Option<String>,
    source: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<RawEvent>, String> {
    query_raw_events(
        &state.paths.database_path,
        since.as_deref(),
        source.as_deref(),
        limit.unwrap_or(100),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_kaizen_tasks(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<KaizenTask>, String> {
    get_kaizen_tasks(&state.paths.database_path, status.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_mobile_server_info(
    port_state: State<'_, MobileServerPort>,
) -> MobileServerInfo {
    let port = port_state.0.lock().unwrap().clone();
    let local_ip = get_local_ip();

    let qr_url = if let (Some(p), Some(ref ip)) = (port, &local_ip) {
        Some(format!("http://{}:{}", ip, p))
    } else {
        None
    };

    MobileServerInfo {
        running: port.is_some(),
        port,
        local_ip,
        qr_url,
    }
}

fn get_local_ip() -> Option<String> {
    // Connect to a remote address to discover local IP (no packet sent)
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

fn find_free_port(start: u16) -> u16 {
    for port in start..start + 100 {
        if TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok() {
            return port;
        }
    }
    start
}
