<script lang="ts">
	import { onMount } from 'svelte';

	import {
		createEmptyKeyMap,
		KEY_FIELDS,
		KEY_LABELS,
		loadKeys,
		saveKeys,
		type KeyField,
		type KeyMap
	} from '$lib/utils/secure-store';

	let keys: KeyMap = createEmptyKeyMap();
	let loading = true;
	let saving = false;
	let status = 'Loading secure vault...';
	let error = '';
	let revealed: Set<KeyField> = new Set();

	const providerGroups = [
		{
			label: 'AI Providers',
			color: 'cyan',
			fields: ['ANTHROPIC_API_KEY', 'OPENAI_API_KEY', 'GROK_API_KEY', 'GEMINI_API_KEY', 'GLM_API_KEY'] as KeyField[]
		},
		{
			label: 'Local Models',
			color: 'violet',
			fields: ['OLLAMA_API_KEY', 'OLLAMA_CLOUD_API_KEY'] as KeyField[]
		},
		{
			label: 'Integrations',
			color: 'amber',
			fields: ['NOTION_API_KEY', 'GITHUB_TOKEN', 'HF_TOKEN', 'PERPLEXICA_KEYS'] as KeyField[]
		}
	];

	onMount(async () => {
		try {
			keys = await loadKeys();
			status = 'Keys loaded from Stronghold.';
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Failed to load keys.';
			status = 'Vault unavailable — running in browser preview mode.';
		} finally {
			loading = false;
		}
	});

	async function handleSave() {
		saving = true;
		error = '';
		status = 'Saving keys to Stronghold...';

		try {
			await saveKeys(keys);
			status = 'All keys saved securely. Deploy flows and tools will auto-inject them.';
		} catch (saveError) {
			error = saveError instanceof Error ? saveError.message : 'Failed to save keys.';
			status = 'Save failed.';
		} finally {
			saving = false;
		}
	}

	function toggleReveal(field: KeyField) {
		if (revealed.has(field)) {
			revealed.delete(field);
		} else {
			revealed.add(field);
		}
		revealed = new Set(revealed);
	}

	function hasKey(field: KeyField) {
		return keys[field]?.trim().length > 0;
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Settings</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">API Key Manager — Stronghold-secured.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			All secrets stored in Tauri Stronghold (argon2 encrypted, never leaves your machine).
			Auto-injected into managed workspaces and tools on launch.
		</p>
	</div>

	<div class="grid gap-6 xl:grid-cols-[1.4fr_0.7fr]">
		<div class="space-y-6">
			{#each providerGroups as group}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
					<div class="flex items-center justify-between">
						<h2 class="text-lg font-semibold text-white">{group.label}</h2>
						<span class="text-xs text-slate-500">
							{group.fields.filter(hasKey).length}/{group.fields.length} configured
						</span>
					</div>

					<form class="mt-4 space-y-3" on:submit|preventDefault={handleSave}>
						{#each group.fields as field}
							{@const meta = KEY_LABELS[field]}
							<div class="rounded-2xl border border-white/8 bg-slate-950/60 p-4">
								<div class="flex items-center justify-between">
									<div>
										<label class="text-sm font-medium text-white" for={field}>{meta.label}</label>
										<p class="mt-0.5 text-xs text-slate-500">{meta.provider}</p>
									</div>
									<div class="flex items-center gap-2">
										{#if hasKey(field)}
											<span class="rounded-full bg-green-400/15 px-2 py-0.5 text-xs text-green-300">saved</span>
										{/if}
										<a
											href={meta.url}
											target="_blank"
											rel="noopener noreferrer"
											class="rounded-full border border-white/10 px-2 py-0.5 text-xs text-slate-400 hover:text-white"
										>
											Get key ↗
										</a>
									</div>
								</div>
								<div class="relative mt-3">
									<input
										id={field}
										type={revealed.has(field) ? 'text' : 'password'}
										bind:value={keys[field]}
										disabled={loading || saving}
										autocomplete="off"
										placeholder="Paste key value..."
										class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 pr-16 text-sm text-white outline-none placeholder:text-slate-600 disabled:opacity-50"
									/>
									<button
										type="button"
										on:click={() => toggleReveal(field)}
										class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-slate-500 hover:text-white"
									>
										{revealed.has(field) ? 'Hide' : 'Show'}
									</button>
								</div>
							</div>
						{/each}
					</form>
				</div>
			{/each}

			<div class="flex flex-wrap items-center gap-3">
				<button
					on:click={handleSave}
					disabled={loading || saving}
					class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-6 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:cursor-not-allowed disabled:opacity-60"
				>
					{saving ? 'Saving...' : 'Save all keys to Stronghold'}
				</button>
				<p class="text-sm {error ? 'text-red-400' : 'text-slate-400'}">{status}</p>
			</div>
		</div>

		<div class="space-y-4">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Key summary</h2>
				<div class="mt-4 space-y-2">
					{#each KEY_FIELDS as field}
						<div class="flex items-center justify-between">
							<span class="text-xs text-slate-400">{KEY_LABELS[field].provider}</span>
							<span class="text-xs {hasKey(field) ? 'text-green-400' : 'text-slate-600'}">
								{hasKey(field) ? '✓ saved' : '— empty'}
							</span>
						</div>
					{/each}
				</div>
			</div>

			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Injection policy</h2>
				<ul class="mt-4 space-y-2 text-sm leading-6 text-slate-400 list-disc list-inside">
					<li>Keys never leave your machine (Stronghold vault)</li>
					<li>Injected into .env files only on explicit approval</li>
					<li>Ollama local endpoint overrides remote keys</li>
					<li>All tools auto-detect which keys they need</li>
				</ul>
			</div>

			<div class="rounded-3xl border border-violet-400/20 bg-violet-500/8 p-6 backdrop-blur">
				<h2 class="text-sm font-semibold text-violet-200">Local-first model priority</h2>
				<p class="mt-2 text-xs leading-5 text-slate-400">
					When Ollama is running locally, all tools route requests through it by default.
					Remote API keys are fallback. Configure in the Ollama tab.
				</p>
			</div>
		</div>
	</div>
</section>
