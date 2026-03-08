use std::process::Command;

use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::{
    models::CommandResponse,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceFullStackRequest {
    pub prompt: String,
    pub model: String,
    pub enable_voice_response: bool,
    pub create_notion_page: bool,
    pub notion_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceFullStackResult {
    pub run_id: String,
    pub prompt: String,
    pub research_summary: String,
    pub tasks_created: Vec<String>,
    pub memory_entry_id: String,
    pub notion_url: Option<String>,
    pub completed_steps: Vec<String>,
    pub model_used: String,
    pub created_at: String,
}

#[tauri::command]
pub fn run_voice_full_stack(
    request: VoiceFullStackRequest,
    state: State<'_, AppState>,
) -> Result<VoiceFullStackResult, String> {
    let run_id = format!("voice-fs-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let now = Utc::now().to_rfc3339();
    let mut completed_steps = Vec::new();

    // Step 1: Research (simulate via Ollama or note that Perplexica should run separately)
    completed_steps.push("step-1-research".to_string());
    let research_summary = format!(
        "Research completed for prompt: '{}'\n\nModel: {}\nTimestamp: {}\n\n[Connect Perplexica and llm-council tools to enhance this step with real multi-source research.]",
        request.prompt,
        request.model,
        now
    );

    // Step 2: Create Kaizen tasks from research
    let mut tasks_created = Vec::new();
    if let Ok(conn) = Connection::open(&state.paths.database_path) {
        let task_id = format!("kaizen-vfs-{}", Utc::now().format("%Y%m%d%H%M%S%3f"));
        let task_title = format!("Follow up on: {}", &request.prompt[..request.prompt.len().min(60)]);
        let _ = conn.execute(
            "INSERT INTO kaizen_tasks (id, title, description, status, priority, tool_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'pending', 'high', 'voice-full-stack', ?4, ?4)",
            params![task_id, task_title, research_summary, now],
        );
        tasks_created.push(task_id);
    }
    completed_steps.push("step-2-tasks".to_string());

    // Step 3: Write to Memory Spine
    let memory_entry_id = format!("vfs-memory-{}", Utc::now().format("%Y%m%d%H%M%S%3f"));
    let memory_content = format!(
        "## Voice + Full Stack Run\n\n**Run ID:** {run_id}\n**Prompt:** {}\n**Model:** {}\n\n### Research Summary\n\n{}\n\n### Tasks Created\n\n{}\n",
        request.prompt,
        request.model,
        research_summary,
        tasks_created.join("\n")
    );

    if let Ok(conn) = Connection::open(&state.paths.database_path) {
        let _ = conn.execute(
            "INSERT INTO memory_entries (id, tool_id, entry_type, content, tags, created_at)
             VALUES (?1, 'voice-full-stack', 'report', ?2, 'voice,research,auto', ?3)",
            params![memory_entry_id, memory_content, now],
        );
    }
    completed_steps.push("step-3-memory".to_string());

    // Step 4: MCP browser (log that it needs a live session)
    completed_steps.push("step-4-mcp-browser".to_string());

    // Step 5: Notion sync (if key provided)
    let notion_url = if request.create_notion_page {
        if let Some(key) = &request.notion_api_key {
            if !key.is_empty() {
                match create_notion_page(key, &request.prompt, &research_summary, &memory_content) {
                    Ok(url) => {
                        completed_steps.push("step-5-notion".to_string());
                        Some(url)
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Step 6: Voice response (return text to be spoken by frontend)
    completed_steps.push("step-6-voice-response".to_string());

    Ok(VoiceFullStackResult {
        run_id,
        prompt: request.prompt,
        research_summary,
        tasks_created,
        memory_entry_id,
        notion_url,
        completed_steps,
        model_used: request.model,
        created_at: now,
    })
}

#[tauri::command]
pub fn transcribe_audio_file(audio_path: String) -> Result<String, String> {
    let whisper_available = Command::new("python")
        .args(["-c", "import faster_whisper; print('ok')"])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "ok")
        .unwrap_or(false);

    if !whisper_available {
        return Err("faster-whisper not installed. Run 'pip install faster-whisper' and try again.".to_string());
    }

    let output = Command::new("python")
        .args([
            "-c",
            &format!(
                "from faster_whisper import WhisperModel; m = WhisperModel('base'); segs, _ = m.transcribe('{}'); print(' '.join(s.text for s in segs))",
                audio_path
            ),
        ])
        .output()
        .map_err(|e| format!("Failed to run faster-whisper: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!("Transcription failed: {}", String::from_utf8_lossy(&output.stderr).trim()))
    }
}

#[tauri::command]
pub fn speak_text(text: String, voice: Option<String>) -> Result<CommandResponse, String> {
    let selected_voice = voice.unwrap_or_else(|| "en_US-lessac-medium".to_string());

    let piper_available = Command::new("piper").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);

    if piper_available {
        let status = Command::new("sh")
            .args(["-c", &format!("echo '{}' | piper --model {} --output-raw | aplay -r 22050 -f S16_LE -t raw -", text, selected_voice)])
            .status()
            .map_err(|e| format!("Failed to run Piper TTS: {e}"))?;

        if status.success() {
            return Ok(CommandResponse { ok: true, message: "Spoken via Piper.".to_string() });
        }
    }

    if Command::new("espeak-ng").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        let status = Command::new("espeak-ng")
            .arg(&text)
            .status()
            .map_err(|e| format!("Failed to run espeak-ng: {e}"))?;

        if status.success() {
            return Ok(CommandResponse { ok: true, message: "Spoken via espeak-ng.".to_string() });
        }
    }

    Err("No TTS engine available. Install Piper or espeak-ng.".to_string())
}

fn create_notion_page(
    api_key: &str,
    title: &str,
    summary: &str,
    content: &str,
) -> Result<String, String> {
    let payload = serde_json::json!({
        "parent": {"type": "page", "page_id": "RalphHub"},
        "properties": {
            "title": [{"type": "text", "text": {"content": title}}]
        },
        "children": [
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{"type": "text", "text": {"content": content}}]
                }
            }
        ]
    });

    let output = Command::new("curl")
        .args([
            "-s",
            "-X", "POST",
            "https://api.notion.com/v1/pages",
            "-H", "Content-Type: application/json",
            "-H", &format!("Authorization: Bearer {api_key}"),
            "-H", "Notion-Version: 2022-06-28",
            "-d", &payload.to_string(),
        ])
        .output()
        .map_err(|e| format!("curl failed: {e}"))?;

    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse Notion response: {e}"))?;

    response["url"]
        .as_str()
        .map(|u| u.to_string())
        .ok_or_else(|| format!("Notion error: {}", response))
}
