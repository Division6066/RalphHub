<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import ModelSwitcher from '$lib/components/ModelSwitcher.svelte';
	import {
		loadProviders,
		activeModelStore,
		activeProviderIdStore,
		getProviderForTool,
		logApiUsage,
		enabledProvidersStore,
		providersStore
	} from '$lib/utils/provider-registry';
	import { loadDynamicKeys } from '$lib/utils/secure-store';
	import { buildEnvInjection as _buildEnvInjection } from '$lib/utils/provider-registry';

	type ToolManifest = {
		id: string;
		name: string;
		repoUrl: string;
		description: string;
		status: string;
		openInCode: boolean;
		requiredKeys: string[];
	};

	type DeployResult = {
		workspacePath: string;
		branch: string;
		message: string;
		envPath: string;
	};

	let tools: ToolManifest[] = [];
	let loading = true;
	let busyTool = '';
	let status = '';

	onMount(async () => {
		await loadProviders();

		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}

		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		loading = false;
	});

	async function launchTool(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			status = `${tool.name} is an internal RalphHub capability.`;
			return;
		}

		busyTool = tool.id;
		status = `Launching ${tool.name}...`;

		try {
			await invokeTauri('ensure_bun');
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });

			// Auto-inject all enabled provider keys
			if (tool.requiredKeys?.length || $enabledProvidersStore.length > 0) {
				const allEnvKeys = $enabledProvidersStore.map((p) => p.apiKeyEnv).filter(Boolean);
				const keyValues = await loadDynamicKeys(allEnvKeys);
				const envEntries = Object.entries(_buildEnvInjection($enabledProvidersStore, keyValues))
					.map(([key, value]) => ({ key, value }));

				if (envEntries.length > 0) {
					await invokeTauri('inject_keys', {
						request: {
							workspacePath: result.workspacePath,
							entries: envEntries
						}
					});
				}
			}

			await invokeTauri('open_in_code', {
				workspacePath: result.workspacePath,
				branch: result.branch
			});

			// Log to memory spine
			const injectedProvider = getProviderForTool($providersStore, tool.requiredKeys ?? []);
			if (injectedProvider && $activeModelStore) {
				await logApiUsage({
					providerId: injectedProvider.id,
					providerName: injectedProvider.name,
					model: $activeModelStore,
					tokensIn: 0,
					tokensOut: 0,
					costUsd: 0,
					outputSummary: `Tool "${tool.name}" launched with provider ${injectedProvider.name}`,
					toolId: tool.id,
					workflowId: ''
				});
			}

			status = `${tool.name} ready — provider keys injected, opened in editor.`;
		} catch (error) {
			status = error instanceof Error ? error.message : `Failed to launch ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}

	function getToolProviderInfo(tool: ToolManifest) {
		if (!tool.requiredKeys?.length) return null;
		return getProviderForTool($providersStore, tool.requiredKeys);
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Tools</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">All tools — with universal provider injection.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Every tool auto-receives keys from the Provider Registry. Fallback to local Ollama when no remote key is set.
		</p>
		<div class="mt-6 flex items-center gap-3 flex-wrap">
			<span class="text-sm text-slate-400">Active model:</span>
			<ModelSwitcher compact />
			{#if $enabledProvidersStore.length > 0}
				<span class="text-xs text-green-400">{$enabledProvidersStore.length} providers active</span>
			{:else}
				<a href="/settings" class="text-xs text-amber-400 underline hover:text-amber-300">Connect providers →</a>
			{/if}
		</div>
	</div>

	<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-4 text-sm text-slate-300 backdrop-blur">
		{#if loading}
			Loading tool manifests...
		{:else}
			{status || 'Choose a tool to clone, inject provider keys, and open in your editor.'}
		{/if}
	</div>

	<div class="grid gap-4 xl:grid-cols-3">
		{#each tools as tool}
			{@const injectedProvider = getToolProviderInfo(tool)}
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-center justify-between gap-4">
					<h2 class="text-lg font-semibold text-white">{tool.name}</h2>
					<span class="rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-medium text-amber-100">
						{tool.status}
					</span>
				</div>

				<p class="mt-4 text-sm leading-6 text-slate-400">{tool.description}</p>

				<!-- Provider injection indicator -->
				{#if tool.requiredKeys?.length}
					<div class="mt-3 flex items-center gap-2">
						{#if injectedProvider}
							<span class="rounded-full bg-green-500/15 border border-green-400/20 px-2 py-0.5 text-xs text-green-400">
								{injectedProvider.logoEmoji} {injectedProvider.name}
							</span>
						{:else}
							<span class="rounded-full bg-amber-500/10 border border-amber-400/20 px-2 py-0.5 text-xs text-amber-400">
								⚠ No provider — will fallback to Ollama
							</span>
						{/if}
					</div>
				{/if}

				<div class="mt-6 flex flex-wrap gap-2 text-sm">
					<button
						type="button"
						on:click={() => launchTool(tool)}
						disabled={busyTool === tool.id}
						class="rounded-full bg-cyan-400/12 px-4 py-2 font-medium text-cyan-100 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{busyTool === tool.id ? 'Launching...' : 'Launch + Inject Keys'}
					</button>
					<a href="/workflows" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Compose</a>
				</div>
			</div>
		{/each}
	</div>
</section>
