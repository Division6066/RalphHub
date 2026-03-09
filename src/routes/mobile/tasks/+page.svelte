<script lang="ts">
	import { mobileState, addTask } from '$lib/mobile/store.svelte.js';

	let newTaskTitle = $state('');
	let newTaskPriority = $state<'urgent' | 'high' | 'normal' | 'low'>('normal');
	let showAdd = $state(false);
	let filter = $state<'all' | 'todo' | 'in_progress' | 'done'>('all');

	let filteredTasks = $derived(
		filter === 'all' ? mobileState.tasks : mobileState.tasks.filter((t) => t.status === filter)
	);

	const priorityBadge: Record<string, string> = {
		urgent: 'bg-red-400/15 text-red-300 border-red-400/30',
		high: 'bg-orange-400/15 text-orange-300 border-orange-400/30',
		normal: 'bg-cyan-400/15 text-cyan-300 border-cyan-400/30',
		low: 'bg-slate-400/15 text-slate-400 border-slate-400/30',
	};

	async function handleAdd() {
		if (!newTaskTitle.trim()) return;
		await addTask(newTaskTitle.trim(), newTaskPriority);
		newTaskTitle = '';
		showAdd = false;
	}
</script>

<div class="space-y-5 py-2">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-xl font-bold text-white">Tasks</h1>
			<p class="text-xs text-slate-500 mt-0.5">{mobileState.tasks.length} total</p>
		</div>
		<button
			onclick={() => showAdd = !showAdd}
			class="w-9 h-9 rounded-2xl bg-cyan-400/15 border border-cyan-400/30 text-cyan-300 text-xl flex items-center justify-center"
		>+</button>
	</div>

	{#if showAdd}
		<div class="rounded-2xl border border-cyan-400/20 bg-slate-900/60 p-4 space-y-3">
			<input
				bind:value={newTaskTitle}
				placeholder="Task title..."
				class="w-full bg-transparent border-b border-white/10 pb-2 text-sm text-white placeholder:text-slate-600 focus:outline-none focus:border-cyan-400/50"
				onkeydown={(e) => e.key === 'Enter' && handleAdd()}
			/>
			<div class="flex gap-2 flex-wrap">
				{#each ['urgent', 'high', 'normal', 'low'] as p}
					<button
						onclick={() => newTaskPriority = p as typeof newTaskPriority}
						class={`px-3 py-1.5 rounded-xl border text-xs font-medium transition-all ${newTaskPriority === p ? priorityBadge[p] : 'border-white/10 text-slate-500'}`}
					>{p}</button>
				{/each}
			</div>
			<button
				onclick={handleAdd}
				class="w-full rounded-xl bg-cyan-500/20 border border-cyan-500/30 text-cyan-300 text-sm font-medium py-2.5"
			>Add Task →</button>
		</div>
	{/if}

	<!-- Filters -->
	<div class="flex gap-2 overflow-x-auto pb-1">
		{#each ['all', 'todo', 'in_progress', 'done'] as f}
			<button
				onclick={() => filter = f as typeof filter}
				class={`px-3 py-1.5 rounded-full text-xs font-medium whitespace-nowrap transition-all border ${
					filter === f ? 'bg-cyan-400/15 text-cyan-300 border-cyan-400/30' : 'border-white/8 text-slate-500'
				}`}
			>{f === 'in_progress' ? 'In Progress' : f.charAt(0).toUpperCase() + f.slice(1)}</button>
		{/each}
	</div>

	<!-- Task list -->
	<div class="space-y-2">
		{#each filteredTasks as task}
			<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-4">
				<div class="flex items-start gap-3">
					<div class="mt-1">
						<div class={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${task.status === 'done' ? 'bg-emerald-400/30 border-emerald-400' : 'border-slate-600'}`}>
							{#if task.status === 'done'}<span class="text-emerald-400 text-xs">✓</span>{/if}
						</div>
					</div>
					<div class="flex-1 min-w-0">
						<p class={`text-sm font-medium ${task.status === 'done' ? 'line-through text-slate-500' : 'text-white'}`}>
							{task.title}
						</p>
						<div class="flex items-center gap-2 mt-1.5 flex-wrap">
							<span class={`text-xs border rounded-full px-2 py-0.5 ${priorityBadge[task.priority]}`}>{task.priority}</span>
							{#if task.dueDate}<span class="text-xs text-slate-600">Due {task.dueDate}</span>{/if}
							<span class="text-xs text-slate-700">{task.source}</span>
						</div>
					</div>
				</div>
			</div>
		{:else}
			<div class="text-center py-12 text-slate-600">
				<p class="text-3xl mb-2">○</p>
				<p class="text-sm">No tasks here</p>
			</div>
		{/each}
	</div>
</div>
