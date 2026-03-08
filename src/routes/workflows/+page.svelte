<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type ToolManifest = {
		id: string;
		name: string;
		supportsOllama: boolean;
		supportsVoice: boolean;
	};

	type WorkflowRun = {
		id: string;
		workflowName: string;
		modelName: string;
		configPath: string;
		statePath: string;
		status: string;
	};

	// Preset workflow templates (Milestone 8)
	const voiceFullStackFlow = [
		{ step: 1, label: 'Voice input', detail: 'Whisper captures voice command → text prompt' },
		{ step: 2, label: 'Research', detail: 'Perplexica/llm-council multi-source research' },
		{ step: 3, label: 'Task creation', detail: 'Kaizen Tasks: auto-create tasks from findings' },
		{ step: 4, label: 'MCP browser', detail: 'Playwright MCP executes browser actions' },
		{ step: 5, label: 'Memory write', detail: 'Memory Spine records results + evidence' },
		{ step: 6, label: 'Notion sync', detail: 'Write summary to Notion via API' },
		{ step: 7, label: 'Voice response', detail: 'Piper TTS speaks back the final summary' }
	];

	const presetWorkflows = [
		{
			id: 'voice-full-stack',
			name: 'Voice + Full Stack',
			description: 'Voice command → Research → Tasks → Browser → Memory → Notion → Voice response',
			tools: ['perplexica', 'llm-council', 'kaizen-tasks', 'browser-agent', 'memory-spine'],
			model: 'ollama/mistral',
			steps: voiceFullStackFlow
		},
		{
			id: 'overnight-ralph',
			name: 'Overnight Ralph Loop',
			description: 'Multi-tool overnight execution with memory writes and task tracking',
			tools: ['get-shit-done', 'perplexica', 'llm-council', 'memory-spine', 'kaizen-tasks'],
			model: 'ollama/mistral',
			steps: [
				{ step: 1, label: 'Research phase', detail: 'Perplexica + llm-council gather context' },
				{ step: 2, label: 'Execute tasks', detail: 'get-shit-done runs Ralph loop overnight' },
				{ step: 3, label: 'Memory write', detail: 'Memory Spine captures all run reports' },
				{ step: 4, label: 'Task update', detail: 'Kaizen Tasks updated with results' }
			]
		},
		{
			id: 'research-deep-dive',
			name: 'Deep Research',
			description: 'AutoResearch + DeerFlow + Perplexica for thorough iterative research',
			tools: ['autoresearch', 'deer-flow', 'perplexica', 'memory-spine'],
			model: 'ollama/qwen2.5:3b',
			steps: [
				{ step: 1, label: 'Seed query', detail: 'User provides initial research topic' },
				{ step: 2, label: 'Multi-source', detail: 'Perplexica searches web + docs' },
				{ step: 3, label: 'Deep dive', detail: 'DeerFlow iterates and refines' },
				{ step: 4, label: 'Synthesis', detail: 'llm-council votes on best findings' },
				{ step: 5, label: 'Export', detail: 'MarkItDown converts results to Markdown' }
			]
		}
	];

	let tools: ToolManifest[] = [];
	let selectedTools: string[] = ['perplexica', 'llm-council', 'get-shit-done', 'memory-spine'];
	let workflowName = 'Overnight Ralph Chain';
	let modelName = 'ollama/mistral';
	let workflowStatus = '';
	let runs: WorkflowRun[] = [];
	let activePreset = '';
	let voiceEnabled = false;

	const ollamaModels = [
		'ollama/mistral',
		'ollama/qwen2.5:3b',
		'ollama/llama3.2:3b',
		'ollama/phi3.5',
		'anthropic/claude-sonnet',
		'openai/gpt-4o',
		'openai/gpt-4o-mini'
	];

	onMount(async () => {
		if (!isDesktopRuntime()) return;
		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		runs = await invokeTauri<WorkflowRun[]>('list_workflow_runs');
	});

	function applyPreset(preset: typeof presetWorkflows[0]) {
		workflowName = preset.name;
		modelName = preset.model;
		selectedTools = preset.tools;
		activePreset = preset.id;
	}

	async function createWorkflow() {
		if (!isDesktopRuntime()) {
			workflowStatus = 'Available in RalphHub desktop runtime.';
			return;
		}

		workflowStatus = 'Preparing workflow...';
		try {
			const run = await invokeTauri<WorkflowRun>('create_workflow_run', {
				request: { name: workflowName, modelName, toolIds: selectedTools }
			});

			// Write to Memory Spine automatically
			try {
				await invokeTauri('write_memory_entry', {
					toolId: 'workflow-composer',
					entryType: 'report',
					content: `Workflow "${workflowName}" prepared with tools: ${selectedTools.join(', ')} using model ${modelName}.`,
					tags: 'workflow,auto'
				});
			} catch {}

			runs = [run, ...runs];
			workflowStatus = `Workflow prepared: ${run.id}`;
		} catch (e) {
			workflowStatus = e instanceof Error ? e.message : 'Failed to create workflow.';
		}
	}

	function toggleTool(id: string) {
		selectedTools = selectedTools.includes(id)
			? selectedTools.filter((t) => t !== id)
			: [...selectedTools, id];
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-violet-400/20 bg-slate-950/55 p-8 shadow-2xl shadow-violet-950/20 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-violet-200/80">Workflows</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Multi-tool overnight Ralph loops.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Choose a preset or compose your own. All workflows write to Memory Spine and Kaizen Tasks
			automatically. Voice input and local Ollama models supported by default.
		</p>
	</div>

	{#if workflowStatus}
		<div class="rounded-3xl border border-violet-400/20 bg-violet-500/10 p-4 text-sm text-violet-100">
			{workflowStatus}
		</div>
	{/if}

	<!-- Preset templates (Milestone 8) -->
	<div class="space-y-3">
		<h2 class="text-lg font-semibold text-white">Preset workflows</h2>
		<div class="grid gap-4 xl:grid-cols-3">
			{#each presetWorkflows as preset}
				<button
					on:click={() => applyPreset(preset)}
					class="rounded-3xl border text-left p-6 backdrop-blur transition {
						activePreset === preset.id
							? 'border-cyan-400/30 bg-cyan-500/10'
							: 'border-white/10 bg-slate-950/45 hover:border-white/20'
					}"
				>
					<div class="flex items-start justify-between gap-2">
						<h3 class="text-sm font-semibold text-white">{preset.name}</h3>
						{#if activePreset === preset.id}
							<span class="rounded-full bg-cyan-400/15 px-2 py-0.5 text-xs text-cyan-300">selected</span>
						{/if}
					</div>
					<p class="mt-2 text-xs text-slate-400 leading-5">{preset.description}</p>
					<div class="mt-3 flex flex-wrap gap-1">
						{#each preset.tools as t}
							<span class="rounded-full border border-white/8 px-2 py-0.5 text-xs text-slate-500">{t}</span>
						{/each}
					</div>
				</button>
			{/each}
		</div>
	</div>

	<!-- Voice + Full Stack flow visualization -->
	{#if activePreset === 'voice-full-stack'}
		<div class="rounded-3xl border border-cyan-400/20 bg-cyan-500/8 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-white">Voice + Full Stack Flow</h2>
				<button
					on:click={() => (voiceEnabled = !voiceEnabled)}
					class="flex items-center gap-2 rounded-full border px-4 py-2 text-sm transition {voiceEnabled ? 'border-cyan-400/40 bg-cyan-500/15 text-cyan-200' : 'border-white/10 text-slate-400'}"
				>
					{voiceEnabled ? 'Voice ON' : 'Enable voice input'}
				</button>
			</div>

			{#if voiceEnabled}
				<div class="mt-3 rounded-2xl border border-cyan-400/30 bg-cyan-500/10 p-3 text-sm text-cyan-200">
					Mic active — speak your research query. faster-whisper will transcribe it.
				</div>
			{/if}

			<div class="mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
				{#each voiceFullStackFlow as step}
					<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex items-center gap-3">
							<div class="flex h-7 w-7 items-center justify-center rounded-full bg-violet-400/15 text-xs font-semibold text-violet-200">
								{step.step}
							</div>
							<p class="text-sm font-medium text-white">{step.label}</p>
						</div>
						<p class="mt-2 text-xs text-slate-400">{step.detail}</p>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<div class="grid gap-4 lg:grid-cols-[1.3fr_0.9fr]">
		<!-- Composer -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Workflow composer</h2>
			<div class="mt-4 space-y-4">
				<input
					bind:value={workflowName}
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white outline-none placeholder:text-slate-600"
					placeholder="Workflow name"
				/>
				<select
					bind:value={modelName}
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white outline-none"
				>
					{#each ollamaModels as model}
						<option value={model}>{model}</option>
					{/each}
				</select>
				<div class="max-h-64 overflow-y-auto space-y-1.5 pr-1">
					{#each tools as tool}
						<label class="flex cursor-pointer items-center gap-3 rounded-xl border border-white/6 bg-white/3 px-3 py-2 text-sm text-slate-300 hover:bg-white/5">
							<input
								type="checkbox"
								checked={selectedTools.includes(tool.id)}
								on:change={() => toggleTool(tool.id)}
								class="accent-cyan-400"
							/>
							<span class="flex-1">{tool.name}</span>
							{#if tool.supportsOllama}
								<span class="text-xs text-violet-400">ollama</span>
							{/if}
							{#if tool.supportsVoice}
								<span class="text-xs text-cyan-400">voice</span>
							{/if}
						</label>
					{/each}
				</div>
				<button
					on:click={createWorkflow}
					class="w-full rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg"
				>
					Create Workflow
				</button>
			</div>
		</div>

		<!-- Runs -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-white">Prepared runs</h2>
				<span class="text-sm text-slate-500">{runs.length}</span>
			</div>
			<div class="mt-4 space-y-3">
				{#if !runs.length}
					<p class="text-sm text-slate-500">No runs yet. Create one from a preset or composer.</p>
				{:else}
					{#each runs as run}
						<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
							<p class="text-sm font-medium text-white">{run.workflowName}</p>
							<p class="mt-1 text-xs text-slate-500 font-mono truncate">{run.configPath}</p>
							<div class="mt-2 flex items-center gap-2">
								<span class="rounded-full bg-cyan-400/15 px-2 py-0.5 text-xs text-cyan-300">{run.status}</span>
								<span class="text-xs text-slate-500">{run.modelName}</span>
							</div>
						</div>
					{/each}
				{/if}
			</div>
		</div>
	</div>
</section>
