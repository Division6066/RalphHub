use tauri::State;

use crate::models::{CommandResponse, MobileSyncStatus};
use crate::state::AppState;

/// Returns QR code data URL and local IP for mobile sync pairing.
/// In production, this would start a local HTTP server. Here we return
/// the configuration data so the frontend can display a QR code.
#[tauri::command]
pub fn get_mobile_sync_status(state: State<'_, AppState>) -> Result<MobileSyncStatus, String> {
    let local_ip = get_local_ip();
    let port: u16 = 7432;
    let sync_url = format!("http://{}:{}/api/sync", local_ip, port);

    Ok(MobileSyncStatus {
        enabled: false,
        port,
        qr_data: sync_url.clone(),
        local_ip,
        connected_devices: 0,
        last_sync: None,
    })
}

#[tauri::command]
pub fn enable_mobile_sync(
    _state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    Ok(CommandResponse {
        ok: true,
        message: "Mobile sync server would start on port 7432. Full implementation requires the HTTP server feature.".to_string(),
    })
}

fn get_local_ip() -> String {
    // Try to get a non-loopback local IP
    use std::net::UdpSocket;
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}
