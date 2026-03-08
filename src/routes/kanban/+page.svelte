<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface KaizenTask {
		id: string;
		projectId: string | null;
		title: string;
		domain: string;
		energy: string;
		estimateMinutes: number;
		status: string;
		doDate: string | null;
		agentMode: string;
		approvalRequired: boolean;
		notes: string;
		subtaskCount: number;
		createdAt: string;
		updatedAt: string;
	}

	interface KanbanColumn {
		status: string;
		label: string;
		tasks: KaizenTask[];
	}

	let columns = $state<KanbanColumn[]>([]);
	let loading = $state(true);
	let error = $state('');
	let draggedTask = $state<KaizenTask | null>(null);

	const colStyle: Record<string, string> = {
		inbox: 'border-slate-600/40 bg-slate-900/30',
		todo: 'border-blue-500/25 bg-blue-950/20',
		doing: 'border-cyan-500/25 bg-cyan-950/20',
		blocked: 'border-red-500/25 bg-red-950/20',
		done: 'border-emerald-500/25 bg-emerald-950/20',
		cancelled: 'border-slate-700/30 bg-slate-900/20',
	};

	const colHeaderStyle: Record<string, string> = {
		inbox: 'text-slate-400',
		todo: 'text-blue-300',
		doing: 'text-cyan-300',
		blocked: 'text-red-300',
		done: 'text-emerald-300',
		cancelled: 'text-slate-500',
	};

	const energyDot: Record<string, string> = {
		low: 'bg-emerald-400',
		medium: 'bg-amber-400',
		high: 'bg-red-400',
	};

	const domainEmoji: Record<string, string> = {
		work: '💼', health: '🏃', learning: '📚', personal: '🏠', system: '⚙️',
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
				const demoTask = (title: string, status: string, energy: string): KaizenTask => ({
					id: Math.random().toString(), projectId: null, title, domain: 'work', energy,
					estimateMinutes: 45, status, doDate: null, agentMode: 'manual',
					approvalRequired: false, notes: '', subtaskCount: 0,
					createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(),
				});
				columns = [
					{ status: 'inbox', label: 'Inbox', tasks: [demoTask('Review AmitOS spec', 'inbox', 'low')] },
					{ status: 'todo', label: 'To Do', tasks: [demoTask('Write Memory Spine tests', 'todo', 'high')] },
					{ status: 'doing', label: 'Doing', tasks: [demoTask('Build Kanban UI', 'doing', 'high')] },
					{ status: 'blocked', label: 'Blocked', tasks: [] },
					{ status: 'done', label: 'Done', tasks: [demoTask('Set up Tauri project', 'done', 'medium')] },
					{ status: 'cancelled', label: 'Cancelled', tasks: [] },
				];
				loading = false;
				return;
			}
			columns = await invokeTauri<KanbanColumn[]>('get_kanban_board', {});
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function moveTask(task: KaizenTask, newStatus: string) {
		if (task.status === newStatus) return;
		// Optimistic UI update
		columns = columns.map((col) => ({
			...col,
			tasks: col.status === task.status
				? col.tasks.filter((t) => t.id !== task.id)
				: col.status === newStatus
				? [...col.tasks, { ...task, status: newStatus }]
				: col.tasks,
		}));

		if (isDesktopRuntime()) {
			try {
				await invokeTauri('update_kaizen_task_status', { id: task.id, status: newStatus });
			} catch (e) {
				error = String(e);
				await load();
			}
		}
	}

	function onDragStart(task: KaizenTask) {
		draggedTask = task;
	}

	function onDragOver(e: DragEvent) {
		e.preventDefault();
	}

	function onDrop(e: DragEvent, status: string) {
		e.preventDefault();
		if (draggedTask) {
			moveTask(draggedTask, status);
			draggedTask = null;
		}
	}

	onMount(load);
</script>

<div class="space-y-5">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-white">📋 Kanban Board</h1>
			<p class="mt-1 text-sm text-slate-400">Drag tasks between columns — changes save automatically.</p>
		</div>
		<div class="flex gap-2">
			<a href="/tasks" class="rounded-xl border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm font-medium text-cyan-300 hover:bg-cyan-400/15">+ New Task</a>
			<button onclick={load} class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300 hover:bg-white/10">↻</button>
		</div>
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	{#if loading}
		<div class="py-16 text-center text-slate-500">Loading board…</div>
	{:else}
		<!-- Kanban columns -->
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-3 xl:grid-cols-6">
			{#each columns as col}
				<div
					class="min-h-[300px] rounded-2xl border p-3 transition {colStyle[col.status] ?? 'border-white/8 bg-slate-900/30'}"
					ondragover={onDragOver}
					ondrop={(e) => onDrop(e, col.status)}
					role="region"
					aria-label="Column {col.label}"
				>
					<!-- Column header -->
					<div class="mb-3 flex items-center justify-between">
						<span class="text-xs font-bold uppercase tracking-wider {colHeaderStyle[col.status] ?? 'text-slate-400'}">{col.label}</span>
						<span class="rounded-full bg-slate-800 px-2 py-0.5 text-[10px] text-slate-400">{col.tasks.length}</span>
					</div>

					<!-- Task cards -->
					<div class="space-y-2">
						{#each col.tasks as task}
							<div
								draggable="true"
								role="listitem"
								ondragstart={() => onDragStart(task)}
								class="group cursor-grab rounded-xl border border-white/8 bg-slate-900/70 p-3 shadow-sm transition hover:border-white/15 hover:bg-slate-800/70 active:cursor-grabbing"
							>
								<div class="flex items-start gap-2">
									<span class={`mt-1 h-2 w-2 shrink-0 rounded-full ${energyDot[task.energy] ?? 'bg-slate-500'}`}></span>
									<p class="text-xs font-medium leading-snug text-white">{task.title}</p>
								</div>
								<div class="mt-2 flex flex-wrap items-center gap-1">
									<span class="text-[9px] text-slate-600">{domainEmoji[task.domain]}</span>
									<span class="text-[9px] text-slate-600">⏱{fmtMinutes(task.estimateMinutes)}</span>
									{#if task.approvalRequired}
										<span class="text-[9px] text-amber-500">⚠</span>
									{/if}
									{#if task.doDate}
										<span class="text-[9px] text-slate-600">{task.doDate}</span>
									{/if}
								</div>
								<!-- Quick move buttons (show on hover) -->
								<div class="mt-2 hidden flex-wrap gap-1 group-hover:flex">
									{#each columns.filter((c) => c.status !== col.status && c.status !== 'cancelled') as targetCol}
										<button
											onclick={() => moveTask(task, targetCol.status)}
											class="rounded-lg bg-slate-700/60 px-1.5 py-0.5 text-[9px] text-slate-300 hover:bg-slate-600/80"
										>→ {targetCol.label}</button>
									{/each}
								</div>
							</div>
						{/each}

						{#if col.tasks.length === 0}
							<div class="rounded-xl border border-dashed border-white/8 py-6 text-center text-[10px] text-slate-700">
								Drop tasks here
							</div>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
