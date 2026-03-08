<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface DailyLogEntry {
		id: string;
		logDate: string;
		entryType: string;
		title: string;
		content: string;
		createdAt: string;
	}

	interface MorningDigest {
		date: string;
		todayTasks: Array<{ id: string; title: string; domain: string; energy: string; estimateMinutes: number; status: string }>;
		inboxCount: number;
		memoryUpdates: number;
		yesterdaySummary: string;
	}

	let entries = $state<DailyLogEntry[]>([]);
	let digest = $state<MorningDigest | null>(null);
	let loading = $state(true);
	let error = $state('');
	let selectedDate = $state(new Date().toISOString().slice(0, 10));
	let activeTab = $state<'log' | 'digest' | 'add'>('log');
	let adding = $state(false);
	let addMsg = $state('');

	let newEntryType = $state('agent_run');
	let newTitle = $state('');
	let newContent = $state('');

	const entryTypeIcon: Record<string, string> = {
		agent_run: '🤖',
		task_complete: '✅',
		browser_action: '🌐',
		notion_sync: '🔲',
		morning_digest: '🌅',
		nightly_wrap: '🌙',
		memory_write: '🧠',
		manual: '📝',
	};

	const entryTypeColor: Record<string, string> = {
		agent_run: 'border-purple-400/20 bg-purple-400/8 text-purple-300',
		task_complete: 'border-emerald-400/20 bg-emerald-400/8 text-emerald-300',
		browser_action: 'border-blue-400/20 bg-blue-400/8 text-blue-300',
		notion_sync: 'border-gray-400/20 bg-gray-400/8 text-gray-300',
		morning_digest: 'border-amber-400/20 bg-amber-400/8 text-amber-300',
		nightly_wrap: 'border-indigo-400/20 bg-indigo-400/8 text-indigo-300',
		memory_write: 'border-cyan-400/20 bg-cyan-400/8 text-cyan-300',
		manual: 'border-white/10 bg-white/5 text-slate-300',
	};

	const energyDot: Record<string, string> = {
		low: 'bg-emerald-400', medium: 'bg-amber-400', high: 'bg-red-400',
	};

	function fmtMinutes(m: number): string {
		if (m < 60) return `${m}m`;
		return `${Math.floor(m / 60)}h ${m % 60 > 0 ? m % 60 + 'm' : ''}`.trim();
	}

	async function load() {
		loading = true;
		error = '';
		try {
			if (!isDesktopRuntime()) {
				const now = new Date().toISOString();
				entries = [
					{ id: '1', logDate: selectedDate, entryType: 'morning_digest', title: 'Morning Digest', content: '3 tasks today, 2 inbox items', createdAt: now },
					{ id: '2', logDate: selectedDate, entryType: 'agent_run', title: 'Perplexica research run', content: 'Researched "local-first apps for ADHD productivity"', createdAt: now },
					{ id: '3', logDate: selectedDate, entryType: 'task_complete', title: 'Completed: Write Memory Spine spec', content: '', createdAt: now },
					{ id: '4', logDate: selectedDate, entryType: 'memory_write', title: 'Memory ingested: AI productivity research', content: '120 chars captured', createdAt: now },
				];
				digest = {
					date: selectedDate,
					todayTasks: [
						{ id: '1', title: 'Build AmitOS Dashboard', domain: 'work', energy: 'high', estimateMinutes: 120, status: 'doing' },
						{ id: '2', title: 'Review Kaizen task schema', domain: 'work', energy: 'medium', estimateMinutes: 30, status: 'todo' },
					],
					inboxCount: 2,
					memoryUpdates: 4,
					yesterdaySummary: '• Completed: Bootstrap RalphHub\n• Agent run: Code review workflow\n• Memory write: 3 items',
				};
				loading = false;
				return;
			}
			const [logEntries, morningDigest] = await Promise.all([
				invokeTauri<DailyLogEntry[]>('list_daily_log', { date: selectedDate }),
				invokeTauri<MorningDigest>('get_morning_digest'),
			]);
			entries = logEntries;
			digest = morningDigest;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function addEntry() {
		if (!newTitle.trim()) { addMsg = 'Title is required.'; return; }
		adding = true;
		addMsg = '';
		try {
			if (isDesktopRuntime()) {
				await invokeTauri('add_daily_log_entry', {
					request: {
						logDate: selectedDate,
						entryType: newEntryType,
						title: newTitle,
						content: newContent,
					},
				});
			}
			addMsg = '✓ Entry added';
			newTitle = '';
			newContent = '';
			setTimeout(() => { addMsg = ''; }, 2500);
			await load();
			activeTab = 'log';
		} catch (e) {
			addMsg = '✗ ' + String(e);
		} finally {
			adding = false;
		}
	}

	// Group entries by type for display
	const groupedEntries = $derived(() => {
		const groups: Record<string, DailyLogEntry[]> = {};
		for (const e of entries) {
			const key = e.entryType;
			if (!groups[key]) groups[key] = [];
			groups[key].push(e);
		}
		return Object.entries(groups);
	});

	onMount(load);
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-start justify-between">
		<div>
			<h1 class="text-2xl font-bold text-white">📓 Daily Log</h1>
			<p class="mt-1 text-sm text-slate-400">Auto-fed from every agent run, task completion, browser action, and Notion sync.</p>
		</div>
		<div class="flex items-center gap-2">
			<input
				type="date"
				bind:value={selectedDate}
				onchange={load}
				class="rounded-xl border border-white/10 bg-slate-800/60 px-3 py-2 text-sm text-white outline-none focus:border-cyan-400/40"
			/>
			<button onclick={load} class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300 hover:bg-white/10">↻</button>
		</div>
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	<!-- Tabs -->
	<div class="flex gap-1 rounded-2xl border border-white/8 bg-slate-900/40 p-1">
		{#each [
			{ key: 'log', label: '📋 Log Entries' },
			{ key: 'digest', label: '🌅 Morning Digest' },
			{ key: 'add', label: '➕ Add Entry' },
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

	<!-- Log Tab -->
	{#if activeTab === 'log'}
		{#if loading}
			<div class="py-12 text-center text-slate-500">Loading log…</div>
		{:else if entries.length === 0}
			<div class="rounded-2xl border border-dashed border-white/10 py-16 text-center">
				<p class="text-4xl">📓</p>
				<p class="mt-3 text-slate-400">No entries for {selectedDate}. Agent runs and task completions will appear here automatically.</p>
			</div>
		{:else}
			<!-- Timeline -->
			<div class="relative space-y-1">
				{#each entries as entry}
					<div class="flex items-start gap-4">
						<div class="flex w-8 flex-col items-center">
							<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-white/10 bg-slate-900 text-base">{entryTypeIcon[entry.entryType] ?? '📌'}</div>
							<div class="mt-1 h-6 w-px bg-white/8"></div>
						</div>
						<div class="flex-1 min-w-0 pb-2">
							<div class="flex items-center gap-2">
								<span class="rounded-full border px-2 py-0.5 text-[10px] font-medium {entryTypeColor[entry.entryType] ?? 'border-white/10 text-slate-400'}">{entry.entryType.replace('_', ' ')}</span>
								<span class="text-[10px] text-slate-600">{new Date(entry.createdAt).toLocaleTimeString()}</span>
							</div>
							<p class="mt-1 font-medium text-white text-sm">{entry.title}</p>
							{#if entry.content}
								<p class="mt-0.5 text-xs text-slate-400">{entry.content}</p>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{/if}

	<!-- Morning Digest Tab -->
	{#if activeTab === 'digest'}
		{#if loading}
			<div class="py-12 text-center text-slate-500">Loading digest…</div>
		{:else if digest}
			<div class="space-y-5">
				<!-- Date banner -->
				<div class="rounded-2xl border border-amber-400/20 bg-gradient-to-r from-amber-400/8 to-orange-400/5 px-6 py-5">
					<p class="text-xs font-semibold uppercase tracking-wider text-amber-400">🌅 Morning Digest</p>
					<h2 class="mt-1 text-xl font-bold text-white">{new Date(digest.date).toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}</h2>
					<div class="mt-3 grid grid-cols-3 gap-3">
						<div class="rounded-xl bg-black/20 p-3 text-center">
							<p class="text-lg font-bold text-white">{digest.todayTasks.length}</p>
							<p class="text-[10px] text-amber-300/60">Tasks Today</p>
						</div>
						<div class="rounded-xl bg-black/20 p-3 text-center">
							<p class="text-lg font-bold text-white">{digest.inboxCount}</p>
							<p class="text-[10px] text-amber-300/60">In Inbox</p>
						</div>
						<div class="rounded-xl bg-black/20 p-3 text-center">
							<p class="text-lg font-bold text-white">{digest.memoryUpdates}</p>
							<p class="text-[10px] text-amber-300/60">Yesterday Events</p>
						</div>
					</div>
				</div>

				<!-- Today's tasks -->
				{#if digest.todayTasks.length > 0}
					<div>
						<h3 class="mb-3 text-sm font-bold uppercase tracking-wider text-slate-500">Today's Tasks</h3>
						<div class="space-y-2">
							{#each digest.todayTasks as task}
								<div class="flex items-center gap-3 rounded-xl border border-white/8 bg-slate-900/50 px-4 py-3">
									<span class={`h-2.5 w-2.5 shrink-0 rounded-full ${energyDot[task.energy] ?? 'bg-slate-500'}`}></span>
									<p class="flex-1 text-sm text-white">{task.title}</p>
									<span class="text-[10px] text-slate-500">{fmtMinutes(task.estimateMinutes)}</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Yesterday summary -->
				<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-5">
					<h3 class="mb-3 text-sm font-bold uppercase tracking-wider text-slate-500">Yesterday</h3>
					<pre class="whitespace-pre-wrap font-sans text-sm leading-relaxed text-slate-300">{digest.yesterdaySummary}</pre>
				</div>

				<div class="flex gap-2">
					<a href="/tasks" class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-5 py-2.5 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400">→ Today Board</a>
					<a href="/inbox" class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 text-sm font-medium text-slate-300 hover:bg-white/10">Process Inbox</a>
				</div>
			</div>
		{/if}
	{/if}

	<!-- Add Entry Tab -->
	{#if activeTab === 'add'}
		<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-6">
			<h2 class="mb-5 text-lg font-bold text-white">➕ Add Log Entry</h2>

			<div class="space-y-4">
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Entry Type</label>
					<div class="flex flex-wrap gap-2">
						{#each Object.keys(entryTypeIcon) as t}
							<button
								onclick={() => (newEntryType = t)}
								class={`rounded-xl border px-3 py-1.5 text-sm transition ${
									newEntryType === t
										? 'border-cyan-400/40 bg-cyan-400/15 text-cyan-100'
										: 'border-white/10 text-slate-400 hover:text-white'
								}`}
							>
								{entryTypeIcon[t]} {t.replace('_', ' ')}
							</button>
						{/each}
					</div>
				</div>

				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Title</label>
					<input
						bind:value={newTitle}
						type="text"
						placeholder="What happened?"
						class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40"
					/>
				</div>

				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Content (optional)</label>
					<textarea
						bind:value={newContent}
						rows={3}
						placeholder="Additional details, links, evidence…"
						class="w-full resize-none rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40"
					></textarea>
				</div>

				<div class="flex items-center gap-3">
					<button
						onclick={addEntry}
						disabled={adding || !newTitle.trim()}
						class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-6 py-3 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400 disabled:opacity-40"
					>
						{adding ? 'Adding…' : '📓 Add Entry'}
					</button>
					{#if addMsg}
						<p class="text-sm {addMsg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{addMsg}</p>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>
