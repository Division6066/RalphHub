# AmitOS v1.0.0 — Initial Release

> The single desktop surface for deploying AI tools, managing knowledge, staying focused, and shipping work — designed for ADHD and dyslexia.

---

## ⬇️ Download

| Platform | File | Size |
|----------|------|------|
| macOS (Universal .dmg) | `AmitOS_1.0.0_universal.dmg` | ~15 MB |
| Windows (.exe installer) | `AmitOS_1.0.0_x64-setup.exe` | ~12 MB |
| Linux (.AppImage) | `AmitOS_1.0.0_amd64.AppImage` | ~14 MB |
| Android APK (Panda) | `AmitOS_1.0.0.apk` | ~8 MB |
| Raspberry Pi ARM64 | `amitos-arm64` | ~6 MB |
| Raspberry Pi ARMv7 | `amitos-armv7` | ~5 MB |
| RPi installer script | `install-rpi.sh` | 2 KB |

---

## 🆕 What's New in v1.0.0

### Core Features
- **☀️ Today Board** — ADHD-friendly daily focus with Minimum Version tasks, energy levels (Low/Medium/High), and time estimates
- **♾️ Kaizen OS** — Full life-domains task system with 8 domains (Work, Health, Learning, Creative, Relationships, Finance, Home, General), task decomposition, and drag-to-reorder
- **🧠 Memory Spine** — Capture, tag, and full-text-search persistent knowledge entries across all domains
- **⚡ Workflow Composer** — Chain AI tools into overnight research and coding pipelines

### New in 1.0: Vy & Panda
- **🖥️ Vy Desktop Agent** — AI agent that watches your screen and executes desktop actions with explicit per-action approval. Watch tutorials, extract steps, set up dev environments — without leaving your chair
- **🐼 Panda Phone Control** — Use your phone as a remote control: approve Vy actions from anywhere, capture voice memos, check Today board, get workflow notifications

### AI Integrations
- **35+ AI Tools** — One-click deploy: Perplexica, Aider, OpenHands, LiteLLM, LangChain, Firecrawl, Apify, Mem0, STORM, AutoResearch, Playwright MCP, and more
- **50+ API Providers** — Universal key manager: Anthropic, OpenAI, Gemini, Grok, Mistral, Cohere, Together AI, Groq, Perplexity, DeepSeek, Fireworks, OpenRouter, ElevenLabs, Deepgram, Firecrawl, Apify, and 34 more
- **🎙️ Voice Mode** — Web Speech API command routing — say "open today" or "memory" to navigate hands-free
- **⚡ MCP Browser** — Toggle Playwright MCP, Firecrawl, GitHub MCP; generate Claude Desktop `claude_desktop_config.json`

### Infrastructure
- Tauri Stronghold vault — zero plaintext secrets on disk
- SQLite persistence for all tasks, memories, workflows, and API usage logs
- Mobile Sync server with QR-code pairing
- GitHub Actions CI: 37 E2E tests on every push
- Release workflow: macOS + Windows + Linux + Android + Raspberry Pi

---

## 🚀 Install in 30 Seconds

### macOS
```bash
open AmitOS_1.0.0_universal.dmg
# Drag to Applications → Launch
```

### Windows
```bash
# Run AmitOS_1.0.0_x64-setup.exe → Install → Launch
```

### Linux
```bash
chmod +x AmitOS_1.0.0_amd64.AppImage
./AmitOS_1.0.0_amd64.AppImage
```

### Android (Panda)
```bash
# Enable "Install from unknown sources" → tap AmitOS_1.0.0.apk → Install
# Then: AmitOS desktop → Mobile Sync → scan QR code
```

### One-Click Raspberry Pi
```bash
wget https://github.com/amitos/amitos/releases/download/v1.0.0/install-rpi.sh
bash install-rpi.sh
```

---

## ❓ First Steps

1. Launch AmitOS
2. Click **🔑 API Keys** → add your first provider (e.g. Anthropic or OpenAI)
3. Click **☀️ Today** → add today's 3 tasks
4. Click **🛠️ Tools** → deploy Perplexica for AI research
5. Try **🖥️ Vy Agent** → grant permission → type "Set up a coding project"
6. Scan the QR code in **📱 Mobile Sync** to enable Panda

---

## 🔮 What's Coming in v1.1

- Local Ollama models (Mistral, Qwen, Llama, Phi) — fully offline LLM
- Faster-Whisper local STT + Piper local TTS for offline voice
- Notion / Linear two-way sync
- Automated overnight Ralph loops with morning summary

---

*Full changelog: [CHANGELOG.md](CHANGELOG.md)*
*Built with Tauri · SvelteKit · Bun · Rust*
