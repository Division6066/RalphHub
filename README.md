# AmitOS — Universal AI Operating System

> **The single desktop surface for deploying AI tools, managing knowledge, staying focused, and shipping work — designed for ADHD and dyslexia.**

[![CI](https://github.com/amitos/amitos/actions/workflows/ci.yml/badge.svg)](https://github.com/amitos/amitos/actions)
[![Release](https://img.shields.io/github/v/release/amitos/amitos?color=violet)](https://github.com/amitos/amitos/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| **☀️ Today Board** | ADHD-friendly daily focus — pick your Minimum Version tasks and ship them |
| **♾️ Kaizen OS** | Full life-domains task system (Work, Health, Learning, Creative, and more) |
| **🧠 Memory Spine** | Capture, tag, and search everything you want to remember |
| **🛠️ 35+ AI Tools** | One-click deploy: Perplexica, Aider, OpenHands, LiteLLM, Firecrawl, and more |
| **🔑 50+ API Providers** | Universal key manager with Stronghold vault — never plaintext |
| **⚡ MCP Browser** | Toggle Playwright MCP, Firecrawl, GitHub MCP and generate Claude Desktop config |
| **🎙️ Voice Mode** | Web Speech API — say "open today" or "memory" to navigate hands-free |
| **⚡ Workflow Composer** | Chain AI tools into overnight loops: Research → Code → Memory → Notify |
| **📱 Mobile Sync** | QR-code pairing with your local network for on-the-go access |
| **🔐 Secure by Design** | Tauri Stronghold vault, zero plaintext secrets, explicit key injection |

---

## 🚀 Install in 30 Seconds

### macOS
```bash
# Download the .dmg from Releases
open AmitOS_1.0.0_universal.dmg
# Drag to Applications → Launch AmitOS
```

### Windows
```bash
# Download AmitOS_1.0.0_x64-setup.exe from Releases
# Double-click → Install → Launch
```

### Linux
```bash
# Download the .AppImage from Releases
chmod +x AmitOS_1.0.0_amd64.AppImage
./AmitOS_1.0.0_amd64.AppImage
```

### Android APK
```bash
# Download AmitOS.apk from Releases
# Enable "Install from unknown sources" in Settings
# Tap the APK to install
```

---

## 🔑 Add Any API Key in 30 Seconds

1. Open AmitOS → click **🔑 API Keys** in the sidebar
2. Find your provider (50+ listed, searchable)
3. Click the provider card
4. Paste your API key
5. Click **Save Key Securely**

Keys are encrypted via Tauri Stronghold and never written to disk in plaintext.

**Supported providers include:** Anthropic, OpenAI, Google Gemini, Grok, Mistral, Cohere, Together AI, Groq, Perplexity, DeepSeek, Fireworks, OpenRouter, Replicate, HuggingFace, Stability AI, Midjourney, fal.ai, Brave Search, Serper, Tavily, Exa, Firecrawl, Apify, Slack, Discord, Telegram, ElevenLabs, Deepgram, AssemblyAI, GitHub, Vercel, Supabase, Stripe, Notion, Linear, Airtable, and 20 more.

---

## ♾️ Kaizen OS — Full System

AmitOS implements the full Kaizen OS methodology:

### Today Board (`/today`)
- **Minimum Version** tasks — must-do, no excuses (red section)
- Regular today tasks with energy levels and time estimates
- One-click cycle: Todo → In Progress → Done
- Pick from backlog or add on the fly

### Kaizen Board (`/kaizen`)
- 8 life domains: Work, Health, Learning, Creative, Relationships, Finance, Home, General
- Task priority (Urgent → Low), energy level (Low/Medium/High)
- **Decompose** any task into subtasks with one click
- Filter by domain, status; toggle Today and Minimum Version flags

### Task Decomposition
Click **⚡** on any task → enter subtask titles (one per line) → AmitOS creates them all linked to the parent.

---

## 🛠️ Tool Catalog (35+)

| Category | Tools |
|----------|-------|
| **Research** | Perplexica, AutoResearch, STORM |
| **Coding Agents** | OpenHands, Aider, Continue, Goose, Get Shit Done, Claudia |
| **Multi-Model** | LLM Council, LangChain, LiteLLM |
| **Browser / Web** | Playwright MCP, Capture MCP, Stagehand, Firecrawl, Crawlee |
| **Memory** | Mem0, ChromaDB, Obsidian AI Bridge |
| **Voice** | Whisper (local), Whisper.cpp Web, ElevenLabs Voice |
| **MCP Servers** | Filesystem, GitHub, Brave Search, SQLite, Sequential Thinking |
| **Data** | Datasette, Evidence |
| **Design** | v0 (Vercel) |

All tools deploy with one click — AmitOS clones the repo, runs `bun install`, injects your API keys, and opens the workspace in your editor.

---

## ⚡ MCP Browser

Toggle individual MCP servers and AmitOS generates a ready-to-paste `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest"]
    },
    "firecrawl": {
      "command": "npx",
      "args": ["firecrawl-mcp@latest"],
      "env": { "FIRECRAWL_API_KEY": "${FIRECRAWL_API_KEY}" }
    }
  }
}
```

---

## 🎙️ Voice Mode

Say any of these commands (Chrome/Edge/Safari required):

| Say | Action |
|-----|--------|
| "open today" | Opens Today Board |
| "kaizen" or "tasks" | Opens Kaizen Board |
| "memory" or "remember" | Opens Memory Spine |
| "tools" | Opens Tool Catalog |
| "settings" or "api keys" | Opens API Key Manager |
| "dashboard" or "home" | Goes to Dashboard |

**Keyboard:** `Space` to start/stop, `Escape` to cancel.

---

## 🧠 Memory Spine

Capture anything:
- **Manual** notes and insights
- **Voice** transcriptions
- **AI-generated** summaries
- **Research** findings
- **Web** content
- **Book** highlights

Search full-text across all memories. Filter by domain. Edit and tag entries.

---

## ⚡ Workflow Composer

Pre-built chains:

| Preset | Chain |
|--------|-------|
| Deep Research Loop | Perplexica → AutoResearch → Memory Spine |
| Full Coding Agent | OpenHands → Aider → Get Shit Done |
| Web Scrape + Analyze | Playwright MCP → Firecrawl → LiteLLM |
| Multi-Model Council | LLM Council → LiteLLM |
| Overnight Kaizen | Memory Spine → Universal AI Loop |

---

## 🔒 Security

- **Tauri Stronghold**: All API keys encrypted at rest using Argon2-derived keys
- **Zero plaintext**: Keys never appear in logs, files, or environment variables
- **Explicit injection**: Keys only go to a workspace when you confirm
- **No telemetry**: AmitOS makes no outbound requests without your action

---

## 🏗️ Architecture

```
AmitOS/
├── src/                      # SvelteKit frontend
│   ├── routes/
│   │   ├── +page.svelte      # Dashboard
│   │   ├── today/            # Today Board
│   │   ├── kaizen/           # Kaizen OS
│   │   ├── memory/           # Memory Spine
│   │   ├── voice/            # Voice Mode
│   │   ├── tools/            # Tool Catalog
│   │   ├── workflows/        # Workflow Composer
│   │   ├── deploy/           # Deploy UI
│   │   ├── mcp/              # MCP Browser
│   │   ├── mobile/           # Mobile Sync
│   │   └── settings/         # API Key Manager
│   └── lib/utils/
│       ├── desktop.ts        # Tauri bridge
│       └── secure-store.ts   # Stronghold wrapper
├── src-tauri/src/            # Rust backend
│   ├── commands.rs           # Core commands
│   ├── kaizen.rs             # Kaizen CRUD
│   ├── memory.rs             # Memory CRUD
│   ├── mobile_sync.rs        # Mobile sync
│   ├── orchestrator.rs       # Deploy logic
│   ├── workflow.rs           # Workflow runs
│   ├── tool_registry.rs      # 35+ tool manifests + 50+ API providers
│   └── state.rs              # SQLite state
├── tests/
│   └── e2e.test.ts           # 23 passing tests
└── .github/workflows/
    ├── ci.yml                # PR checks
    └── release.yml           # Multi-platform builds
```

**Database**: SQLite via Rusqlite (bundled). Schema: `managed_projects`, `workflow_runs`, `kaizen_tasks`, `kaizen_domains`, `memory_entries`, `api_keys`, `overnight_loops`.

---

## 🛠️ Development

### Prerequisites
- [Bun](https://bun.sh) (only package manager used)
- [Rust](https://rustup.rs) stable
- Linux: `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev`

### Run in dev mode
```bash
bun install
bun run tauri:dev
```

### Build for production
```bash
bun run tauri:build
```

### Run tests
```bash
bun run test
```

---

## 📱 Android Development

```bash
bun run tauri android init
bun run tauri android dev
bun run tauri android build --apk
```

---

## 🤝 Contributing

1. Fork the repo
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `bun run test`
5. Submit a PR

---

## 📄 License

MIT — see [LICENSE](LICENSE)

---

## 🙏 Built With

- [Tauri v2](https://tauri.app) — Rust + WebView desktop shell
- [SvelteKit](https://kit.svelte.dev) — Frontend framework
- [Tailwind CSS v4](https://tailwindcss.com) — Styling
- [Rusqlite](https://github.com/rusqlite/rusqlite) — SQLite in Rust
- [Tauri Stronghold](https://github.com/tauri-apps/tauri-plugin-stronghold) — Encrypted secret storage

---

*AmitOS is built for humans who think differently. ADHD-friendly design, zero friction, everything in one place.*
