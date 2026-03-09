/**
 * Computer Control Utilities — Vy-style Desktop + Android Panda
 *
 * Wraps all Tauri computer_control commands and provides:
 * - Svelte stores for tasks, settings, workflows
 * - Permission modal helpers
 * - Real-time polling for background task status
 */

import { writable, derived, get } from 'svelte/store';
import { invokeTauri, isDesktopRuntime } from './desktop';

// ─── Types ────────────────────────────────────────────────────────────────────

export type ActionKind =
  | 'screenshot'
  | 'analyze_screen'
  | 'mouse_move'
  | 'mouse_click'
  | 'mouse_double_click'
  | 'mouse_right_click'
  | 'mouse_scroll'
  | 'type_text'
  | 'key_press'
  | 'key_combo'
  | 'open_app'
  | 'close_app'
  | 'shell'
  | 'wait';

export interface ComputerAction {
  kind: ActionKind;
  x?: number;
  y?: number;
  text?: string;
  keys?: string[];
  appName?: string;
  command?: string;
  durationMs?: number;
  scrollDelta?: number;
  description?: string;
}

export interface ComputerActionResult {
  ok: boolean;
  message: string;
  screenshotB64?: string;
  screenAnalysis?: string;
  timestamp: string;
}

export type AgentTaskStatus = 'queued' | 'running' | 'paused' | 'completed' | 'failed' | 'killed';

export interface AgentTask {
  id: string;
  title: string;
  description: string;
  goal: string;
  status: AgentTaskStatus;
  mode: string;
  progressPct: number;
  stepsCompleted: number;
  stepsTotal: number;
  currentStep: string;
  log: string[];
  screenshotB64?: string;
  kaizenTaskId?: string;
  memoryEntries: string[];
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
}

export interface CreateAgentTaskRequest {
  title: string;
  description: string;
  goal: string;
  mode: 'supervised' | 'autonomous';
  providerId?: string;
  model?: string;
}

export interface ComputerControlSettings {
  enabled: boolean;
  mode: 'supervised' | 'autonomous';
  allowBackgroundTasks: boolean;
  requireConfirmation: boolean;
  killSwitchActive: boolean;
  androidPandaEnabled: boolean;
  desktopAgentProvider: string;
  desktopAgentModel: string;
  maxConcurrentTasks: number;
  screenshotIntervalMs: number;
  updatedAt: string;
}

export interface PermissionRequest {
  taskId: string;
  action: ComputerAction;
  reason: string;
  riskLevel: 'low' | 'medium' | 'high';
}

export interface ParallelWorkflow {
  id: string;
  name: string;
  foregroundTask: string;
  backgroundTasks: string[];
  status: string;
  createdAt: string;
}

// ─── Stores ───────────────────────────────────────────────────────────────────

export const ccSettingsStore = writable<ComputerControlSettings | null>(null);
export const ccTasksStore = writable<AgentTask[]>([]);
export const ccWorkflowsStore = writable<ParallelWorkflow[]>([]);
export const ccKillSwitchStore = writable<boolean>(false);
export const ccScreenshotStore = writable<string | null>(null);
export const ccPermissionPendingStore = writable<PermissionRequest | null>(null);
export const ccLastScreenshotTimeStore = writable<string | null>(null);

export const ccRunningTasksStore = derived(ccTasksStore, ($tasks) =>
  $tasks.filter((t) => t.status === 'running' || t.status === 'queued')
);

export const ccCompletedTasksStore = derived(ccTasksStore, ($tasks) =>
  $tasks.filter((t) => t.status === 'completed')
);

// ─── Settings ─────────────────────────────────────────────────────────────────

export async function loadCcSettings(): Promise<ComputerControlSettings> {
  if (!isDesktopRuntime()) {
    return defaultSettings();
  }
  try {
    const s = await invokeTauri<ComputerControlSettings>('cc_get_settings');
    ccSettingsStore.set(s);
    ccKillSwitchStore.set(s.killSwitchActive);
    return s;
  } catch {
    const s = defaultSettings();
    ccSettingsStore.set(s);
    return s;
  }
}

export async function saveCcSettings(settings: ComputerControlSettings): Promise<ComputerControlSettings> {
  if (!isDesktopRuntime()) {
    ccSettingsStore.set(settings);
    return settings;
  }
  const saved = await invokeTauri<ComputerControlSettings>('cc_save_settings', { newSettings: settings });
  ccSettingsStore.set(saved);
  ccKillSwitchStore.set(saved.killSwitchActive);
  return saved;
}

function defaultSettings(): ComputerControlSettings {
  return {
    enabled: false,
    mode: 'supervised',
    allowBackgroundTasks: false,
    requireConfirmation: true,
    killSwitchActive: false,
    androidPandaEnabled: false,
    desktopAgentProvider: '',
    desktopAgentModel: '',
    maxConcurrentTasks: 3,
    screenshotIntervalMs: 2000,
    updatedAt: new Date().toISOString()
  };
}

// ─── Kill Switch ──────────────────────────────────────────────────────────────

export async function activateKillSwitch(): Promise<void> {
  ccKillSwitchStore.set(true);
  if (!isDesktopRuntime()) return;
  await invokeTauri('cc_toggle_kill_switch', { active: true });
  // Update settings store
  ccSettingsStore.update((s) => s ? { ...s, killSwitchActive: true } : null);
  // Mark all running tasks as killed
  ccTasksStore.update((tasks) =>
    tasks.map((t) =>
      t.status === 'running' || t.status === 'queued'
        ? { ...t, status: 'killed' as AgentTaskStatus }
        : t
    )
  );
}

export async function deactivateKillSwitch(): Promise<void> {
  ccKillSwitchStore.set(false);
  if (!isDesktopRuntime()) return;
  await invokeTauri('cc_toggle_kill_switch', { active: false });
  ccSettingsStore.update((s) => s ? { ...s, killSwitchActive: false } : null);
}

// ─── Screenshot ───────────────────────────────────────────────────────────────

export async function takeScreenshot(): Promise<ComputerActionResult> {
  if (!isDesktopRuntime()) {
    return { ok: false, message: 'Desktop runtime required', timestamp: new Date().toISOString() };
  }
  const result = await invokeTauri<ComputerActionResult>('cc_take_screenshot');
  if (result.screenshotB64) {
    ccScreenshotStore.set(result.screenshotB64);
    ccLastScreenshotTimeStore.set(result.timestamp);
  }
  return result;
}

// ─── Action Execution ─────────────────────────────────────────────────────────

export async function executeAction(action: ComputerAction): Promise<ComputerActionResult> {
  const settings = get(ccSettingsStore);

  // In supervised mode, queue a permission request first
  if (settings?.mode === 'supervised' && settings.requireConfirmation) {
    const riskLevel = getActionRiskLevel(action);
    // Fire and wait for permission grant via the store
    const permitted = await requestPermission({
      taskId: 'manual',
      action,
      reason: action.description ?? `Execute ${action.kind}`,
      riskLevel
    });
    if (!permitted) {
      return { ok: false, message: 'Action denied by user.', timestamp: new Date().toISOString() };
    }
  }

  if (!isDesktopRuntime()) {
    return { ok: true, message: `[Stub] ${action.kind} executed`, timestamp: new Date().toISOString() };
  }

  const result = await invokeTauri<ComputerActionResult>('cc_execute_action', { action });
  if (result.screenshotB64) {
    ccScreenshotStore.set(result.screenshotB64);
  }
  return result;
}

// Permission modal — returns true if allowed, false if denied
let permissionResolve: ((allowed: boolean) => void) | null = null;

async function requestPermission(req: PermissionRequest): Promise<boolean> {
  ccPermissionPendingStore.set(req);
  return new Promise((resolve) => {
    permissionResolve = resolve;
  });
}

export function grantPermission(): void {
  ccPermissionPendingStore.set(null);
  permissionResolve?.(true);
  permissionResolve = null;
}

export function denyPermission(): void {
  ccPermissionPendingStore.set(null);
  permissionResolve?.(false);
  permissionResolve = null;
}

function getActionRiskLevel(action: ComputerAction): 'low' | 'medium' | 'high' {
  if (action.kind === 'screenshot' || action.kind === 'analyze_screen' || action.kind === 'mouse_move') {
    return 'low';
  }
  if (action.kind === 'shell' || action.kind === 'close_app') {
    return 'high';
  }
  return 'medium';
}

// ─── Agent Tasks ──────────────────────────────────────────────────────────────

export async function startAgentTask(req: CreateAgentTaskRequest): Promise<AgentTask> {
  if (!isDesktopRuntime()) {
    const stub: AgentTask = {
      id: Math.random().toString(36).slice(2),
      title: req.title,
      description: req.description,
      goal: req.goal,
      status: 'running',
      mode: req.mode,
      progressPct: 0,
      stepsCompleted: 0,
      stepsTotal: 10,
      currentStep: 'Initializing (stub)…',
      log: [`[${new Date().toISOString()}] Task started (browser stub)`],
      memoryEntries: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };
    ccTasksStore.update((t) => [stub, ...t]);
    simulateTaskProgress(stub.id);
    return stub;
  }

  const task = await invokeTauri<AgentTask>('cc_start_agent_task', { req });
  ccTasksStore.update((t) => [task, ...t]);
  return task;
}

export async function stopAgentTask(taskId: string): Promise<AgentTask> {
  if (!isDesktopRuntime()) {
    ccTasksStore.update((tasks) =>
      tasks.map((t) => (t.id === taskId ? { ...t, status: 'killed' as AgentTaskStatus } : t))
    );
    return ccTasksStore && (get(ccTasksStore).find((t) => t.id === taskId) as AgentTask);
  }
  const task = await invokeTauri<AgentTask>('cc_stop_agent_task', { taskId });
  ccTasksStore.update((tasks) => tasks.map((t) => (t.id === task.id ? task : t)));
  return task;
}

export async function loadAgentTasks(): Promise<AgentTask[]> {
  if (!isDesktopRuntime()) return [];
  const tasks = await invokeTauri<AgentTask[]>('cc_list_agent_tasks');
  ccTasksStore.set(tasks);
  return tasks;
}

export async function pollTaskStatus(taskId: string): Promise<AgentTask> {
  if (!isDesktopRuntime()) {
    return get(ccTasksStore).find((t) => t.id === taskId) as AgentTask;
  }
  const task = await invokeTauri<AgentTask>('cc_get_task_status', { taskId });
  ccTasksStore.update((tasks) => tasks.map((t) => (t.id === task.id ? task : t)));
  return task;
}

// ─── Parallel Workflows ───────────────────────────────────────────────────────

export async function startParallelWorkflow(
  name: string,
  foregroundTask: string,
  backgroundGoals: string[]
): Promise<ParallelWorkflow> {
  if (!isDesktopRuntime()) {
    const wf: ParallelWorkflow = {
      id: Math.random().toString(36).slice(2),
      name,
      foregroundTask,
      backgroundTasks: backgroundGoals.map(() => Math.random().toString(36).slice(2)),
      status: 'running',
      createdAt: new Date().toISOString()
    };
    ccWorkflowsStore.update((ws) => [wf, ...ws]);
    return wf;
  }

  const wf = await invokeTauri<ParallelWorkflow>('cc_start_parallel_workflow', {
    name,
    foregroundTask,
    backgroundGoals
  });
  ccWorkflowsStore.update((ws) => [wf, ...ws]);
  return wf;
}

export async function loadWorkflows(): Promise<ParallelWorkflow[]> {
  if (!isDesktopRuntime()) return [];
  const workflows = await invokeTauri<ParallelWorkflow[]>('cc_list_parallel_workflows');
  ccWorkflowsStore.set(workflows);
  return workflows;
}

// ─── Android Panda ────────────────────────────────────────────────────────────

export async function getPandaStatus(): Promise<Record<string, unknown>> {
  if (!isDesktopRuntime()) {
    return {
      enabled: false,
      connected: false,
      version: '1.0.0',
      capabilities: ['screen_reader', 'click', 'scroll', 'type_text', 'open_app'],
      description: 'Panda Agent — AmitOS Android control'
    };
  }
  return invokeTauri<Record<string, unknown>>('cc_get_android_panda_status');
}

// ─── Browser Stub Simulation ──────────────────────────────────────────────────

function simulateTaskProgress(taskId: string): void {
  const steps = [
    'Taking initial screenshot',
    'Analyzing screen with vision model',
    'Planning action sequence',
    'Executing action 1/3',
    'Taking verification screenshot',
    'Analyzing progress',
    'Executing action 2/3',
    'Verifying state',
    'Executing action 3/3',
    'Finalizing and writing report'
  ];

  let i = 0;
  const interval = setInterval(() => {
    const killed = get(ccKillSwitchStore);
    if (killed) {
      ccTasksStore.update((tasks) =>
        tasks.map((t) =>
          t.id === taskId ? { ...t, status: 'killed' as AgentTaskStatus } : t
        )
      );
      clearInterval(interval);
      return;
    }

    if (i >= steps.length) {
      ccTasksStore.update((tasks) =>
        tasks.map((t) =>
          t.id === taskId
            ? {
                ...t,
                status: 'completed' as AgentTaskStatus,
                progressPct: 100,
                stepsCompleted: steps.length,
                currentStep: 'Complete',
                completedAt: new Date().toISOString()
              }
            : t
        )
      );
      clearInterval(interval);
      return;
    }

    ccTasksStore.update((tasks) =>
      tasks.map((t) =>
        t.id === taskId
          ? {
              ...t,
              status: 'running' as AgentTaskStatus,
              progressPct: ((i + 1) / steps.length) * 100,
              stepsCompleted: i,
              currentStep: steps[i],
              log: [...t.log, `[${new Date().toISOString()}] ${steps[i]}`]
            }
          : t
      )
    );
    i++;
  }, 900);
}
