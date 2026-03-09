<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import ModelSwitcher from '$lib/components/ModelSwitcher.svelte';
	import {
		loadProviders,
		activeProviderIdStore,
		activeModelStore,
		logApiUsage,
		createKaizenTask,
		enabledProvidersStore,
		getProviderForTool,
		providersStore
	} from '$lib/utils/provider-registry';

	type ToolManifest = {
		id: string;
		name: string;
		requiredKeys: string[];
	};

	type WorkflowRun = {
		id: string;
		workflowName: string;
		modelName: string;
		configPath: string;
		statePath: string;
		status: string;
	};

	const flow = [
		'Firecrawl researches web sources',
		'Perplexica searches and filters',
		'llm-council votes on best approach',
		'Superpowers: brainstorm → plan → execute → review (dispatching-parallel-agents)',
		'get-shit-done executes via selected model',
		'Diffusionstudio Agent edits demo video in background (Vy/Panda control)',
		'Fal.ai generates output assets',
		'vibe-kanban updates board',
		'claudia consolidates memory → Kaizen tasks'
	];

	let tools: ToolManifest[] = [];
	let selectedTools: string[] = ['perplexica', 'llm-council', 'get-shit-done'];
	let workflowName = 'Universal Ralph Chain';
	let workflowStatus = 'Ready to compose a workflow.';
	let runs: WorkflowRun[] = [];
	let creatingWorkflow = false;

	$: activeModelDisplay = $activeModelStore
		? `${$activeModelStore}`
		: 'No model selected';

	$: activeProvider = $providersStore.find((p) => p.id === $activeProviderIdStore);

	onMount(async () => {
		// Load providers for injection
		await loadProviders();

		if (!isDesktopRuntime()) return;

		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		runs = await invokeTauri<WorkflowRun[]>('list_workflow_runs');
	});

	async function createWorkflow() {
		if (!isDesktopRuntime()) {
			workflowStatus = 'Workflow preparation is available in the RalphHub desktop runtime.';
			return;
		}

		if (!$activeModelStore) {
			workflowStatus = 'Please select a model using the model switcher above.';
			return;
		}

		creatingWorkflow = true;

		try {
			const run = await invokeTauri<WorkflowRun>('create_workflow_run', {
				request: {
					name: workflowName,
					modelName: `${$activeProviderIdStore}/${$activeModelStore}`,
					toolIds: selectedTools
				}
			});

			runs = [run, ...runs];
			workflowStatus = `Workflow prepared. Config: ${run.configPath}`;

			// Log to Memory Spine
			if ($activeProviderIdStore && $activeModelStore) {
				await logApiUsage({
					providerId: $activeProviderIdStore,
					providerName: activeProvider?.name ?? $activeProviderIdStore,
					model: $activeModelStore,
					tokensIn: 0,
					tokensOut: 0,
					costUsd: 0,
					outputSummary: `Workflow "${workflowName}" created with tools: ${selectedTools.join(', ')}`,
					toolId: 'workflow-composer',
					workflowId: run.id
				});
			}

			// Auto-create Kaizen task
			await createKaizenTask({
				title: `Run workflow: ${workflowName}`,
				description: `Tools: ${selectedTools.join(', ')} | Model: ${$activeModelStore}`,
				priority: 'normal',
				source: 'workflow-composer',
				providerId: $activeProviderIdStore,
				usageLogId: ''
			});
		} catch (error) {
			workflowStatus = error instanceof Error ? error.message : 'Failed to create workflow.';
		} finally {
			creatingWorkflow = false;
		}
	}

	function toggleTool(toolId: string) {
		selectedTools = selectedTools.includes(toolId)
			? selectedTools.filter((id) => id !== toolId)
			: [...selectedTools, toolId];
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-violet-400/20 bg-slate-950/55 p-8 shadow-2xl shadow-violet-950/20 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-violet-200/80">Workflows</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Compose a universal multi-tool Ralph loop.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Every workflow auto-injects keys from the Provider Registry, logs all API usage to Memory Spine, and creates Kaizen tasks automatically.
		</p>

		<!-- Model Switcher at top level -->
		<div class="mt-6 flex items-center gap-3 flex-wrap">
			<span class="text-sm text-slate-400">Active model:</span>
			<ModelSwitcher />
			{#if $enabledProvidersStore.length > 0}
				<span class="text-xs text-slate-500">{$enabledProvidersStore.length} providers connected</span>
			{:else}
				<a href="/settings" class="text-xs text-amber-400 hover:text-amber-300 underline">
					No providers connected — go to Settings →
				</a>
			{/if}
		</div>
	</div>

	<div class="grid gap-4 lg:grid-cols-[1.25fr_0.9fr]">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Universal Ralph chain with provider injection</h2>
			<div class="mt-6 space-y-2">
				{#each flow as step, index}
					<div class="flex items-center gap-4 rounded-2xl border border-white/6 bg-white/3 p-3">
						<div class="flex h-7 w-7 items-center justify-center rounded-full bg-violet-400/15 text-xs font-semibold text-violet-100 flex-shrink-0">
							{index + 1}
						</div>
						<p class="text-sm text-slate-300">{step}</p>
					</div>
				{/each}
			</div>

			<!-- Provider auto-injection info -->
			<div class="mt-4 rounded-2xl border border-cyan-400/10 bg-cyan-400/5 p-4">
				<p class="text-xs font-semibold text-cyan-300 mb-2">Auto-Injected Provider Keys</p>
				{#if $enabledProvidersStore.length === 0}
					<p class="text-xs text-slate-500">No providers enabled. <a href="/settings" class="text-cyan-400 underline">Enable providers →</a></p>
				{:else}
					<div class="flex flex-wrap gap-1">
						{#each $enabledProvidersStore as p}
							<span class="rounded-lg bg-slate-800 px-2 py-1 text-xs font-mono text-slate-300">
								{p.logoEmoji} {p.apiKeyEnv || p.name}
							</span>
						{/each}
					</div>
				{/if}
			</div>
		</div>

		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Workflow composer</h2>
			<div class="mt-4 space-y-4">
				<input
					bind:value={workflowName}
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
					placeholder="Workflow name"
				/>

				<!-- Inline model switcher -->
				<div>
					<label class="block text-xs font-medium text-slate-500 mb-2">Model</label>
					<ModelSwitcher compact={false} />
				</div>

				<div class="space-y-2">
					<label class="block text-xs font-medium text-slate-500">Tools</label>
					{#each tools as tool}
						<label class="flex items-center gap-3 rounded-2xl border border-white/8 bg-white/3 px-4 py-3 text-sm text-slate-300 cursor-pointer hover:bg-white/5">
							<input
								type="checkbox"
								checked={selectedTools.includes(tool.id)}
								on:change={() => toggleTool(tool.id)}
								class="rounded border-white/20"
							/>
							<span class="flex-1">{tool.name}</span>
							{#if tool.requiredKeys?.length}
								{@const injected = getProviderForTool($providersStore, tool.requiredKeys)}
								{#if injected}
									<span class="text-xs text-green-400">{injected.logoEmoji}</span>
								{:else}
									<span class="text-xs text-amber-400" title="No matching key configured">⚠</span>
								{/if}
							{/if}
						</label>
					{/each}
					{#if tools.length === 0}
						<p class="text-sm text-slate-500">Loading tools...</p>
					{/if}
				</div>

				<button
					type="button"
					on:click={createWorkflow}
					disabled={creatingWorkflow || !$activeModelStore}
					class="w-full rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 disabled:opacity-60 disabled:cursor-not-allowed"
				>
					{creatingWorkflow ? 'Creating...' : 'Create Workflow + Log to Memory'}
				</button>
				<p class="text-sm text-slate-400">{workflowStatus}</p>
			</div>
		</div>
	</div>

	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-semibold text-white">Prepared runs</h2>
			<span class="text-sm text-slate-500">{runs.length} total</span>
		</div>
		<div class="mt-6 space-y-3">
			{#if !runs.length}
				<p class="text-sm text-slate-500">No workflow runs prepared yet.</p>
			{:else}
				{#each runs as run}
					<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex items-center justify-between gap-3">
							<p class="text-sm font-medium text-white">{run.workflowName}</p>
							<span class="rounded-full bg-slate-800 px-2 py-1 text-xs text-slate-400">{run.modelName}</span>
						</div>
						<p class="mt-1 text-xs text-slate-500 font-mono truncate">{run.configPath}</p>
						<p class="mt-2 text-xs text-cyan-100">{run.status}</p>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
