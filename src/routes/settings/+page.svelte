<script lang="ts">
	import { onMount } from 'svelte';

	import {
		createEmptyKeyMap,
		KEY_FIELDS,
		loadKeys,
		saveKeys,
		loadDynamicKeys,
		saveDynamicKey,
		type KeyField,
		type KeyMap
	} from '$lib/utils/secure-store';

	import {
		loadProviders,
		createProvider,
		updateProvider,
		searchProviders,
		toggleProvider,
		getMemoryStats,
		loadKaizenTasks,
		providersStore,
		providersByCategoryStore,
		memoryStatsStore,
		kaizenTasksStore,
		CATEGORY_LABELS,
		CATEGORY_ORDER,
		type Provider,
		type CreateProviderRequest,
		type ProviderCategory
	} from '$lib/utils/provider-registry';

	import ProviderCard from '$lib/components/ProviderCard.svelte';

	// ─── Tab State ───────────────────────────────────────────────────────────────
	let activeTab: 'providers' | 'legacy-keys' | 'memory' | 'tasks' = 'providers';

	// ─── Legacy Keys ─────────────────────────────────────────────────────────────
	const labels: Record<KeyField, string> = {
		ANTHROPIC_API_KEY: 'Anthropic',
		OPENAI_API_KEY: 'OpenAI',
		GROK_API_KEY: 'Grok',
		GEMINI_API_KEY: 'Gemini',
		PERPLEXICA_KEYS: 'Perplexica'
	};

	let keys: KeyMap = createEmptyKeyMap();
	let loadingKeys = true;
	let savingKeys = false;
	let keyStatus = 'Loading...';
	let keyError = '';

	// ─── Provider Search + Filter ────────────────────────────────────────────────
	let searchQuery = '';
	let selectedCategory = 'all';
	let loadingProviders = true;
	let providerError = '';

	// ─── Add New Provider Form ───────────────────────────────────────────────────
	let showAddForm = false;
	let addingProvider = false;
	let addForm: CreateProviderRequest = emptyProviderForm();
	let addModelsText = '';
	let addStatus = '';

	function emptyProviderForm(): CreateProviderRequest {
		return {
			name: '',
			category: 'llm',
			baseUrl: '',
			authType: 'bearer',
			apiKeyEnv: '',
			models: [],
			isLocal: false,
			description: '',
			docsUrl: '',
			logoEmoji: '🔌'
		};
	}

	// ─── Memory Stats ─────────────────────────────────────────────────────────────
	let loadingMemory = false;

	// ─── Computed: filtered providers ────────────────────────────────────────────
	let displayProviders: Provider[] = [];
	let searchTimeout: ReturnType<typeof setTimeout>;

	$: {
		if (searchQuery) {
			clearTimeout(searchTimeout);
			searchTimeout = setTimeout(async () => {
				try {
					displayProviders = await searchProviders(searchQuery);
				} catch {
					displayProviders = [];
				}
			}, 200);
		} else if (selectedCategory === 'all') {
			displayProviders = $providersStore;
		} else {
			displayProviders = ($providersByCategoryStore[selectedCategory] ?? []);
		}
	}

	$: categoryCounts = Object.fromEntries(
		Object.entries($providersByCategoryStore).map(([k, v]) => [k, v.length])
	);

	$: enabledCount = $providersStore.filter((p) => p.enabled).length;

	onMount(async () => {
		// Load legacy keys
		try {
			keys = await loadKeys();
			keyStatus = 'Keys loaded from Stronghold.';
		} catch (e) {
			keyError = e instanceof Error ? e.message : 'Failed to load keys.';
			keyStatus = 'Vault unavailable.';
		} finally {
			loadingKeys = false;
		}

		// Load providers
		try {
			await loadProviders();
		} catch (e) {
			providerError = e instanceof Error ? e.message : 'Failed to load providers.';
		} finally {
			loadingProviders = false;
		}

		// Load memory stats
		try {
			loadingMemory = true;
			await getMemoryStats();
		} catch {
			// Non-critical
		} finally {
			loadingMemory = false;
		}

		// Load tasks
		try {
			await loadKaizenTasks();
		} catch {
			// Non-critical
		}
	});

	async function handleSaveKeys() {
		savingKeys = true;
		keyError = '';
		keyStatus = 'Saving...';
		try {
			await saveKeys(keys);
			keyStatus = 'Keys saved securely.';
		} catch (e) {
			keyError = e instanceof Error ? e.message : 'Failed to save.';
			keyStatus = 'Save failed.';
		} finally {
			savingKeys = false;
		}
	}

	async function handleAddProvider() {
		addingProvider = true;
		addStatus = '';
		try {
			addForm.models = addModelsText
				.split(/[\n,]/)
				.map((m) => m.trim())
				.filter(Boolean);
			const created = await createProvider(addForm);
			addStatus = `✓ Provider "${created.name}" added.`;
			addForm = emptyProviderForm();
			addModelsText = '';
			showAddForm = false;
		} catch (e) {
			addStatus = e instanceof Error ? e.message : 'Failed to add provider.';
		} finally {
			addingProvider = false;
		}
	}

	function handleProviderUpdated(event: CustomEvent<Provider>) {
		providersStore.update((ps) => ps.map((p) => (p.id === event.detail.id ? event.detail : p)));
	}

	async function handleConnectAndTest(event: CustomEvent<Provider>) {
		const p = event.detail;
		// Stub: real test would call provider's health endpoint
		addStatus = `Testing ${p.name}...`;
		await new Promise((r) => setTimeout(r, 500));
		addStatus = `${p.name}: connection OK (stub).`;
	}

	const AUTH_TYPES = [
		{ value: 'bearer', label: 'Bearer Token (Authorization: Bearer ...)' },
		{ value: 'x-api-key', label: 'X-Api-Key Header' },
		{ value: 'api-key-header', label: 'Api-Key Header' },
		{ value: 'api-key-param', label: 'API Key Query Param' },
		{ value: 'api-key-body', label: 'API Key in Request Body' },
		{ value: 'key-secret', label: 'Key:Secret (Fal.ai style)' },
		{ value: 'xi-api-key', label: 'Xi-Api-Key (ElevenLabs)' },
		{ value: 'authorization', label: 'Authorization Header' },
		{ value: 'token', label: 'Token Header' },
		{ value: 'basic', label: 'HTTP Basic Auth' },
		{ value: 'apikey', label: 'Apikey Header' },
		{ value: 'none', label: 'No Auth (Local / Open)' },
	];

	const CATEGORIES: Array<{ value: string; label: string }> = [
		{ value: 'llm', label: '🤖 LLM' },
		{ value: 'image', label: '🎨 Image' },
		{ value: 'video', label: '🎬 Video' },
		{ value: 'audio', label: '🔊 Audio' },
		{ value: 'search', label: '🔍 Search' },
		{ value: 'automation', label: '🤖 Automation' },
		{ value: 'voice', label: '📞 Voice' },
		{ value: 'mcp', label: '🧩 MCP' },
		{ value: 'database', label: '🗄️ Database' },
		{ value: 'custom', label: '⚙️ Custom' },
	];
</script>

<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Settings</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Universal API Provider System</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Connect any API provider — LLMs, video, image, audio, search, automation, voice, and more. Keys are stored in Stronghold and auto-injected into every tool and workflow.
		</p>
		<div class="mt-4 flex gap-3 text-sm">
			<span class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-cyan-100">
				{$providersStore.length} providers loaded
			</span>
			<span class="rounded-full border border-green-400/20 bg-green-400/10 px-3 py-1 text-green-100">
				{enabledCount} active
			</span>
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex gap-1 rounded-2xl border border-white/10 bg-slate-950/40 p-1 backdrop-blur">
		{#each [
			{ id: 'providers', label: '🔌 Providers' },
			{ id: 'memory', label: '🧠 Memory Spine' },
			{ id: 'tasks', label: '✅ Kaizen Tasks' },
			{ id: 'legacy-keys', label: '🔑 Legacy Keys' },
		] as tab}
			<button
				type="button"
				on:click={() => (activeTab = tab.id as typeof activeTab)}
				class="flex-1 rounded-xl px-4 py-2.5 text-sm font-medium transition-colors
					{activeTab === tab.id
					? 'bg-white/10 text-white'
					: 'text-slate-500 hover:text-slate-300'}"
			>
				{tab.label}
			</button>
		{/each}
	</div>

	<!-- ═══════════════════ PROVIDERS TAB ═══════════════════ -->
	{#if activeTab === 'providers'}
		<div class="space-y-4">
			<!-- Search + Filter + Add -->
			<div class="flex flex-wrap gap-3">
				<div class="relative flex-1 min-w-48">
					<svg class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
					</svg>
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search 50+ providers..."
						class="w-full rounded-2xl border border-white/10 bg-slate-950/70 pl-10 pr-4 py-3 text-sm text-white placeholder:text-slate-500 outline-none focus:border-cyan-400/40"
					/>
				</div>

				<select
					bind:value={selectedCategory}
					class="rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
				>
					<option value="all">All Categories ({$providersStore.length})</option>
					{#each CATEGORY_ORDER as cat}
						{#if categoryCounts[cat]}
							<option value={cat}>{CATEGORY_LABELS[cat] ?? cat} ({categoryCounts[cat]})</option>
						{/if}
					{/each}
				</select>

				<button
					type="button"
					on:click={() => (showAddForm = !showAddForm)}
					class="rounded-2xl bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 transition-shadow"
				>
					+ Add Provider
				</button>
			</div>

			<!-- Add Provider Form -->
			{#if showAddForm}
				<div class="rounded-3xl border border-cyan-400/20 bg-slate-950/60 p-6 backdrop-blur space-y-4">
					<h2 class="text-lg font-semibold text-white">Add Any Provider</h2>
					<p class="text-sm text-slate-400">Connect any API that the world has — or will ever have.</p>

					<div class="grid gap-4 sm:grid-cols-2">
						<div>
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Provider Name *</label>
							<input
								type="text"
								bind:value={addForm.name}
								placeholder="e.g. My Custom LLM"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							/>
						</div>
						<div>
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Category *</label>
							<select
								bind:value={addForm.category}
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							>
								{#each CATEGORIES as cat}
									<option value={cat.value}>{cat.label}</option>
								{/each}
							</select>
						</div>
						<div class="sm:col-span-2">
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Base URL *</label>
							<input
								type="url"
								bind:value={addForm.baseUrl}
								placeholder="https://api.myprovider.com/v1"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							/>
						</div>
						<div>
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Auth Type</label>
							<select
								bind:value={addForm.authType}
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							>
								{#each AUTH_TYPES as t}
									<option value={t.value}>{t.label}</option>
								{/each}
							</select>
						</div>
						<div>
							<label class="block text-xs font-medium text-slate-400 mb-1.5">API Key Env Var</label>
							<input
								type="text"
								bind:value={addForm.apiKeyEnv}
								placeholder="MY_PROVIDER_API_KEY"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white font-mono outline-none focus:border-cyan-400/40"
							/>
						</div>
						<div>
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Logo Emoji</label>
							<input
								type="text"
								bind:value={addForm.logoEmoji}
								placeholder="🔌"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							/>
						</div>
						<div>
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Docs URL</label>
							<input
								type="url"
								bind:value={addForm.docsUrl}
								placeholder="https://docs.myprovider.com"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							/>
						</div>
						<div class="sm:col-span-2">
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Models (comma or newline separated)</label>
							<textarea
								bind:value={addModelsText}
								placeholder="model-name-1, model-name-2&#10;model-name-3"
								rows="3"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white font-mono outline-none focus:border-cyan-400/40 resize-none"
							></textarea>
						</div>
						<div class="sm:col-span-2">
							<label class="block text-xs font-medium text-slate-400 mb-1.5">Description</label>
							<input
								type="text"
								bind:value={addForm.description}
								placeholder="Short description of this provider"
								class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
							/>
						</div>
						<div class="flex items-center gap-3">
							<input
								type="checkbox"
								id="is-local"
								bind:checked={addForm.isLocal}
								class="rounded border-white/20"
							/>
							<label for="is-local" class="text-sm text-slate-300">Local / Self-hosted (no API key needed)</label>
						</div>
					</div>

					<div class="flex items-center gap-3">
						<button
							type="button"
							on:click={handleAddProvider}
							disabled={addingProvider || !addForm.name || !addForm.baseUrl}
							class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg disabled:opacity-60"
						>
							{addingProvider ? 'Adding...' : 'Add Provider'}
						</button>
						<button
							type="button"
							on:click={() => { showAddForm = false; addForm = emptyProviderForm(); addModelsText = ''; }}
							class="rounded-full border border-white/10 px-5 py-3 text-sm text-slate-400 hover:text-white"
						>
							Cancel
						</button>
						{#if addStatus}
							<p class="text-sm {addStatus.startsWith('✓') ? 'text-green-400' : 'text-rose-400'}">{addStatus}</p>
						{/if}
					</div>
				</div>
			{/if}

			<!-- Special first-class providers: Firecrawl + Apify -->
			{#if selectedCategory === 'all' || selectedCategory === 'search' || selectedCategory === 'automation'}
				{@const firecrawl = $providersStore.find(p => p.name === 'Firecrawl')}
				{@const apify = $providersStore.find(p => p.name === 'Apify')}
				{#if firecrawl || apify}
					<div class="rounded-3xl border border-amber-400/20 bg-amber-400/5 p-4">
						<p class="text-xs font-semibold uppercase tracking-wider text-amber-300/70 mb-3">⭐ First-Class Integrations</p>
						<div class="grid gap-3 sm:grid-cols-2">
							{#if firecrawl}
								<div class="rounded-2xl border border-amber-400/20 bg-slate-950/60 p-4">
									<div class="flex items-center gap-2 mb-2">
										<span class="text-xl">🔥</span>
										<div>
											<p class="text-sm font-semibold text-white">Firecrawl</p>
											<p class="text-xs text-slate-500">Web scraping + LLM-ready output</p>
										</div>
										<div class="ml-auto flex items-center gap-2">
											<a href="https://firecrawl.dev" target="_blank" rel="noopener noreferrer" class="text-xs text-amber-400 hover:text-amber-300">Get Key →</a>
											<button
												type="button"
												on:click={() => toggleProvider(firecrawl.id, !firecrawl.enabled)}
												class="rounded-xl {firecrawl.enabled ? 'bg-amber-400/20 text-amber-300' : 'bg-slate-800 text-slate-400'} px-3 py-1.5 text-xs font-medium transition-colors"
											>
												{firecrawl.enabled ? '✓ Connected' : 'Connect & Test'}
											</button>
										</div>
									</div>
									<p class="text-xs text-slate-500">Endpoints: scrape, crawl, map, extract, search</p>
								</div>
							{/if}
							{#if apify}
								<div class="rounded-2xl border border-amber-400/20 bg-slate-950/60 p-4">
									<div class="flex items-center gap-2 mb-2">
										<span class="text-xl">🕷️</span>
										<div>
											<p class="text-sm font-semibold text-white">Apify</p>
											<p class="text-xs text-slate-500">1500+ automation actors</p>
										</div>
										<div class="ml-auto flex items-center gap-2">
											<a href="https://apify.com" target="_blank" rel="noopener noreferrer" class="text-xs text-amber-400 hover:text-amber-300">Get Key →</a>
											<button
												type="button"
												on:click={() => toggleProvider(apify.id, !apify.enabled)}
												class="rounded-xl {apify.enabled ? 'bg-amber-400/20 text-amber-300' : 'bg-slate-800 text-slate-400'} px-3 py-1.5 text-xs font-medium transition-colors"
											>
												{apify.enabled ? '✓ Connected' : 'Connect & Test'}
											</button>
										</div>
									</div>
									<p class="text-xs text-slate-500">Endpoints: web-scraper, browser-scraper, cheerio-scraper</p>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			{/if}

			<!-- Provider Grid -->
			{#if loadingProviders}
				<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-12 text-center text-sm text-slate-500">
					Loading providers...
				</div>
			{:else if providerError}
				<div class="rounded-3xl border border-rose-400/20 bg-rose-500/10 p-6 text-sm text-rose-300">
					{providerError}
				</div>
			{:else if displayProviders.length === 0}
				<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-12 text-center text-sm text-slate-500">
					{searchQuery ? `No providers match "${searchQuery}"` : 'No providers in this category'}
				</div>
			{:else}
				<!-- Group by category when showing all -->
				{#if selectedCategory === 'all' && !searchQuery}
					{#each CATEGORY_ORDER as cat}
						{@const catProviders = $providersByCategoryStore[cat] ?? []}
						{#if catProviders.length > 0}
							<div>
								<h3 class="mb-3 text-sm font-semibold text-slate-400">{CATEGORY_LABELS[cat] ?? cat}</h3>
								<div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
									{#each catProviders as provider (provider.id)}
										<ProviderCard
											{provider}
											on:updated={handleProviderUpdated}
											on:test={handleConnectAndTest}
										/>
									{/each}
								</div>
							</div>
						{/if}
					{/each}
				{:else}
					<div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
						{#each displayProviders as provider (provider.id)}
							<ProviderCard
								{provider}
								on:updated={handleProviderUpdated}
								on:test={handleConnectAndTest}
							/>
						{/each}
					</div>
				{/if}
			{/if}
		</div>

	<!-- ═══════════════════ MEMORY SPINE TAB ═══════════════════ -->
	{:else if activeTab === 'memory'}
		<div class="space-y-4">
			<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
				{#each [
					{ label: 'Memory Entries', value: String($memoryStatsStore?.totalEntries ?? 0), sub: 'API calls logged' },
					{ label: 'Total Tokens', value: ($memoryStatsStore?.totalTokens ?? 0).toLocaleString(), sub: 'in + out' },
					{ label: 'Total Cost', value: `$${($memoryStatsStore?.totalCostUsd ?? 0).toFixed(4)}`, sub: 'USD estimated' },
					{ label: 'Providers Used', value: String($memoryStatsStore?.providersUsed?.length ?? 0), sub: 'distinct APIs called' },
				] as stat}
					<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
						<p class="text-xs text-slate-500">{stat.label}</p>
						<p class="mt-2 text-2xl font-semibold text-white">{stat.value}</p>
						<p class="mt-1 text-xs text-slate-600">{stat.sub}</p>
					</div>
				{/each}
			</div>

			{#if $memoryStatsStore?.providersUsed?.length}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
					<h3 class="text-sm font-semibold text-white mb-3">Providers Called</h3>
					<div class="flex flex-wrap gap-2">
						{#each $memoryStatsStore.providersUsed as name}
							<span class="rounded-full bg-cyan-400/10 border border-cyan-400/20 px-3 py-1 text-xs text-cyan-300">{name}</span>
						{/each}
					</div>
				</div>
			{/if}

			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
				<h3 class="text-sm font-semibold text-white mb-3">Recent API Calls</h3>
				{#if !$memoryStatsStore?.recentLogs?.length}
					<p class="text-sm text-slate-500">No API calls logged yet. Enable providers and start making calls to see logs here.</p>
				{:else}
					<div class="space-y-2">
						{#each $memoryStatsStore.recentLogs as log}
							<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-3">
								<div class="flex items-center justify-between gap-2 flex-wrap">
									<div class="flex items-center gap-2">
										<span class="text-xs font-semibold text-white">{log.providerName}</span>
										<span class="rounded-lg bg-slate-800 px-2 py-0.5 text-xs font-mono text-slate-400">{log.model}</span>
									</div>
									<div class="flex items-center gap-3 text-xs text-slate-500">
										<span>{log.tokensIn + log.tokensOut} tokens</span>
										<span>${log.costUsd.toFixed(6)}</span>
										<span>{new Date(log.createdAt).toLocaleTimeString()}</span>
									</div>
								</div>
								{#if log.outputSummary}
									<p class="mt-1.5 text-xs text-slate-400 line-clamp-2">{log.outputSummary}</p>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>

	<!-- ═══════════════════ KAIZEN TASKS TAB ═══════════════════ -->
	{:else if activeTab === 'tasks'}
		<div class="space-y-4">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
				<div class="flex items-center justify-between mb-4">
					<h3 class="text-sm font-semibold text-white">Kaizen Task Board</h3>
					<span class="text-xs text-slate-500">{$kaizenTasksStore.length} total tasks</span>
				</div>
				{#if !$kaizenTasksStore.length}
					<p class="text-sm text-slate-500">No Kaizen tasks yet. Tasks are auto-created after API calls with substantial output summaries.</p>
				{:else}
					<div class="space-y-2">
						{#each $kaizenTasksStore as task}
							<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-4">
								<div class="flex items-start justify-between gap-3">
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-2 flex-wrap">
											<span class="text-sm font-medium text-white truncate">{task.title}</span>
											<span class="rounded-full px-2 py-0.5 text-xs
												{task.status === 'done' ? 'bg-green-500/20 text-green-400' :
												task.status === 'in_progress' ? 'bg-cyan-500/20 text-cyan-400' :
												task.status === 'blocked' ? 'bg-rose-500/20 text-rose-400' :
												'bg-slate-700 text-slate-400'}">
												{task.status}
											</span>
											<span class="rounded-full px-2 py-0.5 text-xs
												{task.priority === 'urgent' ? 'bg-red-500/20 text-red-400' :
												task.priority === 'high' ? 'bg-orange-500/20 text-orange-400' :
												'bg-slate-700/50 text-slate-500'}">
												{task.priority}
											</span>
										</div>
										{#if task.description}
											<p class="mt-1 text-xs text-slate-500 line-clamp-2">{task.description}</p>
										{/if}
										<p class="mt-1 text-xs text-slate-600">{task.source} · {new Date(task.createdAt).toLocaleDateString()}</p>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>

	<!-- ═══════════════════ LEGACY KEYS TAB ═══════════════════ -->
	{:else if activeTab === 'legacy-keys'}
		<div class="grid gap-4 xl:grid-cols-[1.3fr_0.8fr]">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-start justify-between gap-4">
					<div>
						<h2 class="text-lg font-semibold text-white">Legacy key inventory</h2>
						<p class="mt-2 text-sm text-slate-400">
							Stored in Stronghold. Use the Providers tab to manage all 50+ provider keys dynamically.
						</p>
					</div>
					<span class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-xs font-medium text-cyan-100">
						{loadingKeys ? 'Loading' : 'Ready'}
					</span>
				</div>

				<form class="mt-6 space-y-4" on:submit|preventDefault={handleSaveKeys}>
					{#each KEY_FIELDS as field}
						<div class="rounded-2xl border border-white/10 bg-slate-950/70 p-4">
							<label class="block text-sm font-medium text-white" for={field}>{field}</label>
							<p class="mt-2 text-xs uppercase tracking-[0.2em] text-slate-500">{labels[field]}</p>
							<input
								id={field}
								type="password"
								bind:value={keys[field]}
								disabled={loadingKeys || savingKeys}
								autocomplete="off"
								placeholder="Paste key value"
								class="mt-4 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
							/>
						</div>
					{/each}

					<div class="flex flex-wrap items-center gap-3">
						<button
							type="submit"
							disabled={loadingKeys || savingKeys}
							class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:cursor-not-allowed disabled:opacity-60"
						>
							{savingKeys ? 'Saving...' : 'Save to Stronghold'}
						</button>
						<p class="text-sm text-slate-400">{keyStatus}</p>
					</div>
				</form>
			</div>

			<div class="space-y-4">
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
					<h2 class="text-lg font-semibold text-white">Injection policy</h2>
					<ul class="mt-4 space-y-3 text-sm leading-6 text-slate-400">
						<li>All provider keys are stored in Stronghold and never committed to git.</li>
						<li>Keys are auto-injected into every tool and workflow via the provider registry.</li>
						<li>Fallback to local Ollama when no remote key is set for a required provider.</li>
					</ul>
				</div>

				<div class="rounded-3xl border border-rose-400/20 bg-rose-500/10 p-6 backdrop-blur">
					<h2 class="text-lg font-semibold text-white">Vault state</h2>
					<p class="mt-3 text-sm leading-6 text-slate-300">
						{#if keyError}
							{keyError}
						{:else}
							No secrets stored in the repo. This screen + Stronghold is the only source of truth.
						{/if}
					</p>
				</div>
			</div>
		</div>
	{/if}
</section>
