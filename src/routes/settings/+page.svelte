<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import {
		createEmptyKeyMap,
		KEY_FIELDS,
		loadKeys,
		saveKeys,
		type KeyField,
		type KeyMap
	} from '$lib/utils/secure-store';

	const keyLabels: Record<KeyField, string> = {
		ANTHROPIC_API_KEY: 'Anthropic',
		OPENAI_API_KEY: 'OpenAI',
		GROK_API_KEY: 'Grok',
		GEMINI_API_KEY: 'Gemini',
		PERPLEXICA_KEYS: 'Perplexica'
	};

	// ── API keys ──────────────────────────────────────────────────────────────
	let keys: KeyMap = createEmptyKeyMap();
	let keysLoading = true;
	let keysSaving = false;
	let keysStatus = 'Loading secure vault...';
	let keysError = '';

	// ── Browser settings ──────────────────────────────────────────────────────
	type BrowserSettings = { preferredBrowser: string; agentMode: string };

	let browser: BrowserSettings = { preferredBrowser: 'system', agentMode: 'permission' };
	let browserSaving = false;
	let browserStatus = '';
	let browserError = '';

	const browserOptions = [
		{ value: 'system', label: 'System default' },
		{ value: 'edge', label: 'Microsoft Edge (with profile)' },
		{ value: 'chrome', label: 'Google Chrome' },
		{ value: 'firefox', label: 'Firefox' }
	];

	onMount(async () => {
		if (!isDesktopRuntime()) {
			keysLoading = false;
			return;
		}

		try {
			keys = await loadKeys();
			keysStatus = 'Keys loaded from Stronghold.';
		} catch (err) {
			keysError = err instanceof Error ? err.message : 'Failed to load keys.';
			keysStatus = 'Vault unavailable.';
		} finally {
			keysLoading = false;
		}

		try {
			browser = await invokeTauri<BrowserSettings>('get_browser_settings');
		} catch {
			// leave defaults
		}
	});

	async function saveApiKeys() {
		keysSaving = true;
		keysError = '';
		keysStatus = 'Saving keys to Stronghold...';
		try {
			await saveKeys(keys);
			keysStatus = 'Keys saved. Deploy flows can inject them on confirmation.';
		} catch (err) {
			keysError = err instanceof Error ? err.message : 'Failed to save keys.';
			keysStatus = 'Save failed.';
		} finally {
			keysSaving = false;
		}
	}

	async function saveBrowserSettings() {
		if (!isDesktopRuntime()) return;
		browserSaving = true;
		browserError = '';
		try {
			const res = await invokeTauri<{ ok: boolean; message: string }>('save_browser_settings', {
				settings: browser
			});
			browserStatus = res.message;
		} catch (err) {
			browserError = err instanceof Error ? err.message : 'Failed to save browser settings.';
		} finally {
			browserSaving = false;
		}
	}
</script>

<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Settings</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Central key manager.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			API keys are stored in Tauri Stronghold and never leave the vault unless you explicitly
			approve an injection. Browser preferences control how agents interact with the web.
		</p>
	</div>

	<!-- API keys + injection policy -->
	<div class="grid gap-4 xl:grid-cols-[1.3fr_0.8fr]">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 class="text-lg font-semibold text-white">Key inventory</h2>
					<p class="mt-2 text-sm text-slate-400">
						Stored in a Stronghold vault and only materialized into project
						<code class="rounded bg-slate-800/80 px-1">.env</code> files when you explicitly approve
						an injection step.
					</p>
				</div>
				<span
					class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-xs font-medium text-cyan-100"
				>
					{keysLoading ? 'Loading' : 'Ready'}
				</span>
			</div>

			<form class="mt-6 space-y-4" on:submit|preventDefault={saveApiKeys}>
				{#each KEY_FIELDS as field}
					<div class="rounded-2xl border border-white/10 bg-slate-950/70 p-4">
						<label class="block text-sm font-medium text-white" for={field}>{field}</label>
						<p class="mt-2 text-xs uppercase tracking-[0.2em] text-slate-500">{keyLabels[field]}</p>
						<input
							id={field}
							type="password"
							bind:value={keys[field]}
							disabled={keysLoading || keysSaving}
							autocomplete="off"
							placeholder="Paste key value"
							class="mt-4 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
						/>
					</div>
				{/each}

				<div class="flex flex-wrap items-center gap-3">
					<button
						type="submit"
						disabled={keysLoading || keysSaving}
						class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{keysSaving ? 'Saving...' : 'Save to Stronghold'}
					</button>
					<p class="text-sm text-slate-400">{keysStatus}</p>
				</div>
			</form>
		</div>

		<div class="space-y-4">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Injection policy</h2>
				<ul class="mt-4 space-y-3 text-sm leading-6 text-slate-400">
					<li>Bun is the only package manager for RalphHub and managed projects.</li>
					<li>Unknown repos are flagged for sandbox review before launch.</li>
					<li><code class="rounded bg-slate-800/80 px-1">Open in Code</code> is required after deploy and launch operations.</li>
				</ul>
			</div>

			<div class="rounded-3xl border border-rose-400/20 bg-rose-500/10 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Vault state</h2>
				<p class="mt-3 text-sm leading-6 text-slate-300">
					{#if keysError}
						{keysError}
					{:else}
						No secrets are stored in the repo. This screen is the sole source of truth for managed
						deploys and tool runs.
					{/if}
				</p>
			</div>
		</div>
	</div>

	<!-- Browser settings -->
	<div class="rounded-3xl border border-cyan-400/20 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-start justify-between gap-4">
			<div>
				<h2 class="text-lg font-semibold text-white">Browser Agent preferences</h2>
				<p class="mt-2 text-sm text-slate-400">
					Controls how the Browser Agent and Ralph loop workflows open URLs and perform web
					automation. Edge is preferred for persistent-profile access (your real session, cookies,
					extensions).
				</p>
			</div>
			<a
				href="/browser-agent"
				class="shrink-0 rounded-full border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-xs font-medium text-cyan-100"
			>
				Open Browser Agent →
			</a>
		</div>

		<div class="mt-6 grid gap-6 lg:grid-cols-2">
			<!-- Preferred browser -->
			<div>
				<label class="block text-sm font-medium text-white" for="preferred-browser">
					Preferred browser
				</label>
				<p class="mt-1 text-xs text-slate-500">
					When Edge is selected, agents launch with <code class="rounded bg-slate-800/60 px-1">--user-data-dir</code>
					pointing at your real Edge profile.
				</p>
				<select
					id="preferred-browser"
					bind:value={browser.preferredBrowser}
					class="mt-3 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
				>
					{#each browserOptions as opt}
						<option value={opt.value}>{opt.label}</option>
					{/each}
				</select>
			</div>

			<!-- Agent mode -->
			<div>
				<p class="block text-sm font-medium text-white">Agent mode</p>
				<p class="mt-1 text-xs text-slate-500">
					Permission mode (default) shows an approval modal for every action. Autonomous mode runs
					unsupervised — a kill switch is always visible in the Browser Agent tab.
				</p>
				<div class="mt-3 space-y-2">
					<label
						class={`flex cursor-pointer items-center gap-3 rounded-2xl border p-4 text-sm transition ${
							browser.agentMode === 'permission'
								? 'border-cyan-400/30 bg-cyan-400/8 text-cyan-100'
								: 'border-white/10 bg-white/3 text-slate-300'
						}`}
					>
						<input type="radio" bind:group={browser.agentMode} value="permission" />
						<div>
							<span class="font-medium">Permission</span>
							<span class="ml-2 text-xs text-slate-500">— approve every action (default)</span>
						</div>
					</label>
					<label
						class={`flex cursor-pointer items-center gap-3 rounded-2xl border p-4 text-sm transition ${
							browser.agentMode === 'autonomous'
								? 'border-rose-400/30 bg-rose-500/8 text-rose-100'
								: 'border-white/10 bg-white/3 text-slate-300'
						}`}
					>
						<input type="radio" bind:group={browser.agentMode} value="autonomous" />
						<div>
							<span class="font-medium">Autonomous</span>
							<span class="ml-2 text-xs text-slate-500">— unsupervised, kill switch available</span>
						</div>
					</label>
				</div>
			</div>
		</div>

		<div class="mt-6 flex flex-wrap items-center gap-3">
			<button
				type="button"
				on:click={saveBrowserSettings}
				disabled={browserSaving}
				class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:opacity-60"
			>
				{browserSaving ? 'Saving...' : 'Save browser preferences'}
			</button>
			{#if browserStatus}
				<p class="text-sm text-slate-300">{browserStatus}</p>
			{/if}
			{#if browserError}
				<p class="text-sm text-rose-300">{browserError}</p>
			{/if}
		</div>
	</div>
</section>
