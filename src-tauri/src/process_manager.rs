use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

use chrono::Utc;

use crate::models::{EnvEntry, ToolProcessStatus};

pub struct RunningTool {
    pub tool_id: String,
    pub name: String,
    pub pid: u32,
    pub started_at: String,
    pub log_path: String,
    pub child: Option<Child>,
}

impl std::fmt::Debug for RunningTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunningTool")
            .field("tool_id", &self.tool_id)
            .field("name", &self.name)
            .field("pid", &self.pid)
            .field("started_at", &self.started_at)
            .field("log_path", &self.log_path)
            .finish()
    }
}

pub type ProcessRegistry = Arc<Mutex<HashMap<String, RunningTool>>>;

pub fn new_registry() -> ProcessRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn launch_background(
    registry: &ProcessRegistry,
    tool_id: &str,
    name: &str,
    workspace_path: &str,
    command: &str,
    env_entries: &[EnvEntry],
    logs_dir: &PathBuf,
) -> Result<ToolProcessStatus, String> {
    let log_path = logs_dir.join(format!("{tool_id}.log"));
    let err_path = logs_dir.join(format!("{tool_id}.err.log"));

    let log_file = fs::File::create(&log_path)
        .map_err(|e| format!("Failed to create log file: {e}"))?;
    let err_file = fs::File::create(&err_path)
        .map_err(|e| format!("Failed to create error log file: {e}"))?;

    let parts: Vec<&str> = command.split_whitespace().collect();
    let (prog, args) = parts.split_first().ok_or("Empty launch command")?;

    let mut cmd = Command::new(prog);
    cmd.args(args)
        .current_dir(workspace_path)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(err_file));

    for entry in env_entries {
        cmd.env(&entry.key, &entry.value);
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn '{command}' in '{workspace_path}': {e}"))?;

    let pid = child.id();
    let started_at = Utc::now().to_rfc3339();
    let log_path_str = log_path.display().to_string();

    let mut reg = registry.lock().map_err(|e| format!("Registry lock: {e}"))?;
    reg.insert(
        tool_id.to_string(),
        RunningTool {
            tool_id: tool_id.to_string(),
            name: name.to_string(),
            pid,
            started_at: started_at.clone(),
            log_path: log_path_str.clone(),
            child: Some(child),
        },
    );

    Ok(ToolProcessStatus {
        tool_id: tool_id.to_string(),
        name: name.to_string(),
        status: "running".to_string(),
        pid: Some(pid),
        started_at: Some(started_at),
        log_path: Some(log_path_str),
    })
}

pub fn get_status(registry: &ProcessRegistry, tool_id: &str) -> ToolProcessStatus {
    let mut reg = match registry.lock() {
        Ok(r) => r,
        Err(_) => {
            return ToolProcessStatus {
                tool_id: tool_id.to_string(),
                name: tool_id.to_string(),
                status: "unknown".to_string(),
                pid: None,
                started_at: None,
                log_path: None,
            }
        }
    };

    match reg.get_mut(tool_id) {
        None => ToolProcessStatus {
            tool_id: tool_id.to_string(),
            name: tool_id.to_string(),
            status: "idle".to_string(),
            pid: None,
            started_at: None,
            log_path: None,
        },
        Some(tool) => {
            let alive = check_alive(tool);
            ToolProcessStatus {
                tool_id: tool.tool_id.clone(),
                name: tool.name.clone(),
                status: if alive { "running".to_string() } else { "stopped".to_string() },
                pid: Some(tool.pid),
                started_at: Some(tool.started_at.clone()),
                log_path: Some(tool.log_path.clone()),
            }
        }
    }
}

pub fn stop_tool(registry: &ProcessRegistry, tool_id: &str) -> Result<(), String> {
    let mut reg = registry.lock().map_err(|e| format!("Registry lock: {e}"))?;
    if let Some(tool) = reg.get_mut(tool_id) {
        if let Some(ref mut child) = tool.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        tool.child = None;
    }
    Ok(())
}

pub fn list_all(registry: &ProcessRegistry) -> Vec<ToolProcessStatus> {
    let mut reg = match registry.lock() {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    reg.iter_mut()
        .map(|(_, tool)| {
            let alive = check_alive(tool);
            ToolProcessStatus {
                tool_id: tool.tool_id.clone(),
                name: tool.name.clone(),
                status: if alive { "running".to_string() } else { "stopped".to_string() },
                pid: Some(tool.pid),
                started_at: Some(tool.started_at.clone()),
                log_path: Some(tool.log_path.clone()),
            }
        })
        .collect()
}

pub fn read_logs(registry: &ProcessRegistry, tool_id: &str, tail_lines: usize) -> Vec<String> {
    let reg = match registry.lock() {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let log_path = match reg.get(tool_id) {
        Some(tool) => tool.log_path.clone(),
        None => return vec!["No log found for this tool.".to_string()],
    };

    drop(reg);

    let file = match fs::File::open(&log_path) {
        Ok(f) => f,
        Err(e) => return vec![format!("Cannot read log: {e}")],
    };

    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .collect();

    let start = if all_lines.len() > tail_lines {
        all_lines.len() - tail_lines
    } else {
        0
    };

    all_lines[start..].to_vec()
}

fn check_alive(tool: &mut RunningTool) -> bool {
    if let Some(ref mut child) = tool.child {
        match child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => {
                tool.child = None;
                false
            }
            Err(_) => false,
        }
    } else {
        is_pid_alive(tool.pid)
    }
}

fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()),
            Err(_) => false,
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}
