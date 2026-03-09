# AmitOS — Universal AI OS

> **One surface to deploy, research, remember, and control every device.**
> Designed for ADHD and dyslexia: scan in 5 seconds, neon clarity, big touch targets.

[![CI](https://github.com/your-org/amitos/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/amitos/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/your-org/amitos?color=7c3aed)](https://github.com/your-org/amitos/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-cyan.svg)](LICENSE)

---

## ⬇️ One-Click Install

| Platform | Download |
|----------|----------|
| **macOS** (Universal .dmg) | [AmitOS_1.0.0_universal.dmg](https://github.com/your-org/amitos/releases/latest) |
| **Windows** (.exe installer) | [AmitOS_1.0.0_x64-setup.exe](https://github.com/your-org/amitos/releases/latest) |
| **Linux** (.AppImage) | [AmitOS_1.0.0_amd64.AppImage](https://github.com/your-org/amitos/releases/latest) |
| **Android APK** (Panda companion) | [AmitOS_1.0.0.apk](https://github.com/your-org/amitos/releases/latest) |
| **Raspberry Pi ARM64** | [amitos-arm64](https://github.com/your-org/amitos/releases/latest) |

**One-click RPi install:**
```bash
curl -fsSL https://raw.githubusercontent.com/your-org/amitos/main/scripts/build-rpi.sh | bash
```

---

## 🎙️ Voice Demo (30 seconds)

1. Open AmitOS → **Parallel Run** tab (or Voice + Chat)
2. Say: **"Run parallel"** — Superpowers starts coding your feature + Diffusionstudio edits your demo video _simultaneously_
3. Vy (desktop agent) watches your screen; Panda (phone) shows approval prompt
4. Tap **Allow** on your phone → both agents run in background
5. Memory Spine logs every action, Kaizen Board auto-creates tasks for review

**Or try individual voice commands:**
- `"launch superpowers"` — starts agentic coding framework
- `"start video edit"` — launches background video agent
- `"stop all"` — pauses everything immediately
- `"status"` — reads what's running

---

## What's Inside

| Feature | Description |
|---------|-------------|
| 📅 **Today Board** | Focus dashboard — energy-tagged tasks (🌱/⚡/🔥), one-tap status, minimum-version flag |
| 🖥️ **Vy Desktop Agent** | AI vision + mouse/keyboard takeover. Goal-driven with per-action approval |
| 🐼 **Panda Phone Control** | Android companion for remote approvals, voice capture, today-board at your thumb |
| ⚡ **Parallel Run** | Launch superpowers (coding) + diffusionstudio/agent (video) simultaneously |
| 🎙️ **Voice + Chat** | Web Speech API, intent parser, real-time chat UI + push notifications |
| ♾️ **Kaizen Tasks** | Rich domain/energy/today fields. Auto-created from every agent action |
| 🧠 **Memory Spine** | Every action logged with cost tracking. Searchable, persistent, exportable |
| ☁️ **VPS + RPi Deploy** | One-click SSH deploy to any server. Systemd daemon + WebSocket sync |
| 🔌 **MCP Browser** | Playwright, Firecrawl, GitHub MCP — toggle on/off, instant JSON config |
| 📱 **Mobile Sync** | QR-code pairing for real-time sync with Android companion |
| 🔑 **50+ API Providers** | Tauri Stronghold vault, zero plaintext. Auto-injected into every tool |
| 🛠️ **Tool Registry** | superpowers, diffusionstudio/agent, Firecrawl, Apify — 35+ tools pre-wired |

---

## Quick Start (3 minutes)

```bash
# 1. Clone
git clone https://github.com/your-org/amitos
cd amitos

# 2. Install Bun (fastest JS runtime)
curl -fsSL https://bun.sh/install | bash

# 3. Install deps + run desktop app
bun install
bun run tauri dev
```

**Or just download the installer and skip the build step.**

---

## Adding Your First API Key

1. Open AmitOS → **API Keys** tab
2. Search for your provider (50+ listed: OpenAI, Anthropic, Fal.ai, Firecrawl…)
3. Click the provider card → paste your API key → **Save**
4. Go to any tool → your key is auto-injected

---

## Vy Desktop Agent

Vy watches your screen and executes with explicit approval:

```
Start session → describe task → Vy takes screenshot
→ plans actions → each action requires your approval
→ executes (click, type, scroll) → logs to Memory Spine
→ creates Kaizen task for review
```

**Example tasks:**
- "Watch this tutorial and set up the dev environment"
- "Fill the W2 form in Excel using my tax documents"
- "Update all Notion project pages with today's progress"
- "Run my test suite and fix any failures"

---

## Panda Phone Control

Your phone becomes a remote control for AmitOS:

1. Download the Android APK (link above or QR in Mobile tab)
2. Open → scan QR code shown in AmitOS
3. See pending approval requests in real time
4. **One tap to approve or deny** any Vy desktop action
5. Voice capture: speak tasks → synced to desktop instantly

---

## Architecture

```
AmitOS (Tauri 2.0 + SvelteKit + Bun)
├── Frontend (SvelteKit 5, Svelte 5, Tailwind 4)
│   ├── Dashboard          — live system health, quick-launch features
│   ├── Today Board (/today) — ADHD-friendly focus board with energy tags
│   ├── Vy Agent (/vy)     — desktop AI takeover, tutorial extraction
│   ├── Parallel Run (/parallel) — superpowers + diffusionstudio simultaneously
│   ├── Panda (/panda)     — phone remote control + approval relay
│   ├── Voice + Chat (/voice) — Web Speech API + intent parser + push
│   ├── Kaizen (/kaizen)   — Kanban board with domain/energy/priority
│   ├── Memory (/memory)   — persistent knowledge spine + search
│   ├── MCP Browser (/mcp) — Playwright, Firecrawl, GitHub MCP toggles
│   ├── Tools (/tools)     — 35+ pre-wired AI tools with one-click launch
│   ├── Workflows (/workflows) — overnight chain execution composer
│   ├── Deploy (/deploy)   — one-click PC/Colab/VPS/RPi
│   ├── Mobile Sync (/mobile) — QR pairing + real-time WebSocket
│   └── API Keys (/settings) — 50+ provider vault
│
└── Backend (Rust, Tauri 2.0)
    ├── process_manager.rs — background tool launch, log streaming, stop/pause
    ├── kaizen.rs          — rich task commands (domains, is_today, energy, subtasks)
    ├── computer_agent.rs  — Vy vision + mouse/keyboard + parallel tasks
    ├── voice_assistant.rs — chat sessions + push notifications + intent router
    ├── provider_registry.rs — 50+ providers, usage logs, memory spine
    ├── orchestrator.rs    — deploy flows (PC, Colab, VPS, RPi)
    ├── tool_registry.rs   — manifest-driven launcher (category, parallel_capable, tags)
    ├── workflow.rs        — workflow run persistence
    └── state.rs / Stronghold — SQLite + zero-plaintext Argon2 key vault
```

---

## Builds

| Platform | Command | Output |
|----------|---------|--------|
| All desktop | `bun run tauri build` | .dmg, .exe, .AppImage |
| macOS universal | `bun run tauri build --target universal-apple-darwin` | Universal .dmg |
| Android APK | `bun run tauri android build --apk` | .apk |
| Raspberry Pi | `bash scripts/build-rpi.sh` | ARM64/ARMv7 binary |

CI/CD: GitHub Actions builds all platforms on every tag push.

---

## Test Coverage

**59 E2E tests** — all passing. Run with: `bun test tests/e2e.test.ts`

Verified end-to-end flows:
- ✓ Today Board — energy-tagged tasks, domain picker, minimum-version flag
- ✓ Voice command `"run parallel"` → superpowers + diffusionstudio launched simultaneously
- ✓ Vy desktop takeover → permission request → tutorial extraction → Kaizen auto-task
- ✓ Panda APK → ADB bridge → Android Accessibility Service → one-tap approval
- ✓ VPS deploy → systemd daemon → WebSocket sync
- ✓ RPi ARM64/ARMv7 deploy via SSH bash script
- ✓ MCP Playwright → browser control → screenshot capture
- ✓ 50+ API provider auto-injection into any background tool
- ✓ Memory Spine logs every action with cost tracking
- ✓ Parallel workflow engine — run_parallel_workflow → Memory + Kaizen auto-created

---

## License

MIT — see [LICENSE](LICENSE)

---

## Contributing

1. Fork → branch → PR
2. Run `bun run check` before submitting
3. All PRs run CI (frontend build + Rust clippy)

---

*AmitOS is built for people who need to do a lot, forget nothing, and control everything — from one beautiful surface.*
