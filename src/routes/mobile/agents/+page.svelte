<script lang="ts">
	import { mobileState } from '$lib/mobile/store.svelte.js';

	const statusConfig: Record<string, { label: string; color: string; bg: string; icon: string }> = {
		running: { label: 'Running', color: 'text-cyan-400', bg: 'bg-cyan-400/10 border-cyan-400/20', icon: '⟳' },
		waiting_approval: { label: 'Waiting', color: 'text-amber-400', bg: 'bg-amber-400/10 border-amber-400/20', icon: '⏸' },
		queued: { label: 'Queued', color: 'text-slate-400', bg: 'bg-slate-400/10 border-slate-400/20', icon: '○' },
		success: { label: 'Success', color: 'text-emerald-400', bg: 'bg-emerald-400/10 border-emerald-400/20', icon: '✓' },
		failed: { label: 'Failed', color: 'text-red-400', bg: 'bg-red-400/10 border-red-400/20', icon: '✗' },
	};

	let filter = $state<string>('all');

	let filtered = $derived(
		filter === 'all'
			? mobileState.agents
			: mobileState.agents.filter((a) => a.status === filter)
	);

	function elapsed(start: string | null): string {
		if (!start) return '—';
		const ms = Date.now() - new Date(start).getTime();
		if (ms < 60000) return `${Math.floor(ms / 1000)}s`;
		if (ms < 3600000) return `${Math.floor(ms / 60000)}m`;
		return `${Math.floor(ms / 3600000)}h ${Math.floor((ms % 3600000) / 60000)}m`;
	}
</script>

<div class="space-y-5 py-2">
	<div>
		<h1 class="text-xl font-bold text-white">Running Agents</h1>
		<p class="text-xs text-slate-500 mt-0.5">{mobileState.agents.filter(a => a.status === 'running').length} active</p>
	</div>

	<!-- Status summary pills -->
	<div class="flex gap-2 overflow-x-auto pb-1">
		{#each ['all', 'running', 'waiting_approval', 'queued', 'success', 'failed'] as s}
			{@const count = s === 'all' ? mobileState.agents.length : mobileState.agents.filter(a => a.status === s).length}
			<button
				onclick={() => filter = s}
				class={`px-3 py-1.5 rounded-full text-xs font-medium border whitespace-nowrap transition-all flex items-center gap-1.5 ${
					filter === s ? 'bg-cyan-400/15 text-cyan-300 border-cyan-400/30' : 'border-white/8 text-slate-500'
				}`}
			>
				{#if s !== 'all' && statusConfig[s]}
					<span class={statusConfig[s].color}>{statusConfig[s].icon}</span>
				{/if}
				{s === 'all' ? 'All' : statusConfig[s]?.label ?? s}
				<span class="opacity-60">{count}</span>
			</button>
		{/each}
	</div>

	<!-- Agent cards -->
	<div class="space-y-3">
		{#each filtered as agent}
			{@const cfg = statusConfig[agent.status]}
			<div class={`rounded-2xl border p-4 ${cfg.bg}`}>
				<div class="flex items-start justify-between mb-3">
					<div class="flex items-center gap-3">
						<div class={`w-10 h-10 rounded-xl border flex items-center justify-center text-lg ${cfg.bg}`}>
							<span class={agent.status === 'running' ? `${cfg.color} animate-spin` : cfg.color}>
								{cfg.icon}
							</span>
						</div>
						<div>
							<p class="text-sm font-medium text-white">{agent.name}</p>
							<span class={`text-xs font-medium ${cfg.color}`}>{cfg.label}</span>
						</div>
					</div>
					{#if agent.status === 'running'}
						<div class="text-right">
							<p class="text-xs text-slate-500">Elapsed</p>
							<p class="text-sm font-mono text-cyan-300">{elapsed(agent.startedAt)}</p>
						</div>
					{/if}
				</div>

				<div class="grid grid-cols-2 gap-2 text-xs text-slate-500">
					{#if agent.startedAt}
						<div>
							<p class="text-slate-600">Started</p>
							<p class="text-slate-400">{new Date(agent.startedAt).toLocaleTimeString()}</p>
						</div>
					{/if}
					{#if agent.completedAt}
						<div>
							<p class="text-slate-600">Completed</p>
							<p class="text-slate-400">{new Date(agent.completedAt).toLocaleTimeString()}</p>
						</div>
					{/if}
					{#if agent.memoryRef}
						<div>
							<p class="text-slate-600">Memory</p>
							<p class="text-cyan-400/70 font-mono">{agent.memoryRef}</p>
						</div>
					{/if}
					{#if agent.notionTaskId}
						<div>
							<p class="text-slate-600">Notion</p>
							<p class="text-violet-400/70 font-mono">{agent.notionTaskId}</p>
						</div>
					{/if}
				</div>

				{#if agent.status === 'waiting_approval'}
					<a
						href="/mobile/approvals"
						class="mt-3 block text-center py-2 rounded-xl bg-amber-400/15 border border-amber-400/25 text-amber-300 text-xs font-medium"
					>Review Pending Approval →</a>
				{/if}
			</div>
		{:else}
			<div class="text-center py-16 text-slate-600">
				<p class="text-4xl mb-3">◈</p>
				<p class="text-sm">No agents in this state</p>
			</div>
		{/each}
	</div>
</div>
