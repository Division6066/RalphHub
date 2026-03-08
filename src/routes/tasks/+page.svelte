<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface KaizenTask {
		id: string;
		projectId: string | null;
		parentTaskId: string | null;
		title: string;
		domain: string;
		energy: string;
		estimateMinutes: number;
		status: string;
		doDate: string | null;
		deadline: string | null;
		agentMode: string;
		approvalRequired: boolean;
		evidence: Record<string, unknown>;
		notes: string;
		subtaskCount: number;
		createdAt: string;
		updatedAt: string;
	}

	interface KaizenProject {
		id: string;
		title: string;
		domain: string;
		description: string;
		status: string;
		taskCount: number;
		createdAt: string;
		updatedAt: string;
	}

	interface TodayBoardGroup {
		domain: string;
		tasks: KaizenTask[];
	}

	let projects = $state<KaizenProject[]>([]);
	let tasks = $state<KaizenTask[]>([]);
	let todayBoard = $state<TodayBoardGroup[]>([]);
	let loading = $state(true);
	let error = $state('');
	let activeTab = $state<'today' | 'all' | 'new'>('today');
	let creating = $state(false);
	let msg = $state('');

	// New task form
	let newTitle = $state('');
	let newDomain = $state('work');
	let newEnergy = $state('medium');
	let newEstimate = $state(30);
	let newDoDate = $state('');
	let newDeadline = $state('');
	let newNotes = $state('');
	let newAgentMode = $state('manual');
	let newApproval = $state(false);
	let selectedProject = $state('');

	const domainEmoji: Record<string, string> = {
		work: '💼',
		health: '🏃',
		learning: '📚',
		personal: '🏠',
		system: '⚙️',
	};

	const energyColor: Record<string, string> = {
		low: 'text-emerald-400 border-emerald-400/30 bg-emerald-400/8',
		medium: 'text-amber-400 border-amber-400/30 bg-amber-400/8',
		high: 'text-red-400 border-red-400/30 bg-red-400/8',
	};

	const statusColor: Record<string, string> = {
		inbox: 'bg-slate-700 text-slate-300',
		todo: 'bg-blue-500/20 text-blue-300',
		doing: 'bg-cyan-500/20 text-cyan-300',
		blocked: 'bg-red-500/20 text-red-300',
		done: 'bg-emerald-500/20 text-emerald-300',
		cancelled: 'bg-slate-600/20 text-slate-500',
	};

	function fmtMinutes(m: number): string {
		if (m < 60) return `${m}m`;
		const h = Math.floor(m / 60);
		const rem = m % 60;
		return rem > 0 ? `${h}h ${rem}m` : `${h}h`;
	}

	async function load() {
		loading = true;
		error = '';
		try {
			if (!isDesktopRuntime()) {
				const demoTask: KaizenTask = {
					id: '1', projectId: null, parentTaskId: null,
					title: 'Build AmitOS Memory Spine', domain: 'work', energy: 'high',
					estimateMinutes: 90, status: 'doing', doDate: new Date().toISOString().slice(0, 10),
					deadline: null, agentMode: 'manual', approvalRequired: false, evidence: {}, notes: '',
					subtaskCount: 0, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(),
				};
				todayBoard = [{ domain: 'work', tasks: [demoTask] }];
				tasks = [demoTask];
				projects = [{ id: '1', title: 'AmitOS Build', domain: 'work', description: '', status: 'active', taskCount: 1, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() }];
				loading = false;
				return;
			}
			const [board, allTasks, projs] = await Promise.all([
				invokeTauri<TodayBoardGroup[]>('get_today_board'),
				invokeTauri<KaizenTask[]>('list_kaizen_tasks', {}),
				invokeTauri<KaizenProject[]>('list_kaizen_projects'),
			]);
			todayBoard = board;
			tasks = allTasks;
			projects = projs;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function createTask() {
		if (!newTitle.trim()) { msg = 'Title is required.'; return; }
		creating = true;
		msg = '';
		try {
			if (isDesktopRuntime()) {
				await invokeTauri('create_kaizen_task', {
					request: {
						projectId: selectedProject || null,
						parentTaskId: null,
						title: newTitle,
						domain: newDomain,
						energy: newEnergy,
						estimateMinutes: newEstimate,
						doDate: newDoDate || null,
						deadline: newDeadline || null,
						agentMode: newAgentMode,
						approvalRequired: newApproval,
						notes: newNotes,
					},
				});
			}
			msg = '✓ Task created';
			newTitle = '';
			newNotes = '';
			newDoDate = '';
			setTimeout(() => { msg = ''; }, 3000);
			await load();
			activeTab = 'today';
		} catch (e) {
			msg = '✗ ' + String(e);
		} finally {
			creating = false;
		}
	}

	async function updateStatus(id: string, status: string) {
		if (!isDesktopRuntime()) return;
		try {
			await invokeTauri('update_kaizen_task_status', { id, status });
			await load();
		} catch (e) {
			error = String(e);
		}
	}

	async function decompose(id: string) {
		if (!isDesktopRuntime()) return;
		try {
			await invokeTauri('decompose_task', { taskId: id });
			await load();
		} catch (e) {
			error = String(e);
		}
	}

	onMount(load);
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-start justify-between">
		<div>
			<h1 class="text-2xl font-bold text-white">✅ Kaizen Tasks</h1>
			<p class="mt-1 text-sm text-slate-400">Verb-first • Minimum Version • 90-min decomposition • Grouped by Domain</p>
		</div>
		<div class="flex gap-2">
			<button onclick={() => { activeTab = 'new'; }} class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-4 py-2 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400">+ New Task</button>
			<button onclick={load} class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300 hover:bg-white/10">↻</button>
		</div>
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	<!-- Tabs -->
	<div class="flex gap-1 rounded-2xl border border-white/8 bg-slate-900/40 p-1">
		{#each [
			{ key: 'today', label: '📅 Today Board' },
			{ key: 'all', label: '📋 All Tasks' },
			{ key: 'new', label: '➕ New Task' },
		] as tab}
			<button
				onclick={() => (activeTab = tab.key as typeof activeTab)}
				class={`flex-1 rounded-xl px-4 py-2.5 text-sm font-medium transition-all ${
					activeTab === tab.key
						? 'bg-cyan-400/15 text-cyan-100 shadow-md ring-1 ring-cyan-400/20'
						: 'text-slate-400 hover:text-white'
				}`}
			>
				{tab.label}
			</button>
		{/each}
	</div>

	<!-- Today Board -->
	{#if activeTab === 'today'}
		{#if loading}
			<div class="py-12 text-center text-slate-500">Loading today's tasks…</div>
		{:else if todayBoard.length === 0}
			<div class="rounded-2xl border border-dashed border-white/10 py-16 text-center">
				<p class="text-4xl">📅</p>
				<p class="mt-3 text-slate-400">No tasks scheduled for today.</p>
				<button onclick={() => (activeTab = 'new')} class="mt-4 rounded-xl bg-cyan-500/20 px-5 py-2.5 text-sm font-medium text-cyan-300 hover:bg-cyan-500/30">+ Add a task for today</button>
			</div>
		{:else}
			{#each todayBoard as group}
				<div class="mb-6">
					<div class="mb-3 flex items-center gap-2">
						<span class="text-xl">{domainEmoji[group.domain] ?? '📌'}</span>
						<h2 class="text-base font-bold capitalize text-white">{group.domain}</h2>
						<span class="rounded-full bg-slate-800 px-2 py-0.5 text-xs text-slate-400">{group.tasks.length}</span>
					</div>
					<div class="space-y-2">
						{#each group.tasks as task}
							<div class="group rounded-2xl border border-white/8 bg-slate-900/50 p-4 transition hover:border-cyan-400/20">
								<div class="flex items-start gap-3">
									<!-- Status toggle -->
									<button
										onclick={() => updateStatus(task.id, task.status === 'doing' ? 'done' : 'doing')}
										class="mt-0.5 h-5 w-5 shrink-0 rounded-full border-2 {task.status === 'done' ? 'border-emerald-400 bg-emerald-400' : task.status === 'doing' ? 'border-cyan-400' : 'border-slate-600'} transition hover:border-cyan-400"
									></button>

									<div class="flex-1 min-w-0">
										<p class="font-medium {task.status === 'done' ? 'text-slate-500 line-through' : 'text-white'}">{task.title}</p>
										<div class="mt-1.5 flex flex-wrap items-center gap-1.5">
											<span class="rounded-full border px-2 py-0.5 text-[10px] font-medium {energyColor[task.energy] ?? 'text-slate-400'}">{task.energy}</span>
											<span class="text-[10px] text-slate-500">⏱ {fmtMinutes(task.estimateMinutes)}</span>
											{#if task.agentMode !== 'manual'}
												<span class="rounded-full bg-purple-500/20 px-2 py-0.5 text-[10px] text-purple-300">{task.agentMode}</span>
											{/if}
											{#if task.approvalRequired}
												<span class="rounded-full bg-amber-500/20 px-2 py-0.5 text-[10px] text-amber-300">⚠ Approval</span>
											{/if}
											{#if task.estimateMinutes > 90}
												<button onclick={() => decompose(task.id)} class="rounded-full bg-orange-500/20 px-2 py-0.5 text-[10px] font-medium text-orange-300 hover:bg-orange-500/30">✂ Decompose >{90}m</button>
											{/if}
										</div>
									</div>

									<div class="flex shrink-0 gap-1 opacity-0 transition group-hover:opacity-100">
										{#each ['todo', 'doing', 'blocked', 'done'] as s}
											<button
												onclick={() => updateStatus(task.id, s)}
												class="rounded-lg px-2 py-1 text-[10px] font-medium transition {statusColor[s] ?? ''} hover:opacity-80"
											>{s}</button>
										{/each}
									</div>
								</div>
								{#if task.notes}
									<p class="mt-2 text-xs text-slate-500 pl-8">{task.notes}</p>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{/each}
		{/if}
	{/if}

	<!-- All Tasks -->
	{#if activeTab === 'all'}
		<div class="space-y-2">
			{#if loading}
				<div class="py-12 text-center text-slate-500">Loading tasks…</div>
			{:else if tasks.length === 0}
				<div class="py-12 text-center text-slate-500">No tasks yet.</div>
			{:else}
				{#each tasks as task}
					<div class="group flex items-center gap-3 rounded-xl border border-white/6 bg-slate-900/40 px-4 py-3">
						<span class="text-lg shrink-0">{domainEmoji[task.domain] ?? '📌'}</span>
						<div class="flex-1 min-w-0">
							<p class="text-sm font-medium text-white truncate">{task.title}</p>
							<div class="flex items-center gap-2 mt-0.5">
								<span class="text-[10px] text-slate-500">{task.domain}</span>
								<span class="text-[10px] text-slate-600">•</span>
								<span class="text-[10px] text-slate-500">{fmtMinutes(task.estimateMinutes)}</span>
								{#if task.doDate}
									<span class="text-[10px] text-slate-500">• {task.doDate}</span>
								{/if}
							</div>
						</div>
						<span class="shrink-0 rounded-full px-2.5 py-1 text-[10px] font-medium {statusColor[task.status] ?? ''}">{task.status}</span>
					</div>
				{/each}
			{/if}
		</div>
	{/if}

	<!-- New Task Form -->
	{#if activeTab === 'new'}
		<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-6">
			<h2 class="mb-5 text-lg font-bold text-white">➕ New Kaizen Task</h2>
			<p class="mb-5 -mt-3 text-xs text-slate-500">Use verb-first titles: "Write X", "Build Y", "Research Z"</p>

			<div class="grid gap-4 sm:grid-cols-2">
				<!-- Title -->
				<div class="sm:col-span-2">
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Task Title *</label>
					<input
						bind:value={newTitle}
						type="text"
						placeholder="Write the landing page copy…"
						class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-400/10"
					/>
				</div>

				<!-- Domain -->
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Domain</label>
					<select bind:value={newDomain} class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40">
						{#each ['work', 'health', 'learning', 'personal', 'system'] as d}
							<option value={d}>{domainEmoji[d]} {d}</option>
						{/each}
					</select>
				</div>

				<!-- Energy -->
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Energy Required</label>
					<select bind:value={newEnergy} class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40">
						<option value="low">🟢 Low</option>
						<option value="medium">🟡 Medium</option>
						<option value="high">🔴 High</option>
					</select>
				</div>

				<!-- Estimate -->
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">
						Estimate: {fmtMinutes(newEstimate)}
						{#if newEstimate > 90}<span class="text-orange-400">— will auto-decompose</span>{/if}
					</label>
					<input type="range" bind:value={newEstimate} min={5} max={480} step={5} class="w-full accent-cyan-400" />
					<div class="mt-1 flex justify-between text-[10px] text-slate-600">
						<span>5m</span><span>1h30m</span><span>8h</span>
					</div>
				</div>

				<!-- Do Date -->
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Do Date</label>
					<input type="date" bind:value={newDoDate} class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40" />
				</div>

				<!-- Agent Mode -->
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Agent Mode</label>
					<select bind:value={newAgentMode} class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40">
						<option value="manual">🤚 Manual</option>
						<option value="auto">🤖 Auto</option>
						<option value="approval_required">⚠ Approval Required</option>
					</select>
				</div>

				<!-- Project -->
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Project (optional)</label>
					<select bind:value={selectedProject} class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40">
						<option value="">— No project —</option>
						{#each projects as p}
							<option value={p.id}>{p.title}</option>
						{/each}
					</select>
				</div>

				<!-- Approval Required -->
				<div class="flex items-center gap-3">
					<input type="checkbox" bind:checked={newApproval} class="h-4 w-4 rounded" />
					<label class="text-sm text-slate-300">Requires approval before agent runs</label>
				</div>

				<!-- Notes -->
				<div class="sm:col-span-2">
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Notes</label>
					<textarea
						bind:value={newNotes}
						rows={3}
						placeholder="Context, links, minimum version details…"
						class="w-full resize-none rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-400/10"
					></textarea>
				</div>
			</div>

			<div class="mt-5 flex items-center gap-3">
				<button
					onclick={createTask}
					disabled={creating || !newTitle.trim()}
					class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-6 py-3 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 transition hover:from-cyan-400 disabled:opacity-40"
				>
					{creating ? 'Creating…' : '✅ Create Task'}
				</button>
				{#if msg}
					<p class="text-sm {msg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{msg}</p>
				{/if}
			</div>
		</div>
	{/if}
</div>
