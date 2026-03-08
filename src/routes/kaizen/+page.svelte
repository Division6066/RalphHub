<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type KaizenTask = {
		id: string;
		title: string;
		description: string;
		status: string;
		priority: string;
		toolId: string;
		createdAt: string;
		updatedAt: string;
	};

	let tasks: KaizenTask[] = [];
	let loading = true;
	let busy = false;
	let message = '';
	let statusFilter = '';

	let newTitle = '';
	let newDescription = '';
	let newPriority = 'medium';
	let newToolId = '';

	const statuses = ['pending', 'in_progress', 'done', 'cancelled'];
	const priorities = ['high', 'medium', 'low'];

	onMount(async () => {
		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}
		await refresh();
	});

	async function refresh() {
		loading = true;
		try {
			tasks = await invokeTauri<KaizenTask[]>('list_kaizen_tasks', {
				statusFilter: statusFilter || null
			});
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to load tasks.';
		} finally {
			loading = false;
		}
	}

	async function createTask() {
		if (!newTitle.trim()) return;
		busy = true;
		try {
			await invokeTauri('create_kaizen_task', {
				title: newTitle,
				description: newDescription,
				priority: newPriority,
				toolId: newToolId || ''
			});
			newTitle = '';
			newDescription = '';
			message = 'Task created.';
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to create task.';
		} finally {
			busy = false;
		}
	}

	async function updateStatus(id: string, status: string) {
		try {
			await invokeTauri('update_kaizen_task_status', { id, status });
			tasks = tasks.map((t) => (t.id === id ? { ...t, status } : t));
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to update task.';
		}
	}

	async function deleteTask(id: string) {
		try {
			await invokeTauri('delete_kaizen_task', { id });
			tasks = tasks.filter((t) => t.id !== id);
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to delete task.';
		}
	}

	function priorityColor(p: string) {
		switch (p) {
			case 'high': return 'bg-red-400/15 text-red-300';
			case 'medium': return 'bg-amber-400/15 text-amber-300';
			default: return 'bg-slate-600/30 text-slate-400';
		}
	}

	function statusColor(s: string) {
		switch (s) {
			case 'done': return 'bg-green-400/15 text-green-300';
			case 'in_progress': return 'bg-cyan-400/15 text-cyan-300';
			case 'cancelled': return 'bg-slate-600/30 text-slate-500';
			default: return 'bg-violet-400/15 text-violet-300';
		}
	}

	$: grouped = statuses.reduce<Record<string, KaizenTask[]>>((acc, s) => {
		acc[s] = tasks.filter((t) => t.status === s);
		return acc;
	}, {} as Record<string, KaizenTask[]>);
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-amber-300/80">Kaizen Tasks</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Continuous improvement task board.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Every tool run, workflow, and agent loop can create and update tasks here automatically.
			Kaizen — small continuous improvements every session.
		</p>
	</div>

	{#if message}
		<div class="rounded-3xl border border-amber-400/20 bg-amber-500/10 p-4 text-sm text-amber-100">
			{message}
		</div>
	{/if}

	<!-- Create task -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white">New task</h2>
		<div class="mt-4 grid gap-3 sm:grid-cols-2">
			<input
				bind:value={newTitle}
				placeholder="Task title"
				class="rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600"
			/>
			<input
				bind:value={newToolId}
				placeholder="Tool ID (optional)"
				class="rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600"
			/>
		</div>
		<textarea
			bind:value={newDescription}
			rows="2"
			placeholder="Description..."
			class="mt-3 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600 resize-none"
		></textarea>
		<div class="mt-3 flex items-center gap-3">
			<select
				bind:value={newPriority}
				class="rounded-full border border-white/10 bg-slate-900/60 px-4 py-2 text-sm text-white outline-none"
			>
				{#each priorities as p}
					<option value={p}>{p}</option>
				{/each}
			</select>
			<button
				on:click={createTask}
				disabled={busy || !newTitle.trim()}
				class="rounded-full bg-amber-400/15 px-5 py-2 text-sm font-medium text-amber-100 hover:bg-amber-400/25 disabled:opacity-60"
			>
				{busy ? 'Creating...' : 'Create task'}
			</button>
		</div>
	</div>

	<!-- Filter + board -->
	<div class="flex items-center gap-3">
		<h2 class="flex-1 text-lg font-semibold text-white">Task board</h2>
		<select
			bind:value={statusFilter}
			on:change={refresh}
			class="rounded-full border border-white/10 bg-slate-900/60 px-4 py-2 text-sm text-white outline-none"
		>
			<option value="">All statuses</option>
			{#each statuses as s}
				<option value={s}>{s}</option>
			{/each}
		</select>
		<button on:click={refresh} class="rounded-full border border-white/10 px-3 py-2 text-xs text-slate-400 hover:text-white">
			Refresh
		</button>
	</div>

	{#if loading}
		<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-6 text-sm text-slate-400 backdrop-blur">Loading...</div>
	{:else if !tasks.length}
		<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-6 text-sm text-slate-500 backdrop-blur">
			No tasks yet. Tools write tasks here automatically or create them manually above.
		</div>
	{:else}
		<div class="grid gap-4 xl:grid-cols-4">
			{#each statuses as s}
				{#if grouped[s]?.length}
					<div class="space-y-3">
						<div class="flex items-center gap-2">
							<span class="rounded-full px-2 py-0.5 text-xs {statusColor(s)}">{s}</span>
							<span class="text-xs text-slate-500">{grouped[s].length}</span>
						</div>
						{#each grouped[s] as task}
							<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
								<div class="flex items-start justify-between gap-2">
									<p class="text-sm font-medium text-white leading-5">{task.title}</p>
									<span class="shrink-0 rounded-full px-2 py-0.5 text-xs {priorityColor(task.priority)}">{task.priority}</span>
								</div>
								{#if task.description}
									<p class="mt-2 text-xs text-slate-400 leading-5">{task.description}</p>
								{/if}
								{#if task.toolId}
									<p class="mt-1 text-xs text-slate-600 font-mono">{task.toolId}</p>
								{/if}
								<div class="mt-3 flex flex-wrap gap-1.5">
									{#each statuses.filter((ss) => ss !== task.status) as nextStatus}
										<button
											on:click={() => updateStatus(task.id, nextStatus)}
											class="rounded-full border border-white/10 px-2 py-0.5 text-xs text-slate-400 hover:text-white hover:border-white/30"
										>
											→ {nextStatus}
										</button>
									{/each}
									<button
										on:click={() => deleteTask(task.id)}
										class="rounded-full border border-red-400/20 px-2 py-0.5 text-xs text-red-400/60 hover:text-red-400"
									>
										Delete
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			{/each}
		</div>
	{/if}
</section>
