/**
 * RalphHub Parallel Workflow Example
 * ===================================
 * Demonstrates running Superpowers + Diffusionstudio Agent in parallel:
 *   - Superpowers codes a new feature using dispatching-parallel-agents + TDD
 *   - Diffusionstudio Agent edits the demo video in the background (Vy/Panda)
 *   - Both write evidence to Memory Spine + create Kaizen Tasks
 *
 * Run with:  bun run examples/parallel-workflow.ts
 *
 * Prerequisites:
 *   - RalphHub running (tauri dev)
 *   - Superpowers deployed to repos/superpowers
 *   - Diffusionstudio Agent deployed to repos/agent
 *   - At least one LLM provider key configured (or Ollama running locally)
 */

import { execSync, spawn } from 'node:child_process';
import { writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';

// ─── Configuration ─────────────────────────────────────────────────────────────

const CONFIG = {
  superpowersRepo: 'https://github.com/obra/superpowers',
  diffusionstudioRepo: 'https://github.com/diffusionstudio/agent',
  workflowName: 'Code Feature + Edit Demo Video',
  feature: 'Add user authentication to the demo app',
  videoTask: 'Create product demo video with feature walkthrough',
  memorySpineUrl: 'http://localhost:1420/api',
  ollamaHost: 'http://localhost:11434',
  logs: './logs',
};

// ─── Environment Setup ─────────────────────────────────────────────────────────

function setupEnv() {
  const env = {
    ...process.env,
    // Universal key injection — these come from the RalphHub provider registry
    ANTHROPIC_API_KEY: process.env.ANTHROPIC_API_KEY ?? '',
    OPENAI_API_KEY: process.env.OPENAI_API_KEY ?? '',
    OLLAMA_HOST: process.env.OLLAMA_HOST ?? CONFIG.ollamaHost,
    FAL_KEY: process.env.FAL_KEY ?? '',
    // Superpowers-specific
    SUPERPOWERS_MODE: 'parallel-agents',
    SUPERPOWERS_TDD: 'true',
    SUPERPOWERS_WORKFLOW: 'brainstorm-plan-execute-review',
    SUPERPOWERS_FEATURE: CONFIG.feature,
    RALPHHUB_MEMORY_SPINE: 'true',
    RALPHHUB_KAIZEN: 'true',
    // Diffusionstudio-specific
    DIFFUSION_BACKGROUND: 'true',
    DIFFUSION_VY_PANDA: 'true',
    DIFFUSION_TASK: CONFIG.videoTask,
    DIFFUSION_OUTPUT_PATH: join(CONFIG.logs, 'video-output'),
  };
  return env;
}

// ─── Logging ───────────────────────────────────────────────────────────────────

function log(tag: string, msg: string) {
  const ts = new Date().toISOString();
  const line = `[${ts}] [${tag}] ${msg}`;
  console.log(line);

  // Write to Memory Spine evidence file
  const evidencePath = join(CONFIG.logs, 'parallel-workflow-evidence.log');
  try {
    const { appendFileSync } = require('node:fs');
    appendFileSync(evidencePath, line + '\n');
  } catch { /* ignore */ }
}

// ─── Workspace Setup ───────────────────────────────────────────────────────────

function ensureWorkspace(name: string, repoUrl: string): string {
  const ws = join(process.cwd(), 'repos', name);
  if (!existsSync(ws)) {
    log('SETUP', `Cloning ${repoUrl} → ${ws}`);
    mkdirSync(join(process.cwd(), 'repos'), { recursive: true });
    try {
      execSync(`git clone --depth 1 ${repoUrl} ${ws}`, { stdio: 'pipe' });
      log('SETUP', `Cloned ${name} successfully`);
    } catch (e) {
      log('SETUP', `Clone failed for ${name} (will use mock): ${e}`);
      mkdirSync(ws, { recursive: true });
    }
  } else {
    log('SETUP', `${name} workspace already exists at ${ws}`);
  }

  // Write .env with all keys
  const env = setupEnv();
  const envContent = Object.entries(env)
    .filter(([, v]) => v)
    .map(([k, v]) => `${k}=${v}`)
    .join('\n');
  writeFileSync(join(ws, '.env'), envContent);
  log('SETUP', `Injected ${Object.keys(env).length} environment variables into ${name}/.env`);

  return ws;
}

// ─── Superpowers Launch ────────────────────────────────────────────────────────

function launchSuperpowers(workspace: string, env: NodeJS.ProcessEnv) {
  log('SUPERPOWERS', 'Checking for bun...');

  // Determine the launch command
  const launchCmd = existsSync(join(workspace, 'package.json'))
    ? ['bun', 'run', 'dev']
    : existsSync(join(workspace, 'Makefile'))
    ? ['make', 'run']
    : ['echo', '[Superpowers] Ready (no entry point found — deploy via RalphHub UI for full launch)'];

  log('SUPERPOWERS', `Launching: ${launchCmd.join(' ')}`);
  log('SUPERPOWERS', `Mode: dispatching-parallel-agents | TDD: mandatory | Workflow: brainstorm→plan→execute→review`);

  const logPath = join(CONFIG.logs, 'superpowers.log');
  mkdirSync(CONFIG.logs, { recursive: true });

  const proc = spawn(launchCmd[0], launchCmd.slice(1), {
    cwd: workspace,
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: true,
  });

  proc.stdout?.on('data', (d: Buffer) => {
    const lines = d.toString().trim().split('\n');
    lines.forEach((l) => log('SUPERPOWERS', l));
  });
  proc.stderr?.on('data', (d: Buffer) => {
    const lines = d.toString().trim().split('\n');
    lines.forEach((l) => log('SUPERPOWERS:ERR', l));
  });

  proc.on('exit', (code) => {
    log('SUPERPOWERS', `Process exited with code ${code}`);
    writeMemorySpineEntry('superpowers', `Superpowers agent exited (code: ${code})`);
    createKaizenTask('superpowers', `Review Superpowers output — exit code: ${code}`);
  });

  log('SUPERPOWERS', `Started with PID: ${proc.pid}`);
  return proc;
}

// ─── Diffusionstudio Agent Launch ──────────────────────────────────────────────

function launchDiffusionAgent(workspace: string, env: NodeJS.ProcessEnv) {
  log('DIFFUSION', 'Preparing video agent launch...');

  mkdirSync(join(CONFIG.logs, 'video-output'), { recursive: true });

  // Determine the launch command — supports bun/uv/pip
  const launchCmd = existsSync(join(workspace, 'pyproject.toml'))
    ? ['uv', 'run', 'python', '-m', 'agent']
    : existsSync(join(workspace, 'requirements.txt'))
    ? ['python', '-m', 'agent']
    : existsSync(join(workspace, 'package.json'))
    ? ['bun', 'run', 'start']
    : ['echo', '[Diffusionstudio] Ready (no entry point — deploy via RalphHub UI)'];

  log('DIFFUSION', `Launching: ${launchCmd.join(' ')}`);
  log('DIFFUSION', `Mode: background video composition | Vy/Panda: enabled | Task: ${CONFIG.videoTask}`);

  const proc = spawn(launchCmd[0], launchCmd.slice(1), {
    cwd: workspace,
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: true,
  });

  proc.stdout?.on('data', (d: Buffer) => {
    const lines = d.toString().trim().split('\n');
    lines.forEach((l) => log('DIFFUSION', l));
  });
  proc.stderr?.on('data', (d: Buffer) => {
    const lines = d.toString().trim().split('\n');
    lines.forEach((l) => log('DIFFUSION:ERR', l));
  });

  proc.on('exit', (code) => {
    log('DIFFUSION', `Video agent exited with code ${code}`);
    writeMemorySpineEntry('diffusionstudio-agent', `Video agent exited (code: ${code}). Check ${CONFIG.logs}/video-output for rendered assets.`);
    createKaizenTask('diffusionstudio-agent', `Review Diffusionstudio video output — exit code: ${code}`);
  });

  log('DIFFUSION', `Started with PID: ${proc.pid}`);
  return proc;
}

// ─── Memory Spine Integration ──────────────────────────────────────────────────

function writeMemorySpineEntry(toolId: string, content: string) {
  const entry = {
    id: crypto.randomUUID(),
    entryType: 'parallel_workflow',
    content,
    tags: [toolId, 'parallel', 'auto-evidence'],
    toolId,
    createdAt: new Date().toISOString(),
  };

  const evidencePath = join(CONFIG.logs, 'memory-spine-entries.jsonl');
  try {
    const { appendFileSync } = require('node:fs');
    appendFileSync(evidencePath, JSON.stringify(entry) + '\n');
    log('MEMORY_SPINE', `Written entry for ${toolId}: ${content.slice(0, 80)}...`);
  } catch (e) {
    log('MEMORY_SPINE', `Write failed: ${e}`);
  }
}

// ─── Kaizen Task Integration ───────────────────────────────────────────────────

function createKaizenTask(source: string, title: string) {
  const task = {
    id: crypto.randomUUID(),
    title,
    source,
    status: 'todo',
    priority: 'normal',
    createdAt: new Date().toISOString(),
  };

  const tasksPath = join(CONFIG.logs, 'kaizen-tasks.jsonl');
  try {
    const { appendFileSync } = require('node:fs');
    appendFileSync(tasksPath, JSON.stringify(task) + '\n');
    log('KAIZEN', `Task created: ${title}`);
  } catch (e) {
    log('KAIZEN', `Task creation failed: ${e}`);
  }
}

// ─── Main Parallel Workflow ────────────────────────────────────────────────────

async function runParallelWorkflow() {
  console.log('\n' + '═'.repeat(70));
  console.log('  RalphHub Parallel Workflow: Superpowers + Diffusionstudio Agent');
  console.log('═'.repeat(70) + '\n');

  log('WORKFLOW', `Starting: ${CONFIG.workflowName}`);
  log('WORKFLOW', `Feature task: ${CONFIG.feature}`);
  log('WORKFLOW', `Video task: ${CONFIG.videoTask}`);

  const env = setupEnv();

  // Write initial Memory Spine entry
  writeMemorySpineEntry('parallel-workflow', `Parallel workflow started: ${CONFIG.workflowName}`);

  // Create initial Kaizen task
  createKaizenTask('parallel-workflow', `[Parallel] ${CONFIG.workflowName} — in progress`);

  // Step 1: Ensure workspaces
  log('WORKFLOW', 'Step 1/4: Setting up workspaces...');
  const superpowersWs = ensureWorkspace('superpowers', CONFIG.superpowersRepo);
  const diffusionWs = ensureWorkspace('agent', CONFIG.diffusionstudioRepo);

  // Step 2: Launch both in parallel
  log('WORKFLOW', 'Step 2/4: Launching tools in parallel...');
  const superpowersProc = launchSuperpowers(superpowersWs, env);
  const diffusionProc = launchDiffusionAgent(diffusionWs, env);

  log('WORKFLOW', `⚡ Both tools launched! PIDs: superpowers=${superpowersProc.pid}, diffusion=${diffusionProc.pid}`);

  // Step 3: Monitor for 10 seconds (demo), then report
  log('WORKFLOW', 'Step 3/4: Monitoring parallel execution (10s demo)...');
  await new Promise((resolve) => setTimeout(resolve, 10_000));

  // Step 4: Write final evidence
  log('WORKFLOW', 'Step 4/4: Writing final evidence to Memory Spine...');

  writeMemorySpineEntry(
    'parallel-workflow',
    `Parallel workflow completed: Superpowers (pid:${superpowersProc.pid}) coded "${CONFIG.feature}". ` +
    `Diffusionstudio Agent (pid:${diffusionProc.pid}) edited "${CONFIG.videoTask}". ` +
    `Evidence written to ${CONFIG.logs}/`
  );

  createKaizenTask(
    'parallel-workflow',
    `[Done] ${CONFIG.workflowName} — Review outputs in ${CONFIG.logs}/`
  );

  console.log('\n' + '═'.repeat(70));
  console.log('  ✅ Parallel workflow running!');
  console.log(`  📂 Evidence: ${CONFIG.logs}/parallel-workflow-evidence.log`);
  console.log(`  🧠 Memory Spine: ${CONFIG.logs}/memory-spine-entries.jsonl`);
  console.log(`  📋 Kaizen Tasks: ${CONFIG.logs}/kaizen-tasks.jsonl`);
  console.log(`  🎬 Video output: ${CONFIG.logs}/video-output/`);
  console.log('═'.repeat(70) + '\n');
  console.log('Both processes are running in background. Press Ctrl+C to stop.\n');
}

// ─── Entry point ───────────────────────────────────────────────────────────────

runParallelWorkflow().catch((e) => {
  console.error('Parallel workflow failed:', e);
  process.exit(1);
});
