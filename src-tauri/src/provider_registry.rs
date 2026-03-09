use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::models::{
    ApiUsageLog, CreateKaizenTaskRequest, CreateProviderRequest, KaizenTask, LogApiUsageRequest,
    MemorySpineEntry, MemorySpineStats, Provider, UpdateProviderRequest,
};

// ─── Provider CRUD ────────────────────────────────────────────────────────────

pub fn create_provider(conn: &Connection, req: &CreateProviderRequest) -> Result<Provider> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let models_json = serde_json::to_string(&req.models)?;

    conn.execute(
        "INSERT INTO providers (id, name, category, base_url, auth_type, api_key_env, models, enabled, is_local, description, docs_url, logo_emoji, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id, req.name, req.category, req.base_url, req.auth_type,
            req.api_key_env, models_json,
            req.is_local as i64,
            req.description, req.docs_url, req.logo_emoji, now, now,
        ],
    )?;

    get_provider(conn, &id)
}

pub fn get_provider(conn: &Connection, id: &str) -> Result<Provider> {
    let p = conn.query_row(
        "SELECT id, name, category, base_url, auth_type, api_key_env, models, enabled, is_local, description, docs_url, logo_emoji, created_at, updated_at FROM providers WHERE id = ?1",
        params![id],
        row_to_provider,
    )?;
    Ok(p)
}

pub fn list_providers(conn: &Connection, category: Option<&str>) -> Result<Vec<Provider>> {
    let mut stmt = if let Some(cat) = category {
        let mut s = conn.prepare(
            "SELECT id, name, category, base_url, auth_type, api_key_env, models, enabled, is_local, description, docs_url, logo_emoji, created_at, updated_at FROM providers WHERE category = ?1 ORDER BY name ASC",
        )?;
        let rows = s.query_map(params![cat], row_to_provider)?;
        return Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?);
    } else {
        conn.prepare(
            "SELECT id, name, category, base_url, auth_type, api_key_env, models, enabled, is_local, description, docs_url, logo_emoji, created_at, updated_at FROM providers ORDER BY category ASC, name ASC",
        )?
    };
    let rows = stmt.query_map([], row_to_provider)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update_provider(conn: &Connection, req: &UpdateProviderRequest) -> Result<Provider> {
    let now = Utc::now().to_rfc3339();

    if let Some(ref name) = req.name {
        conn.execute("UPDATE providers SET name = ?1, updated_at = ?2 WHERE id = ?3", params![name, now, req.id])?;
    }
    if let Some(ref base_url) = req.base_url {
        conn.execute("UPDATE providers SET base_url = ?1, updated_at = ?2 WHERE id = ?3", params![base_url, now, req.id])?;
    }
    if let Some(ref env) = req.api_key_env {
        conn.execute("UPDATE providers SET api_key_env = ?1, updated_at = ?2 WHERE id = ?3", params![env, now, req.id])?;
    }
    if let Some(ref models) = req.models {
        let json = serde_json::to_string(models)?;
        conn.execute("UPDATE providers SET models = ?1, updated_at = ?2 WHERE id = ?3", params![json, now, req.id])?;
    }
    if let Some(enabled) = req.enabled {
        conn.execute("UPDATE providers SET enabled = ?1, updated_at = ?2 WHERE id = ?3", params![enabled as i64, now, req.id])?;
    }
    if let Some(ref desc) = req.description {
        conn.execute("UPDATE providers SET description = ?1, updated_at = ?2 WHERE id = ?3", params![desc, now, req.id])?;
    }

    get_provider(conn, &req.id)
}

pub fn delete_provider(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM providers WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn search_providers(conn: &Connection, query: &str) -> Result<Vec<Provider>> {
    let pattern = format!("%{}%", query.to_lowercase());
    let mut stmt = conn.prepare(
        "SELECT id, name, category, base_url, auth_type, api_key_env, models, enabled, is_local, description, docs_url, logo_emoji, created_at, updated_at
         FROM providers WHERE lower(name) LIKE ?1 OR lower(category) LIKE ?1 OR lower(description) LIKE ?1
         ORDER BY name ASC",
    )?;
    let rows = stmt.query_map(params![pattern], row_to_provider)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn row_to_provider(row: &rusqlite::Row<'_>) -> rusqlite::Result<Provider> {
    let models_json: String = row.get(6)?;
    let models: Vec<String> = serde_json::from_str(&models_json).unwrap_or_default();
    let enabled_int: i64 = row.get(7)?;
    let is_local_int: i64 = row.get(8)?;

    Ok(Provider {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        base_url: row.get(3)?,
        auth_type: row.get(4)?,
        api_key_env: row.get(5)?,
        models,
        enabled: enabled_int != 0,
        is_local: is_local_int != 0,
        description: row.get(9)?,
        docs_url: row.get(10)?,
        logo_emoji: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

// ─── Usage Logging (Memory Spine) ────────────────────────────────────────────

pub fn log_api_usage(conn: &Connection, req: &LogApiUsageRequest) -> Result<ApiUsageLog> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO api_usage_logs (id, provider_id, provider_name, model, tokens_in, tokens_out, cost_usd, output_summary, tool_id, workflow_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id, req.provider_id, req.provider_name, req.model,
            req.tokens_in, req.tokens_out, req.cost_usd,
            req.output_summary, req.tool_id, req.workflow_id, now,
        ],
    )?;

    // Also write a Memory Spine entry
    let spine_id = Uuid::new_v4().to_string();
    let summary = format!(
        "API call to {} using {}. Tokens: {}in/{}out. Cost: ${:.6}. Summary: {}",
        req.provider_name, req.model, req.tokens_in, req.tokens_out, req.cost_usd, req.output_summary
    );
    let tags = serde_json::to_string(&vec![
        req.provider_name.clone(),
        req.model.clone(),
        if !req.tool_id.is_empty() { req.tool_id.clone() } else { "direct".to_string() },
    ])?;

    conn.execute(
        "INSERT INTO memory_spine (id, entry_type, content, tags, provider_id, model, created_at) VALUES (?1, 'api_call', ?2, ?3, ?4, ?5, ?6)",
        params![spine_id, summary, tags, req.provider_id, req.model, now],
    )?;

    get_usage_log(conn, &id)
}

fn get_usage_log(conn: &Connection, id: &str) -> Result<ApiUsageLog> {
    let log = conn.query_row(
        "SELECT id, provider_id, provider_name, model, tokens_in, tokens_out, cost_usd, output_summary, tool_id, workflow_id, created_at FROM api_usage_logs WHERE id = ?1",
        params![id],
        |row| Ok(ApiUsageLog {
            id: row.get(0)?,
            provider_id: row.get(1)?,
            provider_name: row.get(2)?,
            model: row.get(3)?,
            tokens_in: row.get(4)?,
            tokens_out: row.get(5)?,
            cost_usd: row.get(6)?,
            output_summary: row.get(7)?,
            tool_id: row.get(8)?,
            workflow_id: row.get(9)?,
            created_at: row.get(10)?,
        }),
    )?;
    Ok(log)
}

pub fn list_usage_logs(conn: &Connection, limit: i64) -> Result<Vec<ApiUsageLog>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, provider_name, model, tokens_in, tokens_out, cost_usd, output_summary, tool_id, workflow_id, created_at FROM api_usage_logs ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(ApiUsageLog {
            id: row.get(0)?,
            provider_id: row.get(1)?,
            provider_name: row.get(2)?,
            model: row.get(3)?,
            tokens_in: row.get(4)?,
            tokens_out: row.get(5)?,
            cost_usd: row.get(6)?,
            output_summary: row.get(7)?,
            tool_id: row.get(8)?,
            workflow_id: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_memory_spine_stats(conn: &Connection) -> Result<MemorySpineStats> {
    let total_entries: i64 = conn.query_row("SELECT COUNT(*) FROM memory_spine", [], |r| r.get(0))?;

    let (total_tokens, total_cost): (i64, f64) = conn.query_row(
        "SELECT COALESCE(SUM(tokens_in + tokens_out), 0), COALESCE(SUM(cost_usd), 0.0) FROM api_usage_logs",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let mut pstmt = conn.prepare(
        "SELECT DISTINCT provider_name FROM api_usage_logs WHERE provider_name != '' ORDER BY provider_name",
    )?;
    let providers_used: Vec<String> = pstmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let recent_logs = list_usage_logs(conn, 10)?;

    let active_tasks = list_kaizen_tasks(conn, Some("todo"))?;

    Ok(MemorySpineStats {
        total_entries,
        total_tokens,
        total_cost_usd: total_cost,
        providers_used,
        recent_logs,
        active_tasks,
    })
}

// ─── Kaizen Tasks ─────────────────────────────────────────────────────────────

pub fn create_kaizen_task(conn: &Connection, req: &CreateKaizenTaskRequest) -> Result<KaizenTask> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO kaizen_tasks (id, title, description, status, priority, source, provider_id, usage_log_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'todo', ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id, req.title, req.description, req.priority,
            req.source, req.provider_id, req.usage_log_id, now, now,
        ],
    )?;

    get_kaizen_task(conn, &id)
}

fn get_kaizen_task(conn: &Connection, id: &str) -> Result<KaizenTask> {
    let task = conn.query_row(
        "SELECT id, title, description, status, priority, source, provider_id, usage_log_id, created_at, updated_at FROM kaizen_tasks WHERE id = ?1",
        params![id],
        row_to_task,
    )?;
    Ok(task)
}

pub fn list_kaizen_tasks(conn: &Connection, status: Option<&str>) -> Result<Vec<KaizenTask>> {
    if let Some(s) = status {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, status, priority, source, provider_id, usage_log_id, created_at, updated_at FROM kaizen_tasks WHERE status = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![s], row_to_task)?;
        return Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?);
    }
    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, priority, source, provider_id, usage_log_id, created_at, updated_at FROM kaizen_tasks ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_task)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update_kaizen_task_status(conn: &Connection, id: &str, status: &str) -> Result<KaizenTask> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE kaizen_tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, now, id],
    )?;
    get_kaizen_task(conn, id)
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<KaizenTask> {
    Ok(KaizenTask {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: row.get(3)?,
        priority: row.get(4)?,
        source: row.get(5)?,
        provider_id: row.get(6)?,
        usage_log_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

// ─── Memory Spine Entries ─────────────────────────────────────────────────────

pub fn list_memory_entries(conn: &Connection, limit: i64) -> Result<Vec<MemorySpineEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, entry_type, content, tags, provider_id, model, created_at FROM memory_spine ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        let tags_json: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        Ok(MemorySpineEntry {
            id: row.get(0)?,
            entry_type: row.get(1)?,
            content: row.get(2)?,
            tags,
            provider_id: row.get(4)?,
            model: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ─── Seed Pre-populated Providers ─────────────────────────────────────────────

pub fn seed_default_providers(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM providers", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let providers = default_providers();
    for p in &providers {
        let id = format!("builtin-{}", p.name.to_lowercase().replace(' ', "-").replace('/', "-").replace('.', "-"));
        let now = Utc::now().to_rfc3339();
        let models_json = serde_json::to_string(&p.models)?;

        conn.execute(
            "INSERT OR IGNORE INTO providers (id, name, category, base_url, auth_type, api_key_env, models, enabled, is_local, description, docs_url, logo_emoji, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                id, p.name, p.category, p.base_url, p.auth_type, p.api_key_env,
                models_json, p.enabled as i64, p.is_local as i64,
                p.description, p.docs_url, p.logo_emoji, now, now,
            ],
        )?;
    }

    Ok(())
}

struct ProviderSeed {
    name: &'static str,
    category: &'static str,
    base_url: &'static str,
    auth_type: &'static str,
    api_key_env: &'static str,
    models: Vec<&'static str>,
    is_local: bool,
    enabled: bool,
    description: &'static str,
    docs_url: &'static str,
    logo_emoji: &'static str,
}

fn default_providers() -> Vec<ProviderSeed> {
    vec![
        // ── LLMs ──────────────────────────────────────────────────────────
        ProviderSeed { name: "OpenAI", category: "llm", base_url: "https://api.openai.com/v1", auth_type: "bearer", api_key_env: "OPENAI_API_KEY", models: vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo", "o1", "o1-mini", "o3-mini"], is_local: false, enabled: false, description: "OpenAI GPT-4o, o1, and o3 models", docs_url: "https://platform.openai.com/docs", logo_emoji: "🟢" },
        ProviderSeed { name: "Anthropic", category: "llm", base_url: "https://api.anthropic.com/v1", auth_type: "x-api-key", api_key_env: "ANTHROPIC_API_KEY", models: vec!["claude-opus-4-5", "claude-sonnet-4-5", "claude-3-5-haiku-20241022", "claude-3-opus-20240229"], is_local: false, enabled: false, description: "Claude 3.5 and Claude 4 family", docs_url: "https://docs.anthropic.com", logo_emoji: "🧡" },
        ProviderSeed { name: "Google Gemini", category: "llm", base_url: "https://generativelanguage.googleapis.com/v1beta", auth_type: "api-key-param", api_key_env: "GEMINI_API_KEY", models: vec!["gemini-2.0-flash-exp", "gemini-2.0-pro", "gemini-1.5-pro", "gemini-1.5-flash"], is_local: false, enabled: false, description: "Google Gemini 2.0 and 1.5 family", docs_url: "https://ai.google.dev/docs", logo_emoji: "💙" },
        ProviderSeed { name: "Grok (xAI)", category: "llm", base_url: "https://api.x.ai/v1", auth_type: "bearer", api_key_env: "GROK_API_KEY", models: vec!["grok-3", "grok-3-mini", "grok-2-1212", "grok-2-vision-1212"], is_local: false, enabled: false, description: "Grok 3 from xAI with real-time X data", docs_url: "https://docs.x.ai", logo_emoji: "⚡" },
        ProviderSeed { name: "OpenRouter", category: "llm", base_url: "https://openrouter.ai/api/v1", auth_type: "bearer", api_key_env: "OPENROUTER_API_KEY", models: vec!["openai/gpt-4o", "anthropic/claude-3-5-sonnet", "google/gemini-2.0-flash-exp", "meta-llama/llama-3.3-70b-instruct", "deepseek/deepseek-r1", "mistralai/mistral-large-2411", "cohere/command-r-plus", "qwen/qwq-32b-preview", "nvidia/llama-3.1-nemotron-70b-instruct"], is_local: false, enabled: false, description: "Unified gateway to 200+ models", docs_url: "https://openrouter.ai/docs", logo_emoji: "🔀" },
        ProviderSeed { name: "Ollama (Local)", category: "llm", base_url: "http://localhost:11434/api", auth_type: "none", api_key_env: "OLLAMA_HOST", models: vec!["llama3.3", "llama3.2", "mistral", "qwen2.5", "deepseek-r1", "phi4", "gemma2", "codellama", "mixtral"], is_local: true, enabled: true, description: "Local LLM runtime — runs 100% offline", docs_url: "https://ollama.ai", logo_emoji: "🦙" },
        ProviderSeed { name: "Ollama Cloud", category: "llm", base_url: "https://api.ollama.ai/v1", auth_type: "bearer", api_key_env: "OLLAMA_CLOUD_API_KEY", models: vec!["llama3.3-70b", "mistral-large", "qwen2.5-72b"], is_local: false, enabled: false, description: "Managed Ollama-compatible cloud endpoint", docs_url: "https://ollama.ai/cloud", logo_emoji: "☁️" },
        ProviderSeed { name: "DeepSeek", category: "llm", base_url: "https://api.deepseek.com/v1", auth_type: "bearer", api_key_env: "DEEPSEEK_API_KEY", models: vec!["deepseek-reasoner", "deepseek-chat", "deepseek-coder"], is_local: false, enabled: false, description: "DeepSeek R1 reasoning and chat models", docs_url: "https://platform.deepseek.com/docs", logo_emoji: "🌊" },
        ProviderSeed { name: "Mistral AI", category: "llm", base_url: "https://api.mistral.ai/v1", auth_type: "bearer", api_key_env: "MISTRAL_API_KEY", models: vec!["mistral-large-latest", "mistral-small-latest", "codestral-latest", "pixtral-large-latest"], is_local: false, enabled: false, description: "Mistral Large, Small, and Codestral", docs_url: "https://docs.mistral.ai", logo_emoji: "🌬️" },
        ProviderSeed { name: "Cohere", category: "llm", base_url: "https://api.cohere.ai/v1", auth_type: "bearer", api_key_env: "COHERE_API_KEY", models: vec!["command-r-plus-08-2024", "command-r-08-2024", "command-light"], is_local: false, enabled: false, description: "Cohere Command R+ with RAG support", docs_url: "https://docs.cohere.com", logo_emoji: "🔵" },
        ProviderSeed { name: "GLM / Zhipu AI", category: "llm", base_url: "https://open.bigmodel.cn/api/paas/v4", auth_type: "bearer", api_key_env: "ZHIPU_API_KEY", models: vec!["glm-4-plus", "glm-4-air", "glm-4-flash", "cogvideox"], is_local: false, enabled: false, description: "GLM-4 bilingual LLM + CogVideoX", docs_url: "https://open.bigmodel.cn/dev/howuse/overview", logo_emoji: "🧠" },
        ProviderSeed { name: "Together AI", category: "llm", base_url: "https://api.together.xyz/v1", auth_type: "bearer", api_key_env: "TOGETHER_API_KEY", models: vec!["meta-llama/Llama-3.3-70B-Instruct-Turbo", "mistralai/Mixtral-8x7B-Instruct-v0.1", "Qwen/QwQ-32B-Preview"], is_local: false, enabled: false, description: "Open-source model inference at scale", docs_url: "https://docs.together.ai", logo_emoji: "🤝" },
        ProviderSeed { name: "Groq", category: "llm", base_url: "https://api.groq.com/openai/v1", auth_type: "bearer", api_key_env: "GROQ_API_KEY", models: vec!["llama-3.3-70b-versatile", "llama-3.1-8b-instant", "gemma2-9b-it", "mixtral-8x7b-32768"], is_local: false, enabled: false, description: "Ultra-fast LPU inference", docs_url: "https://console.groq.com/docs", logo_emoji: "⚡" },
        ProviderSeed { name: "Perplexity AI", category: "llm", base_url: "https://api.perplexity.ai", auth_type: "bearer", api_key_env: "PERPLEXITY_API_KEY", models: vec!["sonar-pro", "sonar", "sonar-reasoning-pro"], is_local: false, enabled: false, description: "Web-grounded search-augmented LLM", docs_url: "https://docs.perplexity.ai", logo_emoji: "🔍" },

        // ── Image Generation ──────────────────────────────────────────────
        ProviderSeed { name: "Fal.ai", category: "image", base_url: "https://fal.run", auth_type: "key-secret", api_key_env: "FAL_KEY", models: vec!["fal-ai/flux/dev", "fal-ai/flux/schnell", "fal-ai/stable-diffusion-xl", "fal-ai/aura-flow", "fal-ai/kolors", "fal-ai/hidream-i1-full"], is_local: false, enabled: false, description: "Fal.ai fast image & video inference platform", docs_url: "https://fal.ai/docs", logo_emoji: "🎨" },
        ProviderSeed { name: "Stability AI", category: "image", base_url: "https://api.stability.ai/v2beta", auth_type: "bearer", api_key_env: "STABILITY_API_KEY", models: vec!["stable-image/generate/core", "stable-image/generate/ultra", "stable-image/edit/inpaint", "stable-image/control/structure"], is_local: false, enabled: false, description: "Stable Diffusion 3.5 and SDXL", docs_url: "https://platform.stability.ai/docs", logo_emoji: "🖼️" },
        ProviderSeed { name: "Replicate", category: "image", base_url: "https://api.replicate.com/v1", auth_type: "bearer", api_key_env: "REPLICATE_API_KEY", models: vec!["black-forest-labs/flux-1.1-pro", "stability-ai/sdxl", "recraft-ai/recraft-v3", "ideogram-ai/ideogram-v2"], is_local: false, enabled: false, description: "Run any AI model in the cloud", docs_url: "https://replicate.com/docs", logo_emoji: "🔁" },
        ProviderSeed { name: "Ideogram", category: "image", base_url: "https://api.ideogram.ai/generate", auth_type: "api-key-header", api_key_env: "IDEOGRAM_API_KEY", models: vec!["V_2_TURBO", "V_2", "V_1_TURBO"], is_local: false, enabled: false, description: "Text-to-image with accurate typography", docs_url: "https://ideogram.ai/api/docs", logo_emoji: "✍️" },

        // ── Video Generation ──────────────────────────────────────────────
        ProviderSeed { name: "Runway ML", category: "video", base_url: "https://api.dev.runwayml.com/v1", auth_type: "bearer", api_key_env: "RUNWAY_API_KEY", models: vec!["gen3a_turbo", "gen3a", "gen2"], is_local: false, enabled: false, description: "Gen-3 Alpha video generation", docs_url: "https://docs.dev.runwayml.com", logo_emoji: "🎬" },
        ProviderSeed { name: "Kling AI", category: "video", base_url: "https://api.klingai.com/v1", auth_type: "bearer", api_key_env: "KLING_API_KEY", models: vec!["kling-v1.6-pro", "kling-v1.6-standard", "kling-v1.5-pro"], is_local: false, enabled: false, description: "Kling video generation by Kuaishou", docs_url: "https://platform.klingai.com/docs", logo_emoji: "🎥" },
        ProviderSeed { name: "Luma AI (Dream Machine)", category: "video", base_url: "https://api.lumalabs.ai/dream-machine/v1", auth_type: "bearer", api_key_env: "LUMA_API_KEY", models: vec!["ray-2", "ray-flash-2", "ray-1-6"], is_local: false, enabled: false, description: "Dream Machine text-to-video and image-to-video", docs_url: "https://lumalabs.ai/dream-machine/api/docs", logo_emoji: "🌙" },
        ProviderSeed { name: "Pika Labs", category: "video", base_url: "https://api.pika.art/v1", auth_type: "bearer", api_key_env: "PIKA_API_KEY", models: vec!["pika-2.2", "pika-2.1", "pika-1.5"], is_local: false, enabled: false, description: "Pika text-to-video with motion brush", docs_url: "https://pika.art/api", logo_emoji: "⚡" },
        ProviderSeed { name: "Sora (OpenAI)", category: "video", base_url: "https://api.openai.com/v1", auth_type: "bearer", api_key_env: "OPENAI_API_KEY", models: vec!["sora"], is_local: false, enabled: false, description: "OpenAI Sora video generation", docs_url: "https://platform.openai.com/docs/api-reference", logo_emoji: "🌅" },
        ProviderSeed { name: "Minimax Video", category: "video", base_url: "https://api.minimax.io/v1", auth_type: "bearer", api_key_env: "MINIMAX_API_KEY", models: vec!["video-01", "video-01-live2d"], is_local: false, enabled: false, description: "Hailuo video generation by MiniMax", docs_url: "https://platform.minimaxi.com/document/video-generation", logo_emoji: "🎦" },

        // ── Audio Generation ──────────────────────────────────────────────
        ProviderSeed { name: "ElevenLabs", category: "audio", base_url: "https://api.elevenlabs.io/v1", auth_type: "xi-api-key", api_key_env: "ELEVENLABS_API_KEY", models: vec!["eleven_multilingual_v2", "eleven_turbo_v2_5", "eleven_flash_v2_5", "eleven_monolingual_v1"], is_local: false, enabled: false, description: "Ultra-realistic AI voice cloning and TTS", docs_url: "https://elevenlabs.io/docs/api-reference", logo_emoji: "🔊" },
        ProviderSeed { name: "OpenAI TTS", category: "audio", base_url: "https://api.openai.com/v1", auth_type: "bearer", api_key_env: "OPENAI_API_KEY", models: vec!["tts-1-hd", "tts-1", "whisper-1"], is_local: false, enabled: false, description: "OpenAI TTS and Whisper speech models", docs_url: "https://platform.openai.com/docs/api-reference/audio", logo_emoji: "🎙️" },
        ProviderSeed { name: "Suno AI", category: "audio", base_url: "https://studio-api.suno.ai/api", auth_type: "bearer", api_key_env: "SUNO_API_KEY", models: vec!["chirp-v3-5", "chirp-v3-0"], is_local: false, enabled: false, description: "AI music generation", docs_url: "https://suno.com", logo_emoji: "🎵" },
        ProviderSeed { name: "Udio", category: "audio", base_url: "https://www.udio.com/api", auth_type: "bearer", api_key_env: "UDIO_API_KEY", models: vec!["udio-130"], is_local: false, enabled: false, description: "AI music and song generation", docs_url: "https://udio.com", logo_emoji: "🎶" },
        ProviderSeed { name: "AssemblyAI", category: "audio", base_url: "https://api.assemblyai.com/v2", auth_type: "api-key-header", api_key_env: "ASSEMBLYAI_API_KEY", models: vec!["best", "nano", "slam-1"], is_local: false, enabled: false, description: "Speech-to-text and audio intelligence", docs_url: "https://www.assemblyai.com/docs", logo_emoji: "📝" },

        // ── Search & Research ─────────────────────────────────────────────
        ProviderSeed { name: "Firecrawl", category: "search", base_url: "https://api.firecrawl.dev/v1", auth_type: "bearer", api_key_env: "FIRECRAWL_API_KEY", models: vec!["scrape", "crawl", "map", "extract", "search"], is_local: false, enabled: false, description: "Web scraping and crawling with LLM-ready output", docs_url: "https://docs.firecrawl.dev", logo_emoji: "🔥" },
        ProviderSeed { name: "Perplexica (Self-hosted)", category: "search", base_url: "http://localhost:3001/api", auth_type: "none", api_key_env: "PERPLEXICA_URL", models: vec!["webSearch", "academicSearch", "writingAssistant", "youtubeSearch"], is_local: true, enabled: false, description: "Open-source AI search engine", docs_url: "https://github.com/ItzCrazyKns/Perplexica", logo_emoji: "🔍" },
        ProviderSeed { name: "Tavily", category: "search", base_url: "https://api.tavily.com/v1", auth_type: "api-key-body", api_key_env: "TAVILY_API_KEY", models: vec!["search", "extract"], is_local: false, enabled: false, description: "AI search API for agents", docs_url: "https://docs.tavily.com", logo_emoji: "🗺️" },
        ProviderSeed { name: "Brave Search", category: "search", base_url: "https://api.search.brave.com/res/v1", auth_type: "api-key-header", api_key_env: "BRAVE_API_KEY", models: vec!["web/search", "news/search"], is_local: false, enabled: false, description: "Privacy-first web search API", docs_url: "https://api.search.brave.com/app/documentation/web-search", logo_emoji: "🦁" },
        ProviderSeed { name: "SerpAPI", category: "search", base_url: "https://serpapi.com/search", auth_type: "api-key-param", api_key_env: "SERPAPI_KEY", models: vec!["google", "bing", "duckduckgo", "scholar"], is_local: false, enabled: false, description: "Google, Bing, and DuckDuckGo search APIs", docs_url: "https://serpapi.com/docs", logo_emoji: "🐍" },
        ProviderSeed { name: "You.com", category: "search", base_url: "https://api.ydc-index.io", auth_type: "bearer", api_key_env: "YOU_API_KEY", models: vec!["web", "news", "research"], is_local: false, enabled: false, description: "AI-powered search and research API", docs_url: "https://documentation.you.com", logo_emoji: "🔵" },

        // ── Automation & Agents ───────────────────────────────────────────
        ProviderSeed { name: "Apify", category: "automation", base_url: "https://api.apify.com/v2", auth_type: "bearer", api_key_env: "APIFY_TOKEN", models: vec!["web-scraper", "browser-scraper", "apify/web-scraper", "apify/cheerio-scraper"], is_local: false, enabled: false, description: "Web scraping and automation platform with 1500+ actors", docs_url: "https://docs.apify.com", logo_emoji: "🕷️" },
        ProviderSeed { name: "Browserbase", category: "automation", base_url: "https://api.browserbase.com/v1", auth_type: "x-bb-api-key", api_key_env: "BROWSERBASE_API_KEY", models: vec!["browser/create", "browser/navigate", "browser/screenshot"], is_local: false, enabled: false, description: "Headless browser automation cloud", docs_url: "https://docs.browserbase.com", logo_emoji: "🌐" },
        ProviderSeed { name: "E2B (Code Interpreter)", category: "automation", base_url: "https://api.e2b.dev/v1", auth_type: "e2b-api-key", api_key_env: "E2B_API_KEY", models: vec!["code-interpreter-v1", "desktop"], is_local: false, enabled: false, description: "Secure sandboxed code execution", docs_url: "https://e2b.dev/docs", logo_emoji: "💻" },
        ProviderSeed { name: "Make (Integromat)", category: "automation", base_url: "https://hook.eu1.make.com", auth_type: "bearer", api_key_env: "MAKE_API_KEY", models: vec!["webhook", "scenario"], is_local: false, enabled: false, description: "Visual automation and workflow builder", docs_url: "https://www.make.com/en/api-documentation", logo_emoji: "🔧" },

        // ── Phone / Voice / Communication ─────────────────────────────────
        ProviderSeed { name: "Twilio", category: "voice", base_url: "https://api.twilio.com/2010-04-01", auth_type: "basic", api_key_env: "TWILIO_AUTH_TOKEN", models: vec!["calls", "messages", "voice-intelligence"], is_local: false, enabled: false, description: "SMS, voice, and communication APIs", docs_url: "https://www.twilio.com/docs", logo_emoji: "📞" },
        ProviderSeed { name: "Vapi AI", category: "voice", base_url: "https://api.vapi.ai", auth_type: "bearer", api_key_env: "VAPI_API_KEY", models: vec!["assistant", "call", "phone-number"], is_local: false, enabled: false, description: "Voice AI for phone agents and calls", docs_url: "https://docs.vapi.ai", logo_emoji: "🤖" },
        ProviderSeed { name: "Bland AI", category: "voice", base_url: "https://api.bland.ai/v1", auth_type: "authorization", api_key_env: "BLAND_API_KEY", models: vec!["calls", "agent", "pathway"], is_local: false, enabled: false, description: "AI phone calls at scale", docs_url: "https://docs.bland.ai", logo_emoji: "📱" },
        ProviderSeed { name: "Deepgram", category: "voice", base_url: "https://api.deepgram.com/v1", auth_type: "token", api_key_env: "DEEPGRAM_API_KEY", models: vec!["nova-2", "nova-2-medical", "whisper-cloud"], is_local: false, enabled: false, description: "Real-time speech-to-text and TTS", docs_url: "https://developers.deepgram.com/docs", logo_emoji: "🎤" },

        // ── MCP Tools ─────────────────────────────────────────────────────
        ProviderSeed { name: "MCP Sequential Thinking", category: "mcp", base_url: "npx:@modelcontextprotocol/server-sequential-thinking", auth_type: "none", api_key_env: "", models: vec!["think", "reflect"], is_local: true, enabled: false, description: "Step-by-step reasoning via MCP protocol", docs_url: "https://github.com/modelcontextprotocol/servers", logo_emoji: "🧩" },
        ProviderSeed { name: "MCP Filesystem", category: "mcp", base_url: "npx:@modelcontextprotocol/server-filesystem", auth_type: "none", api_key_env: "", models: vec!["read_file", "write_file", "list_directory"], is_local: true, enabled: false, description: "Local filesystem access via MCP", docs_url: "https://github.com/modelcontextprotocol/servers", logo_emoji: "📁" },
        ProviderSeed { name: "MCP Memory", category: "mcp", base_url: "npx:@modelcontextprotocol/server-memory", auth_type: "none", api_key_env: "", models: vec!["create_entities", "search_nodes", "add_observations"], is_local: true, enabled: false, description: "Persistent entity memory via MCP", docs_url: "https://github.com/modelcontextprotocol/servers", logo_emoji: "🧠" },
        ProviderSeed { name: "MCP GitHub", category: "mcp", base_url: "npx:@modelcontextprotocol/server-github", auth_type: "bearer", api_key_env: "GITHUB_TOKEN", models: vec!["list_repos", "create_issue", "get_file_contents", "push_files"], is_local: false, enabled: false, description: "GitHub operations via MCP protocol", docs_url: "https://github.com/modelcontextprotocol/servers", logo_emoji: "🐙" },

        // ── Database / Storage ────────────────────────────────────────────
        ProviderSeed { name: "Supabase", category: "database", base_url: "https://YOUR_PROJECT.supabase.co/rest/v1", auth_type: "apikey", api_key_env: "SUPABASE_ANON_KEY", models: vec!["postgres", "storage", "auth", "realtime"], is_local: false, enabled: false, description: "Open-source Firebase alternative with Postgres", docs_url: "https://supabase.com/docs", logo_emoji: "🟩" },
        ProviderSeed { name: "Pinecone", category: "database", base_url: "https://api.pinecone.io", auth_type: "api-key-header", api_key_env: "PINECONE_API_KEY", models: vec!["upsert", "query", "fetch", "delete"], is_local: false, enabled: false, description: "Vector database for AI embeddings", docs_url: "https://docs.pinecone.io", logo_emoji: "🌲" },
        ProviderSeed { name: "Weaviate", category: "database", base_url: "http://localhost:8080/v1", auth_type: "none", api_key_env: "WEAVIATE_URL", models: vec!["batch", "graphql", "objects"], is_local: true, enabled: false, description: "Open-source vector search engine", docs_url: "https://weaviate.io/developers/weaviate", logo_emoji: "⚡" },
    ]
}
