# RalphHub — Central AI Workstation

**Production-grade AI orchestration: desktop + mobile companion + Memory Spine.**

---

## Architecture Overview (Layers 0–5)

```
┌──────────────────────────────────────────────────────────────────────┐
│  LAYER 5 — External Integrations                                      │
│  Notion  ·  Google Colab  ·  GitHub  ·  VPS endpoints               │
├──────────────────────────────────────────────────────────────────────┤
│  LAYER 4 — Agent Layer                                                │
│  Browser Agent  ·  Ralph Overnight Loop  ·  Workflow Chains          │
│  → Every run writes to Memory Spine  →  Creates Kaizen tasks         │
├──────────────────────────────────────────────────────────────────────┤
│  LAYER 3 — Orchestration (Desktop Tauri App)                         │
│  Tool Registry  ·  Workflow Engine  ·  Mobile Sync HTTP Server       │
│  Deploy (PC / Colab)  ·  Editor Handoff  ·  Stronghold Vault        │
├──────────────────────────────────────────────────────────────────────┤
│  LAYER 2 — Memory Spine  ◄── CANONICAL SOURCE OF TRUTH              │
│  Tier 0: Raw Events (immutable append-only log)                      │
│  Tier 1: Working Memory (7d TTL, LRU eviction, 500 cap)             │
│  Tier 2: Long-Term Memory (summarized, categorized, Notion-synced)   │
│  Tier 3: Kaizen Tasks (auto-generated from errors/rejections)        │
├──────────────────────────────────────────────────────────────────────┤
│  LAYER 1 — Mobile Companion (Android APK via Capacitor)              │
│  Thin companion: Capture · Approvals · Monitoring · Voice            │
│  Local-first · AES-256 encrypted · Offline queue · LAN sync         │
├──────────────────────────────────────────────────────────────────────┤
│  LAYER 0 — Local Persistence                                         │
│  SQLite (ralphhub.db)  ·  Stronghold vault  ·  Encrypted localStorage│
└──────────────────────────────────────────────────────────────────────┘
```

---

## Canonical Memory Rule

**Every** agent action, Browser Agent run, API call, mobile capture, approval, and error
**MUST** write to the Memory Spine before being considered complete. The rule is:

```
ACTION → memory.write(raw_event) → working_memory → [optional] long_term → [optional] kaizen_task → [optional] notion_sync
```

No state lives only in memory or only in a database without a corresponding raw event.

---

## Mobile Companion

### Philosophy

The mobile companion is a **thin client** — it captures, monitors, and approves.
It does **not** replicate the full desktop orchestration. Desktop remains the brain.

### Mobile Screens

| Screen | Purpose |
|--------|---------|
| Home | Top 3 tasks + today's habits + pending approvals + running agents |
| Tasks | Full task list with priority filters and quick-add |
| Habits | Daily/weekly habit tracker with 7-day visual and streak |
| Capture | Quick text/task/voice capture → immediate sync to desktop |
| Approvals | Agent approval queue — approve/reject with single tap |
| Agents | Live agent monitoring with status badges |
| Digest | Daily summary with Memory Spine stats + Kaizen backlog |
| Voice | Full voice capture with Web Speech API (on-device STT) |

### Sync Architecture

```
Mobile Device                          Desktop (Tauri)
──────────────                         ───────────────
EncryptedStorage (AES-256)             SQLite (memory_raw)
OfflineQueue (localStorage)            SQLite (memory_working)
SyncEngine (vector clocks)    ←──→     SQLite (memory_long_term)
Web Speech API (STT)          REST     SQLite (kaizen_tasks)
SvelteKit PWA                 HTTP     Mobile Sync Server :7842
                              LAN
```

### Encrypted Sync Protocol

1. **Local-first**: All writes go to AES-256-GCM encrypted localStorage first
2. **Offline queue**: Events queued when desktop unreachable, drained on reconnect
3. **Vector clocks**: Detect concurrent edits across devices
4. **Conflict resolution**: Last-write-wins by default; server wins for safety on ties
5. **Pull polling**: Mobile polls desktop every 15s for new events
6. **Endpoint discovery**: QR code in Settings → Mobile encodes `http://{local-ip}:7842`

### APK Build

```bash
# Build SvelteKit
bun run build

# Sync to Android project
npx cap sync android

# Build APK (requires Android SDK)
cd android && ./gradlew assembleRelease

# Output: android/app/build/outputs/apk/release/app-release.apk
```

### Install via QR

1. Open RalphHub desktop → Settings → Mobile ↗
2. Scan the QR code with your Android phone
3. Download and install the APK
4. Open the app and scan the sync QR to connect

---

## Memory Spine API

### Tauri Commands (Desktop → Rust)

```typescript
// Write any event to memory
invoke('write_to_memory', {
  source: 'agent' | 'mobile' | 'desktop' | 'browser_agent' | 'api',
  eventType: 'task.create' | 'approval.submit' | 'agent.complete' | ...,
  payload: { ... },
  deviceId?: string,
  sessionId?: string,
  kaizenHint?: string,  // auto-creates Kaizen task if set
})

// Read raw events
invoke('read_memory', { since?: string, source?: string, limit?: number })

// List Kaizen tasks
invoke('list_kaizen_tasks', { status?: string })

// Get mobile server info (IP, port, QR URL)
invoke('get_mobile_server_info')
```

### REST API (Mobile → Desktop via LAN)

```
GET  /api/ping                         Health check
POST /api/sync/events                  Push sync event from mobile
GET  /api/sync/events?since=&deviceId  Pull events for mobile
POST /api/memory/write                 Write to memory spine
GET  /api/memory/read                  Read memory (raw + kaizen)
GET  /api/tasks                        Task list
GET  /api/approvals                    Pending approvals
POST /api/approvals/:id/resolve        Approve or reject
GET  /api/agents                       Agent run list
GET  /api/digest                       Daily digest
```

---

## Kaizen Integration

Every failure, rejection, or gap automatically generates a Kaizen task:

| Trigger | Kaizen Type | Default Priority |
|---------|------------|-----------------|
| Approval rejected on mobile | `approval_reject` | `high` |
| Agent run failed | `agent_error` | `urgent` |
| Voice capture not processed | `memory_gap` | `normal` |
| Habit missed 3+ days | `habit_miss` | `low` |
| Memory write with `kaizenHint` | `memory_gap` | `normal` |

Kaizen tasks are:
- Stored in `kaizen_tasks` SQLite table
- Shown in mobile Digest screen
- Synced to Notion (when Notion integration is active)
- Reviewed in next Ralph Loop iteration

---

## Agent → Memory → Notion Flow

```
Browser Agent runs
     ↓
Completes action (e.g., create Notion page)
     ↓
write_to_memory('browser_agent', 'agent.complete', { result, notionPageId })
     ↓
Raw event appended to memory_raw
Working memory entry created (7d TTL)
     ↓
Ralph Loop processes working memory
     ↓
Promotes to long_term (category: 'agent_outcome')
Syncs to Notion via Notion API
     ↓
Mobile pulls new events on next 15s poll
Mobile Digest screen updated
```

---

## End-to-End Test Scenario

**Test: Mobile capture → Desktop agent → Memory → Notion**

1. **Mobile**: Open Capture screen, type "Review memory schema" → tap Capture
2. **Mobile → Desktop**: `POST /api/sync/events` with `type: 'task.create'`
3. **Desktop**: `write_to_memory('mobile', 'task.create', { title: 'Review memory schema' })`
4. **Desktop**: Ralph Loop detects new task in working memory
5. **Desktop**: Browser Agent opens Notion, creates task page
6. **Desktop**: `write_to_memory('browser_agent', 'agent.complete', { notionPageId: 'abc123' })`
7. **Desktop**: Promotes to long_term memory with `notionPageId`
8. **Mobile**: Pulls updated events on next poll
9. **Mobile**: Digest screen shows "1 agent completed, 1 Notion task created"

**Validation commands:**
```bash
# Check memory event log
SELECT * FROM memory_raw ORDER BY timestamp DESC LIMIT 10;

# Check Kaizen backlog
SELECT * FROM kaizen_tasks WHERE status = 'backlog';

# Verify mobile server is up
curl http://localhost:7842/api/ping
```

---

## Project Structure

```
src/
  routes/
    /              → Desktop Dashboard
    /deploy        → PC + Colab deployment
    /tools         → Built-in tool manifests
    /workflows     → Workflow chain composer
    /settings      → API key manager (Stronghold)
    /mobile-download → APK download + QR sync code
    /mobile/       → Mobile companion (Capacitor target)
      +layout      → Mobile shell + bottom tab bar
      +page        → Home (top tasks + habits + approvals)
      /tasks        → Full task list
      /habits       → Habit tracker
      /capture      → Quick capture (text/task/voice)
      /approvals    → Approval queue
      /agents       → Agent monitoring
      /digest       → Daily digest + Memory Spine stats
      /voice        → Full voice capture
  lib/
    sync/          → Encrypted sync layer
      types.ts     → Canonical type definitions
      encrypted-storage.ts → AES-GCM localStorage
      offline-queue.ts     → Offline event queue
      conflict-resolver.ts → CRDT vector clock resolution
      sync-engine.ts       → Core sync orchestrator
    memory/        → Memory Spine (frontend)
      types.ts           → Memory type definitions
      memory-spine.ts    → In-memory + sync bridge
    mobile/
      store.svelte.ts → Svelte 5 runes global state
    utils/
      mobile-server.ts → Tauri command wrappers
      desktop.ts       → Tauri runtime detection
      secure-store.ts  → Stronghold vault CRUD

src-tauri/src/
  lib.rs            → Tauri setup + tokio spawn for mobile server
  commands.rs       → Desktop Tauri commands
  memory.rs         → Memory Spine (SQLite backend)
  mobile_commands.rs → Memory + server Tauri commands
  mobile_server.rs  → Axum HTTP server for mobile sync
  models.rs         → Shared serde structs
  orchestrator.rs   → Deploy/inject orchestration
  state.rs          → AppState, SQLite init, paths
  tool_registry.rs  → Built-in tool manifests
  workflow.rs       → Workflow run persistence
```

---

## Dependency Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri v2 (Rust) |
| Mobile framework | Capacitor v8 (Android) |
| Frontend | SvelteKit v2 + Svelte 5 |
| Styling | Tailwind CSS v4 |
| Build | Bun + Vite v7 |
| Database | SQLite via rusqlite (bundled) |
| Encryption (desktop) | Tauri Stronghold (Argon2 + AES) |
| Encryption (mobile) | Web Crypto API (PBKDF2 + AES-256-GCM) |
| Mobile HTTP server | Axum 0.7 + Tower + tokio |
| QR codes | qrcode npm package |
| Sync protocol | Vector clocks + LWW |

---

*MOBILE + ARCHITECTURE MEGA COMPLETE — push when ready for afternoon merge.*
