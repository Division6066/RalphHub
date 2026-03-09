<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import ModelSwitcher from '$lib/components/ModelSwitcher.svelte';
	import {
		loadProviders,
		activeModelStore,
		activeProviderIdStore,
		logApiUsage,
		createKaizenTask,
		enabledProvidersStore,
		providersStore
	} from '$lib/utils/provider-registry';
	import { loadDynamicKeys } from '$lib/utils/secure-store';
	import { buildEnvInjection as _buildEnvInjection } from '$lib/utils/provider-registry';

	type ToolProcessStatus = {
		toolId: string;
		name: string;
		status: string;
		pid?: number;
		startedAt?: string;
		logPath?: string;
	};

	type ParallelWorkflowResult = {
		workflowId: string;
		workflowName: string;
		statuses: ToolProcessStatus[];
		memorySpineId: string;
		kaizenTaskId: string;
	};

	type ToolLogsResult = {
		toolId: string;
		logPath: string;
		lines: string[];
	};

	type VoiceCommandResult = {
		action: string;
		toolId?: string;
		message: string;
		success: boolean;
	};

	// The two parallel tools
	const SUPERPOWERS_ID = 'superpowers';
	const DIFFUSION_ID = 'diffusionstudio-agent';

	let workflowName = 'Code Feature + Edit Demo Video';
	let superpowersWorkspace = '';
	let diffusionWorkspace = '';
	let running = false;
	let lastResult: ParallelWorkflowResult | null = null;
	let statusMessage = '';
	let superpowersStatus: ToolProcessStatus | null = null;
	let diffusionStatus: ToolProcessStatus | null = null;
	let superpowersLogs: string[] = [];
	let diffusionLogs: string[] = [];
	let showSuperpowersLogs = false;
	let showDiffusionLogs = false;
	let pollingInterval: ReturnType<typeof setInterval> | null = null;

	// Voice command state
	let voiceTranscript = '';
	let voiceResult: VoiceCommandResult | null = null;
	let voiceListening = false;
	let voiceSupported = false;
	let recognition: unknown = null;

	// Workflow step log
	let workflowLog: string[] = [];

	// Past parallel workflow history
	type ParallelWorkflowRecord = {
		id: string;
		workflowName: string;
		toolIds: string;
		status: string;
		createdAt: string;
		memorySpineId: string;
		kaizenTaskId: string;
	};
	let pastWorkflows: ParallelWorkflowRecord[] = [];

	function log(msg: string) {
		workflowLog = [`[${new Date().toLocaleTimeString()}] ${msg}`, ...workflowLog.slice(0, 49)];
	}

	onMount(async () => {
		await loadProviders();
		voiceSupported = typeof window !== 'undefined' && ('SpeechRecognition' in window || 'webkitSpeechRecognition' in window);

		if (!isDesktopRuntime()) return;

		// Pre-fill workspace paths
		try {
			const snapshot = await invokeTauri<{ paths: { reposDir: string } }>('get_dashboard_snapshot');
			superpowersWorkspace = `${snapshot.paths.reposDir}/superpowers`;
			diffusionWorkspace = `${snapshot.paths.reposDir}/agent`;
		} catch { /* ignore */ }

		// Start polling
		pollingInterval = setInterval(() => pollStatuses(), 3000);
		pollStatuses();

		// Load past parallel workflows
		try {
			pastWorkflows = await invokeTauri<ParallelWorkflowRecord[]>('list_parallel_workflows');
		} catch { /* ignore */ }
	});

	onDestroy(() => {
		if (pollingInterval) clearInterval(pollingInterval);
	});

	async function pollStatuses() {
		if (!isDesktopRuntime()) return;
		try {
			const running: ToolProcessStatus[] = await invokeTauri('list_running_tools');
			superpowersStatus = running.find((s) => s.toolId === SUPERPOWERS_ID) ?? null;
			diffusionStatus = running.find((s) => s.toolId === DIFFUSION_ID) ?? null;
		} catch { /* ignore */ }
	}

	async function launchParallel() {
		if (!isDesktopRuntime()) {
			statusMessage = 'Parallel execution requires the RalphHub desktop runtime.';
			return;
		}

		running = true;
		statusMessage = 'Preparing parallel workflow...';
		log('Starting parallel workflow: Superpowers + Diffusionstudio Agent');

		try {
			// Build env entries from all enabled providers
			const allEnvKeys = $enabledProvidersStore.map((p) => p.apiKeyEnv).filter(Boolean);
			const keyValues = await loadDynamicKeys(allEnvKeys);
			const baseEnv = Object.entries(_buildEnvInjection($enabledProvidersStore, keyValues))
				.map(([key, value]) => ({ key, value }));

			// Superpowers-specific env
			const superpowersEnv = [
				...baseEnv,
				{ key: 'SUPERPOWERS_MODE', value: 'parallel-agents' },
				{ key: 'SUPERPOWERS_TDD', value: 'true' },
				{ key: 'SUPERPOWERS_WORKFLOW', value: 'brainstorm-plan-execute-review' },
				{ key: 'RALPHHUB_MEMORY_SPINE', value: 'true' },
			];

			// Diffusionstudio-specific env
			const diffusionEnv = [
				...baseEnv,
				{ key: 'DIFFUSION_BACKGROUND', value: 'true' },
				{ key: 'DIFFUSION_VY_PANDA', value: 'true' },
				{ key: 'DIFFUSION_OUTPUT_PATH', value: `${diffusionWorkspace}/output` },
				{ key: 'RALPHHUB_MEMORY_SPINE', value: 'true' },
			];

			log('Deploying superpowers...');
			statusMessage = 'Deploying superpowers repository...';

			// Deploy both if not already deployed
			await invokeTauri('ensure_bun').catch(() => null);

			let superpowersWs = superpowersWorkspace;
			let diffusionWs = diffusionWorkspace;

			if (!superpowersWs) {
				const r1 = await invokeTauri<{ workspacePath: string }>('deploy_to_pc', {
					request: { url: 'https://github.com/obra/superpowers' }
				});
				superpowersWs = r1.workspacePath;
				superpowersWorkspace = superpowersWs;
				log(`Superpowers deployed to: ${superpowersWs}`);
			}

			if (!diffusionWs) {
				log('Deploying diffusionstudio/agent...');
				statusMessage = 'Deploying diffusionstudio/agent repository...';
				const r2 = await invokeTauri<{ workspacePath: string }>('deploy_to_pc', {
					request: { url: 'https://github.com/diffusionstudio/agent' }
				});
				diffusionWs = r2.workspacePath;
				diffusionWorkspace = diffusionWs;
				log(`Diffusionstudio agent deployed to: ${diffusionWs}`);
			}

			// Inject keys for both
			if (baseEnv.length > 0) {
				await invokeTauri('inject_keys', {
					request: { workspacePath: superpowersWs, entries: superpowersEnv }
				}).catch((e: unknown) => log(`Key injection warning (superpowers): ${e}`));
				await invokeTauri('inject_keys', {
					request: { workspacePath: diffusionWs, entries: diffusionEnv }
				}).catch((e: unknown) => log(`Key injection warning (diffusion): ${e}`));
			}

			log('Launching both tools in parallel...');
			statusMessage = 'Launching parallel workflow...';

			const result = await invokeTauri<ParallelWorkflowResult>('run_parallel_workflow', {
				request: {
					workflowName,
					toolConfigs: [
						{
							toolId: SUPERPOWERS_ID,
							workspacePath: superpowersWs,
							envEntries: superpowersEnv
						},
						{
							toolId: DIFFUSION_ID,
							workspacePath: diffusionWs,
							envEntries: diffusionEnv
						}
					]
				}
			});

			lastResult = result;
			superpowersStatus = result.statuses.find((s) => s.toolId === SUPERPOWERS_ID) ?? null;
			diffusionStatus = result.statuses.find((s) => s.toolId === DIFFUSION_ID) ?? null;

			log(`Workflow launched! ID: ${result.workflowId}`);
			log(`Memory Spine entry: ${result.memorySpineId}`);
			log(`Kaizen Task created: ${result.kaizenTaskId}`);
			statusMessage = `Parallel workflow active! ${result.statuses.filter((s) => s.status === 'running').length} tools running.`;

			// Also log from frontend
			await logApiUsage({
				providerId: $activeProviderIdStore || 'parallel-executor',
				providerName: 'Parallel Executor',
				model: $activeModelStore || 'parallel',
				tokensIn: 0,
				tokensOut: 0,
				costUsd: 0,
				outputSummary: `Parallel workflow "${workflowName}" launched: Superpowers (pid:${superpowersStatus?.pid ?? 'n/a'}) + Diffusionstudio Agent (pid:${diffusionStatus?.pid ?? 'n/a'})`,
				toolId: 'parallel-workflow',
				workflowId: result.workflowId
			});

		} catch (error) {
			statusMessage = error instanceof Error ? error.message : 'Parallel launch failed.';
			log(`ERROR: ${statusMessage}`);
		} finally {
			running = false;
		}
	}

	async function stopTool(toolId: string) {
		try {
			await invokeTauri('stop_tool_process', { toolId });
			await pollStatuses();
			log(`Stopped: ${toolId}`);
		} catch (e) {
			log(`Stop failed (${toolId}): ${e}`);
		}
	}

	async function refreshLogs(toolId: string) {
		try {
			const result = await invokeTauri<ToolLogsResult>('get_tool_logs', {
				toolId,
				tailLines: 60
			});
			if (toolId === SUPERPOWERS_ID) {
				superpowersLogs = result.lines;
				showSuperpowersLogs = true;
			} else {
				diffusionLogs = result.lines;
				showDiffusionLogs = true;
			}
		} catch (e) {
			const errLines = [`Error: ${e}`];
			if (toolId === SUPERPOWERS_ID) superpowersLogs = errLines;
			else diffusionLogs = errLines;
		}
	}

	// ── Voice command integration ────────────────────────────────────────────
	function startVoice() {
		if (!voiceSupported) return;
		// @ts-expect-error - WebSpeech API not in all TS defs
		const SpeechRecognition = window.SpeechRecognition ?? window.webkitSpeechRecognition;
		recognition = new SpeechRecognition();
		// @ts-expect-error
		recognition.continuous = false;
		// @ts-expect-error
		recognition.interimResults = false;
		// @ts-expect-error
		recognition.onresult = async (event: SpeechRecognitionEvent) => {
			voiceTranscript = event.results[0][0].transcript;
			voiceListening = false;
			await sendVoiceCommand(voiceTranscript, event.results[0][0].confidence);
		};
		// @ts-expect-error
		recognition.onend = () => { voiceListening = false; };
		// @ts-expect-error
		recognition.start();
		voiceListening = true;
	}

	async function sendVoiceCommand(transcript: string, confidence = 0.9) {
		if (!transcript.trim()) return;
		try {
			voiceResult = await invokeTauri<VoiceCommandResult>('handle_voice_command', {
				request: { transcript, confidence }
			});
			log(`Voice: "${transcript}" → ${voiceResult.action} | ${voiceResult.message}`);

			// Auto-act on voice command
			if (voiceResult.success) {
				if (voiceResult.action === 'launch_parallel') {
					await launchParallel();
				} else if (voiceResult.action === 'launch_tool' && voiceResult.toolId) {
					statusMessage = `Voice triggered: ${voiceResult.message}`;
				} else if (voiceResult.action === 'stop_all') {
					await stopTool(SUPERPOWERS_ID);
					await stopTool(DIFFUSION_ID);
				}
			}
		} catch (e) {
			voiceResult = { action: 'error', message: String(e), success: false };
		}
	}

	function statusBadge(s: ToolProcessStatus | null, toolName: string) {
		if (!s) return { text: 'Not started', cls: 'bg-slate-800 text-slate-400' };
		if (s.status === 'running') return { text: `Running (pid: ${s.pid})`, cls: 'bg-green-500/15 text-green-300 border border-green-400/20' };
		if (s.status === 'stopped') return { text: 'Stopped', cls: 'bg-amber-500/10 text-amber-300 border border-amber-400/20' };
		return { text: s.status, cls: 'bg-red-500/10 text-red-300 border border-red-400/20' };
	}
</script>

<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-violet-400/25 bg-slate-950/55 p-8 shadow-2xl shadow-violet-950/30 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-violet-200/80">Parallel Execution</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">
			Superpowers <span class="text-violet-300">codes</span> while Diffusionstudio <span class="text-pink-300">edits video</span>.
		</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Launch both tools simultaneously in background. Superpowers handles agentic software development with dispatching-parallel-agents while Diffusionstudio/agent edits the demo video — both write evidence to Memory Spine and auto-create Kaizen tasks.
		</p>
		<div class="mt-6 flex items-center gap-3 flex-wrap">
			<ModelSwitcher compact />
			{#if $enabledProvidersStore.length > 0}
				<span class="text-xs text-green-400">{$enabledProvidersStore.length} providers active</span>
			{/if}
		</div>
	</div>

	<!-- Status bar -->
	{#if statusMessage}
		<div class="rounded-3xl border border-white/8 bg-slate-950/40 p-4 text-sm text-slate-300 backdrop-blur">
			{statusMessage}
		</div>
	{/if}

	<!-- Main parallel control panel -->
	<div class="grid gap-4 lg:grid-cols-2">
		<!-- Superpowers card -->
		<div class="rounded-3xl border border-violet-400/20 bg-gradient-to-b from-slate-950/60 to-violet-950/15 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-xs uppercase tracking-widest text-violet-300/70">Coding Agent</p>
					<h2 class="mt-1 text-xl font-semibold text-white">⚡ Superpowers</h2>
				</div>
				<span class="rounded-full px-3 py-1 text-xs font-medium {statusBadge(superpowersStatus, 'Superpowers').cls}">
					{statusBadge(superpowersStatus, 'Superpowers').text}
				</span>
			</div>

			<div class="mt-4 space-y-2 text-sm text-slate-400">
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Mode:</span>
					<span class="font-mono text-violet-300">dispatching-parallel-agents</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Workflow:</span>
					<span class="font-mono text-violet-200">brainstorm → plan → execute → review</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-slate-600">TDD:</span>
					<span class="text-green-400">mandatory</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Output:</span>
					<span>Memory Spine + Kaizen Tasks</span>
				</div>
			</div>

			<div class="mt-4">
				<label class="block text-xs text-slate-500 mb-1">Workspace path</label>
				<input
					bind:value={superpowersWorkspace}
					placeholder="Auto-filled on deploy"
					class="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-xs font-mono text-slate-300 outline-none focus:border-violet-400/40"
				/>
			</div>

			<div class="mt-4 flex flex-wrap gap-2">
				{#if superpowersStatus?.status === 'running'}
					<button
						type="button"
						on:click={() => stopTool(SUPERPOWERS_ID)}
						class="rounded-full bg-red-400/12 px-4 py-2 text-sm font-medium text-red-300 hover:bg-red-400/20"
					>⏹ Stop</button>
				{/if}
				<button
					type="button"
					on:click={() => refreshLogs(SUPERPOWERS_ID)}
					class="rounded-full border border-white/10 px-4 py-2 text-sm font-medium text-slate-300 hover:bg-white/5"
				>📋 Logs</button>
			</div>

			{#if showSuperpowersLogs}
				<div class="mt-3 rounded-2xl border border-white/8 bg-black/40 p-3">
					<div class="flex justify-between items-center mb-2">
						<span class="text-xs text-slate-500 font-mono">Superpowers log (last 60 lines)</span>
						<button type="button" on:click={() => (showSuperpowersLogs = false)} class="text-xs text-slate-600">✕</button>
					</div>
					<div class="max-h-40 overflow-y-auto space-y-0.5">
						{#each (superpowersLogs.length ? superpowersLogs : ['No log output yet.']) as line}
							<p class="text-xs font-mono text-slate-400 leading-5">{line}</p>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<!-- Diffusionstudio Agent card -->
		<div class="rounded-3xl border border-pink-400/20 bg-gradient-to-b from-slate-950/60 to-pink-950/15 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-xs uppercase tracking-widest text-pink-300/70">Video Agent</p>
					<h2 class="mt-1 text-xl font-semibold text-white">🎬 Diffusionstudio Agent</h2>
				</div>
				<span class="rounded-full px-3 py-1 text-xs font-medium {statusBadge(diffusionStatus, 'Diffusionstudio Agent').cls}">
					{statusBadge(diffusionStatus, 'Diffusionstudio Agent').text}
				</span>
			</div>

			<div class="mt-4 space-y-2 text-sm text-slate-400">
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Mode:</span>
					<span class="font-mono text-pink-300">background video composition</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Integration:</span>
					<span class="text-cyan-300">Vy/Panda computer control</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Semantic search:</span>
					<span>docs + tool integration</span>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-slate-600">Output:</span>
					<span>Memory Spine + Kaizen Tasks</span>
				</div>
			</div>

			<div class="mt-4">
				<label class="block text-xs text-slate-500 mb-1">Workspace path</label>
				<input
					bind:value={diffusionWorkspace}
					placeholder="Auto-filled on deploy"
					class="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-xs font-mono text-slate-300 outline-none focus:border-pink-400/40"
				/>
			</div>

			<div class="mt-4 flex flex-wrap gap-2">
				{#if diffusionStatus?.status === 'running'}
					<button
						type="button"
						on:click={() => stopTool(DIFFUSION_ID)}
						class="rounded-full bg-red-400/12 px-4 py-2 text-sm font-medium text-red-300 hover:bg-red-400/20"
					>⏹ Stop</button>
				{/if}
				<button
					type="button"
					on:click={() => refreshLogs(DIFFUSION_ID)}
					class="rounded-full border border-white/10 px-4 py-2 text-sm font-medium text-slate-300 hover:bg-white/5"
				>📋 Logs</button>
			</div>

			{#if showDiffusionLogs}
				<div class="mt-3 rounded-2xl border border-white/8 bg-black/40 p-3">
					<div class="flex justify-between items-center mb-2">
						<span class="text-xs text-slate-500 font-mono">Diffusionstudio log (last 60 lines)</span>
						<button type="button" on:click={() => (showDiffusionLogs = false)} class="text-xs text-slate-600">✕</button>
					</div>
					<div class="max-h-40 overflow-y-auto space-y-0.5">
						{#each (diffusionLogs.length ? diffusionLogs : ['No log output yet.']) as line}
							<p class="text-xs font-mono text-slate-400 leading-5">{line}</p>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Launch control -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white mb-4">Launch Parallel Workflow</h2>
		<div class="flex gap-3 flex-wrap items-end">
			<div class="flex-1 min-w-48">
				<label class="block text-xs text-slate-500 mb-1">Workflow name</label>
				<input
					bind:value={workflowName}
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
					placeholder="Workflow name"
				/>
			</div>
			<button
				type="button"
				on:click={launchParallel}
				disabled={running}
				class="rounded-full bg-gradient-to-r from-violet-500 to-pink-500 px-8 py-3 text-sm font-semibold text-white disabled:opacity-60 disabled:cursor-not-allowed hover:from-violet-400 hover:to-pink-400 transition-all"
			>
				{running ? '⏳ Launching...' : '⚡ Launch Both in Parallel'}
			</button>
		</div>

		{#if lastResult}
			<div class="mt-4 rounded-2xl border border-green-400/15 bg-green-400/5 p-4 text-sm">
				<p class="text-green-300 font-semibold mb-2">✅ Parallel workflow active</p>
				<div class="space-y-1 text-slate-400 text-xs font-mono">
					<p>Workflow ID: {lastResult.workflowId}</p>
					<p>Memory Spine: {lastResult.memorySpineId}</p>
					<p>Kaizen Task: {lastResult.kaizenTaskId}</p>
					<p>Tools running: {lastResult.statuses.filter(s => s.status === 'running').length} / {lastResult.statuses.length}</p>
				</div>
			</div>
		{/if}
	</div>

	<!-- Workflow step trace -->
	<div class="grid gap-4 lg:grid-cols-2">
		<!-- Event log -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-base font-semibold text-white mb-4">📜 Workflow Event Log</h2>
			<div class="max-h-64 overflow-y-auto space-y-1">
				{#each workflowLog as entry}
					<p class="text-xs font-mono text-slate-400 leading-5">{entry}</p>
				{:else}
					<p class="text-xs text-slate-600">No events yet. Launch a workflow to start.</p>
				{/each}
			</div>
		</div>

		<!-- Voice command panel -->
		<div class="rounded-3xl border border-cyan-400/15 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-base font-semibold text-white mb-1">🎙️ Voice / Mobile Control</h2>
			<p class="text-xs text-slate-500 mb-4">Control tools via voice or type a command. Works with mobile voice assistant.</p>

			<div class="flex gap-2 mb-3">
				<input
					bind:value={voiceTranscript}
					placeholder='e.g. "start video edit" or "run parallel"'
					class="flex-1 rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-2.5 text-sm text-white outline-none focus:border-cyan-400/40"
					on:keydown={(e) => e.key === 'Enter' && sendVoiceCommand(voiceTranscript)}
				/>
				{#if voiceSupported}
					<button
						type="button"
						on:click={startVoice}
						class="rounded-full {voiceListening ? 'bg-red-500 animate-pulse' : 'bg-cyan-400/15'} px-4 py-2 text-sm font-medium text-white hover:bg-cyan-400/25 transition-colors"
						title="Click to speak"
					>
						{voiceListening ? '🔴' : '🎙️'}
					</button>
				{/if}
				<button
					type="button"
					on:click={() => sendVoiceCommand(voiceTranscript)}
					class="rounded-full bg-cyan-400/15 px-4 py-2 text-sm font-medium text-cyan-100 hover:bg-cyan-400/25 transition-colors"
				>Send</button>
			</div>

			{#if voiceResult}
				<div class="rounded-xl border {voiceResult.success ? 'border-green-400/20 bg-green-400/5' : 'border-amber-400/20 bg-amber-400/5'} p-3 text-sm">
					<p class="font-medium {voiceResult.success ? 'text-green-300' : 'text-amber-300'}">
						{voiceResult.action}
					</p>
					<p class="text-slate-400 mt-0.5 text-xs">{voiceResult.message}</p>
				</div>
			{/if}

			<div class="mt-3 grid grid-cols-2 gap-1.5">
				{#each ['launch superpowers', 'start video edit', 'run parallel', 'status report', 'stop all', 'launch both'] as cmd}
					<button
						type="button"
						on:click={() => { voiceTranscript = cmd; sendVoiceCommand(cmd); }}
						class="rounded-xl border border-white/8 bg-white/3 px-3 py-1.5 text-xs text-slate-400 hover:bg-white/8 hover:text-white text-left transition-colors"
					>{cmd}</button>
				{/each}
			</div>
		</div>
	</div>

	<!-- Vy/Panda + MCP integration status -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-base font-semibold text-white mb-4">🖥️ Integrations</h2>
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
			{#each [
				{ name: 'Vy/Panda Control', desc: 'Computer control for background video editing', status: diffusionStatus?.status === 'running' ? 'active' : 'standby', icon: '🐼' },
				{ name: 'Memory Spine', desc: 'All outputs auto-written as evidence', status: 'active', icon: '🧠' },
				{ name: 'Kaizen Tasks', desc: 'Auto-created per workflow run', status: 'active', icon: '📋' },
				{ name: 'MCP', desc: 'Sequential thinking + memory + filesystem', status: 'connected', icon: '🧩' },
			] as intg}
				<div class="rounded-2xl border border-white/8 bg-white/2 p-3">
					<div class="flex items-center gap-2 mb-1">
						<span class="text-lg">{intg.icon}</span>
						<span class="text-xs font-semibold text-white">{intg.name}</span>
					</div>
					<p class="text-xs text-slate-500 mb-2">{intg.desc}</p>
					<span class="rounded-full px-2 py-0.5 text-xs {
						intg.status === 'active' ? 'bg-green-500/15 text-green-400' :
						intg.status === 'connected' ? 'bg-cyan-500/15 text-cyan-400' :
						'bg-slate-700/50 text-slate-500'
					}">{intg.status}</span>
				</div>
			{/each}
		</div>
	</div>

	<!-- Example workflow reference -->
	<div class="rounded-3xl border border-cyan-400/15 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-base font-semibold text-white mb-2">📖 Example: Code Feature + Edit Demo Video</h2>
		<p class="text-sm text-slate-400 mb-4">
			The canonical parallel workflow: Superpowers dispatches parallel sub-agents to implement a feature using mandatory TDD (brainstorm → plan → execute → review), while Diffusionstudio Agent edits the product demo video in background via Vy/Panda computer control.
		</p>
		<div class="grid gap-3 sm:grid-cols-2">
			<div class="rounded-2xl border border-violet-400/15 bg-violet-950/20 p-4">
				<p class="text-xs font-bold text-violet-300 mb-2">⚡ Superpowers Agent</p>
				<ol class="space-y-1 text-xs text-slate-400 list-decimal list-inside">
					<li>Brainstorm: generate solution approaches</li>
					<li>Plan: decompose into TDD tasks</li>
					<li>Execute: dispatch parallel sub-agents per task</li>
					<li>Review: aggregate, test, commit</li>
					<li>Write evidence → Memory Spine</li>
					<li>Create Kaizen task for each milestone</li>
				</ol>
			</div>
			<div class="rounded-2xl border border-pink-400/15 bg-pink-950/20 p-4">
				<p class="text-xs font-bold text-pink-300 mb-2">🎬 Diffusionstudio Agent</p>
				<ol class="space-y-1 text-xs text-slate-400 list-decimal list-inside">
					<li>Receive video task from voice/workflow</li>
					<li>Semantic search: find relevant doc clips</li>
					<li>Vy/Panda: control editor in background</li>
					<li>AI composition: assemble scenes</li>
					<li>Render: output to logs/video-output/</li>
					<li>Write evidence → Memory Spine</li>
				</ol>
			</div>
		</div>
		<div class="mt-4 rounded-2xl border border-white/8 bg-black/30 p-3">
			<p class="text-xs font-mono text-slate-400 mb-1"># Run the example workflow directly with Bun:</p>
			<p class="text-xs font-mono text-cyan-300">bun run examples/parallel-workflow.ts</p>
		</div>
	</div>

	<!-- Past parallel workflow runs -->
	{#if pastWorkflows.length > 0}
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-base font-semibold text-white">📂 Past Parallel Workflow Runs</h2>
				<span class="text-xs text-slate-500">{pastWorkflows.length} total</span>
			</div>
			<div class="space-y-2">
				{#each pastWorkflows.slice(0, 10) as wf}
					<div class="rounded-2xl border border-white/8 bg-white/2 p-3 flex items-center justify-between gap-3">
						<div class="min-w-0">
							<p class="text-sm font-medium text-white truncate">{wf.workflowName}</p>
							<p class="text-xs text-slate-500 font-mono mt-0.5">{new Date(wf.createdAt).toLocaleString()}</p>
						</div>
						<div class="flex items-center gap-2 flex-shrink-0">
							<span class="rounded-full px-2 py-0.5 text-xs {
								wf.status === 'running' ? 'bg-green-500/15 text-green-400' :
								wf.status === 'partial' ? 'bg-amber-500/10 text-amber-400' :
								'bg-slate-700/50 text-slate-400'
							}">{wf.status}</span>
							<span class="text-xs text-slate-600 font-mono">id:{wf.id.slice(0,8)}</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</section>
