use std::process::Command;

use tauri::State;

use crate::{
    models::{CommandResponse, OllamaModel, OllamaStatus},
    state::AppState,
};

const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

pub const RECOMMENDED_MODELS: &[(&str, &str, &str)] = &[
    ("mistral", "Mistral 7B", "~4GB"),
    ("qwen2.5:3b", "Qwen 2.5 3B", "~2GB"),
    ("llama3.2:3b", "Llama 3.2 3B", "~2GB"),
    ("phi3.5", "Phi-3.5 Mini", "~2.5GB"),
    ("gemma2:2b", "Gemma 2 2B", "~1.6GB"),
];

#[tauri::command]
pub fn get_ollama_status(_state: State<'_, AppState>) -> OllamaStatus {
    let installed = detect_ollama_installed();
    let (running, version) = if installed {
        let ver = get_ollama_version();
        let run = check_ollama_running();
        (run, ver)
    } else {
        (false, None)
    };

    let models = if running {
        list_local_models().unwrap_or_default()
    } else {
        Vec::new()
    };

    OllamaStatus {
        installed,
        version,
        running,
        endpoint: DEFAULT_ENDPOINT.to_string(),
        models,
        installer_hint: ollama_installer_hint(),
    }
}

#[tauri::command]
pub fn ensure_ollama() -> Result<CommandResponse, String> {
    if detect_ollama_installed() {
        let running = ensure_ollama_running();
        return Ok(CommandResponse {
            ok: true,
            message: format!(
                "Ollama is already installed.{}",
                if running { " Server started." } else { "" }
            ),
        });
    }

    install_ollama()?;
    ensure_ollama_running();

    Ok(CommandResponse {
        ok: true,
        message: "Ollama installed and server started.".to_string(),
    })
}

#[tauri::command]
pub fn pull_ollama_model(model_name: String) -> Result<CommandResponse, String> {
    if !detect_ollama_installed() {
        return Err("Ollama is not installed. Run ensure_ollama first.".to_string());
    }

    ensure_ollama_running();

    let status = Command::new("ollama")
        .args(["pull", &model_name])
        .status()
        .map_err(|e| format!("Failed to run ollama pull: {e}"))?;

    if status.success() {
        Ok(CommandResponse {
            ok: true,
            message: format!("Model '{model_name}' pulled successfully."),
        })
    } else {
        Err(format!("Failed to pull model '{model_name}'. Check model name and network."))
    }
}

#[tauri::command]
pub fn pull_recommended_models() -> Result<CommandResponse, String> {
    if !detect_ollama_installed() {
        install_ollama()?;
    }

    ensure_ollama_running();

    let mut pulled = Vec::new();
    let mut failed = Vec::new();

    for (model_id, display_name, _) in RECOMMENDED_MODELS.iter().take(3) {
        let status = Command::new("ollama")
            .args(["pull", model_id])
            .status();

        match status {
            Ok(s) if s.success() => pulled.push(*display_name),
            _ => failed.push(*display_name),
        }
    }

    let msg = if failed.is_empty() {
        format!("Pulled models: {}.", pulled.join(", "))
    } else {
        format!(
            "Pulled: {}. Failed: {}.",
            pulled.join(", "),
            failed.join(", ")
        )
    };

    Ok(CommandResponse { ok: failed.is_empty(), message: msg })
}

#[tauri::command]
pub fn list_ollama_models() -> Vec<OllamaModel> {
    if !detect_ollama_installed() || !check_ollama_running() {
        return recommended_model_stubs();
    }

    let mut models = list_local_models().unwrap_or_default();

    for (id, display_name, size_hint) in RECOMMENDED_MODELS {
        if !models.iter().any(|m| m.name.starts_with(id)) {
            models.push(OllamaModel {
                name: id.to_string(),
                display_name: display_name.to_string(),
                size_hint: size_hint.to_string(),
                status: "not-pulled".to_string(),
                is_default: *id == "mistral",
            });
        }
    }

    models
}

#[tauri::command]
pub fn start_ollama_server() -> Result<CommandResponse, String> {
    if !detect_ollama_installed() {
        return Err("Ollama is not installed.".to_string());
    }

    let already_running = check_ollama_running();
    if already_running {
        return Ok(CommandResponse {
            ok: true,
            message: "Ollama server is already running.".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "/B", "ollama", "serve"])
            .spawn()
            .map_err(|e| format!("Failed to start Ollama: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .args(["-c", "ollama serve &"])
            .spawn()
            .map_err(|e| format!("Failed to start Ollama: {e}"))?;
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(CommandResponse {
        ok: true,
        message: "Ollama server started.".to_string(),
    })
}

fn detect_ollama_installed() -> bool {
    Command::new("ollama")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_ollama_version() -> Option<String> {
    Command::new("ollama")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn check_ollama_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("powershell")
            .args(["-c", "try { (Invoke-WebRequest -Uri 'http://localhost:11434/api/tags' -TimeoutSec 2).StatusCode -eq 200 } catch { $false }"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "True")
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("curl")
            .args(["-sf", "--max-time", "2", "http://localhost:11434/api/tags"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

fn ensure_ollama_running() -> bool {
    if check_ollama_running() {
        return true;
    }

    let spawned = {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", "/B", "ollama", "serve"])
                .spawn()
                .is_ok()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("sh")
                .args(["-c", "ollama serve &"])
                .spawn()
                .is_ok()
        }
    };

    if spawned {
        std::thread::sleep(std::time::Duration::from_secs(2));
        check_ollama_running()
    } else {
        false
    }
}

fn install_ollama() -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-c", "irm https://ollama.ai/install.ps1 | iex"])
            .status()
    } else if cfg!(target_os = "macos") {
        Command::new("sh")
            .args(["-c", "curl -fsSL https://ollama.ai/install.sh | sh"])
            .status()
    } else {
        Command::new("sh")
            .args(["-c", "curl -fsSL https://ollama.ai/install.sh | sh"])
            .status()
    }
    .map_err(|e| format!("Failed to run Ollama installer: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Ollama installation failed. Install manually from https://ollama.ai and restart RalphHub."
        ))
    }
}

fn list_local_models() -> Option<Vec<OllamaModel>> {
    let output = Command::new("ollama").args(["list"]).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut models: Vec<OllamaModel> = stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }
            let name = parts[0].to_string();
            let size_hint = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
            let display_name = friendly_name(&name);
            Some(OllamaModel {
                is_default: name.starts_with("mistral"),
                name,
                display_name,
                size_hint,
                status: "available".to_string(),
            })
        })
        .collect();

    models.sort_by(|a, b| a.name.cmp(&b.name));
    Some(models)
}

fn recommended_model_stubs() -> Vec<OllamaModel> {
    RECOMMENDED_MODELS
        .iter()
        .map(|(id, display_name, size_hint)| OllamaModel {
            name: id.to_string(),
            display_name: display_name.to_string(),
            size_hint: size_hint.to_string(),
            status: "not-available".to_string(),
            is_default: *id == "mistral",
        })
        .collect()
}

fn friendly_name(name: &str) -> String {
    let base = name.split(':').next().unwrap_or(name);
    match base {
        "mistral" => "Mistral 7B".to_string(),
        "llama3" | "llama3.1" | "llama3.2" => "Llama 3".to_string(),
        "qwen2.5" | "qwen3" => "Qwen 2.5".to_string(),
        "phi3" | "phi3.5" => "Phi-3.5 Mini".to_string(),
        "gemma2" | "gemma3" => "Gemma 2".to_string(),
        "codellama" => "Code Llama".to_string(),
        "deepseek-coder" => "DeepSeek Coder".to_string(),
        _ => base
            .split('-')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

pub fn ollama_installer_hint() -> String {
    if cfg!(target_os = "windows") {
        "Download from https://ollama.ai/download/windows".to_string()
    } else if cfg!(target_os = "macos") {
        "Download from https://ollama.ai/download/mac".to_string()
    } else {
        "curl -fsSL https://ollama.ai/install.sh | sh".to_string()
    }
}
