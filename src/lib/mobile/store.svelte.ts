/**
 * Mobile companion global state — Svelte 5 runes.
 * Single source of truth for all mobile UI state.
 */
import type { Task, Habit, CaptureItem, ApprovalItem, AgentRun, DigestEntry } from '../sync/types.js';
import { syncEngine } from '../sync/sync-engine.js';
import { memorySpine } from '../memory/memory-spine.js';
import { offlineQueue } from '../sync/offline-queue.js';

// --- State ---
export const mobileState = $state({
  initialized: false,
  online: navigator.onLine,
  pendingSyncCount: 0,
  desktopConnected: false,

  // Tasks (top 3 shown on home)
  tasks: [] as Task[],
  tasksLoading: false,

  // Habits
  habits: [] as Habit[],
  habitsLoading: false,

  // Captures (inbox)
  captures: [] as CaptureItem[],

  // Approvals
  approvals: [] as ApprovalItem[],
  approvalsLoading: false,

  // Agents
  agents: [] as AgentRun[],

  // Daily digest
  digest: null as DigestEntry | null,
  digestLoading: false,

  // Voice state
  voiceRecording: false,
  voiceTranscription: '',
  voiceError: null as string | null,

  // Desktop endpoint
  desktopEndpoint: '',

  // Nav
  activeTab: 'home' as 'home' | 'capture' | 'approvals' | 'agents' | 'digest',
});

export const PASSPHRASE_KEY = '__rh_mobile_pass__';

export async function initMobile(): Promise<void> {
  const passphrase = localStorage.getItem(PASSPHRASE_KEY) ?? generatePassphrase();
  localStorage.setItem(PASSPHRASE_KEY, passphrase);

  await syncEngine.init(passphrase);
  await memorySpine.load();

  const ep = syncEngine.getDesktopEndpoint();
  if (ep) {
    mobileState.desktopEndpoint = ep;
    memorySpine.setDesktopEndpoint(ep);
  }

  mobileState.initialized = true;
  mobileState.pendingSyncCount = offlineQueue.size();

  window.addEventListener('online', () => {
    mobileState.online = true;
    offlineQueue.drain();
  });
  window.addEventListener('offline', () => {
    mobileState.online = false;
  });

  // Seed demo data if nothing loaded yet
  if (mobileState.tasks.length === 0) seedDemoData();
}

export async function setDesktopEndpoint(endpoint: string): Promise<void> {
  syncEngine.setDesktopEndpoint(endpoint);
  memorySpine.setDesktopEndpoint(endpoint);
  mobileState.desktopEndpoint = endpoint;
  await checkDesktopConnection();
}

export async function checkDesktopConnection(): Promise<boolean> {
  const ep = mobileState.desktopEndpoint;
  if (!ep) return false;
  try {
    const res = await fetch(`${ep}/api/ping`, { signal: AbortSignal.timeout(3000) });
    mobileState.desktopConnected = res.ok;
    return res.ok;
  } catch {
    mobileState.desktopConnected = false;
    return false;
  }
}

export async function quickCapture(text: string): Promise<void> {
  const event = await syncEngine.captureText(text);
  await memorySpine.write({
    tier: 'raw',
    eventType: 'capture.create',
    payload: { text },
    source: 'mobile',
  });
  mobileState.pendingSyncCount = offlineQueue.size();
  mobileState.captures = [
    {
      id: event.id,
      content: text,
      type: 'text',
      processed: false,
      createdAt: new Date().toISOString(),
    },
    ...mobileState.captures,
  ];
}

export async function addTask(title: string, priority: Task['priority'] = 'normal'): Promise<void> {
  const event = await syncEngine.captureTask(title, priority);
  await memorySpine.write({
    tier: 'raw',
    eventType: 'task.create',
    payload: { title, priority },
    source: 'mobile',
  });
  mobileState.pendingSyncCount = offlineQueue.size();
  const task = event.payload as Task;
  mobileState.tasks = [task, ...mobileState.tasks];
}

export async function submitApproval(approvalId: string, decision: 'approved' | 'rejected'): Promise<void> {
  await syncEngine.resolveApproval(approvalId, decision);
  await memorySpine.write({
    tier: 'raw',
    eventType: 'approval.submit',
    payload: { approvalId, decision },
    source: 'mobile',
    kaizenHint: decision === 'rejected' ? `Follow up: rejected approval ${approvalId}` : undefined,
  });
  mobileState.approvals = mobileState.approvals.map((a) =>
    a.id === approvalId
      ? { ...a, status: decision, resolvedAt: new Date().toISOString(), resolvedBy: 'mobile' }
      : a
  );
  mobileState.pendingSyncCount = offlineQueue.size();
}

export async function captureVoice(transcription: string): Promise<void> {
  const event = await syncEngine.captureVoice(transcription);
  await memorySpine.write({
    tier: 'raw',
    eventType: 'voice.capture',
    payload: { transcription },
    source: 'mobile',
  });
  mobileState.voiceTranscription = transcription;
  mobileState.pendingSyncCount = offlineQueue.size();
}

function generatePassphrase(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(24));
  return btoa(String.fromCharCode(...bytes)).replace(/[^a-zA-Z0-9]/g, '').slice(0, 32);
}

function seedDemoData(): void {
  mobileState.tasks = [
    { id: '1', title: 'Review Notion integration PR', priority: 'urgent', status: 'todo', dueDate: new Date().toISOString().slice(0, 10), tags: ['dev'], source: 'desktop', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
    { id: '2', title: 'Run overnight Browser Agent', priority: 'high', status: 'in_progress', dueDate: null, tags: ['agent'], source: 'desktop', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
    { id: '3', title: 'Update memory spine schema', priority: 'normal', status: 'todo', dueDate: null, tags: ['arch'], source: 'mobile', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
  ];
  mobileState.habits = [
    { id: 'h1', name: 'Morning Review', frequency: 'daily', streak: 7, completedDates: [new Date().toISOString().slice(0, 10)], color: '#22d3ee' },
    { id: 'h2', name: 'Ralph Loop Check', frequency: 'daily', streak: 3, completedDates: [], color: '#a855f7' },
    { id: 'h3', name: 'Weekly Architecture Review', frequency: 'weekly', streak: 2, completedDates: [], color: '#10b981' },
  ];
  mobileState.approvals = [
    { id: 'ap1', agentId: 'browser-agent', agentName: 'Browser Agent', action: 'POST /api/notion/create-task', context: 'Agent wants to create a Notion task: "Review memory schema"', status: 'pending', priority: 'normal', createdAt: new Date().toISOString(), resolvedAt: null, resolvedBy: null },
    { id: 'ap2', agentId: 'ralph-loop', agentName: 'Ralph Loop', action: 'Deploy to Colab', context: 'Agent wants to deploy workflow to Google Colab for overnight run', status: 'pending', priority: 'urgent', createdAt: new Date().toISOString(), resolvedAt: null, resolvedBy: null },
  ];
  mobileState.agents = [
    { id: 'ag1', name: 'Browser Agent #7', status: 'running', startedAt: new Date(Date.now() - 3600000).toISOString(), completedAt: null, memoryRef: 'mem_001', notionTaskId: null },
    { id: 'ag2', name: 'Ralph Overnight Loop', status: 'waiting_approval', startedAt: new Date(Date.now() - 7200000).toISOString(), completedAt: null, memoryRef: null, notionTaskId: null },
    { id: 'ag3', name: 'Colab Deploy #3', status: 'success', startedAt: new Date(Date.now() - 86400000).toISOString(), completedAt: new Date(Date.now() - 82800000).toISOString(), memoryRef: 'mem_002', notionTaskId: 'notion_abc' },
  ];
  mobileState.digest = {
    date: new Date().toISOString().slice(0, 10),
    tasksCompleted: 4,
    habitsCompleted: 2,
    agentsRun: 3,
    capturesProcessed: 7,
    highlights: [
      'Browser Agent completed Notion integration sweep',
      'Memory Spine promoted 12 working memories to long-term',
      'Ralph Loop generated 3 Kaizen tasks from overnight run',
    ],
    generatedAt: new Date().toISOString(),
  };
}
