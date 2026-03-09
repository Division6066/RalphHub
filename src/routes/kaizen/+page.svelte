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
		subtasks: string[];
		tags: string[];
		createdAt: string;
		updatedAt: string;
	};

	type Domain = {
		id: string;
		name: string;
		color: string;
		icon: string;
		description: string;
		taskCount: number;
		todayCount: number;
	};

	let tasks: Task[] = [];
	let domains: Domain[] = [];
	let loading = true;
	let selectedDomain = 'all';
	let selectedStatus = 'all';
	let showAddModal = false;
	let showDecomposeModal = false;
	let decomposeTarget: Task | null = null;
	let decomposeInput = '';
	let savingTask = false;
	let statusMsg = '';
	let isDesktop = false;

	// New task form
	let newTitle = '';
	let newDesc = '';
	let newDomain = 'general';
	let newEnergy = 'medium';
	let newPriority = 3;
	let newMinVersion = false;
	let newToday = false;
	let newEstimated = '';
	let newTags = '';

	async function loadData() {
		if (!isDesktopRuntime()) { isDesktop = false; loading = false; return; }
		isDesktop = true;
		try {
			const [t, d] = await Promise.all([
				invokeTauri<Task[]>('list_kaizen_tasks', {}),
				invokeTauri<Domain[]>('list_kaizen_domains')
			]);
			tasks = t;
			domains = d;
		} catch (e) { statusMsg = String(e); }
		finally { loading = false; }
	}

	onMount(loadData);

	async function addTask() {
		if (!newTitle.trim()) return;
		savingTask = true;
		try {
			await invokeTauri('create_kaizen_task', {
				request: {
					title: newTitle.trim(),
					description: newDesc || null,
					domain: newDomain,
					isToday: newToday,
					isMinimumVersion: newMinVersion,
					priority: newPriority,
					energy: newEnergy,
					estimatedMinutes: newEstimated ? parseInt(newEstimated) : null,
					tags: newTags ? newTags.split(',').map((t) => t.trim()).filter(Boolean) : []
				}
			});
			showAddModal = false;
			resetForm();
			await loadData();
		} catch (e) { statusMsg = String(e); }
		finally { savingTask = false; }
	}

	async function toggleStatus(task: Task) {
		const next = task.status === 'done' ? 'todo' : task.status === 'todo' ? 'in-progress' : 'done';
		await invokeTauri('update_kaizen_task', { request: { id: task.id, status: next } });
		await loadData();
	}

	async function toggleToday(task: Task) {
		await invokeTauri('update_kaizen_task', { request: { id: task.id, isToday: !task.isToday } });
		await loadData();
	}

	async function deleteTask(id: string) {
		if (!confirm('Delete this task?')) return;
		await invokeTauri('delete_kaizen_task', { id });
		await loadData();
	}

	async function decompose() {
		if (!decomposeTarget || !decomposeInput.trim()) return;
		const subtaskTitles = decomposeInput.split('\n').map(t => t.trim()).filter(Boolean);
		await invokeTauri('decompose_task', { parentId: decomposeTarget.id, subtaskTitles });
		showDecomposeModal = false;
		decomposeInput = '';
		decomposeTarget = null;
		await loadData();
	}

	function resetForm() {
		newTitle = ''; newDesc = ''; newDomain = 'general'; newEnergy = 'medium';
		newPriority = 3; newMinVersion = false; newToday = false; newEstimated = ''; newTags = '';
	}

	function domainInfo(id: string) {
		return domains.find((d) => d.id === id) ?? { color: '#6366f1', icon: '⭐', name: id };
	}

	$: filteredTasks = tasks.filter((t) => {
		if (selectedDomain !== 'all' && t.domain !== selectedDomain) return false;
		if (selectedStatus !== 'all' && t.status !== selectedStatus) return false;
		return true;
	});

	$: todoCount = tasks.filter(t => t.status === 'todo').length;
	$: inProgressCount = tasks.filter(t => t.status === 'in-progress').length;
	$: doneCount = tasks.filter(t => t.status === 'done').length;

	const PRIORITY_LABEL: Record<number, string> = { 1: '🔴 Urgent', 2: '🟠 High', 3: '🟡 Normal', 4: '🟢 Low' };
	const ENERGY_LABEL: Record<string, string> = { low: '🌱', medium: '⚡', high: '🔥' };
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-violet-400/20 bg-gradient-to-br from-violet-950/50 via-slate-950/80 to-purple-950/30 p-7 backdrop-blur">
		<div class="flex items-start justify-between gap-4">
			<div>
				<p class="text-xs uppercase tracking-[0.3em] text-violet-300/70">Continuous Improvement</p>
				<h1 class="mt-2 text-3xl font-bold text-white">♾️ Kaizen OS</h1>
				<p class="mt-2 text-sm text-slate-400">All your domains, tasks, and goals in one place.</p>
			</div>
			<div class="flex gap-3 text-center">
				<div class="rounded-xl bg-white/5 px-4 py-3">
					<p class="text-2xl font-bold text-white">{todoCount}</p>
					<p class="text-xs text-slate-500">Todo</p>
				</div>
				<div class="rounded-xl bg-amber-400/10 px-4 py-3">
					<p class="text-2xl font-bold text-amber-300">{inProgressCount}</p>
					<p class="text-xs text-slate-500">Active</p>
				</div>
				<div class="rounded-xl bg-emerald-400/10 px-4 py-3">
					<p class="text-2xl font-bold text-emerald-300">{doneCount}</p>
					<p class="text-xs text-slate-500">Done</p>
				</div>
			</div>
		</div>
		<div class="mt-5 flex gap-3">
			<button onclick={() => showAddModal = true} class="rounded-xl bg-violet-500 px-5 py-2.5 text-sm font-bold text-white shadow-lg transition hover:bg-violet-400">
				+ New Task
			</button>
			<a href="/today" class="rounded-xl border border-amber-400/25 bg-amber-400/10 px-5 py-2.5 text-sm font-semibold text-amber-200 transition hover:bg-amber-400/20">
				☀️ Today Board
			</a>
		</div>
	</div>

	<!-- Domain cards -->
	{#if domains.length > 0}
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-4 xl:grid-cols-8">
			<button
				onclick={() => selectedDomain = 'all'}
				class={`rounded-xl border p-3 text-center transition ${selectedDomain === 'all' ? 'border-violet-400/30 bg-violet-400/15' : 'border-white/8 bg-slate-950/50 hover:border-white/15'}`}
			>
				<p class="text-lg">🌐</p>
				<p class="mt-1 text-xs font-medium text-white">All</p>
				<p class="text-xs text-slate-500">{tasks.length}</p>
			</button>
			{#each domains as domain}
				<button
					onclick={() => selectedDomain = domain.id}
					class={`rounded-xl border p-3 text-center transition ${selectedDomain === domain.id ? 'border-white/20 bg-white/10' : 'border-white/8 bg-slate-950/50 hover:border-white/15'}`}
					style={selectedDomain === domain.id ? `border-color: ${domain.color}40; background: ${domain.color}18;` : ''}
				>
					<p class="text-lg">{domain.icon}</p>
					<p class="mt-1 text-xs font-medium text-white truncate">{domain.name.split(' ')[0]}</p>
					<p class="text-xs text-slate-500">{domain.taskCount}</p>
				</button>
			{/each}
		</div>
	{/if}

	<!-- Filters + task list -->
	<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
		<div class="mb-5 flex flex-wrap items-center gap-3">
			<h2 class="text-base font-bold text-white">Tasks</h2>
			<div class="ml-auto flex gap-2">
				{#each ['all', 'todo', 'in-progress', 'done'] as s}
					<button
						onclick={() => selectedStatus = s}
						class={`rounded-lg px-3 py-1 text-xs font-medium transition ${selectedStatus === s ? 'bg-violet-500/25 text-violet-100' : 'text-slate-400 hover:text-white'}`}
					>
						{s === 'all' ? 'All' : s === 'in-progress' ? 'Active' : s.charAt(0).toUpperCase() + s.slice(1)}
					</button>
				{/each}
			</div>
		</div>

		{#if loading}
			<div class="py-10 text-center text-sm text-slate-400">Loading tasks…</div>
		{:else if !isDesktop}
			<div class="py-8 text-center text-sm text-slate-400">
				<p class="font-medium text-white mb-2">Browser mode</p>
				<p>Launch the desktop app to use Kaizen persistence.</p>
			</div>
		{:else if filteredTasks.length === 0}
			<div class="py-10 text-center">
				<p class="text-3xl mb-3">♾️</p>
				<p class="text-sm text-slate-400">No tasks here. Add one above!</p>
			</div>
		{:else}
			<div class="space-y-2">
				{#each filteredTasks as task}
					{@const info = domainInfo(task.domain)}
					<div class={`group rounded-xl border p-4 transition ${task.status === 'done' ? 'border-white/5 bg-white/2 opacity-60' : 'border-white/8 bg-white/3 hover:border-white/15'}`}>
						<div class="flex items-start gap-3">
							<button
								onclick={() => toggleStatus(task)}
								class={`mt-0.5 h-5 w-5 shrink-0 rounded-full border-2 transition ${
									task.status === 'done' ? 'border-emerald-400 bg-emerald-400' : task.status === 'in-progress' ? 'border-amber-400 bg-amber-400/30' : 'border-slate-600 hover:border-violet-400'
								}`}
							>
								{#if task.status === 'done'}<span class="flex h-full items-center justify-center text-[8px] text-white font-bold">✓</span>{/if}
							</button>

							<div class="flex-1 min-w-0">
								<div class="flex flex-wrap items-center gap-2">
									<p class={`text-sm font-medium ${task.status === 'done' ? 'line-through text-slate-500' : 'text-white'}`}>{task.title}</p>
									{#if task.isMinimumVersion}
										<span class="rounded-full bg-rose-400/15 px-2 py-0.5 text-[10px] font-semibold text-rose-300">MVP</span>
									{/if}
									{#if task.isToday}
										<span class="rounded-full bg-amber-400/15 px-2 py-0.5 text-[10px] font-semibold text-amber-300">TODAY</span>
									{/if}
								</div>
								{#if task.description}
									<p class="mt-1 text-xs text-slate-500 line-clamp-1">{task.description}</p>
								{/if}
								<div class="mt-1.5 flex flex-wrap items-center gap-2 text-xs text-slate-500">
									<span style="color: {info.color}">{info.icon}</span>
									<span>{info.name}</span>
									<span>{PRIORITY_LABEL[task.priority] ?? '—'}</span>
									<span>{ENERGY_LABEL[task.energy] ?? ''}</span>
									{#if task.estimatedMinutes}<span>⏱ {task.estimatedMinutes}m</span>{/if}
								</div>
							</div>

							<div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition">
								<button
									onclick={() => toggleToday(task)}
									class="rounded-lg px-2 py-1 text-xs text-slate-500 hover:text-amber-400 transition"
									title="Toggle Today"
								>☀️</button>
								<button
									onclick={() => { decomposeTarget = task; showDecomposeModal = true; }}
									class="rounded-lg px-2 py-1 text-xs text-slate-500 hover:text-violet-400 transition"
									title="Decompose"
								>⚡</button>
								<button
									onclick={() => deleteTask(task.id)}
									class="rounded-lg px-2 py-1 text-xs text-slate-500 hover:text-rose-400 transition"
								>✕</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	{#if statusMsg}
		<div class="rounded-xl border border-rose-400/20 bg-rose-950/20 p-3 text-xs text-rose-300">{statusMsg}</div>
	{/if}
</section>

<!-- Add Task Modal -->
{#if showAddModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
		<div class="w-full max-w-lg rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl max-h-[90vh] overflow-y-auto">
			<div class="flex items-center justify-between mb-5">
				<h3 class="text-lg font-bold text-white">New Kaizen Task</h3>
				<button onclick={() => { showAddModal = false; resetForm(); }} class="text-slate-500 hover:text-white">✕</button>
			</div>
			<div class="space-y-4">
				<input bind:value={newTitle} placeholder="Task title*" class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-violet-400" autofocus />
				<textarea bind:value={newDesc} placeholder="Description (optional)" rows="2" class="w-full resize-none rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-violet-400"></textarea>
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
						<label class="mb-1.5 block text-xs text-slate-400">Energy level</label>
						<select bind:value={newEnergy} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
							<option value="low">🌱 Low energy</option>
							<option value="medium">⚡ Medium</option>
							<option value="high">🔥 High focus</option>
						</select>
					</div>
				</div>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="mb-1.5 block text-xs text-slate-400">Priority</label>
						<select bind:value={newPriority} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
							<option value={1}>🔴 Urgent</option>
							<option value={2}>🟠 High</option>
							<option value={3}>🟡 Normal</option>
							<option value={4}>🟢 Low</option>
						</select>
					</div>
					<div>
						<label class="mb-1.5 block text-xs text-slate-400">Est. minutes</label>
						<input bind:value={newEstimated} type="number" placeholder="e.g. 45" class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-2.5 text-sm text-white outline-none" />
					</div>
				</div>
				<input bind:value={newTags} placeholder="Tags (comma separated)" class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-2.5 text-sm text-white outline-none focus:border-violet-400" />
				<div class="grid grid-cols-2 gap-3">
					<label class="flex cursor-pointer items-center gap-3 rounded-xl border border-amber-400/20 bg-amber-950/20 px-3 py-2.5">
						<input type="checkbox" bind:checked={newToday} class="h-4 w-4 rounded" />
						<span class="text-sm text-amber-200">☀️ Add to Today</span>
					</label>
					<label class="flex cursor-pointer items-center gap-3 rounded-xl border border-rose-400/20 bg-rose-950/20 px-3 py-2.5">
						<input type="checkbox" bind:checked={newMinVersion} class="h-4 w-4 rounded" />
						<span class="text-sm text-rose-200">🎯 Minimum Version</span>
					</label>
				</div>
				<div class="flex gap-3 pt-2">
					<button onclick={addTask} disabled={savingTask || !newTitle.trim()} class="flex-1 rounded-xl bg-violet-500 py-3 text-sm font-bold text-white disabled:opacity-50">
						{savingTask ? 'Saving…' : 'Create Task'}
					</button>
					<button onclick={() => { showAddModal = false; resetForm(); }} class="rounded-xl border border-white/10 bg-white/5 px-5 py-3 text-sm text-white">Cancel</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Decompose Modal -->
{#if showDecomposeModal && decomposeTarget}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
		<div class="w-full max-w-md rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl">
			<h3 class="mb-2 text-lg font-bold text-white">⚡ Decompose Task</h3>
			<p class="mb-4 text-sm text-slate-400">"{decomposeTarget.title}"</p>
			<p class="mb-2 text-xs text-slate-500">Enter one subtask per line:</p>
			<textarea
				bind:value={decomposeInput}
				rows="6"
				placeholder="Research the topic&#10;Write first draft&#10;Review and edit"
				class="w-full resize-none rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-violet-400"
			></textarea>
			<div class="mt-4 flex gap-3">
				<button onclick={decompose} disabled={!decomposeInput.trim()} class="flex-1 rounded-xl bg-violet-500 py-2.5 text-sm font-bold text-white disabled:opacity-50">
					Create Subtasks
				</button>
				<button onclick={() => { showDecomposeModal = false; decomposeInput = ''; decomposeTarget = null; }} class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 text-sm text-white">
					Cancel
				</button>
			</div>
		</div>
	</div>
{/if}
