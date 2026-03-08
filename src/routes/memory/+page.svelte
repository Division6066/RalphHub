<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type MemoryEntry = {
		id: string;
		toolId: string;
		entryType: string;
		content: string;
		tags: string;
		createdAt: string;
	};

	let entries: MemoryEntry[] = [];
	let loading = true;
	let busy = false;
	let message = '';
	let filterToolId = '';

	let newContent = '';
	let newToolId = '';
	let newType = 'note';
	let newTags = '';

	const entryTypes = ['note', 'report', 'result', 'insight', 'task'];

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
			entries = await invokeTauri<MemoryEntry[]>('list_memory_entries', {
				toolId: filterToolId || null
			});
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to load Memory Spine.';
		} finally {
			loading = false;
		}
	}

	async function writeEntry() {
		if (!newContent.trim()) return;
		busy = true;
		try {
			await invokeTauri('write_memory_entry', {
				toolId: newToolId || 'manual',
				entryType: newType,
				content: newContent,
				tags: newTags
			});
			newContent = '';
			newTags = '';
			message = 'Entry written to Memory Spine.';
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to write entry.';
		} finally {
			busy = false;
		}
	}

	async function deleteEntry(id: string) {
		try {
			await invokeTauri('delete_memory_entry', { id });
			entries = entries.filter((e) => e.id !== id);
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to delete entry.';
		}
	}

	function formatDate(iso: string) {
		return new Date(iso).toLocaleString();
	}

	function typeColor(type: string) {
		switch (type) {
			case 'report': return 'bg-cyan-400/15 text-cyan-300';
			case 'result': return 'bg-green-400/15 text-green-300';
			case 'insight': return 'bg-violet-400/15 text-violet-300';
			case 'task': return 'bg-amber-400/15 text-amber-300';
			default: return 'bg-slate-600/30 text-slate-400';
		}
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Memory Spine</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Persistent memory for every tool run.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Every tool and workflow writes run reports, insights, and results here automatically.
			Queryable by tool, type, and tag.
		</p>
	</div>

	{#if message}
		<div class="rounded-3xl border border-cyan-400/20 bg-cyan-500/10 p-4 text-sm text-cyan-100">
			{message}
		</div>
	{/if}

	<!-- Write entry -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white">Write memory entry</h2>
		<div class="mt-4 grid gap-3 sm:grid-cols-2">
			<input
				bind:value={newToolId}
				placeholder="Tool ID (e.g. perplexica)"
				class="rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600"
			/>
			<select
				bind:value={newType}
				class="rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
			>
				{#each entryTypes as t}
					<option value={t}>{t}</option>
				{/each}
			</select>
		</div>
		<textarea
			bind:value={newContent}
			rows="4"
			placeholder="Content, notes, or run summary..."
			class="mt-3 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600 resize-none"
		></textarea>
		<div class="mt-3 flex gap-3">
			<input
				bind:value={newTags}
				placeholder="Tags (comma-separated)"
				class="flex-1 rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600"
			/>
			<button
				on:click={writeEntry}
				disabled={busy || !newContent.trim()}
				class="rounded-full bg-cyan-400/15 px-5 py-2 text-sm font-medium text-cyan-100 hover:bg-cyan-400/25 disabled:opacity-60"
			>
				{busy ? 'Writing...' : 'Write to spine'}
			</button>
		</div>
	</div>

	<!-- Filter + list -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-center gap-3">
			<h2 class="flex-1 text-lg font-semibold text-white">Memory entries</h2>
			<input
				bind:value={filterToolId}
				on:input={refresh}
				placeholder="Filter by tool ID"
				class="w-48 rounded-full border border-white/10 bg-slate-900/60 px-4 py-2 text-sm text-white outline-none placeholder:text-slate-600"
			/>
			<button
				on:click={refresh}
				class="rounded-full border border-white/10 px-3 py-2 text-xs text-slate-400 hover:text-white"
			>
				Refresh
			</button>
		</div>

		<div class="mt-6 space-y-3">
			{#if loading}
				<p class="text-sm text-slate-500">Loading...</p>
			{:else if !entries.length}
				<p class="text-sm text-slate-500">No memory entries yet. Tools write here automatically after each run.</p>
			{:else}
				{#each entries as entry}
					<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex items-start justify-between gap-4">
							<div class="min-w-0 flex-1">
								<div class="flex items-center gap-2 flex-wrap">
									<span class="rounded-full px-2 py-0.5 text-xs {typeColor(entry.entryType)}">{entry.entryType}</span>
									<span class="text-xs text-slate-500 font-mono">{entry.toolId}</span>
									{#if entry.tags}
										{#each entry.tags.split(',') as tag}
											<span class="rounded-full border border-white/10 px-2 py-0.5 text-xs text-slate-500">{tag.trim()}</span>
										{/each}
									{/if}
								</div>
								<p class="mt-2 text-sm text-slate-300 leading-6 line-clamp-3 whitespace-pre-wrap">{entry.content}</p>
								<p class="mt-2 text-xs text-slate-600">{formatDate(entry.createdAt)}</p>
							</div>
							<button
								on:click={() => deleteEntry(entry.id)}
								class="shrink-0 text-xs text-slate-600 hover:text-red-400"
							>
								Delete
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
