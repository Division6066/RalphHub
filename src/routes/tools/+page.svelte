<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import ModelSwitcher from '$lib/components/ModelSwitcher.svelte';
	import {
		loadProviders,
		activeModelStore,
		activeProviderIdStore,
		getProviderForTool,
		logApiUsage,
		createKaizenTask,
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
		category: string;
		parallelCapable: boolean;
		installMethod: string;
		tags: string[];
	};

	type ToolProcessStatus = {
		toolId: string;
		name: string;
		status: string;
		pid?: number;
		startedAt?: string;
		logPath?: string;
	};

	type ToolLogsResult = {
		toolId: string;
		logPath: string;
		lines: string[];
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
	let processStatuses: Record<string, ToolProcessStatus> = {};
	let logsOpen: Record<string, boolean> = {};
	let toolLogs: Record<string, string[]> = {};
	let pollingInterval: ReturnType<typeof setInterval> | null = null;

	// Filter/category state
	let categoryFilter = 'all';
	$: categories = ['all', ...new Set(tools.map((t) => t.category))];
	$: filteredTools = categoryFilter === 'all'
		? tools
		: tools.filter((t) => t.category === categoryFilter);

	onMount(async () => {
		await loadProviders();

		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}

		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		loading = false;

		// Start polling process statuses every 3 seconds
		pollingInterval = setInterval(() => refreshStatuses(), 3000);
	});

	onDestroy(() => {
		if (pollingInterval) clearInterval(pollingInterval);
	});

	async function refreshStatuses() {
		if (!isDesktopRuntime()) return;
		const running = await invokeTauri<ToolProcessStatus[]>('list_running_tools').catch(() => []);
		const map: Record<string, ToolProcessStatus> = {};
		for (const s of running) map[s.toolId] = s;
		processStatuses = map;
	}

	async function launchTool(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			status = `${tool.name} is an internal RalphHub capability.`;
			return;
		}

		busyTool = tool.id;
		status = `Deploying ${tool.name}...`;

		try {
			await invokeTauri('ensure_bun');

			// For uv-based tools, we wrap with bun script
			const result = await invokeTauri<DeployResult>('deploy_to_pc', {
				request: { url: tool.repoUrl }
			});

			// Auto-inject all enabled provider keys + tool-specific keys
			const allEnvKeys = $enabledProvidersStore.map((p) => p.apiKeyEnv).filter(Boolean);
			const keyValues = await loadDynamicKeys(allEnvKeys);
			const envEntries = Object.entries(_buildEnvInjection($enabledProvidersStore, keyValues))
				.map(([key, value]) => ({ key, value }));

			if (envEntries.length > 0) {
				await invokeTauri('inject_keys', {
					request: { workspacePath: result.workspacePath, entries: envEntries }
				});
			}

			await invokeTauri('open_in_code', {
				workspacePath: result.workspacePath,
				branch: result.branch
			});

			// Log to Memory Spine
			const injectedProvider = getProviderForTool($providersStore, tool.requiredKeys ?? []);
			if (injectedProvider && $activeModelStore) {
				await logApiUsage({
					providerId: injectedProvider.id,
					providerName: injectedProvider.name,
					model: $activeModelStore,
					tokensIn: 0,
					tokensOut: 0,
					costUsd: 0,
					outputSummary: `Tool "${tool.name}" launched with provider ${injectedProvider.name} | install: ${tool.installMethod} | keys injected`,
					toolId: tool.id,
					workflowId: ''
				});
			}

			// Auto-create Kaizen Task
			await createKaizenTask({
				title: `[Tool Launch] ${tool.name}`,
				description: `Launched ${tool.name} (${tool.category}) from ${tool.repoUrl}. Install method: ${tool.installMethod}. Keys injected.`,
				priority: 'normal',
				source: 'tools-page',
				providerId: injectedProvider?.id ?? '',
				usageLogId: ''
			});

			status = `${tool.name} ready — keys injected, opened in editor.`;
		} catch (error) {
			status = error instanceof Error ? error.message : `Failed to launch ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}

	async function launchBackground(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			status = `${tool.name} is an internal capability.`;
			return;
		}
		busyTool = tool.id;
		status = `Launching ${tool.name} in background...`;

		try {
			// Get workspace path from deployed repos (use repos_dir/slug)
			const repoSlug = tool.repoUrl.split('/').pop() ?? tool.id;
			const workspacePath = await getWorkspacePath(repoSlug);

			const allEnvKeys = $enabledProvidersStore.map((p) => p.apiKeyEnv).filter(Boolean);
			const keyValues = await loadDynamicKeys(allEnvKeys);
			const envEntries = Object.entries(_buildEnvInjection($enabledProvidersStore, keyValues))
				.map(([key, value]) => ({ key, value }));

			const ps = await invokeTauri<ToolProcessStatus>('launch_tool_background', {
				request: { toolId: tool.id, workspacePath, envEntries }
			});

			processStatuses = { ...processStatuses, [tool.id]: ps };

			// Log to Memory Spine + Kaizen
			await logApiUsage({
				providerId: 'background-launcher',
				providerName: 'Background Launcher',
				model: 'process',
				tokensIn: 0,
				tokensOut: 0,
				costUsd: 0,
				outputSummary: `Background launch: ${tool.name} pid=${ps.pid ?? 'n/a'} at ${workspacePath}`,
				toolId: tool.id,
				workflowId: ''
			});

			status = `${tool.name} running in background (pid: ${ps.pid ?? 'n/a'}).`;
		} catch (error) {
			status = error instanceof Error ? error.message : `Failed to background-launch ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}

	async function stopTool(tool: ToolManifest) {
		busyTool = tool.id;
		try {
			await invokeTauri('stop_tool_process', { toolId: tool.id });
			processStatuses = { ...processStatuses };
			delete processStatuses[tool.id];
			processStatuses = processStatuses;
			status = `${tool.name} stopped.`;
		} catch (error) {
			status = error instanceof Error ? error.message : `Failed to stop ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}

	async function viewLogs(tool: ToolManifest) {
		try {
			const result = await invokeTauri<ToolLogsResult>('get_tool_logs', {
				toolId: tool.id,
				tailLines: 80
			});
			toolLogs = { ...toolLogs, [tool.id]: result.lines };
			logsOpen = { ...logsOpen, [tool.id]: !logsOpen[tool.id] };
		} catch (error) {
			toolLogs = { ...toolLogs, [tool.id]: [`Error reading logs: ${error}`] };
			logsOpen = { ...logsOpen, [tool.id]: true };
		}
	}

	async function getWorkspacePath(repoSlug: string): Promise<string> {
		// Get the repos directory from the snapshot and construct the path
		if (!isDesktopRuntime()) return `/tmp/${repoSlug}`;
		try {
			const snapshot = await invokeTauri<{ paths: { reposDir: string } }>('get_dashboard_snapshot');
			return `${snapshot.paths.reposDir}/${repoSlug}`;
		} catch {
			return `/tmp/${repoSlug}`;
		}
	}

	function getToolProviderInfo(tool: ToolManifest) {
		if (!tool.requiredKeys?.length) return null;
		return getProviderForTool($providersStore, tool.requiredKeys);
	}

	function getProcessStatus(toolId: string): ToolProcessStatus | null {
		return processStatuses[toolId] ?? null;
	}

	function statusColor(s: string | undefined) {
		if (!s) return 'text-slate-500';
		if (s === 'running') return 'text-green-400';
		if (s === 'stopped') return 'text-amber-400';
		if (s?.startsWith('error')) return 'text-red-400';
		return 'text-slate-400';
	}

	function statusDot(s: string | undefined) {
		if (!s || s === 'idle') return '⚪';
		if (s === 'running') return '🟢';
		if (s === 'stopped') return '🟡';
		return '🔴';
	}

	const CATEGORY_ICONS: Record<string, string> = {
		coding: '💻',
		video: '🎬',
		research: '🔍',
		orchestration: '🎛️',
		memory: '🧠',
		productivity: '📋',
		internal: '⚙️',
	};
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Tools</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">All tools — with universal provider injection.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Every tool auto-receives keys from the Provider Registry. Launch in foreground (deploy + open in editor) or background (parallel background process). Includes <span class="text-violet-300 font-semibold">Superpowers</span> agentic skills framework and <span class="text-pink-300 font-semibold">Diffusionstudio Agent</span> video editor.
		</p>
		<div class="mt-6 flex items-center gap-3 flex-wrap">
			<span class="text-sm text-slate-400">Active model:</span>
			<ModelSwitcher compact />
			{#if $enabledProvidersStore.length > 0}
				<span class="text-xs text-green-400">{$enabledProvidersStore.length} providers active</span>
			{:else}
				<a href="/settings" class="text-xs text-amber-400 underline hover:text-amber-300">Connect providers →</a>
			{/if}
			<a href="/parallel" class="ml-auto rounded-full border border-violet-400/30 bg-violet-400/10 px-4 py-1.5 text-xs font-semibold text-violet-200 hover:bg-violet-400/20">
				⚡ Parallel Workflow →
			</a>
		</div>
	</div>

	<!-- Status bar -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-4 text-sm text-slate-300 backdrop-blur">
		{#if loading}
			<span class="animate-pulse">Loading tool manifests...</span>
		{:else}
			{status || 'Choose a tool to deploy + inject keys, or run in background for parallel workflows.'}
		{/if}
	</div>

	<!-- Category filter -->
	<div class="flex flex-wrap gap-2">
		{#each categories as cat}
			<button
				type="button"
				on:click={() => (categoryFilter = cat)}
				class="rounded-full px-3 py-1 text-xs font-medium transition-colors {categoryFilter === cat ? 'bg-cyan-400/20 text-cyan-200 border border-cyan-400/30' : 'border border-white/10 text-slate-400 hover:text-white'}"
			>
				{cat === 'all' ? '🔠 All' : `${CATEGORY_ICONS[cat] ?? '🔧'} ${cat}`}
			</button>
		{/each}
	</div>

	<div class="grid gap-4 xl:grid-cols-3">
		{#each filteredTools as tool}
			{@const injectedProvider = getToolProviderInfo(tool)}
			{@const ps = getProcessStatus(tool.id)}
			{@const isNew = tool.id === 'superpowers' || tool.id === 'diffusionstudio-agent'}
			<div class="rounded-3xl border {isNew ? 'border-violet-400/25 bg-gradient-to-b from-slate-950/60 to-violet-950/20' : 'border-white/10 bg-slate-950/45'} p-6 backdrop-blur">
				<!-- Header -->
				<div class="flex items-start justify-between gap-3">
					<div class="flex-1 min-w-0">
						<div class="flex items-center gap-2 flex-wrap">
							<h2 class="text-lg font-semibold text-white truncate">{tool.name}</h2>
							{#if isNew}
								<span class="rounded-full bg-violet-500/20 border border-violet-400/30 px-2 py-0.5 text-xs font-bold text-violet-300">NEW</span>
							{/if}
							{#if tool.parallelCapable}
								<span class="rounded-full bg-cyan-500/10 border border-cyan-400/20 px-2 py-0.5 text-xs text-cyan-400" title="Parallel execution capable">⚡ parallel</span>
							{/if}
						</div>
						<div class="mt-1 flex items-center gap-2 flex-wrap">
							<span class="rounded-md bg-slate-800/70 px-1.5 py-0.5 text-xs text-slate-400">
								{CATEGORY_ICONS[tool.category] ?? '🔧'} {tool.category}
							</span>
							{#if tool.installMethod !== 'internal'}
								<span class="rounded-md bg-slate-800/70 px-1.5 py-0.5 text-xs text-slate-500">
									via {tool.installMethod}
								</span>
							{/if}
						</div>
					</div>
					<!-- Process status indicator -->
					<div class="flex flex-col items-end gap-1 flex-shrink-0">
						<span class="text-lg" title={ps?.status ?? 'idle'}>{statusDot(ps?.status)}</span>
						{#if ps}
							<span class="text-xs {statusColor(ps.status)}">{ps.status}</span>
						{/if}
					</div>
				</div>

				<p class="mt-4 text-sm leading-6 text-slate-400 line-clamp-3">{tool.description}</p>

				<!-- Tags -->
				{#if tool.tags?.length}
					<div class="mt-3 flex flex-wrap gap-1">
						{#each tool.tags.slice(0, 4) as tag}
							<span class="rounded-md bg-slate-900/60 px-1.5 py-0.5 text-xs text-slate-500">{tag}</span>
						{/each}
					</div>
				{/if}

				<!-- Provider injection indicator -->
				{#if tool.requiredKeys?.length}
					<div class="mt-3 flex items-center gap-2 flex-wrap">
						{#if injectedProvider}
							<span class="rounded-full bg-green-500/15 border border-green-400/20 px-2 py-0.5 text-xs text-green-400">
								{injectedProvider.logoEmoji} {injectedProvider.name}
							</span>
						{:else}
							<span class="rounded-full bg-amber-500/10 border border-amber-400/20 px-2 py-0.5 text-xs text-amber-400">
								⚠ No key — fallback to Ollama
							</span>
						{/if}
						<span class="text-xs text-slate-600">{tool.requiredKeys.join(', ')}</span>
					</div>
				{/if}

				<!-- Process status detail -->
				{#if ps && ps.status === 'running'}
					<div class="mt-3 rounded-xl border border-green-400/15 bg-green-400/5 p-2.5 text-xs">
						<p class="text-green-300 font-medium">Running — pid {ps.pid}</p>
						{#if ps.startedAt}
							<p class="text-slate-500 mt-0.5">Started: {new Date(ps.startedAt).toLocaleTimeString()}</p>
						{/if}
					</div>
				{/if}

				<!-- Action buttons: Launch / Status / Logs / Pause / Open in Code -->
				<div class="mt-5 flex flex-wrap gap-2 text-sm">
					<!-- Launch (foreground) -->
					{#if !tool.repoUrl.startsWith('internal://')}
						<button
							type="button"
							on:click={() => launchTool(tool)}
							disabled={busyTool === tool.id}
							class="rounded-full bg-cyan-400/12 px-3.5 py-1.5 font-medium text-cyan-100 hover:bg-cyan-400/20 disabled:cursor-not-allowed disabled:opacity-60 transition-colors"
							title="Deploy repo, inject keys, open in editor"
						>
							{busyTool === tool.id ? '...' : '▶ Launch'}
						</button>

						<!-- Background launch (parallel-capable only) -->
						{#if tool.parallelCapable}
							<button
								type="button"
								on:click={() => launchBackground(tool)}
								disabled={busyTool === tool.id || ps?.status === 'running'}
								class="rounded-full bg-violet-400/12 px-3.5 py-1.5 font-medium text-violet-200 hover:bg-violet-400/20 disabled:cursor-not-allowed disabled:opacity-60 transition-colors"
								title="Launch as background process (parallel execution)"
							>
								⚡ Background
							</button>
						{/if}

						<!-- Pause/Stop -->
						{#if ps?.status === 'running'}
							<button
								type="button"
								on:click={() => stopTool(tool)}
								disabled={busyTool === tool.id}
								class="rounded-full bg-red-400/12 px-3.5 py-1.5 font-medium text-red-300 hover:bg-red-400/20 disabled:opacity-60 transition-colors"
								title="Stop background process"
							>
								⏹ Pause
							</button>
						{/if}

						<!-- Logs -->
						<button
							type="button"
							on:click={() => viewLogs(tool)}
							class="rounded-full border border-white/10 px-3.5 py-1.5 font-medium text-slate-300 hover:bg-white/5 transition-colors"
							title="View last 80 lines of process log"
						>
							📋 Logs
						</button>
					{/if}

					<!-- Open in Code -->
					{#if tool.openInCode}
						<a
							href="/workflows"
							class="rounded-full border border-white/10 px-3.5 py-1.5 font-medium text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
							title="Compose this tool in a workflow"
						>
							🔗 Compose
						</a>
					{/if}
				</div>

				<!-- Log viewer (collapsible) -->
				{#if logsOpen[tool.id]}
					<div class="mt-4 rounded-2xl border border-white/8 bg-black/40 p-3">
						<div class="flex items-center justify-between mb-2">
							<span class="text-xs font-mono text-slate-400">Logs (last 80 lines)</span>
							<button
								type="button"
								on:click={() => (logsOpen = { ...logsOpen, [tool.id]: false })}
								class="text-xs text-slate-600 hover:text-slate-400"
							>close ✕</button>
						</div>
						<div class="max-h-48 overflow-y-auto space-y-0.5">
							{#each (toolLogs[tool.id] ?? ['No logs available.']) as line}
								<p class="text-xs font-mono text-slate-400 leading-5">{line}</p>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		{/each}
	</div>
</section>
