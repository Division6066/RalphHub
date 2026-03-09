<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import { getMemoryStats, memoryStatsStore, kaizenTasksStore } from '$lib/utils/provider-registry';

	type MemoryEntry = {
		id: string;
		entryType: string;
		content: string;
		tags: string[];
		providerId: string;
		model: string;
		createdAt: string;
	};

	// ─── State ───────────────────────────────────────────────────────────────────
	let entries: MemoryEntry[] = $state([]);
	let loading = $state(true);
	let filterType = $state('all');
	let searchQ = $state('');

	// ─── Derived ─────────────────────────────────────────────────────────────────
	let filtered = $derived(
		entries.filter(e => {
			if (filterType !== 'all' && e.entryType !== filterType) return false;
			if (searchQ.trim()) {
				const q = searchQ.toLowerCase();
				return e.content.toLowerCase().includes(q) ||
					e.model.toLowerCase().includes(q) ||
					e.tags.some(t => t.toLowerCase().includes(q));
			}
			return true;
		})
	);

	let entryTypes = $derived([...new Set(entries.map(e => e.entryType))].sort());

	// ─── Load ───────────────────────────────────────────────────────────────────
	async function load() {
		loading = true;
		try {
			if (isDesktopRuntime()) {
				entries = await invokeTauri<MemoryEntry[]>('list_memory_entries_cmd', { limit: 200 });
			} else {
				// Web mode sample data
				entries = [
					{ id: '1', entryType: 'api_call', content: 'Scraped 3 pages about AI video generation tools. Runway, Kling, Luma pricing found.', tags: ['firecrawl', 'research'], providerId: 'firecrawl', model: 'scrape', createdAt: new Date(Date.now() - 3_600_000).toISOString() },
					{ id: '2', entryType: 'agent_action', content: 'Vy desktop agent opened VS Code and started the dev server for ralphhub project.', tags: ['vy', 'desktop'], providerId: 'vy', model: 'computer-use', createdAt: new Date(Date.now() - 7_200_000).toISOString() },
					{ id: '3', entryType: 'voice_command', content: 'Voice: "Open my email and check for replies to the Figma export". Intent: open_app + read.', tags: ['voice', 'panda'], providerId: 'voice', model: 'web-speech-api', createdAt: new Date(Date.now() - 86_400_000).toISOString() },
					{ id: '4', entryType: 'workflow', content: 'Parallel workflow ran: taxes research + Notion update + phone chat simultaneously.', tags: ['workflow', 'parallel'], providerId: 'orchestrator', model: 'workflow-runner', createdAt: new Date(Date.now() - 172_800_000).toISOString() },
				];
			}
		} catch (e) {
			entries = [];
		} finally {
			loading = false;
		}
	}

	onMount(async () => {
		await load();
		try { await getMemoryStats(); } catch { /* non-critical */ }
	});

	// ─── Helpers ─────────────────────────────────────────────────────────────────
	function timeAgo(iso: string): string {
		try {
			const diff = Date.now() - new Date(iso).getTime();
			if (diff < 60_000) return 'just now';
			if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
			if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
			return `${Math.floor(diff / 86_400_000)}d ago`;
		} catch { return ''; }
	}

	const TYPE_META: Record<string, {emoji: string; color: string}> = {
		'api_call':     { emoji: '⚡', color: 'text-violet-300 bg-violet-900/30' },
		'agent_action': { emoji: '🖥️', color: 'text-cyan-300 bg-cyan-900/30' },
		'voice_command':{ emoji: '🎙️', color: 'text-emerald-300 bg-emerald-900/30' },
		'workflow':     { emoji: '♾️', color: 'text-amber-300 bg-amber-900/30' },
		'deploy':       { emoji: '🚀', color: 'text-blue-300 bg-blue-900/30' },
	};
</script>

<div class="space-y-6 py-2">

	<!-- Header -->
	<div class="page-header">
		<div>
			<h1 class="page-title">🧠 Memory Spine</h1>
			<p class="page-subtitle">Every agent action, API call, voice command, and workflow — logged forever.</p>
		</div>
	</div>

	<!-- Stats -->
	{#if $memoryStatsStore}
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
			<div class="stat-card">
				<p class="text-xs font-medium text-slate-500 uppercase tracking-wider">Entries</p>
				<p class="mt-1.5 text-2xl font-bold text-white tabular-nums">{$memoryStatsStore.totalEntries}</p>
			</div>
			<div class="stat-card">
				<p class="text-xs font-medium text-slate-500 uppercase tracking-wider">Tokens</p>
				<p class="mt-1.5 text-2xl font-bold text-white tabular-nums">{$memoryStatsStore.totalTokens.toLocaleString()}</p>
			</div>
			<div class="stat-card">
				<p class="text-xs font-medium text-slate-500 uppercase tracking-wider">Cost</p>
				<p class="mt-1.5 text-2xl font-bold text-white tabular-nums">${$memoryStatsStore.totalCostUsd.toFixed(4)}</p>
			</div>
			<div class="stat-card">
				<p class="text-xs font-medium text-slate-500 uppercase tracking-wider">Providers</p>
				<p class="mt-1.5 text-2xl font-bold text-white tabular-nums">{$memoryStatsStore.providersUsed.length}</p>
			</div>
		</div>
	{/if}

	<!-- Filters + search -->
	<div class="flex flex-wrap gap-3 items-center">
		<input
			type="text"
			bind:value={searchQ}
			placeholder="Search memories…"
			class="rounded-xl bg-white/5 border border-white/10 px-3 py-2 text-sm text-slate-200 placeholder:text-slate-600 w-64 focus:border-violet-400/40 focus:outline-none"
		/>

		<div class="flex gap-2 flex-wrap">
			<button
				onclick={() => filterType = 'all'}
				class="rounded-xl px-3 py-2 text-xs font-medium transition {filterType === 'all' ? 'bg-violet-500/20 text-violet-200 border border-violet-400/25' : 'bg-white/5 text-slate-400 hover:text-slate-200 border border-white/7'}"
			>All</button>
			{#each entryTypes as t}
				<button
					onclick={() => filterType = t}
					class="rounded-xl px-3 py-2 text-xs font-medium transition {filterType === t ? 'bg-violet-500/20 text-violet-200 border border-violet-400/25' : 'bg-white/5 text-slate-400 hover:text-slate-200 border border-white/7'}"
				>
					{TYPE_META[t]?.emoji ?? '•'} {t.replace('_', ' ')}
				</button>
			{/each}
		</div>
	</div>

	<!-- Entry list -->
	{#if loading}
		<div class="flex items-center justify-center py-16">
			<div class="text-center">
				<div class="spinner mx-auto mb-3" style="width:32px;height:32px;"></div>
				<p class="text-sm text-slate-500">Loading memory entries…</p>
			</div>
		</div>
	{:else if filtered.length === 0}
		<div class="card text-center py-12">
			<p class="text-3xl mb-3">🧠</p>
			<p class="text-slate-300 font-medium">No memories yet</p>
			<p class="text-sm text-slate-500 mt-1">Memory Spine auto-fills as you use agent actions, voice commands, and workflows.</p>
		</div>
	{:else}
		<div class="space-y-2">
			{#each filtered as entry (entry.id)}
				<div class="card flex items-start gap-4 py-3.5 px-4">
					<!-- Type badge -->
					<div class="mt-0.5 shrink-0 rounded-lg px-2 py-1 text-xs font-bold {TYPE_META[entry.entryType]?.color ?? 'text-slate-400 bg-slate-800/50'}">
						{TYPE_META[entry.entryType]?.emoji ?? '•'}
					</div>

					<!-- Content -->
					<div class="flex-1 min-w-0">
						<p class="text-sm text-slate-200 leading-5">{entry.content}</p>
						<div class="mt-2 flex flex-wrap items-center gap-2">
							{#each entry.tags as tag}
								<span class="rounded-full bg-white/7 px-2 py-0.5 text-[10px] text-slate-400">{tag}</span>
							{/each}
							{#if entry.model}
								<span class="text-[10px] text-slate-600">{entry.model}</span>
							{/if}
							<span class="ml-auto text-[10px] text-slate-600">{timeAgo(entry.createdAt)}</span>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Active tasks from memory -->
	{#if $memoryStatsStore?.activeTasks && $memoryStatsStore.activeTasks.length > 0}
		<div class="card">
			<h3 class="mb-3 text-sm font-semibold text-slate-300">Active Kaizen Tasks</h3>
			<div class="space-y-2">
				{#each $memoryStatsStore.activeTasks as task}
					<div class="flex items-center gap-3 rounded-lg bg-white/4 px-3 py-2.5">
						<span class="shrink-0 rounded-full px-2 py-0.5 text-[10px] font-semibold bg-amber-400/15 text-amber-300">{task.priority}</span>
						<p class="text-sm text-slate-300 flex-1 min-w-0 truncate">{task.title}</p>
						<a href="/kaizen" class="shrink-0 text-xs text-violet-400 hover:text-violet-300">View →</a>
					</div>
				{/each}
			</div>
		</div>
	{/if}

</div>
