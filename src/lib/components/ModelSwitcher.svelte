<script lang="ts">
	import { onMount } from 'svelte';
	import {
		allModelsStore,
		activeProviderIdStore,
		activeModelStore,
		loadProviders,
		type Provider
	} from '$lib/utils/provider-registry';

	export let compact = false;
	export let onSelect: ((providerId: string, model: string) => void) | undefined = undefined;

	let open = false;
	let searchQuery = '';

	onMount(async () => {
		await loadProviders();
	});

	$: filteredModels = $allModelsStore.filter(
		(m) =>
			!searchQuery ||
			m.model.toLowerCase().includes(searchQuery.toLowerCase()) ||
			m.providerName.toLowerCase().includes(searchQuery.toLowerCase())
	);

	$: groupedModels = filteredModels.reduce(
		(acc, m) => {
			if (!acc[m.providerName]) acc[m.providerName] = [];
			acc[m.providerName].push(m);
			return acc;
		},
		{} as Record<string, typeof filteredModels>
	);

	$: currentLabel = (() => {
		const pid = $activeProviderIdStore;
		const model = $activeModelStore;
		if (!model) return 'Select Model';
		const entry = $allModelsStore.find((m) => m.providerId === pid && m.model === model);
		return entry ? `${entry.emoji} ${entry.model}` : model;
	})();

	function selectModel(providerId: string, model: string) {
		activeProviderIdStore.set(providerId);
		activeModelStore.set(model);
		open = false;
		onSelect?.(providerId, model);
	}

	function handleOutsideClick(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.model-switcher')) open = false;
	}
</script>

<svelte:window on:click={handleOutsideClick} />

<div class="model-switcher relative">
	<button
		type="button"
		on:click|stopPropagation={() => (open = !open)}
		class={compact
			? 'flex items-center gap-2 rounded-xl border border-white/10 bg-slate-900/80 px-3 py-2 text-sm text-white hover:border-cyan-400/30'
			: 'flex items-center gap-2 rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white hover:border-cyan-400/30 transition-colors'}
	>
		<span class="font-medium">{currentLabel}</span>
		<svg
			class="h-4 w-4 text-slate-400 transition-transform {open ? 'rotate-180' : ''}"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	{#if open}
		<div
			class="absolute left-0 z-50 mt-2 w-80 rounded-2xl border border-white/10 bg-slate-950 shadow-2xl shadow-black/50"
			on:click|stopPropagation
			role="none"
		>
			<div class="border-b border-white/10 p-3">
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search models..."
					class="w-full rounded-xl border border-white/10 bg-slate-900/80 px-3 py-2 text-sm text-white placeholder:text-slate-500 outline-none focus:border-cyan-400/40"
				/>
			</div>

			<div class="max-h-96 overflow-y-auto p-2">
				{#if Object.keys(groupedModels).length === 0}
					<div class="py-6 text-center text-sm text-slate-500">
						{#if $allModelsStore.length === 0}
							No providers enabled. Go to Settings → Providers to connect.
						{:else}
							No models match "{searchQuery}"
						{/if}
					</div>
				{:else}
					{#each Object.entries(groupedModels) as [providerName, models]}
						<div class="mb-2">
							<p class="px-3 py-1 text-xs font-semibold uppercase tracking-wider text-slate-500">
								{models[0]?.emoji ?? '🔌'}
								{providerName}
								{#if models[0]?.isLocal}
									<span class="ml-1 rounded-full bg-green-500/20 px-1.5 py-0.5 text-green-400 text-xs">local</span>
								{/if}
							</p>
							{#each models as entry}
								<button
									type="button"
									on:click={() => selectModel(entry.providerId, entry.model)}
									class="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition-colors
										{$activeProviderIdStore === entry.providerId && $activeModelStore === entry.model
										? 'bg-cyan-400/10 text-cyan-300'
										: 'text-slate-300 hover:bg-white/5'}"
								>
									<span class="flex-1 font-mono text-xs">{entry.model}</span>
									{#if $activeProviderIdStore === entry.providerId && $activeModelStore === entry.model}
										<svg class="h-4 w-4 text-cyan-400 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
											<path d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"/>
										</svg>
									{/if}
								</button>
							{/each}
						</div>
					{/each}
				{/if}
			</div>

			<div class="border-t border-white/10 p-3">
				<a
					href="/settings"
					on:click={() => (open = false)}
					class="block w-full rounded-xl border border-dashed border-white/10 py-2 text-center text-xs text-slate-500 hover:border-cyan-400/30 hover:text-cyan-400 transition-colors"
				>
					+ Connect more providers in Settings
				</a>
			</div>
		</div>
	{/if}
</div>
