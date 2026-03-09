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
        ),
        tool(
            "vibe-kanban",
            "vibe-kanban",
            "https://github.com/BloopAI/vibe-kanban",
            "Tracks work items and Ralph progress in a board-style workspace.",
            "bun run dev",
            vec!["OPENAI_API_KEY"],
            false,
        ),
        tool(
            "claudia",
            "claudia",
            "https://github.com/kbanc85/claudia",
            "Consolidates memory and agent notes into a durable workspace.",
            "bun run dev",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY"],
            false,
        ),
        tool(
            "claude-code-templates",
            "claude-code-templates",
            "https://github.com/davila7/claude-code-templates",
            "Bootstraps prompt and workflow templates for managed coding runs.",
            "bun run dev",
            vec!["ANTHROPIC_API_KEY"],
            false,
        ),
        tool(
            "perplexica",
            "Perplexica",
            "https://github.com/ItzCrazyKns/Perplexica",
            "Research engine for multi-tool overnight workflows.",
            "bun run dev",
            vec!["PERPLEXICA_KEYS", "OPENAI_API_KEY"],
            true,
        ),
        tool(
            "llm-council",
            "llm-council",
            "https://github.com/karpathy/llm-council",
            "Model voting layer for combined workflows and overnight loops.",
            "bun run dev",
            vec!["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GEMINI_API_KEY"],
            false,
        ),
        tool(
            "autoresearch",
            "autoresearch",
            "https://github.com/karpathy/autoresearch",
            "Iterative research runner wired into RalphHub orchestration.",
            "bun run dev",
            vec!["OPENAI_API_KEY", "ANTHROPIC_API_KEY"],
            false,
        ),
        tool(
            "open-in-code",
            "Open in Code",
            "internal://open-in-code",
            "Opens the active managed workspace in Cursor or VS Code with STATE.md ready.",
            "internal",
            Vec::new(),
            false,
        ),
        tool(
            "universal-ralph-loop",
            "Universal Ralph Loop",
            "internal://universal-ralph-loop",
            "Runs a model-selectable overnight Ralph loop across one or more managed tools.",
            "internal",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GEMINI_API_KEY", "GROK_API_KEY"],
            false,
        ),
        // ── Computer Control Tools (Vy-style) ─────────────────────────────────
        tool(
            "computer-agent",
            "Computer Agent (Desktop)",
            "https://github.com/suitedaces/computer-agent",
            "Vy-style vision loop: screenshot → LLM analysis → mouse/keyboard action. Runs in background while you work.",
            "internal://computer-control",
            vec!["ANTHROPIC_API_KEY", "OPENAI_API_KEY"],
            false,
        ),
        tool(
            "agent-s",
            "Agent-S (simular-ai)",
            "https://github.com/simular-ai/Agent-S",
            "Multi-step GUI agent with hierarchical planning and OS-level control.",
            "internal://computer-control",
            vec!["OPENAI_API_KEY"],
            true,
        ),
        tool(
            "cua",
            "CUA Vision Loop (trycua)",
            "https://github.com/trycua/cua",
            "Computer-Use Agent: lightweight screenshot-based tool-use loop with sandboxing.",
            "internal://computer-control",
            vec!["ANTHROPIC_API_KEY"],
            true,
        ),
        tool(
            "android-panda",
            "Android Panda (blurr)",
            "https://github.com/Ayush0Chaudhary/blurr",
            "Android Accessibility Service agent — full control over any app on your Android device.",
            "internal://computer-control/android",
            Vec::new(),
            false,
        ),
        tool(
            "tutorial-excel-workflow",
            "Tutorial + Excel Parallel Workflow",
            "internal://computer-control/workflows",
            "Example: Watch YouTube tutorial in foreground while agent does taxes in Excel + updates Notion in background.",
            "internal://computer-control",
            vec!["OPENAI_API_KEY"],
            false,
        ),
    ]
});

pub fn all_tools() -> Vec<ToolManifest> {
    BUILTIN_TOOLS.clone()
}

fn tool(
    id: &str,
    name: &str,
    repo_url: &str,
    description: &str,
    launch_command: &str,
    required_keys: Vec<&str>,
    needs_sandbox: bool,
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
    }
}
