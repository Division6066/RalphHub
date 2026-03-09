<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type MemoryEntry = {
		id: string;
		title: string;
		content: string;
		tags: string[];
		domain: string;
		source: string;
		createdAt: string;
		updatedAt: string;
	};

	let entries: MemoryEntry[] = [];
	let loading = true;
	let searchQuery = '';
	let searching = false;
	let selectedDomain = 'all';
	let showAddModal = false;
	let editEntry: MemoryEntry | null = null;
	let savingEntry = false;
	let statusMsg = '';
	let isDesktop = false;

	// Form
	let formTitle = '';
	let formContent = '';
	let formDomain = 'general';
	let formTags = '';
	let formSource = 'manual';

	const DOMAINS = [
		{ id: 'all', label: '🌐 All', color: '#94a3b8' },
		{ id: 'general', label: '⭐ General', color: '#64748b' },
		{ id: 'work', label: '💼 Work', color: '#6366f1' },
		{ id: 'learning', label: '📚 Learning', color: '#f59e0b' },
		{ id: 'health', label: '🏃 Health', color: '#10b981' },
		{ id: 'creative', label: '🎨 Creative', color: '#ec4899' },
		{ id: 'relationships', label: '❤️ People', color: '#ef4444' },
		{ id: 'finance', label: '💰 Finance', color: '#14b8a6' },
		{ id: 'home', label: '🏠 Home', color: '#8b5cf6' },
	];

	async function loadEntries() {
		if (!isDesktopRuntime()) { isDesktop = false; loading = false; return; }
		isDesktop = true;
		try {
			entries = await invokeTauri<MemoryEntry[]>('list_memory_entries', {
				domain: selectedDomain === 'all' ? null : selectedDomain,
				limit: 100
			});
		} catch (e) { statusMsg = String(e); }
		finally { loading = false; }
	}

	onMount(loadEntries);

	async function doSearch() {
		if (!searchQuery.trim()) { await loadEntries(); return; }
		searching = true;
		try {
			entries = await invokeTauri<MemoryEntry[]>('search_memory', {
				request: {
					query: searchQuery,
					domain: selectedDomain === 'all' ? null : selectedDomain,
					limit: 50
				}
			});
		} catch (e) { statusMsg = String(e); }
		finally { searching = false; }
	}

	async function saveEntry() {
		if (!formTitle.trim() || !formContent.trim()) return;
		savingEntry = true;
		try {
			const tags = formTags ? formTags.split(',').map(t => t.trim()).filter(Boolean) : [];
			if (editEntry) {
				await invokeTauri('update_memory_entry', {
					id: editEntry.id,
					title: formTitle,
					content: formContent,
					tags,
					domain: formDomain
				});
			} else {
				await invokeTauri('create_memory_entry', {
					request: {
						title: formTitle,
						content: formContent,
						tags,
						domain: formDomain,
						source: formSource
					}
				});
			}
			closeModal();
			await loadEntries();
		} catch (e) { statusMsg = String(e); }
		finally { savingEntry = false; }
	}

	async function deleteEntry(id: string) {
		if (!confirm('Delete this memory?')) return;
		await invokeTauri('delete_memory_entry', { id });
		await loadEntries();
	}

	function openEdit(entry: MemoryEntry) {
		editEntry = entry;
		formTitle = entry.title;
		formContent = entry.content;
		formDomain = entry.domain;
		formTags = entry.tags.join(', ');
		formSource = entry.source;
		showAddModal = true;
	}

	function closeModal() {
		showAddModal = false;
		editEntry = null;
		formTitle = ''; formContent = ''; formDomain = 'general'; formTags = ''; formSource = 'manual';
	}

	function formatDate(iso: string) {
		return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function domainColor(id: string) {
		return DOMAINS.find(d => d.id === id)?.color ?? '#64748b';
	}

	let searchTimeout: ReturnType<typeof setTimeout>;
	function onSearchInput() {
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(doSearch, 300);
	}
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-cyan-400/20 bg-gradient-to-br from-cyan-950/40 via-slate-950/80 to-blue-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-cyan-300/70">Knowledge Base</p>
		<h1 class="mt-2 text-3xl font-bold text-white">🧠 Memory Spine</h1>
		<p class="mt-2 text-sm text-slate-400">Capture everything. Search instantly. Never lose a thought.</p>

		<div class="mt-5 flex flex-wrap gap-3">
			<button onclick={() => showAddModal = true} class="rounded-xl bg-cyan-400 px-5 py-2.5 text-sm font-bold text-cyan-950 shadow-lg transition hover:bg-cyan-300">
				+ Capture Memory
			</button>
			<span class="rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 text-sm text-slate-400">
				{entries.length} memories stored
			</span>
		</div>
	</div>

	<!-- Search + domain filter -->
	<div class="flex flex-wrap gap-3">
		<div class="relative flex-1 min-w-48">
			<span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-500">🔍</span>
			<input
				bind:value={searchQuery}
				oninput={onSearchInput}
				placeholder="Search memories…"
				class="w-full rounded-xl border border-white/10 bg-slate-950/60 pl-10 pr-4 py-2.5 text-sm text-white outline-none focus:border-cyan-400 backdrop-blur"
			/>
		</div>
		<div class="flex gap-1.5 overflow-x-auto">
			{#each DOMAINS as d}
				<button
					onclick={() => { selectedDomain = d.id; loadEntries(); }}
					class={`shrink-0 rounded-xl border px-3 py-2 text-xs font-medium transition ${selectedDomain === d.id ? 'border-cyan-400/30 bg-cyan-400/15 text-cyan-100' : 'border-white/8 bg-slate-950/50 text-slate-400 hover:text-white'}`}
				>
					{d.label.split(' ')[0]}
				</button>
			{/each}
		</div>
	</div>

	<!-- Memory grid -->
	{#if loading}
		<div class="py-10 text-center text-sm text-slate-400">Loading memory spine…</div>
	{:else if !isDesktop}
		<div class="rounded-xl border border-cyan-400/20 bg-cyan-400/8 p-6 text-sm text-cyan-200">
			<p class="font-bold mb-2">Browser mode — no persistence</p>
			<p>Launch the AmitOS desktop app to use Memory Spine.</p>
		</div>
	{:else if entries.length === 0}
		<div class="rounded-2xl border border-dashed border-white/12 p-14 text-center">
			<p class="text-4xl mb-3">🧠</p>
			<p class="text-base font-semibold text-white mb-2">{searchQuery ? 'No memories match your search' : 'Memory Spine is empty'}</p>
			<p class="text-sm text-slate-400 mb-5">{searchQuery ? 'Try different keywords' : 'Capture your first thought, insight, or note.'}</p>
			{#if !searchQuery}
				<button onclick={() => showAddModal = true} class="rounded-xl bg-cyan-400 px-5 py-2.5 text-sm font-bold text-cyan-950">+ Capture Memory</button>
			{/if}
		</div>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
			{#each entries as entry}
				<div class="group relative rounded-xl border border-white/8 bg-slate-950/50 p-5 transition hover:border-white/15 backdrop-blur">
					<div class="flex items-start gap-3">
						<div class="h-2 w-2 shrink-0 mt-1.5 rounded-full" style="background: {domainColor(entry.domain)}"></div>
						<div class="flex-1 min-w-0">
							<h3 class="text-sm font-semibold text-white truncate">{entry.title}</h3>
							<p class="mt-1.5 text-xs leading-5 text-slate-400 line-clamp-3">{entry.content}</p>
							{#if entry.tags.length > 0}
								<div class="mt-2.5 flex flex-wrap gap-1">
									{#each entry.tags as tag}
										<span class="rounded-full bg-white/8 px-2 py-0.5 text-[10px] text-slate-400">{tag}</span>
									{/each}
								</div>
							{/if}
							<div class="mt-2.5 flex items-center gap-2 text-[10px] text-slate-600">
								<span>{entry.domain}</span>
								<span>·</span>
								<span>{formatDate(entry.createdAt)}</span>
								{#if entry.source !== 'manual'}<span>· {entry.source}</span>{/if}
							</div>
						</div>
					</div>
					<div class="absolute right-3 top-3 flex gap-1 opacity-0 group-hover:opacity-100 transition">
						<button onclick={() => openEdit(entry)} class="rounded-lg p-1.5 text-slate-600 hover:text-cyan-400 transition">✏️</button>
						<button onclick={() => deleteEntry(entry.id)} class="rounded-lg p-1.5 text-slate-600 hover:text-rose-400 transition">✕</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	{#if statusMsg}
		<div class="rounded-xl border border-rose-400/20 bg-rose-950/20 p-3 text-xs text-rose-300">{statusMsg}</div>
	{/if}
</section>

<!-- Add/Edit Modal -->
{#if showAddModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
		<div class="w-full max-w-lg rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl">
			<div class="flex items-center justify-between mb-5">
				<h3 class="text-lg font-bold text-white">{editEntry ? 'Edit Memory' : '🧠 Capture Memory'}</h3>
				<button onclick={closeModal} class="text-slate-500 hover:text-white">✕</button>
			</div>
			<div class="space-y-4">
				<input bind:value={formTitle} placeholder="Title*" class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400" autofocus />
				<textarea bind:value={formContent} rows="5" placeholder="Content — paste notes, insights, links, anything*" class="w-full resize-none rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400"></textarea>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="mb-1.5 block text-xs text-slate-400">Domain</label>
						<select bind:value={formDomain} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
							{#each DOMAINS.filter(d => d.id !== 'all') as d}
								<option value={d.id}>{d.label}</option>
							{/each}
						</select>
					</div>
					<div>
						<label class="mb-1.5 block text-xs text-slate-400">Source</label>
						<select bind:value={formSource} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
							<option value="manual">✍️ Manual</option>
							<option value="voice">🎙️ Voice</option>
							<option value="ai">🤖 AI Generated</option>
							<option value="research">🔍 Research</option>
							<option value="web">🌐 Web</option>
							<option value="book">📚 Book</option>
						</select>
					</div>
				</div>
				<input bind:value={formTags} placeholder="Tags (comma separated, optional)" class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-2.5 text-sm text-white outline-none focus:border-cyan-400" />
				<div class="flex gap-3 pt-2">
					<button onclick={saveEntry} disabled={savingEntry || !formTitle.trim() || !formContent.trim()} class="flex-1 rounded-xl bg-cyan-400 py-3 text-sm font-bold text-cyan-950 disabled:opacity-50">
						{savingEntry ? 'Saving…' : editEntry ? 'Update Memory' : 'Save Memory'}
					</button>
					<button onclick={closeModal} class="rounded-xl border border-white/10 bg-white/5 px-5 py-3 text-sm text-white">Cancel</button>
				</div>
			</div>
		</div>
	</div>
{/if}
