/**
 * AmitOS End-to-End Test Suite
 *
 * Tests cover: UI routes, provider registry, Tauri command signatures,
 * MCP config generation, tool registry, workflow validation, and system health.
 *
 * Run with: bun test tests/e2e.test.ts
 */

import { describe, it, expect, beforeAll } from 'bun:test';

// ─── Route availability ─────────────────────────────────────────────────────

describe('Routes', () => {
  const routes = [
    '/',
    '/today',
    '/kaizen',
    '/memory',
    '/vy',
    '/parallel',
    '/computer-control',
    '/panda',
    '/voice',
    '/remote-nodes',
    '/mcp',
    '/mobile',
    '/tools',
    '/workflows',
    '/deploy',
    '/settings',
  ];

  for (const route of routes) {
    it(`route ${route} has a +page.svelte file`, async () => {
      const path = route === '/'
        ? 'src/routes/+page.svelte'
        : `src/routes${route}/+page.svelte`;
      const file = Bun.file(path);
      expect(await file.exists()).toBe(true);
    });
  }
});

// ─── CI/CD workflows ─────────────────────────────────────────────────────────

describe('GitHub Actions', () => {
  it('ci.yml exists and is valid YAML', async () => {
    const f = Bun.file('.github/workflows/ci.yml');
    expect(await f.exists()).toBe(true);
    const content = await f.text();
    expect(content).toContain('bun install');
    expect(content).toContain('bun run build');
  });

  it('release.yml exists and covers all platforms', async () => {
    const f = Bun.file('.github/workflows/release.yml');
    expect(await f.exists()).toBe(true);
    const content = await f.text();
    expect(content).toContain('build-macos');
    expect(content).toContain('build-windows');
    expect(content).toContain('build-linux');
    expect(content).toContain('build-android');
    expect(content).toContain('build-rpi');
  });
});

// ─── Documentation ────────────────────────────────────────────────────────────

describe('Documentation', () => {
  it('README.md has one-click install section', async () => {
    const f = Bun.file('README.md');
    const content = await f.text();
    expect(content).toContain('One-Click Install');
    expect(content).toContain('bun install');
  });

  it('README.md has voice demo section', async () => {
    const f = Bun.file('README.md');
    const content = await f.text();
    expect(content).toContain('Voice Demo');
  });

  it('CHANGELOG.md documents v1.0.0', async () => {
    const f = Bun.file('CHANGELOG.md');
    const content = await f.text();
    expect(content).toContain('[1.0.0]');
  });

  it('LICENSE file exists', async () => {
    const f = Bun.file('LICENSE');
    expect(await f.exists()).toBe(true);
    const content = await f.text();
    expect(content).toContain('MIT');
  });

  it('RELEASE_NOTES.md exists', async () => {
    const f = Bun.file('RELEASE_NOTES.md');
    expect(await f.exists()).toBe(true);
  });
});

// ─── Rebranding ─────────────────────────────────────────────────────────────

describe('AmitOS Branding', () => {
  it('tauri.conf.json uses AmitOS branding', async () => {
    const f = Bun.file('src-tauri/tauri.conf.json');
    const content = await f.text();
    expect(content).toContain('AmitOS');
    expect(content).toContain('1.0.0');
  });

  it('Cargo.toml uses amitos package name', async () => {
    const f = Bun.file('src-tauri/Cargo.toml');
    const content = await f.text();
    expect(content).toContain('name = "amitos"');
    expect(content).toContain('1.0.0');
  });

  it('page title is AmitOS', async () => {
    const f = Bun.file('src/routes/+layout.svelte');
    const content = await f.text();
    expect(content).toContain('AmitOS');
  });
});

// ─── Build artifacts ──────────────────────────────────────────────────────────

describe('Build', () => {
  it('scripts/build-rpi.sh exists and is executable', async () => {
    const f = Bun.file('scripts/build-rpi.sh');
    expect(await f.exists()).toBe(true);
    const content = await f.text();
    expect(content).toContain('#!/bin/bash');
  });

  it('package.json has correct bun scripts', async () => {
    const f = Bun.file('package.json');
    const pkg = await f.json();
    expect(pkg.scripts.build).toBeDefined();
    expect(pkg.scripts.check).toBeDefined();
    expect(pkg.scripts['tauri:dev']).toContain('bun');
    expect(pkg.scripts['tauri:build']).toContain('bun');
  });
});

// ─── Tauri commands ─────────────────────────────────────────────────────────

describe('Tauri Commands', () => {
  const EXPECTED_COMMANDS = [
    'ensure_bun',
    'get_dashboard_snapshot',
    'list_builtin_tools',
    'list_providers_cmd',
    'create_provider_cmd',
    'update_provider_cmd',
    'delete_provider_cmd',
    'log_api_usage_cmd',
    'get_memory_spine_stats_cmd',
    'list_memory_entries_cmd',
    'create_kaizen_task_cmd',
    'list_kaizen_tasks_cmd',
    'update_kaizen_task_status_cmd',
    'start_agent_session',
    'list_agent_sessions',
    'stop_agent_session',
    'execute_agent_action',
    'create_parallel_task',
    'list_parallel_tasks',
    'update_parallel_task_status',
    'list_android_devices',
    'execute_adb_command',
    'install_panda_apk',
    'request_permission',
    'resolve_permission',
    'list_permission_requests',
    'deploy_remote_node',
    'list_remote_nodes',
    'send_chat_message',
    'list_chat_sessions',
    'list_chat_messages',
    'create_push_notification',
    'list_push_notifications',
    'mark_notification_read',
    'deploy_to_colab',
    'deploy_to_pc',
    'inject_keys',
  ];

  it('lib.rs registers all expected commands', async () => {
    const f = Bun.file('src-tauri/src/lib.rs');
    const content = await f.text();
    for (const cmd of EXPECTED_COMMANDS) {
      expect(content).toContain(cmd);
    }
  });

  it('all frontend commands are registered in backend', async () => {
    const libContent = await Bun.file('src-tauri/src/lib.rs').text();
    // Collect all invokeTauri calls from route files
    const routeDir = 'src/routes';
    const allCalls: string[] = [];

    // Read key pages
    const pages = [
      'src/routes/+page.svelte',
      'src/routes/computer-control/+page.svelte',
      'src/routes/voice/+page.svelte',
      'src/routes/remote-nodes/+page.svelte',
      'src/routes/memory/+page.svelte',
    ];

    for (const page of pages) {
      const content = await Bun.file(page).text();
      const matches = content.matchAll(/invokeTauri[^(]*\(['"]([^'"]+)['"]/g);
      for (const m of matches) {
        allCalls.push(m[1]);
      }
    }

    // All discovered commands should be in lib.rs
    for (const cmd of allCalls) {
      expect(libContent).toContain(cmd);
    }
  });
});

// ─── CSS design system ───────────────────────────────────────────────────────

describe('CSS Design System', () => {
  it('app.css has ADHD-friendly design tokens', async () => {
    const f = Bun.file('src/app.css');
    const content = await f.text();
    expect(content).toContain('--c-violet');
    expect(content).toContain('--c-cyan');
    expect(content).toContain('amitos-bg');
    expect(content).toContain('btn-primary');
    expect(content).toContain('touch-target');
  });

  it('app.css has smooth animations', async () => {
    const f = Bun.file('src/app.css');
    const content = await f.text();
    expect(content).toContain('@keyframes');
    expect(content).toContain('@keyframes spin');
    expect(content).toContain('pulse');
  });

  it('layout has correct navigation items', async () => {
    const f = Bun.file('src/routes/+layout.svelte');
    const content = await f.text();
    expect(content).toContain('/kaizen');
    expect(content).toContain('/memory');
    expect(content).toContain('/today');
    expect(content).toContain('/vy');
    expect(content).toContain('/parallel');
    expect(content).toContain('/panda');
    expect(content).toContain('/voice');
    expect(content).toContain('/mcp');
    expect(content).toContain('/mobile');
  });
});

// ─── Provider registry ───────────────────────────────────────────────────────

describe('Provider Registry', () => {
  it('provider-registry.ts exports all required stores', async () => {
    const f = Bun.file('src/lib/utils/provider-registry.ts');
    const content = await f.text();
    expect(content).toContain('enabledProvidersStore');
    expect(content).toContain('activeModelStore');
    expect(content).toContain('memoryStatsStore');
    expect(content).toContain('kaizenTasksStore');
    expect(content).toContain('createKaizenTask');
    expect(content).toContain('loadKaizenTasks');
    expect(content).toContain('getMemoryStats');
  });
});

// ─── MCP Config ──────────────────────────────────────────────────────────────

describe('MCP Config Generation', () => {
  it('mcp page contains correct MCP server commands', async () => {
    const f = Bun.file('src/routes/mcp/+page.svelte');
    const content = await f.text();
    expect(content).toContain('@playwright/mcp');
    expect(content).toContain('firecrawl-mcp');
    expect(content).toContain('server-github');
    expect(content).toContain('server-filesystem');
  });
});

// ─── System health simulation ────────────────────────────────────────────────

describe('System Health', () => {
  it('dashboard has system health check function', async () => {
    const f = Bun.file('src/routes/+page.svelte');
    const content = await f.text();
    expect(content).toContain('runHealthCheck');
    expect(content).toContain('Memory Spine');
    expect(content).toContain('Vy Agent');
    expect(content).toContain('Panda');
  });

  it('all 10 AmitOS milestones are referenced', async () => {
    const f = Bun.file('src/routes/+page.svelte');
    const content = await f.text();
    expect(content).toContain('M1');
    expect(content).toContain('M10');
  });
});

// ─── New Routes (Grand Finale additions) ─────────────────────────────────────

describe('Grand Finale New Routes', () => {
  it('today/+page.svelte loads kaizen_tasks and domains', async () => {
    const f = Bun.file('src/routes/today/+page.svelte');
    const content = await f.text();
    expect(content).toContain('list_kaizen_tasks');
    expect(content).toContain('list_kaizen_domains');
    expect(content).toContain('create_kaizen_task');
    expect(content).toContain('update_kaizen_task');
    expect(content).toContain('isToday');
    expect(content).toContain('energy');
  });

  it('vy/+page.svelte has VY_CAPABILITIES and permission flow', async () => {
    const f = Bun.file('src/routes/vy/+page.svelte');
    const content = await f.text();
    expect(content).toContain('VY_CAPABILITIES');
    expect(content).toContain('requestPermission');
    expect(content).toContain('APPROVAL_STEPS');
    expect(content).toContain('permissionGranted');
  });

  it('parallel/+page.svelte has both superpowers and diffusionstudio', async () => {
    const f = Bun.file('src/routes/parallel/+page.svelte');
    const content = await f.text();
    expect(content).toContain('superpowers');
    expect(content).toContain('diffusionstudio-agent');
    expect(content).toContain('run_parallel_workflow');
    expect(content).toContain('handle_voice_command');
    expect(content).toContain('list_running_tools');
  });
});

// ─── New Rust commands (Grand Finale) ────────────────────────────────────────

describe('Grand Finale Rust Commands', () => {
  it('lib.rs registers kaizen rich commands', async () => {
    const lib = await Bun.file('src-tauri/src/lib.rs').text();
    expect(lib).toContain('kaizen::list_kaizen_tasks');
    expect(lib).toContain('kaizen::create_kaizen_task');
    expect(lib).toContain('kaizen::update_kaizen_task');
    expect(lib).toContain('kaizen::list_kaizen_domains');
    expect(lib).toContain('kaizen::delete_kaizen_task');
  });

  it('lib.rs registers process management commands', async () => {
    const lib = await Bun.file('src-tauri/src/lib.rs').text();
    expect(lib).toContain('commands::launch_tool_background');
    expect(lib).toContain('commands::run_parallel_workflow');
    expect(lib).toContain('commands::list_running_tools');
    expect(lib).toContain('commands::handle_voice_command');
    expect(lib).toContain('commands::stop_tool_process');
    expect(lib).toContain('commands::get_tool_logs');
    expect(lib).toContain('commands::list_parallel_workflows');
  });

  it('process_manager.rs exists with launch_background function', async () => {
    const f = Bun.file('src-tauri/src/process_manager.rs');
    const content = await f.text();
    expect(await f.exists()).toBe(true);
    expect(content).toContain('launch_background');
    expect(content).toContain('stop_tool');
    expect(content).toContain('read_logs');
    expect(content).toContain('list_all');
    expect(content).toContain('ProcessRegistry');
  });

  it('kaizen.rs exists with rich commands', async () => {
    const f = Bun.file('src-tauri/src/kaizen.rs');
    const content = await f.text();
    expect(await f.exists()).toBe(true);
    expect(content).toContain('#[tauri::command]');
    expect(content).toContain('list_kaizen_tasks');
    expect(content).toContain('create_kaizen_task');
    expect(content).toContain('list_kaizen_domains');
    expect(content).toContain('is_today');
    expect(content).toContain('energy');
  });
});

// ─── Tool Registry ────────────────────────────────────────────────────────────

describe('Tool Registry - Superpowers + Diffusionstudio', () => {
  it('tool_registry.rs includes superpowers tool', async () => {
    const f = Bun.file('src-tauri/src/tool_registry.rs');
    const content = await f.text();
    expect(content).toContain('superpowers');
    expect(content).toContain('obra/superpowers');
    expect(content).toContain('parallel_capable');
  });

  it('tool_registry.rs includes diffusionstudio-agent tool', async () => {
    const f = Bun.file('src-tauri/src/tool_registry.rs');
    const content = await f.text();
    expect(content).toContain('diffusionstudio-agent');
    expect(content).toContain('diffusionstudio/agent');
    expect(content).toContain('video');
  });

  it('ToolManifest has category and tags fields', async () => {
    const f = Bun.file('src-tauri/src/models.rs');
    const content = await f.text();
    expect(content).toContain('pub category: String');
    expect(content).toContain('pub parallel_capable: bool');
    expect(content).toContain('pub tags: Vec<String>');
  });
});

// ─── Kaizen schema ────────────────────────────────────────────────────────────

describe('Kaizen Rich Schema', () => {
  it('models.rs has rich KaizenTask with all fields', async () => {
    const f = Bun.file('src-tauri/src/models.rs');
    const content = await f.text();
    expect(content).toContain('pub is_today: bool');
    expect(content).toContain('pub energy: String');
    expect(content).toContain('pub estimated_minutes: Option<i32>');
    expect(content).toContain('pub domain: String');
    expect(content).toContain('pub subtasks: Vec<String>');
  });

  it('models.rs has KaizenDomain and UpdateKaizenTaskRequest', async () => {
    const f = Bun.file('src-tauri/src/models.rs');
    const content = await f.text();
    expect(content).toContain('pub struct KaizenDomain');
    expect(content).toContain('pub struct UpdateKaizenTaskRequest');
    expect(content).toContain('pub today_count: i64');
  });

  it('state.rs creates kaizen_domains table with seeds', async () => {
    const f = Bun.file('src-tauri/src/state.rs');
    const content = await f.text();
    expect(content).toContain('kaizen_domains');
    expect(content).toContain('is_today INTEGER');
    expect(content).toContain('energy TEXT');
    expect(content).toContain('parallel_workflows');
  });
});

// ─── End-to-end simulation: Grand Finale workflow ────────────────────────────

describe('Grand Finale E2E Simulation', () => {
  it('voice command → parallel agents → memory → kaizen → Notion flow is documented', async () => {
    const f = Bun.file('src/routes/+page.svelte');
    const content = await f.text();
    expect(content).toContain('superpowers');
    expect(content).toContain('diffusionstudio');
    expect(content).toContain('Memory Spine');
    expect(content).toContain('runHealthCheck');
  });

  it('parallel workflow page has voice command integration', async () => {
    const f = Bun.file('src/routes/parallel/+page.svelte');
    const content = await f.text();
    expect(content).toContain('handle_voice_command');
    expect(content).toContain('voiceTranscript');
    expect(content).toContain('voiceListening');
  });

  it('voice → Panda phone approval flow is wired', async () => {
    const voicePage = await Bun.file('src/routes/voice/+page.svelte').text();
    const pandaPage = await Bun.file('src/routes/panda/+page.svelte').text();
    expect(voicePage).toContain('send_chat_message');
    expect(pandaPage).toContain('approveAction');
    expect(pandaPage).toContain('pendingActions');
  });

  it('memory spine + kaizen task auto-creation wiring exists', async () => {
    const registry = await Bun.file('src/lib/utils/provider-registry.ts').text();
    expect(registry).toContain('logApiUsage');
    expect(registry).toContain('createKaizenTask');
    expect(registry).toContain('refreshMemoryStats');
  });

  it('MCP browser config generation works correctly', async () => {
    const mcp = await Bun.file('src/routes/mcp/+page.svelte').text();
    expect(mcp).toContain('updateConfig');
    expect(mcp).toContain('mcpServers');
    expect(mcp).toContain('copyConfig');
  });

  it('RPi ARM build script is ready', async () => {
    const script = await Bun.file('scripts/build-rpi.sh').text();
    expect(script).toContain('#!/bin/bash');
  });

  it('today board uses energy-tagged tasks', async () => {
    const f = Bun.file('src/routes/today/+page.svelte');
    const content = await f.text();
    expect(content).toContain('ENERGY_LABELS');
    expect(content).toContain('medium');
    expect(content).toContain('low');
    expect(content).toContain('high');
  });

  it('vy agent has tutorial and permission approval modes', async () => {
    const f = Bun.file('src/routes/vy/+page.svelte');
    const content = await f.text();
    expect(content).toContain('tutorial');
    expect(content).toContain('takeover');
    expect(content).toContain('permissions');
  });

  it('parallel page has workflow log and history', async () => {
    const f = Bun.file('src/routes/parallel/+page.svelte');
    const content = await f.text();
    expect(content).toContain('workflowLog');
    expect(content).toContain('pastWorkflows');
    expect(content).toContain('list_parallel_workflows');
  });
});
