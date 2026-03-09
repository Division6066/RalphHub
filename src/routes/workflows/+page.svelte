<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type ToolManifest = {
		id: string;
		name: string;
		category: string;
		repoUrl: string;
		tags: string[];
	};

	type WorkflowRun = {
		id: string;
		workflowName: string;
		modelName: string;
		configPath: string;
		statePath: string;
		status: string;
		createdAt: string;
	};

	const PRESET_CHAINS = [
		{
			name: 'Deep Research Loop',
			icon: '🔬',
			desc: 'Perplexica searches → AutoResearch iterates → Memory Spine saves',
			tools: ['perplexica', 'autoresearch', 'memory-spine'],
			model: 'anthropic/claude-sonnet-4-5',
			color: 'violet'
		},
		{
			name: 'Full Coding Agent',
			icon: '💻',
			desc: 'OpenHands plans → Aider implements → get-shit-done ships',
			tools: ['open-hands', 'aider', 'get-shit-done'],
			model: 'anthropic/claude-sonnet-4-5',
			color: 'cyan'
		},
		{
			name: 'Web Scrape + Analyze',
			icon: '🌐',
			desc: 'Playwright MCP browses → Firecrawl extracts → LiteLLM analyzes',
			tools: ['playwright-mcp', 'firecrawl', 'litellm'],
			model: 'openai/gpt-4o',
			color: 'blue'
		},
		{
			name: 'Multi-Model Council',
			icon: '🧠',
			desc: 'LLM Council votes across GPT-4o + Claude + Gemini on hard decisions',
			tools: ['llm-council', 'litellm'],
			model: 'council/all',
			color: 'purple'
		},
		{
			name: 'Overnight Kaizen',
			icon: '♾️',
			desc: 'Analyze Kaizen backlog → decompose tasks → write memory → notify',
			tools: ['memory-spine', 'universal-ai-loop'],
			model: 'anthropic/claude-sonnet-4-5',
			color: 'amber'
		},
	];

	const ALL_MODELS = [
		'anthropic/claude-sonnet-4-5',
		'anthropic/claude-opus-4-5',
		'anthropic/claude-haiku-3-5',
		'openai/gpt-4o',
		'openai/gpt-4o-mini',
		'openai/o1',
		'openai/o3-mini',
		'google/gemini-1.5-pro',
		'google/gemini-1.5-flash',
		'xai/grok-2',
		'deepseek/deepseek-r1',
		'mistral/mistral-large',
		'groq/llama-3.3-70b',
		'council/all',
		'local/ollama',
	];

	let tools: ToolManifest[] = [];
	let runs: WorkflowRun[] = [];
	let loading = true;
	let selectedTools: string[] = [];
	let workflowName = 'My AI Workflow';
	let modelName = 'anthropic/claude-sonnet-4-5';
	let workflowDesc = '';
	let status = '';
	let isDesktop = false;

	onMount(async () => {
		isDesktop = isDesktopRuntime();
		if (!isDesktop) { loading = false; return; }
		try {
			[tools, runs] = await Promise.all([
				invokeTauri<ToolManifest[]>('list_builtin_tools'),
				invokeTauri<WorkflowRun[]>('list_workflow_runs')
			]);
		} catch (e) { status = String(e); }
		finally { loading = false; }
	});

	async function createWorkflow() {
		if (!workflowName.trim() || !selectedTools.length) {
			status = 'Enter a name and select at least one tool.';
			return;
		}
		if (!isDesktop) { status = 'Desktop runtime required.'; return; }
		try {
			const run = await invokeTauri<WorkflowRun>('create_workflow_run', {
				request: {
					name: workflowName,
					modelName,
					toolIds: selectedTools,
					description: workflowDesc || null
				}
			});
			runs = [run, ...runs];
			status = `✓ Workflow created. Config: ${run.configPath}`;
		} catch (e) {
			status = String(e);
		}
	}

	function applyPreset(preset: typeof PRESET_CHAINS[0]) {
		workflowName = preset.name;
		modelName = preset.model;
		selectedTools = preset.tools;
		workflowDesc = preset.desc;
		status = `Preset "${preset.name}" loaded. Customize and create.`;
	}

	function toggleTool(id: string) {
		selectedTools = selectedTools.includes(id)
			? selectedTools.filter((t) => t !== id)
			: [...selectedTools, id];
	}

	function formatDate(iso: string) {
		return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
	}

	const colorClass = (c: string) => {
		const m: Record<string, string> = {
			violet: 'border-violet-400/20 bg-violet-400/8 hover:border-violet-400/40',
			cyan: 'border-cyan-400/20 bg-cyan-400/8 hover:border-cyan-400/40',
			blue: 'border-blue-400/20 bg-blue-400/8 hover:border-blue-400/40',
			purple: 'border-purple-400/20 bg-purple-400/8 hover:border-purple-400/40',
			amber: 'border-amber-400/20 bg-amber-400/8 hover:border-amber-400/40',
		};
		return m[c] ?? 'border-white/10 bg-white/5';
	};
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-violet-400/20 bg-gradient-to-br from-violet-950/50 via-slate-950/80 to-purple-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-violet-300/70">Multi-Tool Orchestration</p>
		<h1 class="mt-2 text-3xl font-bold text-white">⚡ Workflows</h1>
		<p class="mt-2 text-sm text-slate-400">Chain AI tools into overnight loops. Research → Code → Memory → Notify.</p>
	</div>

	<!-- Preset chains -->
	<div>
		<h2 class="mb-3 text-sm font-bold text-slate-300">Quick-Start Presets</h2>
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
			{#each PRESET_CHAINS as preset}
				<button
					onclick={() => applyPreset(preset)}
					class={`rounded-xl border p-4 text-left transition ${colorClass(preset.color)}`}
				>
					<p class="text-lg mb-1.5">{preset.icon}</p>
					<p class="text-sm font-bold text-white">{preset.name}</p>
					<p class="mt-1.5 text-xs text-slate-400 leading-5">{preset.desc}</p>
					<div class="mt-2.5 flex flex-wrap gap-1">
						{#each preset.tools as t}
							<span class="rounded-full bg-white/8 px-2 py-0.5 text-[10px] text-slate-400">{t}</span>
						{/each}
					</div>
				</button>
			{/each}
		</div>
	</div>

	<!-- Composer -->
	<div class="grid gap-5 lg:grid-cols-[1fr_1.2fr]">
		<!-- Tool selector -->
		<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
			<h2 class="mb-4 text-sm font-bold text-white">Select Tools ({selectedTools.length} selected)</h2>
			<div class="max-h-80 overflow-y-auto space-y-1.5 pr-1">
				{#each tools.filter(t => !t.repoUrl.startsWith('internal://') || ['memory-spine', 'universal-ai-loop'].includes(t.id)) as tool}
					<label class={`flex cursor-pointer items-center gap-3 rounded-xl border px-3 py-2.5 transition ${selectedTools.includes(tool.id) ? 'border-violet-400/30 bg-violet-400/10' : 'border-white/5 bg-white/2 hover:border-white/10'}`}>
						<input type="checkbox" checked={selectedTools.includes(tool.id)} onchange={() => toggleTool(tool.id)} class="h-4 w-4 rounded" />
						<div class="flex-1 min-w-0">
							<p class="text-sm font-medium text-white truncate">{tool.name}</p>
							<p class="text-xs text-slate-500">{tool.category}</p>
						</div>
					</label>
				{/each}
			</div>
		</div>

		<!-- Config + create -->
		<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
			<h2 class="mb-4 text-sm font-bold text-white">Workflow Config</h2>
			<div class="space-y-4">
				<div>
					<label class="mb-1.5 block text-xs text-slate-400">Name</label>
					<input bind:value={workflowName} class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-2.5 text-sm text-white outline-none focus:border-violet-400" placeholder="My workflow name" />
				</div>
				<div>
					<label class="mb-1.5 block text-xs text-slate-400">Model</label>
					<select bind:value={modelName} class="w-full rounded-xl border border-white/10 bg-slate-800 px-3 py-2.5 text-sm text-white">
						{#each ALL_MODELS as m}
							<option value={m}>{m}</option>
						{/each}
					</select>
				</div>
				<div>
					<label class="mb-1.5 block text-xs text-slate-400">Description (optional)</label>
					<textarea bind:value={workflowDesc} rows="2" class="w-full resize-none rounded-xl border border-white/10 bg-slate-800 px-4 py-2.5 text-sm text-white outline-none focus:border-violet-400"></textarea>
				</div>

				<!-- Selected tools preview -->
				{#if selectedTools.length > 0}
					<div class="rounded-xl border border-violet-400/15 bg-violet-400/8 p-3">
						<p class="text-xs text-violet-300 font-semibold mb-2">Chain ({selectedTools.length} tools):</p>
						<div class="flex flex-wrap gap-1.5">
							{#each selectedTools as toolId, i}
								<div class="flex items-center gap-1">
									<span class="rounded-lg bg-violet-400/20 px-2 py-1 text-xs font-medium text-violet-200">{toolId}</span>
									{#if i < selectedTools.length - 1}
										<span class="text-slate-600 text-xs">→</span>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<button
					onclick={createWorkflow}
					disabled={!selectedTools.length || !workflowName.trim()}
					class="w-full rounded-xl bg-gradient-to-r from-violet-500 to-purple-500 py-3 text-sm font-bold text-white shadow-lg disabled:opacity-50 transition hover:from-violet-400 hover:to-purple-400"
				>
					⚡ Create Workflow
				</button>
				{#if status}
					<p class="text-xs text-slate-400">{status}</p>
				{/if}
			</div>
		</div>
	</div>

	<!-- Workflow runs history -->
	<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-sm font-bold text-white">Run History</h2>
			<span class="text-xs text-slate-500">{runs.length} runs</span>
		</div>
		{#if loading}
			<p class="text-sm text-slate-400">Loading…</p>
		{:else if runs.length === 0}
			<p class="text-sm text-slate-500">No workflows created yet. Use a preset above to get started.</p>
		{:else}
			<div class="space-y-2.5">
				{#each runs as run}
					<div class="flex items-start justify-between gap-4 rounded-xl border border-white/8 bg-white/3 p-4">
						<div>
							<p class="text-sm font-medium text-white">{run.workflowName}</p>
							<p class="mt-0.5 text-xs text-slate-500">{run.modelName} · {run.configPath}</p>
							<p class="mt-0.5 text-xs text-slate-600">{formatDate(run.createdAt)}</p>
						</div>
						<span class={`shrink-0 rounded-full border px-2.5 py-1 text-xs font-semibold ${run.status === 'prepared' ? 'border-amber-400/20 bg-amber-400/10 text-amber-300' : 'border-emerald-400/20 bg-emerald-400/10 text-emerald-300'}`}>
							{run.status}
						</span>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</section>
