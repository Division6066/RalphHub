// Milestone 7: Central store for Computer Control + Voice state
// Provides reactive stores for agent sessions, parallel tasks, permissions, and notifications
import { writable, derived } from 'svelte/store';
import { invokeTauri, isDesktopRuntime } from './desktop';

// ─── Types ────────────────────────────────────────────────────────────────────

export type AgentSession = {
	id: string;
	name: string;
	target: string;
	status: string;
	currentTask: string;
	actionsTaken: number;
	parallelMode: boolean;
	permissionMode: string;
	screenshotPath: string | null;
	logLines: string[];
	startedAt: string | null;
	updatedAt: string;
};

export type ParallelTask = {
	id: string;
	sessionId: string;
	title: string;
	description: string;
	status: string;
	priority: number;
	deviceTarget: string;
	progressPct: number;
	resultSummary: string;
	createdAt: string;
	updatedAt: string;
};

export type PermissionRequest = {
	id: string;
	sessionId: string;
	actionType: string;
	description: string;
	riskLevel: string;
	status: string;
	requestedAt: string;
	resolvedAt: string | null;
	resolvedBy: string;
};

export type PushNotification = {
	id: string;
	title: string;
	body: string;
	notificationType: string;
	payload: string;
	read: boolean;
	createdAt: string;
};

export type RemoteNode = {
	id: string;
	nodeName: string;
	nodeType: string;
	host: string;
	port: number;
	username: string;
	status: string;
	lastPing: string | null;
	agentVersion: string;
	createdAt: string;
};

// ─── Stores ───────────────────────────────────────────────────────────────────

export const agentSessionsStore = writable<AgentSession[]>([]);
export const parallelTasksStore = writable<ParallelTask[]>([]);
export const permissionRequestsStore = writable<PermissionRequest[]>([]);
export const pushNotificationsStore = writable<PushNotification[]>([]);
export const remoteNodesStore = writable<RemoteNode[]>([]);

// ─── Derived ──────────────────────────────────────────────────────────────────

export const runningSessionsStore = derived(agentSessionsStore, ($sessions) =>
	$sessions.filter((s) => s.status === 'running')
);

export const pendingPermissionsStore = derived(permissionRequestsStore, ($perms) =>
	$perms.filter((p) => p.status === 'pending')
);

export const unreadNotificationsStore = derived(pushNotificationsStore, ($notifs) =>
	$notifs.filter((n) => !n.read)
);

export const queuedTasksStore = derived(parallelTasksStore, ($tasks) =>
	$tasks.filter((t) => t.status === 'queued')
);

// ─── Actions ──────────────────────────────────────────────────────────────────

export async function loadComputerControlState(): Promise<void> {
	if (!isDesktopRuntime()) return;

	try {
		const [sessions, tasks, perms, notifs, nodes] = await Promise.all([
			invokeTauri<AgentSession[]>('list_agent_sessions'),
			invokeTauri<ParallelTask[]>('list_parallel_tasks'),
			invokeTauri<PermissionRequest[]>('list_permission_requests', { status: null }),
			invokeTauri<PushNotification[]>('list_push_notifications', { unreadOnly: false }),
			invokeTauri<RemoteNode[]>('list_remote_nodes')
		]);

		agentSessionsStore.set(sessions);
		parallelTasksStore.set(tasks);
		permissionRequestsStore.set(perms);
		pushNotificationsStore.set(notifs);
		remoteNodesStore.set(nodes);
	} catch (e) {
		console.error('Failed to load computer control state:', e);
	}
}

export async function approvePermission(id: string, resolvedBy = 'user'): Promise<void> {
	if (!isDesktopRuntime()) return;
	await invokeTauri('resolve_permission', { id, approved: true, resolvedBy });
	await loadComputerControlState();
}

export async function denyPermission(id: string, resolvedBy = 'user'): Promise<void> {
	if (!isDesktopRuntime()) return;
	await invokeTauri('resolve_permission', { id, approved: false, resolvedBy });
	await loadComputerControlState();
}

export async function sendVoiceCommand(text: string): Promise<string> {
	if (!isDesktopRuntime()) {
		return `Voice command received: "${text}". Processing in web mode.`;
	}
	const reply = await invokeTauri<{ content: string }>('send_chat_message', {
		req: {
			sessionId: null,
			content: text,
			voiceInput: true,
			deviceOrigin: 'voice',
			model: null
		}
	});
	return reply.content;
}

// ─── Milestone 8: Example Parallel Workflow Steps ────────────────────────────

export const MEGA_PARALLEL_WORKFLOW = {
	id: 'mega-parallel-demo',
	title: 'Taxes + Notion + Phone Chat (Milestone 8 Example)',
	description: 'While you watch a tutorial, AmitOS handles everything in parallel across all devices.',
	steps: [
		{
			device: 'desktop' as const,
			icon: '📊',
			title: 'Excel Tax Filing',
			task: 'Vy agent opens Excel, fills W2 data from prior year PDF, calculates deductions, saves to taxes-2025.xlsx',
			agentType: 'vy-desktop'
		},
		{
			device: 'desktop' as const,
			icon: '📝',
			title: 'Notion Update',
			task: 'Second agent thread updates Notion workspace — marks Q1 projects done, creates new sprint tasks',
			agentType: 'vy-desktop'
		},
		{
			device: 'android' as const,
			icon: '💬',
			title: 'Phone Messages',
			task: 'Panda/blurr agent reads WhatsApp, Signal messages, drafts context-aware replies for approval',
			agentType: 'panda-android'
		},
		{
			device: 'vps' as const,
			icon: '🔄',
			title: 'VPS Data Pipeline',
			task: 'Remote node fetches bank CSV via API, categorizes 500+ transactions, generates expense summary',
			agentType: 'vps-node'
		},
		{
			device: 'voice' as const,
			icon: '🎙️',
			title: 'Voice Approval',
			task: 'While you\'re out: "approve the tax deduction" → relayed instantly to desktop agent via Voice Assistant',
			agentType: 'voice'
		},
		{
			device: 'memory' as const,
			icon: '🧠',
			title: 'Memory + Kaizen',
			task: 'ALL actions auto-logged to Memory Spine, Kaizen tasks created for review, Workflow Composer updated',
			agentType: 'memory'
		}
	]
} as const;
