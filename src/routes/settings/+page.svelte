<script lang="ts">
	import { onMount } from 'svelte';

	import {
		createEmptyKeyMap,
		KEY_FIELDS,
		loadKeys,
		saveKeys,
		type KeyField,
		type KeyMap
	} from '$lib/utils/secure-store';

	const labels: Record<KeyField, string> = {
		ANTHROPIC_API_KEY: 'Anthropic',
		OPENAI_API_KEY: 'OpenAI',
		GROK_API_KEY: 'Grok',
		GEMINI_API_KEY: 'Gemini',
		PERPLEXICA_KEYS: 'Perplexica'
	};

	let keys: KeyMap = createEmptyKeyMap();
	let loading = true;
	let saving = false;
	let status = 'Loading secure vault...';
	let error = '';

	onMount(async () => {
		try {
			keys = await loadKeys();
			status = 'Keys loaded from Stronghold.';
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Failed to load keys.';
			status = 'Vault unavailable.';
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
			status = 'Keys saved securely. Deploy flows can now inject them on confirmation.';
		} catch (saveError) {
			error = saveError instanceof Error ? saveError.message : 'Failed to save keys.';
			status = 'Save failed.';
		} finally {
			saving = false;
		}
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Settings</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Central API key manager.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Secrets will be stored via Tauri Stronghold, then injected into managed workspaces only
			after explicit confirmation.
		</p>
	</div>

	<div class="grid gap-4 xl:grid-cols-[1.3fr_0.8fr]">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 class="text-lg font-semibold text-white">Key inventory</h2>
					<p class="mt-2 text-sm text-slate-400">
						Saved into a Stronghold vault under RalphHub app data and only materialized into
						project `.env` files when you explicitly approve an injection step.
					</p>
				</div>
				<span class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-xs font-medium text-cyan-100">
					{loading ? 'Loading' : 'Ready'}
				</span>
			</div>

			<form class="mt-6 space-y-4" on:submit|preventDefault={handleSave}>
				{#each KEY_FIELDS as field}
					<div class="rounded-2xl border border-white/10 bg-slate-950/70 p-4">
						<label class="block text-sm font-medium text-white" for={field}>{field}</label>
						<p class="mt-2 text-xs uppercase tracking-[0.2em] text-slate-500">{labels[field]}</p>
						<input
							id={field}
							type="password"
							bind:value={keys[field]}
							disabled={loading || saving}
							autocomplete="off"
							placeholder="Paste key value"
							class="mt-4 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
						/>
					</div>
				{/each}

				<div class="flex flex-wrap items-center gap-3">
					<button
						type="submit"
						disabled={loading || saving}
						class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{saving ? 'Saving...' : 'Save to Stronghold'}
					</button>
					<p class="text-sm text-slate-400">{status}</p>
				</div>
			</form>
		</div>

		<div class="space-y-4">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Injection policy</h2>
				<ul class="mt-4 space-y-3 text-sm leading-6 text-slate-400">
					<li>Bun remains the only package manager for RalphHub and managed projects.</li>
					<li>Unknown repos will be flagged for sandbox review before launch.</li>
					<li>`Open in Code` is required after deploy and launch operations.</li>
				</ul>
			</div>

			<div class="rounded-3xl border border-rose-400/20 bg-rose-500/10 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Vault state</h2>
				<p class="mt-3 text-sm leading-6 text-slate-300">
					{#if error}
						{error}
					{:else}
						No secrets are stored in the repo. This screen is the central source of truth for managed deploys and tool runs.
					{/if}
				</p>
			</div>
		</div>
	</div>
</section>
