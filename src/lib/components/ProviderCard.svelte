<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { toggleProvider, type Provider, CATEGORY_LABELS } from '$lib/utils/provider-registry';
	import { saveDynamicKey, hasKey } from '$lib/utils/secure-store';

	export let provider: Provider;
	export let keyValue: string = '';
	export const showKeyInput = false;

	const dispatch = createEventDispatcher<{
		updated: Provider;
		deleted: string;
		test: Provider;
	}>();

	let toggling = false;
	let saving = false;
	let testing = false;
	let testResult = '';
	let testOk = false;
	let expanded = false;

	async function handleToggle() {
		toggling = true;
		try {
			const updated = await toggleProvider(provider.id, !provider.enabled);
			dispatch('updated', updated);
		} finally {
			toggling = false;
		}
	}

	async function handleSaveKey() {
		if (!keyValue.trim()) return;
		saving = true;
		try {
			await saveDynamicKey(provider.apiKeyEnv, keyValue);
			if (!provider.enabled) {
				const updated = await toggleProvider(provider.id, true);
				dispatch('updated', updated);
			}
		} finally {
			saving = false;
		}
	}

	async function handleTest() {
		if (!provider.baseUrl || provider.isLocal) return;
		testing = true;
		testResult = '';
		try {
			dispatch('test', provider);
			testOk = true;
			testResult = 'Connection test queued.';
		} catch (e) {
			testOk = false;
			testResult = e instanceof Error ? e.message : 'Test failed';
		} finally {
			testing = false;
		}
	}

	$: categoryLabel = CATEGORY_LABELS[provider.category] ?? provider.category;
</script>

<div
	class="rounded-2xl border transition-colors {provider.enabled
		? 'border-cyan-400/20 bg-slate-950/60'
		: 'border-white/8 bg-slate-950/30'} p-4"
>
	<div class="flex items-start gap-3">
		<span class="text-2xl leading-none pt-0.5 flex-shrink-0">{provider.logoEmoji}</span>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2 flex-wrap">
				<h3 class="text-sm font-semibold text-white truncate">{provider.name}</h3>
				<span class="rounded-full bg-slate-800 px-2 py-0.5 text-xs text-slate-400">
					{categoryLabel}
				</span>
				{#if provider.isLocal}
					<span class="rounded-full bg-green-500/20 px-2 py-0.5 text-xs text-green-400">local</span>
				{/if}
				{#if provider.enabled}
					<span class="rounded-full bg-cyan-400/15 px-2 py-0.5 text-xs text-cyan-300">active</span>
				{/if}
			</div>
			<p class="mt-1 text-xs text-slate-500 leading-relaxed line-clamp-1">{provider.description}</p>

			{#if expanded}
				<div class="mt-3 space-y-2">
					<div class="flex flex-wrap gap-1">
						{#each provider.models.slice(0, 6) as model}
							<span class="rounded-lg bg-slate-800/80 px-2 py-0.5 text-xs font-mono text-slate-300">
								{model}
							</span>
						{/each}
						{#if provider.models.length > 6}
							<span class="rounded-lg bg-slate-800/80 px-2 py-0.5 text-xs text-slate-500">
								+{provider.models.length - 6} more
							</span>
						{/if}
					</div>

					<div class="rounded-xl bg-slate-900/60 px-3 py-2">
						<p class="text-xs text-slate-500 font-mono break-all">{provider.baseUrl}</p>
					</div>

					{#if !provider.isLocal && provider.apiKeyEnv}
						<div class="flex gap-2">
							<input
								type="password"
								bind:value={keyValue}
								placeholder="Paste {provider.apiKeyEnv} value..."
								class="flex-1 rounded-xl border border-white/10 bg-slate-900/80 px-3 py-2 text-xs text-white placeholder:text-slate-600 outline-none focus:border-cyan-400/40"
								autocomplete="off"
							/>
							<button
								type="button"
								on:click={handleSaveKey}
								disabled={saving || !keyValue.trim()}
								class="rounded-xl bg-cyan-400/10 border border-cyan-400/20 px-3 py-2 text-xs text-cyan-300 hover:bg-cyan-400/20 disabled:opacity-50 transition-colors"
							>
								{saving ? '...' : 'Save'}
							</button>
						</div>
					{/if}

					{#if testResult}
						<p class="text-xs {testOk ? 'text-green-400' : 'text-rose-400'}">{testResult}</p>
					{/if}

					{#if provider.docsUrl}
						<a
							href={provider.docsUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="text-xs text-cyan-400/70 hover:text-cyan-400 underline"
						>
							Documentation →
						</a>
					{/if}
				</div>
			{/if}
		</div>

		<div class="flex items-center gap-2 flex-shrink-0">
			<button
				type="button"
				on:click={() => (expanded = !expanded)}
				class="rounded-xl border border-white/10 p-1.5 text-slate-500 hover:text-white hover:border-white/20 transition-colors"
				title={expanded ? 'Collapse' : 'Expand'}
			>
				<svg class="h-4 w-4 transition-transform {expanded ? 'rotate-180' : ''}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>

			<!-- Toggle switch -->
			<button
				type="button"
				on:click={handleToggle}
				disabled={toggling}
				aria-label="{provider.enabled ? 'Disable' : 'Enable'} {provider.name}"
				class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 disabled:opacity-60
					{provider.enabled ? 'bg-cyan-400' : 'bg-slate-700'}"
				role="switch"
				aria-checked={provider.enabled}
			>
				<span
					class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out
						{provider.enabled ? 'translate-x-4' : 'translate-x-0'}"
				></span>
			</button>
		</div>
	</div>
</div>
