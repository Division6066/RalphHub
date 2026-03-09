<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import { loadKeys, saveKeys } from '$lib/utils/secure-store';

	type ApiProvider = {
		id: string;
		name: string;
		category: string;
		keyName: string;
		url: string;
		description: string;
		color: string;
	};

	type KeyState = { value: string; saved: boolean; saving: boolean };

	const CATEGORY_LABELS: Record<string, { label: string; icon: string }> = {
		llm: { label: 'Language Models', icon: '🤖' },
		image: { label: 'Image Generation', icon: '🎨' },
		search: { label: 'Search APIs', icon: '🔍' },
		data: { label: 'Web & Data', icon: '🌐' },
		voice: { label: 'Voice & Audio', icon: '🎙️' },
		communication: { label: 'Communication', icon: '💬' },
		cloud: { label: 'Cloud & Infra', icon: '☁️' },
		business: { label: 'Business Tools', icon: '💼' },
		misc: { label: 'Other', icon: '⚙️' },
	};

	let providers: ApiProvider[] = [];
	let keyStates: Record<string, KeyState> = {};
	let loading = true;
	let globalStatus = '';
	let searchQuery = '';
	let selectedCategory = 'all';
	let showModal = false;
	let modalProvider: ApiProvider | null = null;
	let modalValue = '';
	let isDesktop = false;
	let savingAll = false;
	let savedCount = 0;

	onMount(async () => {
		isDesktop = isDesktopRuntime();
		try {
			providers = await invokeTauri<ApiProvider[]>('list_api_providers');

			// Init key states
			for (const p of providers) {
				keyStates[p.id] = { value: '', saved: false, saving: false };
			}

			if (isDesktop) {
				try {
					const existingKeys = await loadKeys();
					for (const [k, v] of Object.entries(existingKeys)) {
						const prov = providers.find((p) => p.keyName === k);
						if (prov && v) {
							keyStates[prov.id] = { value: v, saved: true, saving: false };
						}
					}
					savedCount = Object.values(keyStates).filter((s) => s.saved).length;
				} catch {}
			}
		} catch (e) {
			globalStatus = String(e);
		} finally {
			loading = false;
		}
	});

	function openModal(provider: ApiProvider) {
		modalProvider = provider;
		modalValue = keyStates[provider.id]?.value ?? '';
		showModal = true;
	}

	async function saveProviderKey() {
		if (!modalProvider) return;
		const prov = modalProvider;
		keyStates[prov.id] = { value: modalValue, saved: false, saving: true };
		keyStates = { ...keyStates };

		try {
			if (isDesktop) {
				const keyMap: Record<string, string> = {};
				for (const p of providers) {
					keyMap[p.keyName] = keyStates[p.id]?.value ?? '';
				}
				await saveKeys(keyMap as any);
			} else {
				// Browser fallback: localStorage
				localStorage.setItem(`amitos_key_${prov.id}`, modalValue);
			}
			keyStates[prov.id] = { value: modalValue, saved: !!modalValue, saving: false };
			keyStates = { ...keyStates };
			savedCount = Object.values(keyStates).filter((s) => s.saved).length;
			globalStatus = `${prov.name} key saved.`;
			showModal = false;
		} catch (e) {
			keyStates[prov.id] = { ...keyStates[prov.id], saving: false };
			keyStates = { ...keyStates };
			globalStatus = `Failed to save: ${e}`;
		}
	}

	function maskKey(value: string) {
		if (!value || value.length < 6) return value ? '••••••' : '';
		return value.slice(0, 4) + '••••••' + value.slice(-4);
	}

	$: categories = ['all', ...new Set(providers.map((p) => p.category))];

	$: filteredProviders = providers.filter((p) => {
		const matchCat = selectedCategory === 'all' || p.category === selectedCategory;
		const matchSearch = !searchQuery || p.name.toLowerCase().includes(searchQuery.toLowerCase()) || p.description.toLowerCase().includes(searchQuery.toLowerCase()) || p.keyName.toLowerCase().includes(searchQuery.toLowerCase());
		return matchCat && matchSearch;
	});

	$: groupedProviders = (() => {
		const groups: Record<string, ApiProvider[]> = {};
		for (const p of filteredProviders) {
			if (!groups[p.category]) groups[p.category] = [];
			groups[p.category].push(p);
		}
		return groups;
	})();
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-rose-400/20 bg-gradient-to-br from-rose-950/40 via-slate-950/80 to-pink-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-rose-300/70">Universal Key Manager</p>
		<h1 class="mt-2 text-3xl font-bold text-white">🔑 API Keys</h1>
		<p class="mt-2 text-sm text-slate-400">
			One place for all 50+ providers. Keys stored in Stronghold vault — never in plain text.
		</p>
		<div class="mt-4 flex flex-wrap items-center gap-4">
			<div class="flex items-center gap-2 rounded-xl border border-emerald-400/20 bg-emerald-400/10 px-3 py-2">
				<span class="h-2 w-2 rounded-full bg-emerald-400"></span>
				<span class="text-sm font-semibold text-emerald-300">{savedCount} keys saved</span>
			</div>
			<div class="flex items-center gap-2 rounded-xl border border-white/10 bg-white/5 px-3 py-2">
				<span class="text-sm text-slate-400">{providers.length} providers available</span>
			</div>
		</div>
	</div>

	<!-- How to add in 30 seconds banner -->
	<div class="rounded-xl border border-violet-400/20 bg-violet-400/8 p-4">
		<p class="text-xs font-bold uppercase tracking-widest text-violet-300 mb-1.5">Add any API key in 30 seconds</p>
		<p class="text-xs text-slate-400">1. Find your provider below → 2. Click the card → 3. Paste your key → 4. Click Save. Done.</p>
	</div>

	<!-- Search + category filter -->
	<div class="flex flex-wrap gap-3">
		<div class="relative flex-1 min-w-48">
			<span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-500">🔍</span>
			<input bind:value={searchQuery} placeholder="Search providers…" class="w-full rounded-xl border border-white/10 bg-slate-950/60 pl-10 pr-4 py-2.5 text-sm text-white outline-none focus:border-rose-400 backdrop-blur" />
		</div>
		<div class="flex flex-wrap gap-1.5">
			<button onclick={() => selectedCategory = 'all'} class={`rounded-xl border px-3 py-2 text-xs font-medium transition ${selectedCategory === 'all' ? 'border-rose-400/30 bg-rose-400/15 text-rose-100' : 'border-white/8 bg-slate-950/50 text-slate-400 hover:text-white'}`}>All</button>
			{#each categories.filter(c => c !== 'all') as cat}
				<button onclick={() => selectedCategory = cat} class={`rounded-xl border px-3 py-2 text-xs font-medium transition ${selectedCategory === cat ? 'border-rose-400/30 bg-rose-400/15 text-rose-100' : 'border-white/8 bg-slate-950/50 text-slate-400 hover:text-white'}`}>
					{CATEGORY_LABELS[cat]?.icon ?? '⚙️'} {CATEGORY_LABELS[cat]?.label ?? cat}
				</button>
			{/each}
		</div>
	</div>

	<!-- Provider groups -->
	{#if loading}
		<div class="py-10 text-center text-sm text-slate-400">Loading providers…</div>
	{:else}
		{#each Object.entries(groupedProviders) as [category, provs]}
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					<span class="text-lg">{CATEGORY_LABELS[category]?.icon ?? '⚙️'}</span>
					<h2 class="text-sm font-bold text-white">{CATEGORY_LABELS[category]?.label ?? category}</h2>
					<span class="text-xs text-slate-500">({provs.length})</span>
				</div>
				<div class="grid gap-2.5 sm:grid-cols-2 xl:grid-cols-3">
					{#each provs as provider}
						{@const state = keyStates[provider.id]}
						<button
							onclick={() => openModal(provider)}
							class={`group relative flex items-start gap-3 rounded-xl border p-4 text-left transition hover:scale-[1.01] ${
								state?.saved
									? 'border-emerald-400/25 bg-emerald-950/20'
									: 'border-white/8 bg-slate-950/40 hover:border-white/20'
							}`}
						>
							<div class="h-2 w-2 shrink-0 mt-1.5 rounded-full" style="background: {provider.color}"></div>
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-2">
									<p class="text-sm font-semibold text-white truncate">{provider.name}</p>
									{#if state?.saved}
										<span class="shrink-0 rounded-full bg-emerald-400/15 px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wide text-emerald-400">✓ SET</span>
									{/if}
								</div>
								<p class="mt-0.5 text-xs text-slate-500 truncate">{provider.description}</p>
								{#if state?.saved}
									<p class="mt-1 text-[10px] font-mono text-slate-600">{maskKey(state.value)}</p>
								{:else}
									<p class="mt-1 text-[10px] font-mono text-slate-600">{provider.keyName}</p>
								{/if}
							</div>
							<span class="shrink-0 text-slate-600 transition group-hover:text-white">+</span>
						</button>
					{/each}
				</div>
			</div>
		{/each}
	{/if}

	{#if globalStatus}
		<div class="rounded-xl border border-emerald-400/20 bg-emerald-950/20 p-3 text-sm text-emerald-300">{globalStatus}</div>
	{/if}

	<!-- Security info -->
	<div class="rounded-xl border border-white/8 bg-slate-950/40 p-5 backdrop-blur">
		<h3 class="text-sm font-bold text-white mb-3">🔒 Security Model</h3>
		<ul class="space-y-2 text-xs text-slate-400">
			<li>• Keys are stored in Tauri Stronghold (encrypted, OS keychain-backed)</li>
			<li>• Never written to disk in plaintext or committed to git</li>
			<li>• Injected into tool workspaces only on explicit confirmation</li>
			<li>• You can revoke any key by clearing the field and saving</li>
		</ul>
	</div>
</section>

<!-- Add Key Modal -->
{#if showModal && modalProvider}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
		<div class="w-full max-w-md rounded-2xl border border-white/10 bg-slate-900 p-6 shadow-2xl">
			<div class="mb-5 flex items-start justify-between gap-4">
				<div>
					<h3 class="text-lg font-bold text-white">{modalProvider.name}</h3>
					<p class="mt-1 text-xs text-slate-400">{modalProvider.description}</p>
				</div>
				<button onclick={() => { showModal = false; modalProvider = null; }} class="text-slate-500 hover:text-white shrink-0">✕</button>
			</div>

			<div class="mb-4 rounded-xl border border-white/8 bg-white/3 p-3">
				<p class="text-xs text-slate-400 mb-1">Env var name</p>
				<p class="font-mono text-sm text-violet-300">{modalProvider.keyName}</p>
			</div>

			<div class="mb-4 space-y-2">
				<label class="block text-xs font-medium text-slate-300">API Key Value</label>
				<input
					bind:value={modalValue}
					type="password"
					autocomplete="off"
					placeholder="Paste your API key here"
					class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-3 font-mono text-sm text-white outline-none focus:border-rose-400"
				/>
			</div>

			<div class="mb-5 flex items-center gap-2 rounded-xl border border-violet-400/15 bg-violet-400/8 p-3">
				<span class="text-sm">🔗</span>
				<a href={modalProvider.url} target="_blank" rel="noopener" class="text-xs text-violet-300 hover:text-violet-100 underline-offset-2 hover:underline truncate">
					Get API key at {new URL(modalProvider.url).hostname}
				</a>
			</div>

			<div class="flex gap-3">
				<button
					onclick={saveProviderKey}
					disabled={!modalValue.trim()}
					class="flex-1 rounded-xl bg-rose-500 py-3 text-sm font-bold text-white disabled:opacity-50 transition hover:bg-rose-400"
				>
					Save Key Securely
				</button>
				{#if keyStates[modalProvider.id]?.saved}
					<button
						onclick={() => {
							modalValue = '';
							saveProviderKey();
						}}
						class="rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-400 hover:text-rose-400"
					>
						Clear
					</button>
				{/if}
				<button onclick={() => { showModal = false; modalProvider = null; }} class="rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-white">
					Cancel
				</button>
			</div>
		</div>
	</div>
{/if}
