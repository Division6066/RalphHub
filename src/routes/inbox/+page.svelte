<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface InboxItem {
		id: string;
		content: string;
		contentType: string;
		processed: boolean;
		source: string;
		createdAt: string;
	}

	let items = $state<InboxItem[]>([]);
	let loading = $state(true);
	let error = $state('');
	let showProcessed = $state(false);
	let capturing = $state(false);
	let captureMsg = $state('');

	let captureContent = $state('');
	let captureType = $state('text');

	let convertId = $state('');
	let convertDomain = $state('work');
	let convertEnergy = $state('medium');
	let convertEstimate = $state(30);
	let convertDate = $state('');
	let converting = $state(false);
	let convertMsg = $state('');

	const typeIcons: Record<string, string> = {
		text: '📝', url: '🔗', screenshot: '📷', voice: '🎙️', file: '📄',
	};

	async function load() {
		loading = true;
		error = '';
		try {
			if (!isDesktopRuntime()) {
				items = [
					{ id: '1', content: 'Read: https://www.lesswrong.com/posts/cognition-article', contentType: 'url', processed: false, source: 'manual', createdAt: new Date().toISOString() },
					{ id: '2', content: 'Idea: Build a local-first PARA system with auto-tagging', contentType: 'text', processed: false, source: 'manual', createdAt: new Date().toISOString() },
					{ id: '3', content: 'Follow up with team about sprint review', contentType: 'text', processed: true, source: 'manual', createdAt: new Date().toISOString() },
				];
				loading = false;
				return;
			}
			items = await invokeTauri<InboxItem[]>('list_inbox_items', { unprocessedOnly: !showProcessed });
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
			if (isDesktopRuntime()) {
				await invokeTauri('add_inbox_item', {
					request: { content: captureContent, contentType: captureType, source: 'manual' },
				});
			}
			captureMsg = '✓ Added to inbox';
			captureContent = '';
			setTimeout(() => { captureMsg = ''; }, 2500);
			await load();
		} catch (e) {
			captureMsg = '✗ ' + String(e);
		} finally {
			capturing = false;
		}
	}

	async function markProcessed(id: string) {
		if (!isDesktopRuntime()) {
			items = items.map((i) => i.id === id ? { ...i, processed: true } : i);
			return;
		}
		try {
			await invokeTauri('mark_inbox_processed', { id });
			await load();
		} catch (e) {
			error = String(e);
		}
	}

	async function convertToTask(id: string) {
		converting = true;
		convertMsg = '';
		try {
			if (isDesktopRuntime()) {
				await invokeTauri('inbox_to_task', {
					inboxId: id,
					domain: convertDomain,
					energy: convertEnergy,
					estimateMinutes: convertEstimate,
					doDate: convertDate || null,
				});
			}
			convertMsg = '✓ Converted to task';
			convertId = '';
			setTimeout(() => { convertMsg = ''; }, 2500);
			await load();
		} catch (e) {
			convertMsg = '✗ ' + String(e);
		} finally {
			converting = false;
		}
	}

	const visibleItems = $derived(showProcessed ? items : items.filter((i) => !i.processed));
	const unprocessedCount = $derived(items.filter((i) => !i.processed).length);

	onMount(load);
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-start justify-between">
		<div>
			<h1 class="text-2xl font-bold text-white">📥 Capture Inbox</h1>
			<p class="mt-1 text-sm text-slate-400">Zero-friction capture for text, links, screenshots, and voice. Process later.</p>
		</div>
		<div class="flex items-center gap-3">
			{#if unprocessedCount > 0}
				<span class="rounded-full bg-red-500/20 px-3 py-1 text-sm font-bold text-red-300">{unprocessedCount} unprocessed</span>
			{/if}
			<button onclick={load} class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300 hover:bg-white/10">↻</button>
		</div>
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	<!-- Quick Capture -->
	<div class="rounded-2xl border border-cyan-400/15 bg-slate-900/60 p-5">
		<h2 class="mb-4 text-base font-bold text-white">⚡ Quick Capture</h2>
		<div class="flex flex-wrap gap-2 mb-3">
			{#each ['text', 'url', 'screenshot', 'file'] as t}
				<button
					onclick={() => (captureType = t)}
					class={`rounded-xl border px-3 py-1.5 text-sm font-medium transition ${
						captureType === t
							? 'border-cyan-400/40 bg-cyan-400/15 text-cyan-100'
							: 'border-white/10 text-slate-400 hover:border-white/20 hover:text-white'
					}`}
				>
					{typeIcons[t]} {t}
				</button>
			{/each}
		</div>
		<div class="flex gap-2">
			<textarea
				bind:value={captureContent}
				rows={2}
				placeholder={captureType === 'url' ? 'Paste URL here…' : captureType === 'screenshot' ? 'Describe screenshot or paste alt-text…' : 'Type or paste anything — process it later…'}
				class="flex-1 resize-none rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-400/10"
				onkeydown={(e) => { if (e.key === 'Enter' && e.ctrlKey) capture(); }}
			></textarea>
			<button
				onclick={capture}
				disabled={capturing || !captureContent.trim()}
				class="shrink-0 self-end rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-5 py-3 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400 disabled:opacity-40"
			>
				{capturing ? '…' : 'Capture'}
			</button>
		</div>
		{#if captureMsg}
			<p class="mt-2 text-sm {captureMsg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{captureMsg}</p>
		{/if}
		<p class="mt-2 text-[10px] text-slate-600">Ctrl+Enter to capture</p>
	</div>

	<!-- Inbox items -->
	<div>
		<div class="mb-3 flex items-center gap-3">
			<h2 class="text-base font-bold text-white">Inbox Items</h2>
			<label class="flex cursor-pointer items-center gap-2 text-xs text-slate-400">
				<input type="checkbox" bind:checked={showProcessed} onchange={load} class="h-3.5 w-3.5" />
				Show processed
			</label>
		</div>

		{#if loading}
			<div class="py-12 text-center text-slate-500">Loading…</div>
		{:else if visibleItems.length === 0}
			<div class="rounded-2xl border border-dashed border-white/10 py-16 text-center">
				<p class="text-4xl">📥</p>
				<p class="mt-3 text-slate-400">{showProcessed ? 'No items yet.' : 'Inbox is clear! Great work.'}</p>
			</div>
		{:else}
			<div class="space-y-2">
				{#each visibleItems as item}
					<div class={`group rounded-2xl border p-4 transition ${item.processed ? 'border-white/6 bg-slate-900/30 opacity-60' : 'border-white/8 bg-slate-900/50 hover:border-cyan-400/20'}`}>
						<div class="flex items-start gap-3">
							<span class="mt-0.5 shrink-0 text-xl">{typeIcons[item.contentType] ?? '📌'}</span>
							<div class="flex-1 min-w-0">
								<p class="text-sm text-white">{item.content}</p>
								<div class="mt-1.5 flex items-center gap-2 text-[10px] text-slate-600">
									<span>{item.source}</span>
									<span>•</span>
									<span>{new Date(item.createdAt).toLocaleString()}</span>
								</div>
							</div>

							{#if !item.processed}
								<div class="flex shrink-0 items-center gap-2 opacity-0 transition group-hover:opacity-100">
									<!-- Convert to task -->
									{#if convertId === item.id}
										<div class="flex items-center gap-1.5">
											<select bind:value={convertDomain} class="rounded-lg border border-white/10 bg-slate-800 px-2 py-1 text-xs text-white">
												{#each ['work', 'health', 'learning', 'personal', 'system'] as d}
													<option value={d}>{d}</option>
												{/each}
											</select>
											<select bind:value={convertEnergy} class="rounded-lg border border-white/10 bg-slate-800 px-2 py-1 text-xs text-white">
												<option value="low">low</option>
												<option value="medium">med</option>
												<option value="high">high</option>
											</select>
											<input type="number" bind:value={convertEstimate} min={5} max={480} step={5} class="w-14 rounded-lg border border-white/10 bg-slate-800 px-2 py-1 text-xs text-white" placeholder="30m" />
											<button
												onclick={() => convertToTask(item.id)}
												disabled={converting}
												class="rounded-lg bg-cyan-500/20 px-2 py-1 text-xs font-medium text-cyan-300 hover:bg-cyan-500/30"
											>→ Task</button>
											<button onclick={() => (convertId = '')} class="rounded-lg bg-slate-700 px-2 py-1 text-xs text-slate-400">✕</button>
										</div>
									{:else}
										<button
											onclick={() => { convertId = item.id; convertDate = new Date().toISOString().slice(0, 10); }}
											class="rounded-xl border border-blue-400/20 bg-blue-400/8 px-3 py-1.5 text-xs font-medium text-blue-300 hover:bg-blue-400/15"
										>→ Task</button>
									{/if}
									<button
										onclick={() => markProcessed(item.id)}
										class="rounded-xl border border-emerald-400/20 bg-emerald-400/8 px-3 py-1.5 text-xs font-medium text-emerald-300 hover:bg-emerald-400/15"
									>✓ Done</button>
								</div>
							{/if}
						</div>
						{#if convertMsg && convertId === item.id}
							<p class="mt-2 text-xs {convertMsg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{convertMsg}</p>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Voice placeholder (M7 stub) -->
	<div class="rounded-2xl border border-dashed border-violet-500/20 bg-violet-500/5 p-4 text-center">
		<p class="text-sm text-violet-400">🎙️ Voice Capture — Coming in Stage 2 Mobile Integration</p>
		<p class="mt-1 text-xs text-slate-600">Will be wired to Android APK mic for hands-free inbox capture.</p>
	</div>
</div>
