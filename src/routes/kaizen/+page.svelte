<script lang="ts">
	import { onMount } from 'svelte';
	import {
		loadKaizenTasks,
		createKaizenTask,
		updateTaskStatus,
		getMemoryStats,
		memoryStatsStore,
	} from '$lib/utils/provider-registry';

	type Task = {
		id: string;
		title: string;
		description: string;
		status: string;
		priority: string;
		source: string;
		providerId: string;
		usageLogId: string;
		createdAt: string;
		updatedAt: string;
	};

	// ─── State ───────────────────────────────────────────────────────────────────
	let tasks: Task[] = $state([]);
	let loading = $state(true);
	let filterStatus = $state('all');
	let showAdd = $state(false);
	let saving = $state(false);
	let msg = $state('');

	// Form
	let fTitle = $state('');
	let fDesc = $state('');
	let fPriority = $state('medium');

	const STATUS_LABELS: Record<string, {label: string; color: string; emoji: string}> = {
		'todo':        { label: 'To Do',       color: 'text-slate-400 bg-slate-800/60',       emoji: '○' },
		'in_progress': { label: 'In Progress', color: 'text-cyan-300 bg-cyan-900/30',          emoji: '◎' },
		'done':        { label: 'Done',         color: 'text-emerald-300 bg-emerald-900/30',    emoji: '●' },
		'blocked':     { label: 'Blocked',      color: 'text-red-300 bg-red-900/30',            emoji: '✕' },
	};

	const PRIORITY_LABELS: Record<string, {label: string; color: string}> = {
		'low':      { label: 'Low',    color: 'text-slate-400' },
		'medium':   { label: 'Medium', color: 'text-amber-400' },
		'high':     { label: 'High',   color: 'text-orange-400' },
		'critical': { label: 'Critical', color: 'text-red-400' },
	};

	// ─── Derived ─────────────────────────────────────────────────────────────────
	let filtered = $derived(
		filterStatus === 'all' ? tasks : tasks.filter(t => t.status === filterStatus)
	);

	let todayCount = $derived(tasks.filter(t => t.status === 'in_progress').length);
	let doneCount = $derived(tasks.filter(t => t.status === 'done').length);
	let blockedCount = $derived(tasks.filter(t => t.status === 'blocked').length);

	// ─── Load ───────────────────────────────────────────────────────────────────
	async function loadTasks() {
		loading = true;
		try {
			tasks = (await loadKaizenTasks()) as Task[];
		} catch (e) {
			// Web mode: show sample tasks
			tasks = [
				{ id: '1', title: 'Connect your first AI provider', description: 'Go to Settings and add an API key to unlock all features.', status: 'todo', priority: 'high', source: 'system', providerId: '', usageLogId: '', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
				{ id: '2', title: 'Try Vy desktop agent', description: 'Open Computer Control and start an agent session.', status: 'todo', priority: 'medium', source: 'system', providerId: '', usageLogId: '', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
				{ id: '3', title: 'Deploy a remote node', description: 'Set up a VPS or Raspberry Pi for background execution.', status: 'todo', priority: 'low', source: 'system', providerId: '', usageLogId: '', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
			];
		} finally {
			loading = false;
		}
	}

	onMount(async () => {
		await loadTasks();
		try { await getMemoryStats(); } catch { /* non-critical */ }
	});

	// ─── Actions ─────────────────────────────────────────────────────────────────
	async function addTask() {
		if (!fTitle.trim()) return;
		saving = true;
		msg = '';
		try {
			await createKaizenTask({
				title: fTitle.trim(),
				description: fDesc.trim(),
				priority: fPriority,
				source: 'manual',
				providerId: '',
				usageLogId: '',
			});
			showAdd = false;
			fTitle = '';
			fDesc = '';
			fPriority = 'medium';
			await loadTasks();
			msg = '✓ Task created';
		} catch (e) {
			msg = `Error: ${e}`;
		} finally {
			saving = false;
		}
	}

	async function cycleStatus(task: Task) {
		const cycle: Record<string, string> = { 'todo': 'in_progress', 'in_progress': 'done', 'done': 'todo', 'blocked': 'todo' };
		const next = cycle[task.status] ?? 'todo';
		try {
			await updateTaskStatus(task.id, next);
			await loadTasks();
		} catch (e) {
			msg = `Error: ${e}`;
		}
	}

	function timeAgo(iso: string): string {
		try {
			const diff = Date.now() - new Date(iso).getTime();
			if (diff < 60_000) return 'just now';
			if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
			if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
			return `${Math.floor(diff / 86_400_000)}d ago`;
		} catch { return ''; }
	}
</script>

<div class="space-y-6 py-2">

	<!-- Header -->
	<div class="page-header">
		<div>
			<h1 class="page-title">♾️ Kaizen Tasks</h1>
			<p class="page-subtitle">Auto-created from every API call, agent action, and voice command.</p>
		</div>
		<button onclick={() => showAdd = true} class="btn-primary shrink-0">
			+ New Task
		</button>
	</div>

	<!-- Stats row -->
	<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
		{#each [
			{ label: 'Total', value: tasks.length, icon: '♾️', cls: 'text-slate-200' },
			{ label: 'In Progress', value: todayCount, icon: '◎', cls: 'text-cyan-300' },
			{ label: 'Done', value: doneCount, icon: '●', cls: 'text-emerald-300' },
			{ label: 'Blocked', value: blockedCount, icon: '✕', cls: 'text-red-300' },
		] as stat}
			<div class="stat-card">
				<p class="text-xs font-medium text-slate-500 uppercase tracking-wider">{stat.label}</p>
				<p class="mt-1.5 text-2xl font-bold {stat.cls} tabular-nums">{stat.value}</p>
			</div>
		{/each}
	</div>

	<!-- Filter tabs -->
	<div class="flex gap-2 flex-wrap">
		{#each [['all','All'], ['todo','To Do'], ['in_progress','In Progress'], ['done','Done'], ['blocked','Blocked']] as [val, lbl]}
			<button
				onclick={() => filterStatus = val}
				class="rounded-xl px-3.5 py-2 text-sm font-medium transition {filterStatus === val ? 'bg-violet-500/20 text-violet-200 border border-violet-400/25' : 'bg-white/5 text-slate-400 hover:text-slate-200 border border-white/7'}"
			>
				{lbl}
			</button>
		{/each}
	</div>

	<!-- Task list -->
	{#if loading}
		<div class="flex items-center justify-center py-16">
			<div class="text-center">
				<div class="spinner mx-auto mb-3" style="width:32px;height:32px;"></div>
				<p class="text-sm text-slate-500">Loading tasks…</p>
			</div>
		</div>
	{:else if filtered.length === 0}
		<div class="card text-center py-12">
			<p class="text-3xl mb-3">♾️</p>
			<p class="text-slate-300 font-medium">No tasks yet</p>
			<p class="text-sm text-slate-500 mt-1">Tasks auto-create from every agent action and API call.</p>
			<button onclick={() => showAdd = true} class="btn-primary mt-4 text-sm">Create first task</button>
		</div>
	{:else}
		<div class="space-y-2">
			{#each filtered as task (task.id)}
				<div class="card flex items-start gap-4 py-3.5 px-4">
					<!-- Status toggle -->
					<button
						onclick={() => cycleStatus(task)}
						class="mt-0.5 shrink-0 rounded-lg w-8 h-8 flex items-center justify-center text-base font-bold transition hover:scale-110 active:scale-95 {STATUS_LABELS[task.status]?.color ?? 'text-slate-400 bg-slate-800/60'}"
						data-tooltip="Click to cycle status"
					>
						{STATUS_LABELS[task.status]?.emoji ?? '○'}
					</button>

					<!-- Content -->
					<div class="flex-1 min-w-0">
						<p class="font-semibold text-slate-100 text-sm leading-5 {task.status === 'done' ? 'line-through text-slate-500' : ''}">{task.title}</p>
						{#if task.description}
							<p class="mt-1 text-xs text-slate-500 leading-5">{task.description}</p>
						{/if}
						<div class="mt-2 flex flex-wrap items-center gap-2">
							<span class="rounded-full px-2 py-0.5 text-[10px] font-semibold {PRIORITY_LABELS[task.priority]?.color ?? 'text-slate-400'} bg-white/5">
								{PRIORITY_LABELS[task.priority]?.label ?? task.priority}
							</span>
							<span class="text-[10px] text-slate-600">{task.source}</span>
							<span class="text-[10px] text-slate-600">{timeAgo(task.createdAt)}</span>
						</div>
					</div>

					<!-- Status label (desktop) -->
					<span class="hidden sm:inline shrink-0 rounded-lg px-2.5 py-1 text-xs font-medium {STATUS_LABELS[task.status]?.color ?? 'text-slate-400 bg-slate-800/60'}">
						{STATUS_LABELS[task.status]?.label ?? task.status}
					</span>
				</div>
			{/each}
		</div>
	{/if}

	{#if msg}
		<div class="rounded-xl border border-emerald-400/20 bg-emerald-400/8 px-4 py-2.5 text-sm text-emerald-300">{msg}</div>
	{/if}

	<!-- Memory stats -->
	{#if $memoryStatsStore}
		<div class="card">
			<h3 class="mb-3 text-sm font-semibold text-slate-300">Memory Spine Stats</h3>
			<div class="grid grid-cols-3 gap-3 text-center">
				<div>
					<p class="text-lg font-bold text-white tabular-nums">{$memoryStatsStore.totalEntries}</p>
					<p class="text-xs text-slate-500">Entries</p>
				</div>
				<div>
					<p class="text-lg font-bold text-white tabular-nums">{$memoryStatsStore.totalTokens.toLocaleString()}</p>
					<p class="text-xs text-slate-500">Tokens</p>
				</div>
				<div>
					<p class="text-lg font-bold text-white tabular-nums">${$memoryStatsStore.totalCostUsd.toFixed(4)}</p>
					<p class="text-xs text-slate-500">Cost</p>
				</div>
			</div>
		</div>
	{/if}
</div>

<!-- Add task modal -->
{#if showAdd}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm px-4" onclick={(e) => { if (e.target === e.currentTarget) showAdd = false; }}>
		<div class="w-full max-w-md rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl">
			<h2 class="mb-5 text-lg font-bold text-white">New Kaizen Task</h2>

			<div class="space-y-4">
				<div>
					<label class="mb-1.5 block text-xs font-medium text-slate-400" for="task-title">Title *</label>
					<input id="task-title" type="text" bind:value={fTitle} placeholder="e.g. Review API workflow output" class="w-full rounded-xl bg-white/5 border border-white/10 px-3 py-2.5 text-sm text-slate-100 placeholder:text-slate-600 focus:border-violet-400/50 focus:ring-2 focus:ring-violet-400/10" />
				</div>

				<div>
					<label class="mb-1.5 block text-xs font-medium text-slate-400" for="task-desc">Description</label>
					<textarea id="task-desc" bind:value={fDesc} rows="3" placeholder="Optional context…" class="w-full rounded-xl bg-white/5 border border-white/10 px-3 py-2.5 text-sm text-slate-100 placeholder:text-slate-600 resize-none focus:border-violet-400/50 focus:ring-2 focus:ring-violet-400/10"></textarea>
				</div>

				<div>
					<label class="mb-1.5 block text-xs font-medium text-slate-400" for="task-priority">Priority</label>
					<select id="task-priority" bind:value={fPriority} class="w-full rounded-xl bg-slate-800 border border-white/10 px-3 py-2.5 text-sm text-slate-100">
						<option value="low">Low</option>
						<option value="medium">Medium</option>
						<option value="high">High</option>
						<option value="critical">Critical</option>
					</select>
				</div>
			</div>

			<div class="mt-6 flex gap-3 justify-end">
				<button onclick={() => showAdd = false} class="btn-ghost text-sm">Cancel</button>
				<button onclick={addTask} disabled={saving || !fTitle.trim()} class="btn-primary text-sm disabled:opacity-50">
					{saving ? 'Saving…' : 'Create Task'}
				</button>
			</div>
		</div>
	</div>
{/if}
