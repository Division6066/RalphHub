# AmitOS Changelog

All notable changes are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

---

## [1.0.0] — 2026-03-09 🎉 Grand Finale — Fully Optimized, Debugged, and Ready to Ship

### Grand Finale Changes
- **Merged all branches** cleanly: amitos-final-polish, amitos-layer-foundation, browser-agent, mobile-companion, new-tools, wishlist-local-models
- **New routes added**: Kaizen Tasks, Memory Spine, MCP Browser, Mobile Sync, Panda Phone Control
- **Design system overhaul**: AmitOS CSS design tokens, ADHD-friendly typography, neon dark theme, smooth animations, big touch targets
- **Layout redesign**: Icon + badge navigation, violet/cyan gradient sidebar, model switcher, memory stats
- **CI/CD**: Fixed test job (no test runner referenced), all build targets validated
- **README**: Full rewrite with one-click install, voice demo, architecture diagram

---

## [1.0.0-rc1] — 2026-03-09 🎉 Initial Release

### Added — Core OS
- **Today Board** (`/today`): ADHD-friendly daily focus board with Minimum Version tasks, energy levels, time estimates, and one-click status cycle (Todo → In Progress → Done)
- **Kaizen OS** (`/kaizen`): Full life-domains task system (Work, Health, Learning, Creative, Relationships, Finance, Home, General) with drag-to-reorder, subtask decomposition, and priority system (Urgent → Low)
- **Memory Spine** (`/memory`): Capture, tag, and semantic-search persistent knowledge entries across 8 domains
- **Universal AI OS** dashboard with live stats (tasks, memories, API keys, workflows)

### Added — Agents
- **Vy Desktop Agent** (`/vy`): AI desktop takeover with explicit per-action approval flow. Screen watching, tutorial extraction, and command execution
- **Panda Phone Control** (`/panda`): Use your phone as a remote control — approve Vy actions, capture voice memos, check Today board from anywhere
- **Voice Mode** (`/voice`): Web Speech API command routing — say "open today", "kaizen", "memory", "settings" to navigate hands-free

### Added — AI Tools & Integrations
- **35+ AI Tool Registry**: One-click deploy of Perplexica, Aider, OpenHands, LiteLLM, LangChain, Firecrawl, Apify, Mem0, STORM, AutoResearch, Playwright MCP, and more
- **MCP Browser** (`/mcp`): Toggle Playwright MCP, Filesystem MCP, GitHub MCP, Brave Search MCP and generate Claude Desktop `claude_desktop_config.json`
- **50+ API Provider Registry**: Universal key manager with Stronghold vault — Anthropic, OpenAI, Gemini, Grok, Mistral, Cohere, Together AI, Groq, Perplexity, DeepSeek, Fireworks, OpenRouter, Replicate, HuggingFace, Stability AI, fal.ai, Brave, Serper, Tavily, Exa, Firecrawl, Apify, ElevenLabs, Deepgram, AssemblyAI, GitHub, Vercel, Supabase, Stripe, Notion, Linear, Airtable, and 20+ more
- **Workflow Composer** (`/workflows`): Chain AI tools into overnight loops — Research → Code → Memory → Notify

### Added — Backend (Tauri / Rust)
- **SQLite persistence** for all tasks, memories, domains, workflows, and API usage logs
- **Tauri Stronghold vault**: Encrypted key storage — zero plaintext secrets on disk
- **Kaizen Task decomposition**: One-click task → subtask expansion
- **Memory entry commands**: `list_memory_entries`, `create_memory_entry`, `search_memory`, `update_memory_entry`, `delete_memory_entry`
- **Mobile Sync server**: Local network QR-code pairing with WebSocket sync
- **Tool registry**: 35+ tool manifests with auto-install hints and Connect & Test flows
- **API provider registry**: Built-in providers loaded from `tool_registry::all_providers()`

### Added — Infrastructure
- **GitHub Actions CI** (`ci.yml`): Frontend build, type check, 37-test E2E suite, and Rust `cargo check` on every push
- **GitHub Actions Release** (`release.yml`): Cross-platform build → macOS (.dmg), Windows (.exe/.msi), Linux (.AppImage/.deb), Android APK, Raspberry Pi ARM64/ARMv7 binaries → automatic GitHub Release draft on tag push
- **E2E Test Suite** (`tests/e2e.test.ts`): 37 tests covering tool registry, API providers, voice commands, MCP config, workflow validation, Vy agent, Panda phone control, and VPS/RPi sync

### Design
- ADHD / dyslexia-friendly: 16px base font, 1.6 line-height, 0.01em letter-spacing, min 36px touch targets
- Dark neon design system: Violet + Cyan gradient, radial glow backgrounds, readable contrast
- Scan-in-5-seconds layout: Sidebar with emoji icons + NOW/NEW/AI/PHONE badges
- Reduced-motion support: Full `prefers-reduced-motion` compliance
- Keyboard navigation: Focus-visible outlines everywhere, skip-link ready

---

## Future Plans

- [ ] RPi background daemon mode (headless AmitOS server)
- [ ] Ollama local model auto-install (Mistral, Qwen, Llama, Phi)
- [ ] Faster-Whisper local STT + Piper local TTS for offline voice
- [ ] Electron-less web companion (serve from RPi/VPS, access from any browser)
- [ ] Notion / Linear two-way sync for Memory and Kaizen tasks
- [ ] Automated overnight loops (Ralph mode) with morning summary notifications
- [ ] Video demo export from Vy sessions
