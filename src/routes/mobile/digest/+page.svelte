<script lang="ts">
	import { mobileState } from '$lib/mobile/store.svelte.js';
	import { memorySpine } from '$lib/memory/memory-spine.js';

	let kaizenTasks = $derived(memorySpine.getKaizenTasks().slice(0, 5));
	let longTermMemory = $derived(memorySpine.getLongTermMemory().slice(0, 5));
	let rawCount = $derived(memorySpine.getRawEvents().length);

	const statCards = $derived([
		{ label: 'Tasks Done', value: mobileState.digest?.tasksCompleted ?? 0, color: 'text-cyan-400', icon: '✓' },
		{ label: 'Habits', value: mobileState.digest?.habitsCompleted ?? 0, color: 'text-emerald-400', icon: '○' },
		{ label: 'Agents Run', value: mobileState.digest?.agentsRun ?? 0, color: 'text-violet-400', icon: '◈' },
		{ label: 'Captures', value: mobileState.digest?.capturesProcessed ?? 0, color: 'text-amber-400', icon: '≡' },
	]);
</script>

<div class="space-y-5 py-2">
	<div>
		<h1 class="text-xl font-bold text-white">Daily Digest</h1>
		<p class="text-xs text-slate-500 mt-0.5">
			{mobileState.digest
				? new Date(mobileState.digest.date).toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })
				: 'Loading...'}
		</p>
	</div>

	{#if mobileState.digest}
		<!-- Stat grid -->
		<div class="grid grid-cols-2 gap-3">
			{#each statCards as card}
				<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-4">
					<div class="flex items-center gap-2 mb-2">
						<span class={`text-lg ${card.color}`}>{card.icon}</span>
						<span class="text-xs text-slate-500">{card.label}</span>
					</div>
					<p class={`text-3xl font-bold ${card.color}`}>{card.value}</p>
				</div>
			{/each}
		</div>

		<!-- Highlights -->
		<section>
			<h2 class="text-xs uppercase tracking-widest text-slate-500 mb-3">Highlights</h2>
			<div class="space-y-2">
				{#each mobileState.digest.highlights as highlight, i}
					<div class="flex items-start gap-3 rounded-xl border border-white/5 bg-slate-900/30 px-4 py-3">
						<span class="text-cyan-400 text-xs mt-0.5 shrink-0">{i + 1}.</span>
						<p class="text-sm text-slate-300 leading-relaxed">{highlight}</p>
					</div>
				{/each}
			</div>
		</section>
	{:else}
		<div class="text-center py-8 text-slate-600">
			<p class="text-3xl mb-2">◉</p>
			<p class="text-sm">No digest for today yet</p>
		</div>
	{/if}

	<!-- Memory Spine status -->
	<section>
		<h2 class="text-xs uppercase tracking-widest text-slate-500 mb-3">Memory Spine</h2>
		<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-4 space-y-3">
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Raw Events</span>
				<span class="font-mono text-cyan-400">{rawCount}</span>
			</div>
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Working Memory</span>
				<span class="font-mono text-violet-400">{memorySpine.getWorkingMemory().length}</span>
			</div>
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Long-term</span>
				<span class="font-mono text-emerald-400">{memorySpine.getLongTermMemory().length}</span>
			</div>
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Kaizen Tasks</span>
				<span class="font-mono text-amber-400">{memorySpine.getKaizenTasks().length}</span>
			</div>
		</div>
	</section>

	<!-- Kaizen tasks -->
	{#if kaizenTasks.length > 0}
		<section>
			<h2 class="text-xs uppercase tracking-widest text-slate-500 mb-3">Kaizen Backlog</h2>
			<div class="space-y-2">
				{#each kaizenTasks as task}
					<div class="rounded-xl border border-amber-400/10 bg-amber-400/5 px-4 py-3">
						<p class="text-sm text-white">{task.title}</p>
						<p class="text-xs text-slate-500 mt-0.5">{task.sourceType} · {task.priority}</p>
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<!-- Long-term memory -->
	{#if longTermMemory.length > 0}
		<section>
			<h2 class="text-xs uppercase tracking-widest text-slate-500 mb-3">Long-Term Memory</h2>
			<div class="space-y-2">
				{#each longTermMemory as mem}
					<div class="rounded-xl border border-violet-400/10 bg-violet-400/5 px-4 py-3">
						<div class="flex items-center gap-2 mb-1">
							<span class="text-xs text-violet-400 bg-violet-400/15 rounded-full px-2 py-0.5">{mem.category}</span>
						</div>
						<p class="text-sm text-white">{mem.summary}</p>
						<p class="text-xs text-slate-500 mt-0.5 line-clamp-1">{mem.detail}</p>
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<p class="text-center text-xs text-slate-700 pb-4">
		Generated {mobileState.digest ? new Date(mobileState.digest.generatedAt).toLocaleTimeString() : '—'}
	</p>
</div>
