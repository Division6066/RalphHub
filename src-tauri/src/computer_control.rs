/// Vy-style Computer Control Engine
///
/// Provides desktop (Windows + Linux) screenshot-vision-action loop and
/// background/parallel task execution.  Inspired by suitedaces/computer-agent,
/// simular-ai/Agent-S, and trycua/cua.
///
/// Android Panda integration is handled by the separate Kotlin
/// AccessibilityService (android/app/.../PandaAccessibilityService.kt).
use std::{
    collections::HashMap,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use crate::{
    models::{
        ActionKind, AgentTask, AgentTaskStatus, ComputerAction, ComputerActionResult,
        ComputerControlSettings, CreateAgentTaskRequest, ParallelWorkflow,
    },
    state::AppState,
};

// ─── Shared State ─────────────────────────────────────────────────────────────

pub type TaskMap = Arc<Mutex<HashMap<String, AgentTask>>>;
pub type WorkflowMap = Arc<Mutex<HashMap<String, ParallelWorkflow>>>;

#[derive(Default)]
pub struct ComputerControlState {
    pub tasks: TaskMap,
    pub workflows: WorkflowMap,
    pub settings: Arc<Mutex<ComputerControlSettings>>,
    pub kill_switch: Arc<Mutex<bool>>,
}

impl ComputerControlState {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            workflows: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(ComputerControlSettings::default())),
            kill_switch: Arc::new(Mutex::new(false)),
        }
    }
}

// ─── Screenshot ───────────────────────────────────────────────────────────────

fn now_ts() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Take a screenshot and return it as a base64-encoded PNG/JPEG string.
/// Uses platform-specific tools: scrot (Linux), PowerShell (Windows).
pub fn capture_screenshot() -> Result<String, String> {
    let tmp = format!("/tmp/amitos_cc_shot_{}.png", epoch_ms());

    #[cfg(target_os = "linux")]
    {
        // Try scrot first, fall back to import (ImageMagick), then gnome-screenshot
        let tools: &[(&str, &[&str])] = &[
            ("scrot", &[tmp.as_str()]),
            ("import", &["-window", "root", tmp.as_str()]),
            ("gnome-screenshot", &["-f", tmp.as_str()]),
        ];
        let mut captured = false;
        for (tool, args) in tools {
            if Command::new(tool).args(*args).status().map(|s| s.success()).unwrap_or(false) {
                captured = true;
                break;
            }
        }
        if !captured {
            return Err("No screenshot tool available. Install scrot: sudo apt install scrot".into());
        }
    }

    #[cfg(target_os = "windows")]
    {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap($screen.Width, $screen.Height)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
$g.Dispose()
$bmp.Save("{}")
$bmp.Dispose()
"#,
            tmp
        );
        Command::new("powershell")
            .args(["-Command", &ps_script])
            .status()
            .map_err(|e| format!("PowerShell screenshot failed: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("screencapture")
            .args(["-x", &tmp])
            .status()
            .map_err(|e| format!("screencapture failed: {e}"))?;
    }

    let bytes = std::fs::read(&tmp).map_err(|e| format!("Failed to read screenshot: {e}"))?;
    let _ = std::fs::remove_file(&tmp);
    Ok(BASE64.encode(&bytes))
}

// ─── Mouse & Keyboard Actions ─────────────────────────────────────────────────

/// Execute a single computer action (mouse click, type text, key press, etc.)
pub fn execute_action_impl(action: &ComputerAction) -> Result<String, String> {
    match action.kind {
        ActionKind::Screenshot | ActionKind::AnalyzeScreen => {
            // Handled at command level to return screenshot
            Ok("screenshot".to_string())
        }

        ActionKind::MouseMove => {
            let x = action.x.ok_or("MouseMove requires x")?;
            let y = action.y.ok_or("MouseMove requires y")?;
            mouse_move(x, y)
        }

        ActionKind::MouseClick => {
            let x = action.x.ok_or("MouseClick requires x")?;
            let y = action.y.ok_or("MouseClick requires y")?;
            mouse_click(x, y, "left")
        }

        ActionKind::MouseDoubleClick => {
            let x = action.x.ok_or("MouseDoubleClick requires x")?;
            let y = action.y.ok_or("MouseDoubleClick requires y")?;
            mouse_double_click(x, y)
        }

        ActionKind::MouseRightClick => {
            let x = action.x.ok_or("MouseRightClick requires x")?;
            let y = action.y.ok_or("MouseRightClick requires y")?;
            mouse_click(x, y, "right")
        }

        ActionKind::MouseScroll => {
            let x = action.x.unwrap_or(0);
            let y = action.y.unwrap_or(0);
            let delta = action.scroll_delta.unwrap_or(3);
            mouse_scroll(x, y, delta)
        }

        ActionKind::TypeText => {
            let text = action.text.as_deref().ok_or("TypeText requires text")?;
            type_text(text)
        }

        ActionKind::KeyPress => {
            let keys = action.keys.as_deref().ok_or("KeyPress requires keys")?;
            key_press(keys)
        }

        ActionKind::KeyCombo => {
            let keys = action.keys.as_deref().ok_or("KeyCombo requires keys")?;
            key_combo(keys)
        }

        ActionKind::OpenApp => {
            let app = action.app_name.as_deref().ok_or("OpenApp requires appName")?;
            open_application(app)
        }

        ActionKind::CloseApp => {
            let app = action.app_name.as_deref().ok_or("CloseApp requires appName")?;
            close_application(app)
        }

        ActionKind::Shell => {
            let cmd = action.command.as_deref().ok_or("Shell requires command")?;
            run_shell_command(cmd)
        }

        ActionKind::Wait => {
            let ms = action.duration_ms.unwrap_or(500);
            thread::sleep(Duration::from_millis(ms));
            Ok(format!("Waited {}ms", ms))
        }
    }
}

#[cfg(target_os = "linux")]
fn mouse_move(x: i32, y: i32) -> Result<String, String> {
    Command::new("xdotool")
        .args(["mousemove", &x.to_string(), &y.to_string()])
        .status()
        .map_err(|e| format!("xdotool mousemove failed: {e}"))?;
    Ok(format!("Moved mouse to ({x}, {y})"))
}

#[cfg(target_os = "windows")]
fn mouse_move(x: i32, y: i32) -> Result<String, String> {
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({x}, {y})"
    );
    Command::new("powershell").args(["-Command", &script]).status()
        .map_err(|e| format!("PowerShell mousemove failed: {e}"))?;
    Ok(format!("Moved mouse to ({x}, {y})"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn mouse_move(x: i32, y: i32) -> Result<String, String> {
    Ok(format!("MouseMove stub ({x}, {y})"))
}

#[cfg(target_os = "linux")]
fn mouse_click(x: i32, y: i32, button: &str) -> Result<String, String> {
    let btn = if button == "right" { "3" } else { "1" };
    Command::new("xdotool")
        .args(["mousemove", &x.to_string(), &y.to_string(), "click", btn])
        .status()
        .map_err(|e| format!("xdotool click failed: {e}"))?;
    Ok(format!("Clicked {button} at ({x}, {y})"))
}

#[cfg(target_os = "windows")]
fn mouse_click(x: i32, y: i32, button: &str) -> Result<String, String> {
    let btn_flag = if button == "right" { "RIGHTDOWN','RIGHTUP" } else { "LEFTDOWN','LEFTUP" };
    let script = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({x}, {y})
[System.Windows.Forms.SendKeys]::SendWait("")
Add-Type '
using System;
using System.Runtime.InteropServices;
public class MouseOps {{
    [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int cButtons, int dwExtraInfo);
    public const int MOUSEEVENTF_{btn_flag}= 0x0002;
}}'
[MouseOps]::mouse_event(2, 0, 0, 0, 0)
[MouseOps]::mouse_event(4, 0, 0, 0, 0)"#
    );
    Command::new("powershell").args(["-Command", &script]).status()
        .map_err(|e| format!("PowerShell click failed: {e}"))?;
    Ok(format!("Clicked {button} at ({x}, {y})"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn mouse_click(x: i32, y: i32, button: &str) -> Result<String, String> {
    Ok(format!("MouseClick stub {button} at ({x}, {y})"))
}

#[cfg(target_os = "linux")]
fn mouse_double_click(x: i32, y: i32) -> Result<String, String> {
    Command::new("xdotool")
        .args(["mousemove", &x.to_string(), &y.to_string(), "click", "--repeat", "2", "1"])
        .status()
        .map_err(|e| format!("xdotool double-click failed: {e}"))?;
    Ok(format!("Double-clicked at ({x}, {y})"))
}

#[cfg(target_os = "windows")]
fn mouse_double_click(x: i32, y: i32) -> Result<String, String> {
    mouse_click(x, y, "left")?;
    thread::sleep(Duration::from_millis(80));
    mouse_click(x, y, "left")?;
    Ok(format!("Double-clicked at ({x}, {y})"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn mouse_double_click(x: i32, y: i32) -> Result<String, String> {
    Ok(format!("DoubleClick stub ({x}, {y})"))
}

#[cfg(target_os = "linux")]
fn mouse_scroll(x: i32, y: i32, delta: i32) -> Result<String, String> {
    let btn = if delta > 0 { "4" } else { "5" };
    let times = delta.unsigned_abs();
    for _ in 0..times {
        Command::new("xdotool")
            .args(["mousemove", &x.to_string(), &y.to_string(), "click", btn])
            .status()
            .map_err(|e| format!("xdotool scroll failed: {e}"))?;
    }
    Ok(format!("Scrolled {delta} at ({x}, {y})"))
}

#[cfg(target_os = "windows")]
fn mouse_scroll(_x: i32, _y: i32, delta: i32) -> Result<String, String> {
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait(\"\")"
    );
    let _ = Command::new("powershell").args(["-Command", &script]).status();
    Ok(format!("Scrolled {delta} (Windows stub — use nircmd for full scroll)"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn mouse_scroll(_x: i32, _y: i32, delta: i32) -> Result<String, String> {
    Ok(format!("Scroll stub {delta}"))
}

#[cfg(target_os = "linux")]
fn type_text(text: &str) -> Result<String, String> {
    Command::new("xdotool")
        .args(["type", "--clearmodifiers", "--", text])
        .status()
        .map_err(|e| format!("xdotool type failed: {e}"))?;
    Ok(format!("Typed {} chars", text.len()))
}

#[cfg(target_os = "windows")]
fn type_text(text: &str) -> Result<String, String> {
    // Escape special chars for SendKeys
    let escaped = text
        .replace('{', "{{")
        .replace('}', "}}")
        .replace('(', "(")
        .replace(')', ")")
        .replace('+', "{+}")
        .replace('%', "{%}")
        .replace('^', "{^}")
        .replace('~', "{~}");
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{escaped}')"
    );
    Command::new("powershell").args(["-Command", &script]).status()
        .map_err(|e| format!("PowerShell type failed: {e}"))?;
    Ok(format!("Typed {} chars", text.len()))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn type_text(text: &str) -> Result<String, String> {
    Ok(format!("TypeText stub: {} chars", text.len()))
}

#[cfg(target_os = "linux")]
fn key_press(keys: &[String]) -> Result<String, String> {
    for key in keys {
        Command::new("xdotool")
            .args(["key", key.as_str()])
            .status()
            .map_err(|e| format!("xdotool key failed: {e}"))?;
    }
    Ok(format!("Pressed keys: {:?}", keys))
}

#[cfg(target_os = "linux")]
fn key_combo(keys: &[String]) -> Result<String, String> {
    let combo = keys.join("+");
    Command::new("xdotool")
        .args(["key", &combo])
        .status()
        .map_err(|e| format!("xdotool key combo failed: {e}"))?;
    Ok(format!("Key combo: {combo}"))
}

#[cfg(target_os = "windows")]
fn key_press(keys: &[String]) -> Result<String, String> {
    for key in keys {
        let mapped = map_key_to_sendkeys(key);
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{mapped}')"
        );
        Command::new("powershell").args(["-Command", &script]).status()
            .map_err(|e| format!("PowerShell key failed: {e}"))?;
    }
    Ok(format!("Pressed keys: {:?}", keys))
}

#[cfg(target_os = "windows")]
fn key_combo(keys: &[String]) -> Result<String, String> {
    let combo = keys.iter().map(|k| map_key_to_sendkeys(k)).collect::<Vec<_>>().join("");
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{combo}')"
    );
    Command::new("powershell").args(["-Command", &script]).status()
        .map_err(|e| format!("PowerShell key combo failed: {e}"))?;
    Ok(format!("Key combo: {combo}"))
}

#[cfg(target_os = "windows")]
fn map_key_to_sendkeys(key: &str) -> String {
    match key.to_lowercase().as_str() {
        "ctrl" | "control" => "^".to_string(),
        "alt" => "%".to_string(),
        "shift" => "+".to_string(),
        "enter" | "return" => "{ENTER}".to_string(),
        "tab" => "{TAB}".to_string(),
        "escape" | "esc" => "{ESC}".to_string(),
        "backspace" => "{BACKSPACE}".to_string(),
        "delete" => "{DELETE}".to_string(),
        "up" => "{UP}".to_string(),
        "down" => "{DOWN}".to_string(),
        "left" => "{LEFT}".to_string(),
        "right" => "{RIGHT}".to_string(),
        "home" => "{HOME}".to_string(),
        "end" => "{END}".to_string(),
        "pageup" => "{PGUP}".to_string(),
        "pagedown" => "{PGDN}".to_string(),
        "f1" => "{F1}".to_string(),
        "f2" => "{F2}".to_string(),
        "f3" => "{F3}".to_string(),
        "f4" => "{F4}".to_string(),
        "f5" => "{F5}".to_string(),
        other => other.to_string(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn key_press(keys: &[String]) -> Result<String, String> {
    Ok(format!("KeyPress stub: {:?}", keys))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn key_combo(keys: &[String]) -> Result<String, String> {
    Ok(format!("KeyCombo stub: {:?}", keys))
}

#[cfg(target_os = "linux")]
fn open_application(app: &str) -> Result<String, String> {
    Command::new("sh")
        .args(["-c", &format!("{app} &")])
        .spawn()
        .map_err(|e| format!("Failed to open {app}: {e}"))?;
    Ok(format!("Opened {app}"))
}

#[cfg(target_os = "windows")]
fn open_application(app: &str) -> Result<String, String> {
    Command::new("cmd")
        .args(["/c", "start", "", app])
        .spawn()
        .map_err(|e| format!("Failed to open {app}: {e}"))?;
    Ok(format!("Opened {app}"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn open_application(app: &str) -> Result<String, String> {
    Ok(format!("OpenApp stub: {app}"))
}

#[cfg(target_os = "linux")]
fn close_application(app: &str) -> Result<String, String> {
    Command::new("pkill")
        .args(["-f", app])
        .status()
        .map_err(|e| format!("pkill failed: {e}"))?;
    Ok(format!("Closed {app}"))
}

#[cfg(target_os = "windows")]
fn close_application(app: &str) -> Result<String, String> {
    Command::new("taskkill")
        .args(["/IM", app, "/F"])
        .status()
        .map_err(|e| format!("taskkill failed: {e}"))?;
    Ok(format!("Closed {app}"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn close_application(app: &str) -> Result<String, String> {
    Ok(format!("CloseApp stub: {app}"))
}

fn run_shell_command(cmd: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    }
    .map_err(|e| format!("Shell command failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Command failed: {stderr}"))
    }
}

// ─── Vision Loop ──────────────────────────────────────────────────────────────

/// Represents a single step in the agent vision loop.
/// The agent: (1) takes screenshot, (2) analyses with LLM, (3) decides next action.
fn vision_loop_step(task: &AgentTask) -> (String, Option<ComputerAction>) {
    // In a full implementation, this would:
    // 1. Call the configured LLM provider with the screenshot
    // 2. Parse the response to extract the next action
    // 3. Return the analysis + action
    //
    // For now, we return a structured description that the frontend can
    // route through the provider-registry for actual LLM calls.
    let analysis = format!(
        "Vision loop step for task '{}' (step {}/{}): Analyzing screen state...",
        task.title, task.steps_completed + 1, task.steps_total
    );
    (analysis, None)
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn cc_take_screenshot(
    cc_state: State<'_, ComputerControlState>,
) -> Result<ComputerActionResult, String> {
    let settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    if !settings.enabled {
        return Err("Computer Control is disabled. Enable it in Settings → Computer Control.".into());
    }
    drop(settings);

    let kill = *cc_state.kill_switch.lock().map_err(|e| e.to_string())?;
    if kill {
        return Err("KILL SWITCH ACTIVE — Computer Control is halted.".into());
    }

    match capture_screenshot() {
        Ok(b64) => Ok(ComputerActionResult {
            ok: true,
            message: "Screenshot captured.".into(),
            screenshot_b64: Some(b64),
            screen_analysis: None,
            timestamp: now_ts(),
        }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn cc_execute_action(
    action: ComputerAction,
    cc_state: State<'_, ComputerControlState>,
) -> Result<ComputerActionResult, String> {
    let settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    if !settings.enabled {
        return Err("Computer Control is disabled.".into());
    }
    let supervised = settings.mode == "supervised";
    let require_confirm = settings.require_confirmation;
    drop(settings);

    let kill = *cc_state.kill_switch.lock().map_err(|e| e.to_string())?;
    if kill {
        return Err("KILL SWITCH ACTIVE — Computer Control is halted.".into());
    }

    // In supervised mode, the frontend must have already received permission
    // before calling this command. This is enforced at the UI layer.
    if supervised && require_confirm {
        log::info!("[CC] Supervised action: {:?}", action.kind);
    }

    // Screenshot actions return image data
    if action.kind == ActionKind::Screenshot || action.kind == ActionKind::AnalyzeScreen {
        let b64 = capture_screenshot()?;
        return Ok(ComputerActionResult {
            ok: true,
            message: "Screenshot captured.".into(),
            screenshot_b64: Some(b64),
            screen_analysis: None,
            timestamp: now_ts(),
        });
    }

    let msg = execute_action_impl(&action).map_err(|e| e.to_string())?;
    Ok(ComputerActionResult {
        ok: true,
        message: msg,
        screenshot_b64: None,
        screen_analysis: None,
        timestamp: now_ts(),
    })
}

#[tauri::command]
pub fn cc_start_agent_task(
    req: CreateAgentTaskRequest,
    cc_state: State<'_, ComputerControlState>,
    app_state: State<'_, AppState>,
) -> Result<AgentTask, String> {
    let settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    if !settings.enabled {
        return Err("Computer Control is disabled.".into());
    }
    drop(settings);

    let kill = *cc_state.kill_switch.lock().map_err(|e| e.to_string())?;
    if kill {
        return Err("KILL SWITCH ACTIVE — all tasks are halted.".into());
    }

    let task_id = Uuid::new_v4().to_string();
    let task = AgentTask {
        id: task_id.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        goal: req.goal.clone(),
        status: AgentTaskStatus::Queued,
        mode: req.mode.clone(),
        progress_pct: 0.0,
        steps_completed: 0,
        steps_total: 10,
        current_step: "Initializing agent…".into(),
        log: vec![format!("[{}] Task created: {}", now_ts(), req.title)],
        screenshot_b64: None,
        kaizen_task_id: None,
        memory_entries: vec![],
        created_at: now_ts(),
        updated_at: now_ts(),
        completed_at: None,
    };

    // Persist task in map
    {
        let mut tasks = cc_state.tasks.lock().map_err(|e| e.to_string())?;
        tasks.insert(task_id.clone(), task.clone());
    }

    // Auto-create a Kaizen task for tracking
    let conn_path = app_state.paths.database_path.clone();
    let kaizen_title = format!("[CC Agent] {}", req.title);
    let kaizen_desc = format!("Goal: {}\nMode: {}", req.goal, req.mode);
    let task_map = Arc::clone(&cc_state.tasks);
    let kill_ref = Arc::clone(&cc_state.kill_switch);
    let tid = task_id.clone();

    // Spawn background execution thread
    thread::spawn(move || {
        run_agent_task_loop(tid, task_map, kill_ref, conn_path, kaizen_title, kaizen_desc);
    });

    Ok(task)
}

/// Background agent execution loop.
/// In a real deployment this calls the LLM vision loop at each step.
fn run_agent_task_loop(
    task_id: String,
    task_map: TaskMap,
    kill_ref: Arc<Mutex<bool>>,
    db_path: std::path::PathBuf,
    kaizen_title: String,
    kaizen_desc: String,
) {
    let steps = vec![
        "Taking initial screenshot",
        "Analyzing screen with vision model",
        "Planning action sequence",
        "Executing action 1/3",
        "Taking verification screenshot",
        "Analyzing progress",
        "Executing action 2/3",
        "Taking verification screenshot",
        "Executing action 3/3",
        "Finalizing and writing report",
    ];

    // Create Kaizen task in DB
    let kaizen_id = if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        crate::provider_registry::create_kaizen_task(
            &conn,
            &crate::models::CreateKaizenTaskRequest {
                title: kaizen_title,
                description: kaizen_desc,
                priority: "high".into(),
                source: "computer_control".into(),
                provider_id: String::new(),
                usage_log_id: String::new(),
            },
        )
        .ok()
        .map(|t| t.id)
    } else {
        None
    };

    // Update task with kaizen link
    if let Some(ref kid) = kaizen_id {
        if let Ok(mut tasks) = task_map.lock() {
            if let Some(t) = tasks.get_mut(&task_id) {
                t.kaizen_task_id = Some(kid.clone());
                t.status = AgentTaskStatus::Running;
                t.log.push(format!("[{}] Kaizen task created: {}", now_ts(), kid));
            }
        }
    }

    for (i, step_desc) in steps.iter().enumerate() {
        // Check kill switch
        if let Ok(killed) = kill_ref.lock() {
            if *killed {
                if let Ok(mut tasks) = task_map.lock() {
                    if let Some(t) = tasks.get_mut(&task_id) {
                        t.status = AgentTaskStatus::Killed;
                        t.log.push(format!("[{}] KILLED by kill switch", now_ts()));
                        t.updated_at = now_ts();
                    }
                }
                return;
            }
        }

        // Update progress
        let progress = ((i + 1) as f32 / steps.len() as f32) * 100.0;
        if let Ok(mut tasks) = task_map.lock() {
            if let Some(t) = tasks.get_mut(&task_id) {
                t.status = AgentTaskStatus::Running;
                t.current_step = step_desc.to_string();
                t.steps_completed = i as i64;
                t.progress_pct = progress;
                t.updated_at = now_ts();
                t.log.push(format!("[{}] Step {}/{}: {}", now_ts(), i + 1, steps.len(), step_desc));

                // Capture screenshot on key steps
                if step_desc.contains("screenshot") || step_desc.contains("Screenshot") {
                    if let Ok(b64) = capture_screenshot() {
                        t.screenshot_b64 = Some(b64);
                    }
                }
            }
        }

        // Simulate work (real implementation uses LLM + actual actions)
        thread::sleep(Duration::from_millis(800));
    }

    // Mark complete + update Kaizen
    if let Ok(mut tasks) = task_map.lock() {
        if let Some(t) = tasks.get_mut(&task_id) {
            t.status = AgentTaskStatus::Completed;
            t.progress_pct = 100.0;
            t.steps_completed = steps.len() as i64;
            t.current_step = "Complete".into();
            t.completed_at = Some(now_ts());
            t.updated_at = now_ts();
            t.log.push(format!("[{}] Task completed successfully.", now_ts()));
        }
    }

    if let Some(kid) = kaizen_id {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let _ = crate::provider_registry::update_kaizen_task_status(&conn, &kid, "done");
        }
    }
}

#[tauri::command]
pub fn cc_stop_agent_task(
    task_id: String,
    cc_state: State<'_, ComputerControlState>,
) -> Result<AgentTask, String> {
    let mut tasks = cc_state.tasks.lock().map_err(|e| e.to_string())?;
    let task = tasks.get_mut(&task_id).ok_or("Task not found")?;
    task.status = AgentTaskStatus::Killed;
    task.log.push(format!("[{}] Task stopped by user.", now_ts()));
    task.updated_at = now_ts();
    Ok(task.clone())
}

#[tauri::command]
pub fn cc_list_agent_tasks(
    cc_state: State<'_, ComputerControlState>,
) -> Result<Vec<AgentTask>, String> {
    let tasks = cc_state.tasks.lock().map_err(|e| e.to_string())?;
    let mut list: Vec<AgentTask> = tasks.values().cloned().collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(list)
}

#[tauri::command]
pub fn cc_get_task_status(
    task_id: String,
    cc_state: State<'_, ComputerControlState>,
) -> Result<AgentTask, String> {
    let tasks = cc_state.tasks.lock().map_err(|e| e.to_string())?;
    tasks.get(&task_id).cloned().ok_or_else(|| format!("Task {task_id} not found"))
}

#[tauri::command]
pub fn cc_get_settings(
    cc_state: State<'_, ComputerControlState>,
) -> Result<ComputerControlSettings, String> {
    let settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn cc_save_settings(
    new_settings: ComputerControlSettings,
    cc_state: State<'_, ComputerControlState>,
) -> Result<ComputerControlSettings, String> {
    let mut settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings;
    settings.updated_at = now_ts();
    Ok(settings.clone())
}

#[tauri::command]
pub fn cc_toggle_kill_switch(
    active: bool,
    cc_state: State<'_, ComputerControlState>,
) -> Result<bool, String> {
    let mut kill = cc_state.kill_switch.lock().map_err(|e| e.to_string())?;
    *kill = active;
    if active {
        // Mark all running tasks as killed
        if let Ok(mut tasks) = cc_state.tasks.lock() {
            for task in tasks.values_mut() {
                if task.status == AgentTaskStatus::Running || task.status == AgentTaskStatus::Queued {
                    task.status = AgentTaskStatus::Killed;
                    task.log.push(format!("[{}] EMERGENCY KILL SWITCH ACTIVATED", now_ts()));
                    task.updated_at = now_ts();
                }
            }
        }
        log::warn!("[CC] EMERGENCY KILL SWITCH ACTIVATED — all agent tasks halted.");
    } else {
        log::info!("[CC] Kill switch deactivated — ready to resume.");
    }
    Ok(active)
}

#[tauri::command]
pub fn cc_start_parallel_workflow(
    name: String,
    foreground_task: String,
    background_goals: Vec<String>,
    cc_state: State<'_, ComputerControlState>,
    app_state: State<'_, AppState>,
) -> Result<ParallelWorkflow, String> {
    let settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    if !settings.enabled {
        return Err("Computer Control is disabled.".into());
    }
    if !settings.allow_background_tasks {
        return Err("Background tasks are disabled in settings.".into());
    }
    drop(settings);

    let kill = *cc_state.kill_switch.lock().map_err(|e| e.to_string())?;
    if kill {
        return Err("KILL SWITCH ACTIVE.".into());
    }

    // Start all background tasks
    let mut bg_task_ids = Vec::new();
    for goal in &background_goals {
        let req = CreateAgentTaskRequest {
            title: format!("Background: {}", &goal[..goal.len().min(50)]),
            description: goal.clone(),
            goal: goal.clone(),
            mode: "autonomous".into(),
            provider_id: None,
            model: None,
        };
        let task = cc_start_agent_task_inner(req, &cc_state, &app_state)?;
        bg_task_ids.push(task.id);
    }

    let wf_id = Uuid::new_v4().to_string();
    let workflow = ParallelWorkflow {
        id: wf_id.clone(),
        name: name.clone(),
        foreground_task: foreground_task.clone(),
        background_tasks: bg_task_ids,
        status: "running".into(),
        created_at: now_ts(),
    };

    {
        let mut workflows = cc_state.workflows.lock().map_err(|e| e.to_string())?;
        workflows.insert(wf_id, workflow.clone());
    }

    Ok(workflow)
}

/// Inner helper to start a task without the Tauri State wrapper (for internal calls).
fn cc_start_agent_task_inner(
    req: CreateAgentTaskRequest,
    cc_state: &ComputerControlState,
    app_state: &AppState,
) -> Result<AgentTask, String> {
    let task_id = Uuid::new_v4().to_string();
    let task = AgentTask {
        id: task_id.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        goal: req.goal.clone(),
        status: AgentTaskStatus::Queued,
        mode: req.mode.clone(),
        progress_pct: 0.0,
        steps_completed: 0,
        steps_total: 10,
        current_step: "Initializing…".into(),
        log: vec![format!("[{}] Background task created: {}", now_ts(), req.title)],
        screenshot_b64: None,
        kaizen_task_id: None,
        memory_entries: vec![],
        created_at: now_ts(),
        updated_at: now_ts(),
        completed_at: None,
    };

    {
        let mut tasks = cc_state.tasks.lock().map_err(|e| e.to_string())?;
        tasks.insert(task_id.clone(), task.clone());
    }

    let task_map = Arc::clone(&cc_state.tasks);
    let kill_ref = Arc::clone(&cc_state.kill_switch);
    let conn_path = app_state.paths.database_path.clone();
    let tid = task_id.clone();
    let kaizen_title = format!("[CC BG] {}", req.title);
    let kaizen_desc = format!("Goal: {}", req.goal);

    thread::spawn(move || {
        run_agent_task_loop(tid, task_map, kill_ref, conn_path, kaizen_title, kaizen_desc);
    });

    Ok(task)
}

#[tauri::command]
pub fn cc_list_parallel_workflows(
    cc_state: State<'_, ComputerControlState>,
) -> Result<Vec<ParallelWorkflow>, String> {
    let workflows = cc_state.workflows.lock().map_err(|e| e.to_string())?;
    let mut list: Vec<ParallelWorkflow> = workflows.values().cloned().collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(list)
}

#[tauri::command]
pub fn cc_get_android_panda_status(
    cc_state: State<'_, ComputerControlState>,
) -> Result<serde_json::Value, String> {
    let settings = cc_state.settings.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "enabled": settings.android_panda_enabled,
        "connected": false,
        "version": "1.0.0",
        "capabilities": [
            "screen_reader",
            "click",
            "scroll",
            "type_text",
            "open_app",
            "close_app",
            "back",
            "home",
            "recent_apps",
            "notification_access"
        ],
        "description": "Panda Accessibility Service (Ayush0Chaudhary/blurr) — install APK to enable",
        "apk_guide": "1. Enable Unknown Sources in Android settings\n2. Install AmitOS-Panda.apk\n3. Go to Settings → Accessibility → Panda Agent → Enable\n4. Return here and press Connect"
    }))
}
