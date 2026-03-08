<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface NotionSyncResult {
		pushed: number;
		pulled: number;
		errors: string[];
		syncedAt: string;
	}

	let apiKey = $state('');
	let databaseId = $state('');
	let direction = $state('push');
	let syncing = $state(false);
	let result = $state<NotionSyncResult | null>(null);
	let cursorUrl = $state('');
	let openingCursor = $state(false);
	let cursorResult = $state('');
	let workflowId = $state('');

	async function sync() {
		if (!apiKey.trim() || !databaseId.trim()) return;
		syncing = true;
		result = null;
		try {
			if (isDesktopRuntime()) {
				result = await invokeTauri<NotionSyncResult>('sync_notion', {
					request: { apiKey, databaseId, direction },
				});
			} else {
				result = { pushed: 0, pulled: 0, errors: ['Desktop runtime not available in browser preview.'], syncedAt: new Date().toISOString() };
			}
		} catch (e) {
			result = { pushed: 0, pulled: 0, errors: [String(e)], syncedAt: new Date().toISOString() };
		} finally {
			syncing = false;
		}
	}

	async function openInCursorWeb() {
		openingCursor = true;
		cursorResult = '';
		try {
			if (isDesktopRuntime()) {
				const url = await invokeTauri<string>('open_in_cursor_agent_web', {
					workflowId: workflowId || null,
					memoryIds: null,
				});
				cursorUrl = url;
				cursorResult = '✓ URL generated — click below to launch';
			} else {
				cursorUrl = 'https://cursor.com/agents/new?context=AmitOS+context+pack+preview';
				cursorResult = '✓ Preview URL (desktop runtime needed for full context)';
			}
		} catch (e) {
			cursorResult = '✗ ' + String(e);
		} finally {
			openingCursor = false;
		}
	}
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-2xl font-bold text-white">🔲 Notion Sync + Cursor Agent Web</h1>
		<p class="mt-1 text-sm text-slate-400">Push AmitOS tasks and memory to Notion. Launch workflows in Cursor Agent Web with full Memory Spine context.</p>
	</div>

	<div class="grid gap-5 lg:grid-cols-2">
		<!-- Notion Sync -->
		<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-6">
			<h2 class="mb-5 text-lg font-bold text-white">🔲 Notion Sync</h2>

			<div class="space-y-4">
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Notion API Key</label>
					<input
						bind:value={apiKey}
						type="password"
						placeholder="secret_..."
						class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40"
					/>
					<p class="mt-1 text-[10px] text-slate-600">Or configure in Settings → Stronghold vault</p>
				</div>

				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Notion Database ID</label>
					<input
						bind:value={databaseId}
						type="text"
						placeholder="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
						class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40"
					/>
				</div>

				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Direction</label>
					<div class="flex gap-2">
						{#each ['push', 'pull', 'both'] as d}
							<button
								onclick={() => (direction = d)}
								class={`flex-1 rounded-xl border py-2.5 text-sm font-medium transition ${
									direction === d
										? 'border-cyan-400/40 bg-cyan-400/15 text-cyan-100'
										: 'border-white/10 text-slate-400 hover:border-white/20 hover:text-white'
								}`}
							>
								{d === 'push' ? '↑ Push' : d === 'pull' ? '↓ Pull' : '↕ Both'}
							</button>
						{/each}
					</div>
				</div>

				<button
					onclick={sync}
					disabled={syncing || !apiKey.trim() || !databaseId.trim()}
					class="w-full rounded-xl bg-gradient-to-r from-slate-600 to-slate-500 py-3 text-sm font-bold text-white transition hover:from-slate-500 disabled:opacity-40"
				>
					{syncing ? 'Syncing…' : '🔲 Sync with Notion'}
				</button>

				{#if result}
					<div class={`rounded-xl border p-4 ${result.errors.length === 0 ? 'border-emerald-400/20 bg-emerald-400/8' : 'border-amber-400/20 bg-amber-400/8'}`}>
						<div class="grid grid-cols-2 gap-3 mb-3">
							<div class="text-center">
								<p class="text-lg font-bold text-white">{result.pushed}</p>
								<p class="text-[10px] text-slate-500">Pushed</p>
							</div>
							<div class="text-center">
								<p class="text-lg font-bold text-white">{result.pulled}</p>
								<p class="text-[10px] text-slate-500">Pulled</p>
							</div>
						</div>
						{#if result.errors.length > 0}
							{#each result.errors as err}
								<p class="text-xs text-amber-300">{err}</p>
							{/each}
						{/if}
						<p class="mt-2 text-[10px] text-slate-600">Synced at {new Date(result.syncedAt).toLocaleString()}</p>
					</div>
				{/if}
			</div>

			<!-- Roadmap note -->
			<div class="mt-5 rounded-xl border border-dashed border-white/10 p-4">
				<p class="text-xs font-semibold text-slate-400">Notion HTTP Integration Roadmap</p>
				<ul class="mt-2 space-y-1 text-[11px] text-slate-500">
					<li>• Add <code class="text-slate-400">reqwest</code> HTTP client to Cargo.toml</li>
					<li>• Implement <code class="text-slate-400">notion_push_tasks()</code> mapping KaizenTask → Notion page</li>
					<li>• Implement <code class="text-slate-400">notion_pull_tasks()</code> mapping Notion rows → KaizenTask</li>
					<li>• Add Notion API key to Stronghold vault in Settings</li>
				</ul>
			</div>
		</div>

		<!-- Cursor Agent Web -->
		<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-6">
			<h2 class="mb-5 text-lg font-bold text-white">🖱️ Open in Cursor Agent Web</h2>
			<p class="mb-4 text-sm text-slate-400">Launch the current workflow in Cursor's cloud agent mode with the full AmitOS Memory Spine as context.</p>

			<div class="space-y-4">
				<div>
					<label class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Workflow ID (optional)</label>
					<input
						bind:value={workflowId}
						type="text"
						placeholder="Leave blank to use latest"
						class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40"
					/>
				</div>

				<button
					onclick={openInCursorWeb}
					disabled={openingCursor}
					class="w-full rounded-xl bg-gradient-to-r from-violet-600 to-violet-500 py-3 text-sm font-bold text-white shadow-lg shadow-violet-500/20 transition hover:from-violet-500 disabled:opacity-40"
				>
					{openingCursor ? 'Generating…' : '🖱️ Open in Cursor Agent Web'}
				</button>

				{#if cursorResult}
					<p class="text-sm {cursorResult.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{cursorResult}</p>
				{/if}

				{#if cursorUrl}
					<div class="rounded-xl border border-violet-400/20 bg-violet-400/8 p-4">
						<p class="mb-2 text-xs font-semibold text-violet-300">Generated URL with Memory Context:</p>
						<a href={cursorUrl} target="_blank" class="block break-all text-xs text-violet-400 underline hover:text-violet-300">{cursorUrl.slice(0, 120)}…</a>
						<p class="mt-2 text-[10px] text-slate-600">Opens Cursor Agent Web with AmitOS working memory + recent events as context</p>
					</div>
				{/if}
			</div>

			<!-- Context pack explanation -->
			<div class="mt-5 rounded-xl border border-violet-400/15 bg-violet-400/5 p-4">
				<p class="text-xs font-semibold text-violet-300">What's in the Context Pack?</p>
				<ul class="mt-2 space-y-1 text-[11px] text-slate-400">
					<li>• Last 5 Working Memory items</li>
					<li>• Last 3 Raw Events (with source type)</li>
					<li>• Active workflow ID (if specified)</li>
					<li>• URL-encoded and passed as <code class="text-slate-300">?context=</code></li>
				</ul>
			</div>
		</div>
	</div>
</div>
