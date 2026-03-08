<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type ToolManifest = {
		id: string;
		name: string;
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
		'Perplexica researches',
		'llm-council votes',
		'get-shit-done executes',
		'vibe-kanban updates board',
		'claudia consolidates memory'
	];

	let tools: ToolManifest[] = [];
	let selectedTools: string[] = ['perplexica', 'llm-council', 'get-shit-done'];
	let workflowName = 'Overnight Ralph Chain';
	let modelName = 'anthropic/claude-sonnet';
	let workflowStatus = 'Ready to prepare a workflow.';
	let runs: WorkflowRun[] = [];

	onMount(async () => {
		tools = await invoke<ToolManifest[]>('list_builtin_tools');
		runs = await invoke<WorkflowRun[]>('list_workflow_runs');
	});

	async function createWorkflow() {
		const run = await invoke<WorkflowRun>('create_workflow_run', {
			request: {
				name: workflowName,
				modelName,
				toolIds: selectedTools
			}
		});

		runs = [run, ...runs];
		workflowStatus = `Workflow prepared. Config saved to ${run.configPath}`;
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
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Compose a real multi-tool Ralph loop.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			The final composer will persist a combined manifest, write `STATE.md`, and run a durable
			overnight chain in the background.
		</p>
	</div>

	<div class="grid gap-4 lg:grid-cols-[1.25fr_0.9fr]">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Example chain</h2>
			<div class="mt-6 space-y-3">
				{#each flow as step, index}
					<div class="flex items-center gap-4 rounded-2xl border border-white/6 bg-white/3 p-4">
						<div class="flex h-8 w-8 items-center justify-center rounded-full bg-violet-400/15 text-sm font-semibold text-violet-100">
							{index + 1}
						</div>
						<p class="text-sm text-slate-300">{step}</p>
					</div>
				{/each}
			</div>
		</div>

		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Workflow composer</h2>
			<div class="mt-4 space-y-4">
				<input
					bind:value={workflowName}
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white"
					placeholder="Workflow name"
				/>
				<input
					bind:value={modelName}
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white"
					placeholder="Model name"
				/>
				<div class="space-y-2">
					{#each tools as tool}
						<label class="flex items-center gap-3 rounded-2xl border border-white/8 bg-white/3 px-4 py-3 text-sm text-slate-300">
							<input
								type="checkbox"
								checked={selectedTools.includes(tool.id)}
								on:change={() => toggleTool(tool.id)}
							/>
							<span>{tool.name}</span>
						</label>
					{/each}
				</div>
				<button
					type="button"
					on:click={createWorkflow}
					class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950"
				>
					Create Combined Workflow
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
						<p class="text-sm font-medium text-white">{run.workflowName}</p>
						<p class="mt-1 text-xs text-slate-500">{run.configPath}</p>
						<p class="mt-2 text-xs text-cyan-100">{run.status}</p>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
