<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type Task = {
		id: string;
		title: string;
		description?: string;
		domain: string;
		status: string;
		isToday: boolean;
		isMinimumVersion: boolean;
		priority: number;
		energy: string;
		estimatedMinutes?: number;
		tags: string[];
		dueDate?: string;
		createdAt: string;
	};

	type Domain = {
		id: string;
		name: string;
		color: string;
		icon: string;
	};

	let tasks: Task[] = [];
	let allTasks: Task[] = [];
	let domains: Domain[] = [];
	let loading = true;
	let showAddModal = false;
	let showPickModal = false;
	let newTitle = '';
	let newDomain = 'general';
	let newEnergy = 'medium';
	let newEstimated = '';
	let newMinVersion = false;
	let savingTask = false;
	let statusMsg = '';
	let completedCount = 0;
	let isDesktop = false;

	const ENERGY_LABELS: Record<string, string> = {
		low: '🌱 Low',
		medium: '⚡ Medium',
		high: '🔥 High'
	};

	const STATUS_ORDER = ['todo', 'in-progress', 'done'];

	async function loadData() {
		if (!isDesktopRuntime()) {
			loading = false;
			isDesktop = false;
			return;
		}
		isDesktop = true;
		try {
			const [todayTasks, doms, all] = await Promise.all([
				invokeTauri<Task[]>('list_kaizen_tasks', { todayOnly: true }),
				invokeTauri<Domain[]>('list_kaizen_domains'),
				invokeTauri<Task[]>('list_kaizen_tasks', {})
			]);
			tasks = todayTasks;
			domains = doms;
			allTasks = all.filter((t) => !t.isToday);
			completedCount = todayTasks.filter((t) => t.status === 'done').length;
		} catch (e) {
			statusMsg = String(e);
		} finally {
			loading = false;
		}
	}

	onMount(loadData);

	async function quickAddTask() {
		if (!newTitle.trim()) return;
		savingTask = true;
		try {
			await invokeTauri('create_kaizen_task', {
				request: {
					title: newTitle.trim(),
					domain: newDomain,
					isToday: true,
					isMinimumVersion: newMinVersion,
					energy: newEnergy,
					estimatedMinutes: newEstimated ? parseInt(newEstimated) : null
				}
			});
			newTitle = '';
			newEstimated = '';
			newMinVersion = false;
			showAddModal = false;
			await loadData();
		} catch (e) {
			statusMsg = String(e);
		} finally {
			savingTask = false;
		}
	}

	async function cycleStatus(task: Task) {
		const next = STATUS_ORDER[(STATUS_ORDER.indexOf(task.status) + 1) % STATUS_ORDER.length];
		try {
			await invokeTauri('update_kaizen_task', {
				request: { id: task.id, status: next }
			});
			await loadData();
		} catch {}
	}

	async function removeFromToday(task: Task) {
		await invokeTauri('update_kaizen_task', {
			request: { id: task.id, isToday: false }
		});
		await loadData();
	}

	async function addToToday(task: Task) {
		await invokeTauri('update_kaizen_task', {
			request: { id: task.id, isToday: true }
		});
		showPickModal = false;
		await loadData();
	}

	function domainColor(id: string) {
		return domains.find((d) => d.id === id)?.color ?? '#6366f1';
	}

	function domainIcon(id: string) {
		return domains.find((d) => d.id === id)?.icon ?? '⭐';
	}

	$: mvpTasks = tasks.filter((t) => t.isMinimumVersion);
	$: regularTasks = tasks.filter((t) => !t.isMinimumVersion);
	$: progressPct = tasks.length ? Math.round((completedCount / tasks.length) * 100) : 0;

	const now = new Date();
	const greeting = now.getHours() < 12 ? 'Good morning' : now.getHours() < 17 ? 'Good afternoon' : 'Good evening';
	const dateStr = now.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' });
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-amber-400/20 bg-gradient-to-br from-amber-950/40 via-slate-950/80 to-orange-950/30 p-7 backdrop-blur">
		<div class="flex items-start justify-between gap-4">
			<div>
				<p class="text-xs uppercase tracking-[0.3em] text-amber-300/70">{dateStr}</p>
				<h1 class="mt-2 text-3xl font-bold text-white">{greeting} ☀️</h1>
				<p class="mt-2 text-sm text-slate-400">Focus on today. Everything else waits.</p>
			</div>
			<div class="text-right">
				<p class="text-3xl font-bold text-white">{completedCount}/{tasks.length}</p>
				<p class="text-xs text-slate-500 mt-0.5">tasks done</p>
			</div>
		</div>

		<!-- Progress bar -->
		{#if tasks.length > 0}
			<div class="mt-5">
				<div class="flex justify-between text-xs text-slate-500 mb-1.5">
					<span>Daily Progress</span>
					<span>{progressPct}%</span>
				</div>
				<div class="h-2.5 rounded-full bg-white/8">
					<div
						class="h-full rounded-full bg-gradient-to-r from-amber-400 to-orange-400 transition-all duration-700"
						style="width: {progressPct}%"
					></div>
				</div>
			</div>
		{/if}

		<div class="mt-5 flex gap-3">
			<button
				onclick={() => showAddModal = true}
				class="rounded-xl bg-amber-400 px-5 py-2.5 text-sm font-bold text-amber-950 shadow-lg transition hover:bg-amber-300"
			>
				+ Add Task
			</button>
			<button
				onclick={() => showPickModal = true}
				class="rounded-xl border border-white/12 bg-white/5 px-5 py-2.5 text-sm font-semibold text-white transition hover:bg-white/10"
			>
				📋 Pick from Backlog
			</button>
		</div>
	</div>

	{#if loading}
		<div class="rounded-xl border border-white/8 bg-slate-950/50 p-8 text-center text-sm text-slate-400">
			Loading your day…
		</div>
	{:else if !isDesktop}
		<div class="rounded-xl border border-amber-400/20 bg-amber-400/8 p-6 text-sm text-amber-200">
			<p class="font-bold mb-2">Browser mode — no persistence</p>
			<p>Launch the AmitOS desktop app to save tasks and use the full Today Board.</p>
		</div>
	{:else}

		<!-- Minimum Version section -->
		{#if mvpTasks.length > 0}
			<div class="rounded-2xl border border-rose-400/20 bg-rose-950/20 p-5 backdrop-blur">
				<div class="mb-4 flex items-center gap-2">
					<span class="text-lg">🎯</span>
					<h2 class="font-bold text-rose-100">Minimum Version — Must Do Today</h2>
					<span class="ml-auto rounded-full bg-rose-400/15 px-2 py-0.5 text-xs font-semibold text-rose-300">{mvpTasks.length}</span>
				</div>
				<div class="space-y-2.5">
					{#each mvpTasks as task}
						<div class={`group flex items-start gap-3 rounded-xl border p-4 transition ${task.status === 'done' ? 'border-white/5 bg-white/3 opacity-60' : 'border-rose-400/15 bg-rose-950/30'}`}>
							<button
								onclick={() => cycleStatus(task)}
								class={`mt-0.5 h-5 w-5 shrink-0 rounded-full border-2 transition ${
									task.status === 'done' ? 'border-emerald-400 bg-emerald-400' : task.status === 'in-progress' ? 'border-amber-400 bg-amber-400/30' : 'border-slate-600 hover:border-rose-400'
								}`}
							>
								{#if task.status === 'done'}<span class="flex h-full items-center justify-center text-[9px] text-white">✓</span>{/if}
							</button>
							<div class="flex-1 min-w-0">
								<p class={`text-sm font-medium ${task.status === 'done' ? 'line-through text-slate-500' : 'text-white'}`}>{task.title}</p>
								<div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-slate-500">
									<span>{domainIcon(task.domain)} {task.domain}</span>
									{#if task.estimatedMinutes}<span>⏱ {task.estimatedMinutes}m</span>{/if}
									<span>{ENERGY_LABELS[task.energy] ?? task.energy}</span>
								</div>
							</div>
							<button onclick={() => removeFromToday(task)} class="ml-2 shrink-0 rounded-lg px-2 py-1 text-xs text-slate-600 opacity-0 transition group-hover:opacity-100 hover:text-rose-400">✕</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Regular tasks -->
		{#if regularTasks.length > 0}
			<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
				<div class="mb-4 flex items-center gap-2">
					<span class="text-lg">📋</span>
					<h2 class="font-bold text-white">Today's Tasks</h2>
					<span class="ml-auto rounded-full bg-white/10 px-2 py-0.5 text-xs font-semibold text-slate-300">{regularTasks.length}</span>
				</div>
				<div class="space-y-2.5">
					{#each regularTasks as task}
						<div class={`group flex items-start gap-3 rounded-xl border p-4 transition ${task.status === 'done' ? 'border-white/5 bg-white/3 opacity-60' : 'border-white/8 bg-white/3 hover:border-white/15'}`}>
							<button
								onclick={() => cycleStatus(task)}
								class={`mt-0.5 h-5 w-5 shrink-0 rounded-full border-2 transition ${
									task.status === 'done' ? 'border-emerald-400 bg-emerald-400' : task.status === 'in-progress' ? 'border-amber-400 bg-amber-400/30' : 'border-slate-600 hover:border-violet-400'
								}`}
							>
								{#if task.status === 'done'}<span class="flex h-full items-center justify-center text-[9px] text-white">✓</span>{/if}
							</button>
							<div class="flex-1 min-w-0">
								<p class={`text-sm font-medium ${task.status === 'done' ? 'line-through text-slate-500' : 'text-white'}`}>{task.title}</p>
								<div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-slate-500">
									<span style="color: {domainColor(task.domain)}">{domainIcon(task.domain)}</span>
									<span>{task.domain}</span>
									{#if task.estimatedMinutes}<span>⏱ {task.estimatedMinutes}m</span>{/if}
									<span>{ENERGY_LABELS[task.energy] ?? task.energy}</span>
								</div>
							</div>
							<button onclick={() => removeFromToday(task)} class="ml-2 shrink-0 rounded-lg px-2 py-1 text-xs text-slate-600 opacity-0 transition group-hover:opacity-100 hover:text-rose-400">✕</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		{#if tasks.length === 0}
			<div class="rounded-2xl border border-dashed border-white/12 p-12 text-center">
				<p class="text-4xl mb-3">☀️</p>
				<p class="text-base font-semibold text-white mb-2">No tasks for today</p>
				<p class="text-sm text-slate-400 mb-5">Add a task or pick from your backlog to get started.</p>
				<button onclick={() => showAddModal = true} class="rounded-xl bg-amber-400 px-5 py-2.5 text-sm font-bold text-amber-950">+ Add First Task</button>
			</div>
		{/if}
	{/if}

	{#if statusMsg}
		<div class="rounded-xl border border-rose-400/20 bg-rose-950/20 p-3 text-xs text-rose-300">{statusMsg}</div>
	{/if}
</section>

<!-- Add Task Modal -->
{#if showAddModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
		<div class="w-full max-w-md rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl">
			<h3 class="mb-5 text-lg font-bold text-white">Add Task to Today</h3>
			<div class="space-y-4">
				<input
					bind:value={newTitle}
					placeholder="What needs to get done?"
					class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-violet-400"
					onkeydown={(e) => e.key === 'Enter' && quickAddTask()}
					autofocus
				/>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="mb-1.5 block text-xs text-slate-400">Domain</label>
						<select bind:value={newDomain} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
							<option value="general">⭐ General</option>
							<option value="work">💼 Work</option>
							<option value="health">🏃 Health</option>
							<option value="learning">📚 Learning</option>
							<option value="creative">🎨 Creative</option>
							<option value="relationships">❤️ Relationships</option>
							<option value="finance">💰 Finance</option>
							<option value="home">🏠 Home</option>
						</select>
					</div>
					<div>
						<label class="mb-1.5 block text-xs text-slate-400">Energy</label>
						<select bind:value={newEnergy} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
							<option value="low">🌱 Low</option>
							<option value="medium">⚡ Medium</option>
							<option value="high">🔥 High</option>
						</select>
					</div>
				</div>
				<div>
					<label class="mb-1.5 block text-xs text-slate-400">Estimated minutes (optional)</label>
					<input bind:value={newEstimated} type="number" placeholder="e.g. 30" class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-2.5 text-sm text-white outline-none focus:border-violet-400" />
				</div>
				<label class="flex cursor-pointer items-center gap-3 rounded-xl border border-rose-400/20 bg-rose-950/20 px-4 py-3">
					<input type="checkbox" bind:checked={newMinVersion} class="h-4 w-4 rounded" />
					<div>
						<p class="text-sm font-medium text-rose-100">🎯 Minimum Version</p>
						<p class="text-xs text-slate-400">Must-do today, no excuses</p>
					</div>
				</label>
				<div class="flex gap-3 pt-2">
					<button onclick={quickAddTask} disabled={savingTask || !newTitle.trim()} class="flex-1 rounded-xl bg-amber-400 py-2.5 text-sm font-bold text-amber-950 disabled:opacity-50">
						{savingTask ? 'Adding…' : 'Add Task'}
					</button>
					<button onclick={() => { showAddModal = false; newTitle = ''; }} class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 text-sm text-white">
						Cancel
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Pick from Backlog Modal -->
{#if showPickModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
		<div class="w-full max-w-lg rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl max-h-[80vh] flex flex-col">
			<div class="flex items-center justify-between mb-4">
				<h3 class="text-lg font-bold text-white">Pick from Backlog</h3>
				<button onclick={() => showPickModal = false} class="text-slate-500 hover:text-white">✕</button>
			</div>
			<div class="overflow-y-auto flex-1 space-y-2">
				{#if allTasks.length === 0}
					<p class="py-8 text-center text-sm text-slate-400">No backlog tasks. Add some in Kaizen.</p>
				{:else}
					{#each allTasks.filter(t => t.status !== 'done') as task}
						<button
							onclick={() => addToToday(task)}
							class="w-full rounded-xl border border-white/8 bg-white/3 p-3.5 text-left transition hover:border-violet-400/30 hover:bg-violet-400/8"
						>
							<p class="text-sm font-medium text-white">{task.title}</p>
							<p class="mt-1 text-xs text-slate-500">{task.domain} · {ENERGY_LABELS[task.energy] ?? task.energy}</p>
						</button>
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}
