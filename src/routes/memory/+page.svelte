<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface MemoryStats {
		rawEventsCount: number;
		workingMemoryCount: number;
		longTermCount: number;
		summariesCount: number;
		inboxCount: number;
		dailyLogCount: number;
	}

	interface RawEvent {
		id: string;
		sourceType: string;
		content: string;
		metadata: Record<string, unknown>;
		createdAt: string;
	}

	interface WorkingMemoryItem {
		id: string;
		title: string;
		content: string;
		tags: string[];
		expiresAt: string | null;
		createdAt: string;
		updatedAt: string;
	}

	let stats = $state<MemoryStats | null>(null);
	let rawEvents = $state<RawEvent[]>([]);
	let workingMemory = $state<WorkingMemoryItem[]>([]);
	let loading = $state(true);
	let error = $state('');
	let activeTab = $state<'working' | 'raw' | 'capture'>('working');
	let captureContent = $state('');
	let captureType = $state('text');
	let captureTags = $state('');
	let captureAutoSummarize = $state(true);
	let capturing = $state(false);
	let captureMsg = $state('');

	const sourceIcons: Record<string, string> = {
		text: '📝',
		url: '🔗',
		file: '📄',
		browser_agent: '🌐',
		notion: '🔲',
		workflow: '⚙️',
		task: '✅',
	};

	async function load() {
		loading = true;
		error = '';
		try {
			if (!isDesktopRuntime()) {
				stats = { rawEventsCount: 3, workingMemoryCount: 2, longTermCount: 1, summariesCount: 0, inboxCount: 5, dailyLogCount: 8 };
				rawEvents = [
					{ id: '1', sourceType: 'text', content: 'Sample memory entry from browser preview', metadata: {}, createdAt: new Date().toISOString() },
					{ id: '2', sourceType: 'workflow', content: 'Workflow run completed: Perplexica research on AI memory systems', metadata: {}, createdAt: new Date().toISOString() },
				];
				workingMemory = [
					{ id: '1', title: 'AI Memory Systems Overview', content: 'Key insight: Memory spines require both fast retrieval and slow consolidation.', tags: ['ai', 'memory'], expiresAt: null, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
				];
				loading = false;
				return;
			}
			const [s, re, wm] = await Promise.all([
				invokeTauri<MemoryStats>('get_memory_stats'),
				invokeTauri<RawEvent[]>('list_raw_events', { limit: 30 }),
				invokeTauri<WorkingMemoryItem[]>('list_working_memory'),
			]);
			stats = s;
			rawEvents = re;
			workingMemory = wm;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function capture() {
		if (!captureContent.trim()) return;
		capturing = true;
		captureMsg = '';
		try {
			const tags = captureTags.split(',').map((t) => t.trim()).filter(Boolean);
			if (isDesktopRuntime()) {
				await invokeTauri('ingest_memory', {
					request: {
						sourceType: captureType,
						content: captureContent,
						metadata: { tags },
						autoSummarize: captureAutoSummarize,
					},
				});
			}
			captureMsg = '✓ Captured to Memory Spine';
			captureContent = '';
			captureTags = '';
			setTimeout(() => { captureMsg = ''; }, 3000);
			await load();
		} catch (e) {
			captureMsg = '✗ ' + String(e);
		} finally {
			capturing = false;
		}
	}

	async function promoteToLongTerm(id: string) {
		if (!isDesktopRuntime()) return;
		try {
			await invokeTauri('promote_to_long_term', { workingMemoryId: id });
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
			<h1 class="text-2xl font-bold text-white">🧠 Memory Spine</h1>
			<p class="mt-1 text-sm text-slate-400">Single source of truth — all agents, browser actions, and workflows write here.</p>
		</div>
		<button
			onclick={load}
			class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300 transition hover:bg-white/10"
		>
			↻ Refresh
		</button>
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	<!-- Stats bar -->
	{#if stats}
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-6">
			{#each [
				{ label: 'Raw Events', value: stats.rawEventsCount, color: 'cyan' },
				{ label: 'Working Mem', value: stats.workingMemoryCount, color: 'violet' },
				{ label: 'Long-Term', value: stats.longTermCount, color: 'emerald' },
				{ label: 'Summaries', value: stats.summariesCount, color: 'amber' },
				{ label: 'Inbox', value: stats.inboxCount, color: 'pink' },
				{ label: 'Log Entries', value: stats.dailyLogCount, color: 'sky' },
			] as s}
				<div class="rounded-2xl border border-white/8 bg-slate-900/60 p-3 text-center">
					<p class="text-xl font-bold text-white">{s.value}</p>
					<p class="mt-0.5 text-[10px] font-medium uppercase tracking-wider text-slate-500">{s.label}</p>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Tabs -->
	<div class="flex gap-1 rounded-2xl border border-white/8 bg-slate-900/40 p-1">
		{#each [
			{ key: 'working', label: '⚡ Working Memory' },
			{ key: 'raw', label: '📡 Raw Events' },
			{ key: 'capture', label: '➕ Capture' },
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

	<!-- Working Memory Tab -->
	{#if activeTab === 'working'}
		<div class="space-y-3">
			{#if loading}
				<div class="py-12 text-center text-slate-500">Loading memory…</div>
			{:else if workingMemory.length === 0}
				<div class="rounded-2xl border border-dashed border-white/10 py-12 text-center text-slate-500">
					<p class="text-3xl">🧠</p>
					<p class="mt-2">No working memory yet. Capture something below.</p>
				</div>
			{:else}
				{#each workingMemory as item}
					<div class="group rounded-2xl border border-white/8 bg-slate-900/50 p-4 transition hover:border-cyan-400/20 hover:bg-slate-800/60">
						<div class="flex items-start justify-between gap-3">
							<div class="flex-1 min-w-0">
								<h3 class="font-semibold text-white">{item.title}</h3>
								<p class="mt-1 text-sm leading-relaxed text-slate-400">{item.content}</p>
								{#if item.tags.length > 0}
									<div class="mt-2 flex flex-wrap gap-1">
										{#each item.tags as tag}
											<span class="rounded-full border border-cyan-400/20 bg-cyan-400/8 px-2 py-0.5 text-[10px] font-medium text-cyan-300">{tag}</span>
										{/each}
									</div>
								{/if}
								<p class="mt-2 text-[10px] text-slate-600">{new Date(item.createdAt).toLocaleString()}</p>
							</div>
							<button
								onclick={() => promoteToLongTerm(item.id)}
								class="shrink-0 rounded-xl border border-emerald-400/20 bg-emerald-400/8 px-3 py-1.5 text-xs font-medium text-emerald-300 opacity-0 transition hover:bg-emerald-400/15 group-hover:opacity-100"
							>
								→ Long-Term
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	{/if}

	<!-- Raw Events Tab -->
	{#if activeTab === 'raw'}
		<div class="space-y-2">
			{#if loading}
				<div class="py-12 text-center text-slate-500">Loading events…</div>
			{:else if rawEvents.length === 0}
				<div class="rounded-2xl border border-dashed border-white/10 py-12 text-center text-slate-500">
					<p class="text-3xl">📡</p>
					<p class="mt-2">No raw events yet. Agent runs will appear here automatically.</p>
				</div>
			{:else}
				{#each rawEvents as ev}
					<div class="flex items-start gap-3 rounded-xl border border-white/6 bg-slate-900/40 px-4 py-3 text-sm">
						<span class="mt-0.5 shrink-0 text-lg">{sourceIcons[ev.sourceType] ?? '📌'}</span>
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-2">
								<span class="rounded-full bg-slate-700 px-2 py-0.5 text-[10px] font-medium text-slate-300">{ev.sourceType}</span>
								<span class="text-[10px] text-slate-600">{new Date(ev.createdAt).toLocaleString()}</span>
							</div>
							<p class="mt-1 truncate text-slate-300">{ev.content}</p>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	{/if}

	<!-- Capture Tab -->
	{#if activeTab === 'capture'}
		<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-6">
			<h2 class="mb-5 text-lg font-bold text-white">➕ Capture to Memory Spine</h2>

			<div class="space-y-4">
				<!-- Type selector -->
				<div>
					<label class="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-500">Source Type</label>
					<div class="flex flex-wrap gap-2">
						{#each ['text', 'url', 'file', 'browser_agent', 'notion', 'workflow'] as t}
							<button
								onclick={() => (captureType = t)}
								class={`rounded-xl border px-3 py-2 text-sm font-medium transition ${
									captureType === t
										? 'border-cyan-400/40 bg-cyan-400/15 text-cyan-100'
										: 'border-white/10 text-slate-400 hover:border-white/20 hover:text-white'
								}`}
							>
								{sourceIcons[t]} {t}
							</button>
						{/each}
					</div>
				</div>

				<!-- Content -->
				<div>
					<label class="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-500">Content</label>
					<textarea
						bind:value={captureContent}
						rows={5}
						placeholder="Paste text, URL, screenshot description, or notes here…"
						class="w-full resize-none rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-400/10"
					></textarea>
				</div>

				<!-- Tags -->
				<div>
					<label class="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-500">Tags (comma-separated)</label>
					<input
						bind:value={captureTags}
						type="text"
						placeholder="e.g. ai, research, urgent"
						class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-400/10"
					/>
				</div>

				<!-- Auto-summarize -->
				<label class="flex cursor-pointer items-center gap-3">
					<input type="checkbox" bind:checked={captureAutoSummarize} class="h-4 w-4 rounded" />
					<span class="text-sm text-slate-300">Auto-add to Working Memory</span>
				</label>

				<div class="flex items-center gap-3">
					<button
						onclick={capture}
						disabled={capturing || !captureContent.trim()}
						class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-6 py-3 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 transition hover:from-cyan-400 hover:to-cyan-500 disabled:opacity-40"
					>
						{capturing ? 'Capturing…' : '🧠 Capture to Memory'}
					</button>
					{#if captureMsg}
						<p class="text-sm {captureMsg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{captureMsg}</p>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>
