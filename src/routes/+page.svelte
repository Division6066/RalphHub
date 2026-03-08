<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	interface MemoryStats {
		rawEventsCount: number;
		workingMemoryCount: number;
		longTermCount: number;
		summariesCount: number;
		inboxCount: number;
		dailyLogCount: number;
	}

	interface KaizenTask {
		id: string;
		title: string;
		domain: string;
		energy: string;
		estimateMinutes: number;
		status: string;
		doDate: string | null;
		approvalRequired: boolean;
		agentMode: string;
	}

	interface InboxItem {
		id: string;
		content: string;
		contentType: string;
		processed: boolean;
		createdAt: string;
	}

	interface DailyLogEntry {
		id: string;
		logDate: string;
		entryType: string;
		title: string;
		createdAt: string;
	}

	interface AmitosDashboard {
		memoryStats: MemoryStats;
		todayTasks: KaizenTask[];
		inboxItems: InboxItem[];
		runningAgents: string[];
		approvalQueue: KaizenTask[];
		recentLog: DailyLogEntry[];
		managedProjectCount: number;
		workflowRunCount: number;
	}

	let dashboard = $state<AmitosDashboard | null>(null);
	let loading = $state(true);
	let error = $state('');
	let quickCapture = $state('');
	let capturing = $state(false);
	let captureMsg = $state('');

	const domainEmoji: Record<string, string> = {
		work: '💼', health: '🏃', learning: '📚', personal: '🏠', system: '⚙️',
	};
	const energyDot: Record<string, string> = {
		low: 'bg-emerald-400', medium: 'bg-amber-400', high: 'bg-red-400',
	};
	const logTypeIcon: Record<string, string> = {
		agent_run: '🤖', task_complete: '✅', browser_action: '🌐',
		notion_sync: '🔲', morning_digest: '🌅', nightly_wrap: '🌙', memory_write: '🧠', manual: '📝',
	};

	function fmtMinutes(m: number): string {
		if (m < 60) return `${m}m`;
		return `${Math.floor(m / 60)}h${m % 60 > 0 ? ` ${m % 60}m` : ''}`;
	}

	async function load() {
		loading = true;
		error = '';
		try {
			if (!isDesktopRuntime()) {
				dashboard = {
					memoryStats: { rawEventsCount: 12, workingMemoryCount: 5, longTermCount: 3, summariesCount: 2, inboxCount: 4, dailyLogCount: 18 },
					todayTasks: [
						{ id: '1', title: 'Build AmitOS Dashboard', domain: 'work', energy: 'high', estimateMinutes: 120, status: 'doing', doDate: new Date().toISOString().slice(0, 10), approvalRequired: false, agentMode: 'manual' },
						{ id: '2', title: 'Review Kaizen task schema', domain: 'work', energy: 'medium', estimateMinutes: 30, status: 'todo', doDate: new Date().toISOString().slice(0, 10), approvalRequired: false, agentMode: 'manual' },
						{ id: '3', title: '30-min walk', domain: 'health', energy: 'low', estimateMinutes: 30, status: 'todo', doDate: new Date().toISOString().slice(0, 10), approvalRequired: false, agentMode: 'manual' },
					],
					inboxItems: [
						{ id: '1', content: 'Read: Spaced Repetition for devs article', contentType: 'url', processed: false, createdAt: new Date().toISOString() },
						{ id: '2', content: 'Idea: auto-tag memory entries with domains', contentType: 'text', processed: false, createdAt: new Date().toISOString() },
					],
					runningAgents: [],
					approvalQueue: [],
					recentLog: [
						{ id: '1', logDate: new Date().toISOString().slice(0, 10), entryType: 'morning_digest', title: 'Morning Digest', createdAt: new Date().toISOString() },
						{ id: '2', logDate: new Date().toISOString().slice(0, 10), entryType: 'memory_write', title: 'Memory: AmitOS architecture notes', createdAt: new Date().toISOString() },
					],
					managedProjectCount: 1,
					workflowRunCount: 3,
				};
				loading = false;
				return;
			}
			dashboard = await invokeTauri<AmitosDashboard>('get_amitos_dashboard');
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function quickCaptureToInbox() {
		if (!quickCapture.trim()) return;
		capturing = true;
		captureMsg = '';
		try {
			if (isDesktopRuntime()) {
				await invokeTauri('add_inbox_item', {
					request: { content: quickCapture, contentType: 'text', source: 'dashboard_quick_capture' },
				});
			}
			captureMsg = '✓ Captured';
			quickCapture = '';
			setTimeout(() => { captureMsg = ''; }, 2000);
			await load();
		} catch (e) {
			captureMsg = '✗ ' + String(e);
		} finally {
			capturing = false;
		}
	}

	onMount(load);
</script>

<div class="space-y-6">
	<!-- Hero banner -->
	<div class="rounded-3xl border border-cyan-400/10 bg-gradient-to-br from-cyan-950/40 via-slate-900/50 to-purple-950/30 p-6 shadow-2xl shadow-cyan-950/20">
		<div class="flex items-start justify-between gap-4">
			<div>
				<div class="flex items-center gap-2 mb-3">
					<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-cyan-400 to-purple-500 text-base font-bold text-white shadow-lg shadow-cyan-500/30">A</div>
					<div>
						<p class="text-xs font-bold uppercase tracking-widest text-cyan-300">AmitOS</p>
						<p class="text-[10px] text-slate-500">Personal Operating System</p>
					</div>
				</div>
				<h1 class="text-2xl font-bold leading-tight text-white">
					{new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}
				</h1>
				<p class="mt-1 text-sm text-slate-400">Memory Spine · Kaizen Tasks · Inbox · Daily Log</p>
			</div>
			<div class="flex flex-col gap-2 text-right">
				{#if dashboard}
					<p class="text-2xl font-bold text-white">{dashboard.todayTasks.length}</p>
					<p class="text-xs text-slate-500">tasks today</p>
				{/if}
			</div>
		</div>

		<!-- Quick Capture -->
		<div class="mt-5 flex gap-2">
			<input
				bind:value={quickCapture}
				type="text"
				placeholder="⚡ Quick capture — press Enter to send to inbox…"
				class="flex-1 rounded-xl border border-white/10 bg-slate-800/60 px-4 py-2.5 text-sm text-white placeholder-slate-500 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-400/10"
				onkeydown={(e) => { if (e.key === 'Enter') quickCaptureToInbox(); }}
			/>
			<button
				onclick={quickCaptureToInbox}
				disabled={capturing || !quickCapture.trim()}
				class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400 disabled:opacity-40"
			>
				{capturing ? '…' : 'Capture'}
			</button>
		</div>
		{#if captureMsg}
			<p class="mt-1.5 text-xs {captureMsg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{captureMsg}</p>
		{/if}
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	{#if loading}
		<div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
			{#each Array(4) as _}
				<div class="h-24 animate-pulse rounded-2xl bg-slate-800/40"></div>
			{/each}
		</div>
	{:else if dashboard}
		<!-- Memory stats row -->
		<div>
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-sm font-bold uppercase tracking-wider text-slate-500">🧠 Memory Spine</h2>
				<a href="/memory" class="text-xs text-cyan-400 hover:text-cyan-300">Open →</a>
			</div>
			<div class="grid grid-cols-3 gap-3 sm:grid-cols-6">
				{#each [
					{ label: 'Raw Events', value: dashboard.memoryStats.rawEventsCount },
					{ label: 'Working', value: dashboard.memoryStats.workingMemoryCount },
					{ label: 'Long-Term', value: dashboard.memoryStats.longTermCount },
					{ label: 'Summaries', value: dashboard.memoryStats.summariesCount },
					{ label: 'Inbox', value: dashboard.memoryStats.inboxCount },
					{ label: 'Log', value: dashboard.memoryStats.dailyLogCount },
				] as s}
					<div class="rounded-xl border border-white/8 bg-slate-900/50 p-3 text-center">
						<p class="text-lg font-bold text-white">{s.value}</p>
						<p class="mt-0.5 text-[9px] font-medium uppercase tracking-wider text-slate-600">{s.label}</p>
					</div>
				{/each}
			</div>
		</div>

		<div class="grid gap-5 lg:grid-cols-2">
			<!-- Today Board preview -->
			<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-5">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-sm font-bold uppercase tracking-wider text-slate-500">📅 Today's Tasks</h2>
					<a href="/tasks" class="text-xs text-cyan-400 hover:text-cyan-300">Full Board →</a>
				</div>
				{#if dashboard.todayTasks.length === 0}
					<div class="py-8 text-center">
						<p class="text-slate-500 text-sm">No tasks today.</p>
						<a href="/tasks" class="mt-2 inline-block text-xs text-cyan-400">+ Add tasks →</a>
					</div>
				{:else}
					<div class="space-y-2">
						{#each dashboard.todayTasks.slice(0, 6) as task}
							<div class="flex items-center gap-3 rounded-xl border border-white/6 bg-slate-800/40 px-3 py-2">
								<span class={`h-2 w-2 shrink-0 rounded-full ${energyDot[task.energy] ?? 'bg-slate-500'}`}></span>
								<span class="text-sm shrink-0">{domainEmoji[task.domain] ?? '📌'}</span>
								<p class="flex-1 text-sm text-white truncate">{task.title}</p>
								<span class="shrink-0 text-[10px] text-slate-500">{fmtMinutes(task.estimateMinutes)}</span>
							</div>
						{/each}
						{#if dashboard.todayTasks.length > 6}
							<p class="text-center text-xs text-slate-500">+{dashboard.todayTasks.length - 6} more</p>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Inbox + Approvals -->
			<div class="space-y-4">
				<!-- Inbox -->
				<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-5">
					<div class="mb-3 flex items-center justify-between">
						<h2 class="text-sm font-bold uppercase tracking-wider text-slate-500">📥 Inbox</h2>
						<a href="/inbox" class="text-xs text-cyan-400 hover:text-cyan-300">Open →</a>
					</div>
					{#if dashboard.inboxItems.length === 0}
						<p class="py-2 text-center text-sm text-slate-500">Inbox is clear ✓</p>
					{:else}
						<div class="space-y-1.5">
							{#each dashboard.inboxItems.slice(0, 3) as item}
								<div class="flex items-center gap-2 rounded-xl border border-white/6 bg-slate-800/40 px-3 py-2">
									<span class="text-sm">{item.contentType === 'url' ? '🔗' : item.contentType === 'screenshot' ? '📷' : '📝'}</span>
									<p class="flex-1 text-xs text-slate-300 truncate">{item.content}</p>
								</div>
							{/each}
							{#if dashboard.inboxItems.length > 3}
								<p class="text-center text-xs text-slate-500">+{dashboard.inboxItems.length - 3} more</p>
							{/if}
						</div>
					{/if}
				</div>

				<!-- Approval Queue -->
				{#if dashboard.approvalQueue.length > 0}
					<div class="rounded-2xl border border-amber-400/20 bg-amber-400/5 p-5">
						<div class="mb-3 flex items-center justify-between">
							<h2 class="text-sm font-bold uppercase tracking-wider text-amber-500">⚠ Approval Queue</h2>
							<span class="rounded-full bg-amber-500/20 px-2 py-0.5 text-xs text-amber-300">{dashboard.approvalQueue.length}</span>
						</div>
						<div class="space-y-1.5">
							{#each dashboard.approvalQueue.slice(0, 3) as task}
								<div class="flex items-center gap-2 rounded-xl border border-amber-400/15 bg-amber-400/8 px-3 py-2">
									<p class="flex-1 text-xs text-amber-200 truncate">{task.title}</p>
									<a href="/tasks" class="text-[10px] text-amber-400 hover:text-amber-300">Review →</a>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>

		<!-- Recent Daily Log + Running Agents -->
		<div class="grid gap-5 lg:grid-cols-2">
			<!-- Recent Log -->
			<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-5">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-sm font-bold uppercase tracking-wider text-slate-500">📓 Recent Activity</h2>
					<a href="/daily-log" class="text-xs text-cyan-400 hover:text-cyan-300">Full Log →</a>
				</div>
				{#if dashboard.recentLog.length === 0}
					<p class="py-2 text-center text-sm text-slate-500">No activity yet.</p>
				{:else}
					<div class="space-y-2">
						{#each dashboard.recentLog as entry}
							<div class="flex items-center gap-2.5">
								<span class="text-base">{logTypeIcon[entry.entryType] ?? '📌'}</span>
								<div class="flex-1 min-w-0">
									<p class="text-xs text-white truncate">{entry.title}</p>
									<p class="text-[10px] text-slate-600">{new Date(entry.createdAt).toLocaleTimeString()}</p>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Running Agents + Cross-layer links -->
			<div class="space-y-4">
				<!-- Running Agents -->
				<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-5">
					<h2 class="mb-3 text-sm font-bold uppercase tracking-wider text-slate-500">🤖 Running Agents</h2>
					{#if dashboard.runningAgents.length === 0}
						<p class="py-2 text-center text-sm text-slate-500">No agents running.</p>
					{:else}
						{#each dashboard.runningAgents as agent}
							<div class="flex items-center gap-2 rounded-xl border border-cyan-400/15 bg-cyan-400/8 px-3 py-2">
								<span class="h-2 w-2 animate-pulse rounded-full bg-cyan-400"></span>
								<p class="text-xs text-cyan-200">{agent}</p>
							</div>
						{/each}
					{/if}
				</div>

				<!-- Cross-layer links -->
				<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-5">
					<h2 class="mb-3 text-sm font-bold uppercase tracking-wider text-slate-500">Quick Links</h2>
					<div class="grid grid-cols-2 gap-2">
						{#each [
							{ href: '/memory', label: '🧠 Memory', color: 'border-cyan-400/20 bg-cyan-400/8 text-cyan-300 hover:bg-cyan-400/15' },
							{ href: '/tasks', label: '✅ Tasks', color: 'border-emerald-400/20 bg-emerald-400/8 text-emerald-300 hover:bg-emerald-400/15' },
							{ href: '/kanban', label: '📋 Kanban', color: 'border-blue-400/20 bg-blue-400/8 text-blue-300 hover:bg-blue-400/15' },
							{ href: '/inbox', label: '📥 Inbox', color: 'border-pink-400/20 bg-pink-400/8 text-pink-300 hover:bg-pink-400/15' },
							{ href: '/daily-log', label: '📓 Log', color: 'border-amber-400/20 bg-amber-400/8 text-amber-300 hover:bg-amber-400/15' },
							{ href: '/workflows', label: '⚙️ Workflows', color: 'border-purple-400/20 bg-purple-400/8 text-purple-300 hover:bg-purple-400/15' },
						] as link}
							<a href={link.href} class="rounded-xl border px-3 py-2.5 text-xs font-medium text-center transition {link.color}">{link.label}</a>
						{/each}
					</div>
				</div>

				<!-- RalphHub stats -->
				<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-4">
					<div class="grid grid-cols-2 gap-3">
						<div class="text-center">
							<p class="text-lg font-bold text-white">{dashboard.managedProjectCount}</p>
							<p class="text-[10px] text-slate-500">Managed Repos</p>
						</div>
						<div class="text-center">
							<p class="text-lg font-bold text-white">{dashboard.workflowRunCount}</p>
							<p class="text-[10px] text-slate-500">Workflow Runs</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>
