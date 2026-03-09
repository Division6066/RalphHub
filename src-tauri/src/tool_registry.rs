use once_cell::sync::Lazy;

use crate::models::ToolManifest;

pub static BUILTIN_TOOLS: Lazy<Vec<ToolManifest>> = Lazy::new(|| {
    vec![
        tool(
            "get-shit-done",
            "get-shit-done",
            "https://github.com/gsd-build/get-shit-done",
            "Runs the Ralph execution loop as a managed Bun workspace.",
            "bun run dev",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY"],
            false,
            "coding",
            false,
            "bun",
            vec!["execution", "loop", "ralph"],
        ),
        tool(
            "vibe-kanban",
            "vibe-kanban",
            "https://github.com/BloopAI/vibe-kanban",
            "Tracks work items and Ralph progress in a board-style workspace.",
            "bun run dev",
            vec!["OPENAI_API_KEY"],
            false,
            "productivity",
            false,
            "bun",
            vec!["kanban", "tasks", "board"],
        ),
        tool(
            "claudia",
            "claudia",
            "https://github.com/kbanc85/claudia",
            "Consolidates memory and agent notes into a durable workspace.",
            "bun run dev",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY"],
            false,
            "memory",
            false,
            "bun",
            vec!["memory", "consolidation", "notes"],
        ),
        tool(
            "claude-code-templates",
            "claude-code-templates",
            "https://github.com/davila7/claude-code-templates",
            "Bootstraps prompt and workflow templates for managed coding runs.",
            "bun run dev",
            vec!["ANTHROPIC_API_KEY"],
            false,
            "coding",
            false,
            "bun",
            vec!["templates", "prompts", "coding"],
        ),
        tool(
            "perplexica",
            "Perplexica",
            "https://github.com/ItzCrazyKns/Perplexica",
            "Research engine for multi-tool overnight workflows.",
            "bun run dev",
            vec!["PERPLEXICA_KEYS", "OPENAI_API_KEY"],
            true,
            "research",
            false,
            "bun",
            vec!["search", "research", "multi-tool"],
        ),
        tool(
            "llm-council",
            "llm-council",
            "https://github.com/karpathy/llm-council",
            "Model voting layer for combined workflows and overnight loops.",
            "bun run dev",
            vec!["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GEMINI_API_KEY"],
            false,
            "orchestration",
            true,
            "bun",
            vec!["voting", "multi-model", "orchestration"],
        ),
        tool(
            "autoresearch",
            "autoresearch",
            "https://github.com/karpathy/autoresearch",
            "Iterative research runner wired into RalphHub orchestration.",
            "bun run dev",
            vec!["OPENAI_API_KEY", "ANTHROPIC_API_KEY"],
            false,
            "research",
            true,
            "bun",
            vec!["research", "iterative", "automation"],
        ),
        tool(
            "open-in-code",
            "Open in Code",
            "internal://open-in-code",
            "Opens the active managed workspace in Cursor or VS Code with STATE.md ready.",
            "internal",
            Vec::new(),
            false,
            "internal",
            false,
            "internal",
            vec!["editor", "open", "workspace"],
        ),
        tool(
            "universal-ralph-loop",
            "Universal Ralph Loop",
            "internal://universal-ralph-loop",
            "Runs a model-selectable overnight Ralph loop across one or more managed tools.",
            "internal",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GEMINI_API_KEY", "GROK_API_KEY"],
            false,
            "internal",
            true,
            "internal",
            vec!["overnight", "loop", "automation"],
        ),
        // ── New Tool: Superpowers ──────────────────────────────────────────────
        tool(
            "superpowers",
            "Superpowers",
            "https://github.com/obra/superpowers",
            "Agentic skills framework & software development methodology for coding agents. \
             Features composable skills, mandatory TDD workflows, brainstorm → plan → execute → \
             review cycles, and dispatching-parallel-agents for simultaneous multi-agent execution.",
            "bun run dev",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY"],
            false,
            "coding",
            true,
            "bun",
            vec!["agentic", "skills", "tdd", "parallel-agents", "superpowers", "coding-methodology"],
        ),
        // ── New Tool: Diffusionstudio Agent ───────────────────────────────────
        tool(
            "diffusionstudio-agent",
            "Diffusionstudio Agent",
            "https://github.com/diffusionstudio/agent",
            "Agentic video editing framework with AI-driven composition, semantic documentation \
             search, and tool integration. Runs video workflows in background while you work, \
             integrates with Vy/Panda computer control for background video editing.",
            "bun run start",
            vec!["OPENAI_API_KEY", "FAL_KEY", "ANTHROPIC_API_KEY"],
            false,
            "video",
            true,
            "uv",
            vec!["video", "editing", "diffusion", "ai-composition", "background", "vy-panda"],
        ),
    ]
});

pub fn all_tools() -> Vec<ToolManifest> {
    BUILTIN_TOOLS.clone()
}

pub fn get_tool(id: &str) -> Option<ToolManifest> {
    BUILTIN_TOOLS.iter().find(|t| t.id == id).cloned()
}

#[allow(clippy::too_many_arguments)]
fn tool(
    id: &str,
    name: &str,
    repo_url: &str,
    description: &str,
    launch_command: &str,
    required_keys: Vec<&str>,
    needs_sandbox: bool,
    category: &str,
    parallel_capable: bool,
    install_method: &str,
    tags: Vec<&str>,
) -> ToolManifest {
    ToolManifest {
        id: id.to_string(),
        name: name.to_string(),
        repo_url: repo_url.to_string(),
        description: description.to_string(),
        launch_command: launch_command.to_string(),
        status: "pending".to_string(),
        open_in_code: true,
        needs_sandbox,
        required_keys: required_keys.into_iter().map(str::to_string).collect(),
        category: category.to_string(),
        parallel_capable,
        install_method: install_method.to_string(),
        tags: tags.into_iter().map(str::to_string).collect(),
    }
}
