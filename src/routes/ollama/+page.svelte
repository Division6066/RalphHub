<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type OllamaModel = {
		name: string;
		displayName: string;
		sizeHint: string;
		status: string;
		isDefault: boolean;
	};

	type OllamaStatus = {
		installed: boolean;
		version: string | null;
		running: boolean;
		endpoint: string;
		models: OllamaModel[];
		installerHint: string;
	};

	let status: OllamaStatus | null = null;
	let loading = true;
	let busy = false;
	let pullingModel = '';
	let message = '';

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
			status = await invokeTauri<OllamaStatus>('get_ollama_status');
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to check Ollama status.';
		} finally {
			loading = false;
		}
	}

	async function ensureOllama() {
		busy = true;
		message = 'Installing Ollama...';
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('ensure_ollama');
			message = result.message;
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to install Ollama.';
		} finally {
			busy = false;
		}
	}

	async function startServer() {
		busy = true;
		message = 'Starting Ollama server...';
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('start_ollama_server');
			message = result.message;
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to start server.';
		} finally {
			busy = false;
		}
	}

	async function pullModel(modelName: string) {
		pullingModel = modelName;
		message = `Pulling ${modelName}...`;
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('pull_ollama_model', { modelName });
			message = result.message;
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : `Failed to pull ${modelName}.`;
		} finally {
			pullingModel = '';
		}
	}

	async function pullRecommended() {
		busy = true;
		message = 'Pulling recommended models (Mistral, Qwen, Llama)... this may take several minutes.';
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('pull_recommended_models');
			message = result.message;
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to pull models.';
		} finally {
			busy = false;
		}
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-violet-300/80">Local Models</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Ollama — local model manager.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Auto-install Ollama, pre-pull recommended quantized models (Mistral, Qwen, Llama, Phi), and route
			all tools to local inference by default. Supports Ollama Cloud as fallback.
		</p>
	</div>

	{#if message}
		<div class="rounded-3xl border border-violet-400/20 bg-violet-500/10 p-4 text-sm text-violet-100">
			{message}
		</div>
	{/if}

	{#if loading}
		<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-6 text-sm text-slate-400 backdrop-blur">
			Checking Ollama status...
		</div>
	{:else if status}
		<!-- Status row -->
		<div class="grid gap-4 xl:grid-cols-3">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<p class="text-sm text-slate-400">Installation</p>
				<p class="mt-3 text-xl font-semibold {status.installed ? 'text-green-400' : 'text-red-400'}">
					{status.installed ? 'Installed' : 'Not installed'}
				</p>
				{#if status.version}
					<p class="mt-1 text-xs text-slate-500">{status.version}</p>
				{/if}
				{#if !status.installed}
					<button
						on:click={ensureOllama}
						disabled={busy}
						class="mt-4 rounded-full bg-violet-500/20 px-4 py-2 text-sm font-medium text-violet-100 hover:bg-violet-500/30 disabled:opacity-60"
					>
						{busy ? 'Installing...' : 'Auto-install Ollama'}
					</button>
				{/if}
			</div>

			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<p class="text-sm text-slate-400">Server</p>
				<p class="mt-3 text-xl font-semibold {status.running ? 'text-green-400' : 'text-amber-400'}">
					{status.running ? 'Running' : 'Stopped'}
				</p>
				<p class="mt-1 text-xs text-slate-500">{status.endpoint}</p>
				{#if status.installed && !status.running}
					<button
						on:click={startServer}
						disabled={busy}
						class="mt-4 rounded-full bg-cyan-400/12 px-4 py-2 text-sm font-medium text-cyan-100 hover:bg-cyan-400/20 disabled:opacity-60"
					>
						Start Server
					</button>
				{/if}
			</div>

			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<p class="text-sm text-slate-400">Models pulled</p>
				<p class="mt-3 text-xl font-semibold text-white">
					{status.models.filter((m) => m.status === 'available').length} / {status.models.length}
				</p>
				<button
					on:click={pullRecommended}
					disabled={busy || !status.installed}
					class="mt-4 rounded-full bg-gradient-to-r from-cyan-500/20 to-violet-500/20 px-4 py-2 text-sm font-medium text-white hover:from-cyan-500/30 hover:to-violet-500/30 disabled:opacity-60"
				>
					{busy ? 'Pulling...' : 'Pull recommended models'}
				</button>
			</div>
		</div>

		<!-- Model grid -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-white">Model library</h2>
				<button
					on:click={refresh}
					class="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-400 hover:text-white"
				>
					Refresh
				</button>
			</div>

			<div class="mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
				{#each status.models as model}
					<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex items-start justify-between gap-2">
							<div>
								<p class="text-sm font-medium text-white">
									{model.displayName}
									{#if model.isDefault}
										<span class="ml-1 rounded-full bg-cyan-400/15 px-2 py-0.5 text-xs text-cyan-200">default</span>
									{/if}
								</p>
								<p class="mt-1 text-xs text-slate-500 font-mono">{model.name}</p>
								<p class="mt-1 text-xs text-slate-500">{model.sizeHint}</p>
							</div>
							<span class="shrink-0 rounded-full px-2 py-0.5 text-xs {
								model.status === 'available' ? 'bg-green-400/15 text-green-300' :
								model.status === 'pulling' ? 'bg-amber-400/15 text-amber-300' :
								'bg-slate-600/30 text-slate-400'
							}">
								{model.status}
							</span>
						</div>

						{#if model.status !== 'available'}
							<button
								on:click={() => pullModel(model.name)}
								disabled={pullingModel === model.name || !status?.installed}
								class="mt-3 w-full rounded-xl bg-violet-500/15 px-3 py-1.5 text-xs font-medium text-violet-200 hover:bg-violet-500/25 disabled:opacity-60"
							>
								{pullingModel === model.name ? 'Pulling...' : 'Pull model'}
							</button>
						{/if}
					</div>
				{/each}
			</div>
		</div>

		<!-- Install hint -->
		{#if !status.installed}
			<div class="rounded-3xl border border-amber-400/20 bg-amber-500/8 p-6 backdrop-blur">
				<p class="text-sm font-medium text-amber-200">Manual install command</p>
				<code class="mt-3 block rounded-xl bg-slate-900/80 px-4 py-3 text-sm text-slate-300">
					{status.installerHint}
				</code>
			</div>
		{/if}
	{/if}
</section>
