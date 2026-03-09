# AmitOS — Universal AI Operating System

> **The single desktop surface for deploying AI tools, managing knowledge, staying focused, and shipping work — designed for ADHD and dyslexia.**

[![CI](https://github.com/amitos/amitos/actions/workflows/ci.yml/badge.svg)](https://github.com/amitos/amitos/actions)
[![Release](https://img.shields.io/github/v/release/amitos/amitos?color=violet)](https://github.com/amitos/amitos/releases)
[![Tests](https://img.shields.io/badge/tests-37%20passing-green)](#-test-suite)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

```
┌─────────────────────────────────────────────────────────────────┐
│  AmitOS  ·  Universal AI OS                                     │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐      │
│  │ ☀️ Today │ ♾️ Kaizen│ 🧠 Memory│ 🖥️  Vy  │ 🐼 Panda │      │
│  │  Board   │   Board  │  Spine   │  Agent   │  Phone   │      │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘      │
│  35+ AI Tools · 50+ API Keys · Voice Mode · MCP Browser        │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✨ Features at a Glance

| Feature | Description |
|---------|-------------|
| **☀️ Today Board** | ADHD-friendly daily focus — pick Minimum Version tasks, track energy, ship daily |
| **♾️ Kaizen OS** | Full life-domains task system (Work, Health, Learning, Creative, and 4 more) |
| **🧠 Memory Spine** | Capture, tag, and full-text-search every insight you want to keep |
| **🖥️ Vy Desktop Agent** | AI agent that watches your screen and executes actions with your approval |
| **🐼 Panda Phone** | Remote-control AmitOS from your phone: approve Vy, capture voice, check today |
| **🛠️ 35+ AI Tools** | One-click deploy: Perplexica, Aider, OpenHands, LiteLLM, Firecrawl, and more |
| **🔑 50+ API Providers** | Universal key manager with Stronghold vault — never plaintext |
| **⚡ MCP Browser** | Toggle Playwright MCP, Firecrawl, GitHub MCP; generate Claude Desktop config |
| **🎙️ Voice Mode** | Say "open today" or "memory" to navigate hands-free |
| **⚡ Workflow Composer** | Chain tools into overnight loops: Research → Code → Memory → Notify |
| **📱 Mobile Sync** | QR-code pairing with your local network for on-the-go access |
| **🔐 Secure by Design** | Tauri Stronghold vault, zero plaintext secrets, explicit key injection |

---

## 🚀 Install in 30 Seconds

### macOS
```bash
# 1. Download AmitOS_1.0.0_universal.dmg from Releases
open AmitOS_1.0.0_universal.dmg
# 2. Drag AmitOS to Applications
# 3. Launch AmitOS — done!
```

### Windows
```bash
# 1. Download AmitOS_1.0.0_x64-setup.exe from Releases
# 2. Double-click to install
# 3. Launch AmitOS from Start menu
```

### Linux (.AppImage)
```bash
# 1. Download AmitOS_1.0.0_amd64.AppImage from Releases
chmod +x AmitOS_1.0.0_amd64.AppImage
./AmitOS_1.0.0_amd64.AppImage
```

### Android APK (Panda companion app)
```bash
# 1. Download AmitOS.apk from Releases
# 2. On your phone: Settings → Security → Enable "Install from unknown sources"
# 3. Tap the APK → Install
# 4. Open AmitOS on desktop → Mobile Sync → scan QR code
```

### One-Click Raspberry Pi
```bash
# On your Raspberry Pi (ARM64 or ARMv7):
wget https://github.com/amitos/amitos/releases/latest/download/amitos-arm64
chmod +x amitos-arm64
./amitos-arm64

# Or use the installer script:
wget https://github.com/amitos/amitos/releases/latest/download/install-rpi.sh
bash install-rpi.sh
# Installs to /opt/amitos and creates a systemd service
```

---

## 🔑 Add Any API Key in 30 Seconds

1. Open AmitOS → click **🔑 API Keys** in the sidebar
2. Find your provider (50+ listed, searchable by name or category)
3. Click the provider card
4. Paste your API key
5. Click **Save Key Securely**

Keys are encrypted via Tauri Stronghold and never written to disk in plaintext.

**Supported providers include:** Anthropic, OpenAI, Google Gemini, Grok, Mistral, Cohere, Together AI, Groq, Perplexity, DeepSeek, Fireworks, OpenRouter, Replicate, HuggingFace, Stability AI, Midjourney, fal.ai, Brave Search, Serper, Tavily, Exa, Firecrawl, Apify, Slack, Discord, Telegram, ElevenLabs, Deepgram, AssemblyAI, GitHub, Vercel, Supabase, Stripe, Notion, Linear, Airtable, and 20 more.

---

## 🖥️ How to Use Vy — Desktop AI Agent

**Vy** watches your screen and can control your desktop — but asks for approval before every single action.

### Quick Start
1. Open AmitOS → click **🖥️ Vy Agent** in the sidebar
2. Go to the **Permissions** tab → click **Grant Vy Desktop Permission**
3. Back in **Desktop Takeover** tab → type your goal (e.g. "Set up my dev environment")
4. Click **▶ Start Vy**
5. Vy shows a plan → you click **✅ Approve All** or **Review One-by-One**
6. Press **⏹ Stop Vy** or `Escape` at any time to halt immediately

### Watch a Tutorial with Vy
1. Open **📹 Tutorial Watch** tab in Vy
2. Paste a YouTube or Loom URL
3. Click **▶ Watch with Vy**
4. Vy extracts every actionable step and adds them to your Kaizen board

### Vy Safety Model
- Every action is shown **before** execution — you approve or reject individually
- Screenshots are processed locally — nothing leaves your machine without consent
- Session permission is temporary — expires when you close AmitOS
- `Escape` key or Stop button halts Vy **immediately**

---

## 🐼 How to Use Panda — Phone Remote Control

**Panda** turns your phone into a remote control for AmitOS.

### Setup (3 steps)
1. Open AmitOS → click **📱 Mobile Sync** → click **Enable Sync**
2. On your phone: open a browser → scan the QR code shown
3. *(Optional)* Download the AmitOS APK for the native experience

### What You Can Do from Your Phone
| Action | How |
|--------|-----|
| **Approve Vy actions** | Tap ✅ or ✗ in the Panda Approvals tab |
| **Capture voice memo** | Open Panda → Voice Capture → tap 🎙️ and speak |
| **Check today's tasks** | Panda shows your Today board in real-time |
| **Trigger workflows** | Tap ⚡ to start an overnight research loop |
| **Get notifications** | Panda notifies you when workflows complete |

### Voice Capture Phrases
```
"Remember to..."      → Saved to Memory Spine
"Add task to do..."   → Added to Kaizen board
"Schedule tomorrow..." → Added to Today board
"Note that..."        → Saved as general note
"Idea about..."       → Saved to Learning domain
```

### Phone Voice Chat Permission
1. Open Panda → **Voice Capture** tab
2. Click **Grant Microphone Permission**
3. Allow microphone in your browser/phone settings
4. Tap 🎙️ to start capturing — speak naturally

---

## ♾️ Kaizen OS — Full System

### Today Board (`/today`)
- **Minimum Version** tasks — must-do, no excuses (highlighted red section)
- Regular today tasks with energy levels (🌱 Low / ⚡ Medium / 🔥 High) and time estimates
- One-click cycle: Todo → In Progress → Done
- Pick from your full backlog with the "Pick from Backlog" button

### Kaizen Board (`/kaizen`)
- 8 life domains: Work, Health, Learning, Creative, Relationships, Finance, Home, General
- Task priority (Urgent → Low), energy level, subtasks
- **Decompose** any task: click ⚡ → paste subtask titles → AmitOS creates them all linked to the parent
- Filter by domain, status, and today flag

---

## 🛠️ Tool Catalog (35+)

| Category | Tools |
|----------|-------|
| **Research** | Perplexica, AutoResearch, STORM |
| **Coding Agents** | OpenHands, Aider, Continue, Goose, Claudia |
| **Multi-Model** | LLM Council, LangChain, LiteLLM |
| **Browser** | Playwright MCP, Firecrawl, Stagehand |
| **Memory & RAG** | Mem0, Chroma, Qdrant, Weaviate |
| **Voice** | Faster-Whisper (STT), Piper (TTS) |
| **MCP Servers** | Playwright, Filesystem, GitHub, Brave Search |
| **Automation** | Apify, n8n, Flowise |
| **Dev Tools** | Cursor MCP, GitHub Copilot Workspace |

---

## ⚡ Workflow Composer

Chain AI tools into automated overnight pipelines.

### Example: Research + Code Loop
```
Step 1: Perplexica → research topic
Step 2: Memory Spine → save findings
Step 3: Aider → generate code based on research
Step 4: Memory Spine → save code result
Step 5: Notify → push summary to Slack
```

### Example: Voice → Research → Tasks
```
Step 1: Voice capture → "Research competitor pricing"
Step 2: Perplexica → deep research
Step 3: Kaizen → create tasks for each action item
Step 4: Memory → save research summary
```

---

## 📱 VPS / RPi Sync

### Deploy on a VPS (always-on AmitOS)
```bash
# 1. SSH into your VPS
ssh user@your-vps-ip

# 2. Download the Linux binary
wget https://github.com/amitos/amitos/releases/latest/download/amitos-amd64.AppImage
chmod +x amitos-amd64.AppImage

# 3. Create a systemd service
sudo tee /etc/systemd/system/amitos.service << 'EOF'
[Unit]
Description=AmitOS Universal AI OS
After=network.target

[Service]
Type=simple
User=ubuntu
ExecStart=/home/ubuntu/amitos-amd64.AppImage
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable --now amitos
```

### One-Click RPi Install
```bash
# Download and run the installer
wget https://github.com/amitos/amitos/releases/latest/download/install-rpi.sh
bash install-rpi.sh

# Runs on:
# - Raspberry Pi 4 / 5 (ARM64)
# - Raspberry Pi 3 / CM3 (ARMv7)
# - Raspberry Pi 2 (ARMv7)
```

---

## 🔐 Security Model

| Layer | What it protects |
|-------|-----------------|
| **Tauri Stronghold** | All API keys encrypted at rest — zero plaintext ever written to disk |
| **Memory scope** | Each key is readable only by the app process that wrote it |
| **Vy approval flow** | No desktop action executes without explicit user approval |
| **Mobile pairing** | QR code pairing is local-network-only by default |
| **CSP** | Tauri CSP enforced — no external resource injection |

---

## 🧪 Test Suite

AmitOS ships with a 37-test E2E suite (`tests/e2e.test.ts`):

```bash
bun run test
# ✓ tests/e2e.test.ts (37 tests)
```

Test coverage:
- Tool registry (all required tools present, correct categories, valid manifests)
- API provider registry (50+ providers, all LLMs, voice, search, data)
- Kaizen domain system (8 domains, hex colors, icons)
- Voice command recognition (all routes, case insensitive)
- MCP config generation (playwright, env vars, JSON validity)
- Workflow chain validation (name, tools, model validation)
- Vy Desktop Agent (permission check, session creation, voice goal parsing)
- Panda Phone Control (approval flow, voice capture classification)
- VPS/RPi sync (SSH/rsync command generation)

---

## 🏗️ Architecture

```
AmitOS/
├── src/                     # SvelteKit frontend
│   ├── routes/
│   │   ├── +page.svelte     # Dashboard
│   │   ├── today/           # Today Board (ADHD focus)
│   │   ├── kaizen/          # Kaizen task system
│   │   ├── memory/          # Memory Spine
│   │   ├── vy/              # Vy Desktop Agent
│   │   ├── panda/           # Panda Phone Control
│   │   ├── tools/           # AI Tool catalog
│   │   ├── workflows/       # Workflow Composer
│   │   ├── voice/           # Voice Mode
│   │   ├── mcp/             # MCP Browser
│   │   ├── mobile/          # Mobile Sync
│   │   ├── deploy/          # Deploy flows
│   │   └── settings/        # API Key Manager
│   ├── lib/
│   │   ├── utils/
│   │   │   ├── desktop.ts   # Tauri bridge (safe fallback for browser)
│   │   │   └── secure-store.ts # Stronghold key access
│   │   └── components/      # Shared UI components
│   └── app.css              # Design system (ADHD-friendly)
├── src-tauri/               # Rust backend
│   └── src/
│       ├── commands.rs      # Core IPC commands
│       ├── kaizen.rs        # Kaizen task system
│       ├── memory.rs        # Memory Spine
│       ├── mobile_sync.rs   # Mobile Sync server
│       ├── models.rs        # Shared data types
│       ├── state.rs         # App state + SQLite init
│       ├── tool_registry.rs # 35+ tool manifests
│       ├── orchestrator.rs  # Deploy orchestration
│       └── workflow.rs      # Workflow runs
├── tests/
│   └── e2e.test.ts         # 37-test E2E suite
├── .github/
│   ├── workflows/ci.yml    # CI: test + build on every push
│   └── workflows/release.yml # Release: macOS + Windows + Linux + Android + RPi
├── scripts/
│   └── build-rpi.sh        # Local RPi build script
└── CHANGELOG.md            # Full version history
```

**Stack:**
- **Frontend**: SvelteKit 5 + Tailwind CSS 4 + TypeScript
- **Backend**: Tauri 2 (Rust) + SQLite (bundled via rusqlite)
- **Security**: Tauri Stronghold (Argon2 key derivation)
- **Build**: Bun (no npm/yarn/pnpm)
- **Tests**: Vitest

---

## 🏃 Development

### Prerequisites
- [Bun](https://bun.sh) (required — no npm/yarn fallback)
- [Rust toolchain](https://rustup.rs) (for desktop builds)
- Linux: `libgtk-3-dev libwebkit2gtk-4.1-dev libssl-dev`

### Run in browser mode (fastest, no Rust needed)
```bash
bun install
bun run dev
# Open http://localhost:5173
```

### Run as desktop app (Tauri)
```bash
bun install
bun run tauri:dev
```

### Build for distribution
```bash
# Desktop (current platform)
bun run tauri:build

# Raspberry Pi ARM
bash scripts/build-rpi.sh

# All platforms (via CI)
git tag v1.0.1 && git push --tags
# GitHub Actions builds everything automatically
```

---

## 📸 Screenshots

> The following show AmitOS in action.

**Dashboard** — scan everything in 5 seconds:
```
┌── AmitOS ──────────────────────────────────────────────────────────────────┐
│ 🟣 AmitOS v1.0 — Ready                                                     │
│                                                                             │
│  Your Universal AI Operating System                                         │
│                                                                             │
│  [☀️ Start Today]  [♾️ Kaizen Board]  [🛠️ Launch Tools]                    │
│                                                                             │
│  Today tasks: 3   Memories: 47   API providers: 12   Workflows: 2          │
│                                                                             │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐ │
│  │ ☀️ Today   │ │♾️ Kaizen   │ │🧠 Memory   │ │🖥️ Vy NEW  │ │🐼Panda   │ │
│  │ Board      │ │ OS         │ │ Spine      │ │ Agent      │ │ NEW      │ │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └──────────┘ │
└────────────────────────────────────────────────────────────────────────────┘
```

**Today Board** — ADHD-friendly daily focus:
```
┌── ☀️ Today Board ──────────────────────────────────────────────────────────┐
│  Monday, March 9 · 3/5 complete                                            │
│                                                                             │
│  🔴 MINIMUM VERSION (must ship today)                                       │
│  ✅ Write landing page copy          ⚡ Medium  30 min   ████████░░ Done   │
│  ☐  Review PR #42                   🔥 High    45 min   Working…           │
│                                                                             │
│  📋 Rest of today                                                           │
│  ✅ Morning standup                  🌱 Low     15 min   Done               │
│  ☐  Evening walk 30 min             🌱 Low     30 min   Todo               │
│  ☐  Read chapter 3                  🌱 Low     20 min   Todo               │
│                                                                             │
│  [+ Add Task]  [Pick from Backlog]                                          │
└────────────────────────────────────────────────────────────────────────────┘
```

**Vy Agent** — desktop control with approval:
```
┌── 🖥️ Vy Desktop Agent ─────────────────────────────────────────────────────┐
│  ● Reviewing  Goal: "Set up dev environment"                               │
│                                                                             │
│  Session Log:                                                               │
│  🎯 Goal: "Set up dev environment"                                          │
│  👁️ Vy is observing your screen...                                          │
│  📸 3 screenshots captured                                                  │
│  🤔 Analysing current state...                                              │
│                                                                             │
│  ┌─── Vy has a plan — approve to continue: ─────────────────────────────┐  │
│  │  1. Open terminal in project folder                                   │  │
│  │  2. Run: bun install                                                  │  │
│  │  3. Open browser to localhost:5173                                    │  │
│  │  [✅ Approve All]  [Review One-by-One]                                │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `bun run test`
5. Build to verify: `bun run build`
6. Submit a PR

---

## 📄 License

MIT © AmitOS Contributors. See [LICENSE](LICENSE) for details.

---

*Built with [Tauri](https://tauri.app) · [SvelteKit](https://svelte.dev) · [Bun](https://bun.sh) · Designed for minds that work differently.*
