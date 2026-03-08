use std::process::Command;
use std::path::PathBuf;

use tauri::State;

use crate::{
    models::{CommandResponse, VoiceConfig},
    state::AppState,
};

#[tauri::command]
pub fn get_voice_config(_state: State<'_, AppState>) -> VoiceConfig {
    VoiceConfig {
        enabled: false,
        stt_provider: detect_stt_provider(),
        tts_provider: detect_tts_provider(),
        stt_model: "base".to_string(),
        tts_voice: "en_US-lessac-medium".to_string(),
        whisper_installed: detect_whisper_installed(),
        piper_installed: detect_piper_installed(),
        offline_fallback: false,
    }
}

#[tauri::command]
pub fn ensure_voice(stt_provider: String, tts_provider: String) -> Result<CommandResponse, String> {
    let mut msgs = Vec::new();
    let mut ok = true;

    match stt_provider.as_str() {
        "faster-whisper" | "whisper" => {
            if !detect_whisper_installed() {
                match install_faster_whisper() {
                    Ok(msg) => msgs.push(msg),
                    Err(e) => {
                        msgs.push(format!("STT install failed: {e}"));
                        ok = false;
                    }
                }
            } else {
                msgs.push("Whisper already installed.".to_string());
            }
        }
        _ => msgs.push(format!("Unknown STT provider: {stt_provider}")),
    }

    match tts_provider.as_str() {
        "piper" => {
            if !detect_piper_installed() {
                match install_piper() {
                    Ok(msg) => msgs.push(msg),
                    Err(e) => {
                        msgs.push(format!("TTS install failed: {e}"));
                        ok = false;
                    }
                }
            } else {
                msgs.push("Piper already installed.".to_string());
            }
        }
        "espeak" => {
            msgs.push("espeak-ng is available as system fallback.".to_string());
        }
        _ => msgs.push(format!("Unknown TTS provider: {tts_provider}")),
    }

    Ok(CommandResponse {
        ok,
        message: msgs.join(" "),
    })
}

#[tauri::command]
pub fn check_voice_system() -> Result<VoiceConfig, String> {
    Ok(VoiceConfig {
        enabled: detect_whisper_installed() || detect_piper_installed(),
        stt_provider: detect_stt_provider(),
        tts_provider: detect_tts_provider(),
        stt_model: "base".to_string(),
        tts_voice: "en_US-lessac-medium".to_string(),
        whisper_installed: detect_whisper_installed(),
        piper_installed: detect_piper_installed(),
        offline_fallback: detect_whisper_installed() && detect_piper_installed(),
    })
}

#[tauri::command]
pub fn list_piper_voices() -> Vec<String> {
    vec![
        "en_US-lessac-medium".to_string(),
        "en_US-ryan-high".to_string(),
        "en_GB-alan-medium".to_string(),
        "en_US-amy-medium".to_string(),
        "en_US-joe-medium".to_string(),
    ]
}

fn detect_whisper_installed() -> bool {
    Command::new("faster-whisper")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("python")
            .args(["-c", "import faster_whisper"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        || Command::new("whisper")
            .arg("--help")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

fn detect_piper_installed() -> bool {
    Command::new("piper")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || piper_binary_path().exists()
}

fn piper_binary_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from("C:\\Program Files\\piper\\piper.exe")
    } else {
        PathBuf::from("/usr/local/bin/piper")
    }
}

fn detect_stt_provider() -> String {
    if Command::new("python")
        .args(["-c", "import faster_whisper"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "faster-whisper".to_string()
    } else if detect_whisper_installed() {
        "whisper".to_string()
    } else {
        "none".to_string()
    }
}

fn detect_tts_provider() -> String {
    if detect_piper_installed() {
        "piper".to_string()
    } else if Command::new("espeak-ng")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "espeak".to_string()
    } else {
        "none".to_string()
    }
}

fn install_faster_whisper() -> Result<String, String> {
    let status = Command::new("pip")
        .args(["install", "faster-whisper"])
        .status()
        .map_err(|e| format!("pip not found: {e}"))?;

    if status.success() {
        Ok("faster-whisper installed.".to_string())
    } else {
        Err("pip install faster-whisper failed.".to_string())
    }
}

fn install_piper() -> Result<String, String> {
    if cfg!(target_os = "linux") {
        let status = Command::new("sh")
            .args(["-c", "pip install piper-tts"])
            .status()
            .map_err(|e| format!("Failed: {e}"))?;

        if status.success() {
            return Ok("piper-tts installed via pip.".to_string());
        }
    }

    if cfg!(target_os = "macos") {
        let status = Command::new("brew")
            .args(["install", "piper"])
            .status()
            .map_err(|e| format!("brew not found: {e}"));

        if let Ok(s) = status {
            if s.success() {
                return Ok("Piper installed via Homebrew.".to_string());
            }
        }
    }

    Err("Could not auto-install Piper. Download from https://github.com/rhasspy/piper/releases".to_string())
}
