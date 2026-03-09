<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import { createKaizenTask, logApiUsage } from '$lib/utils/provider-registry';

	// ─── Types ────────────────────────────────────────────────────────────────

	type AgentSession = {
		id: string;
		name: string;
		target: string;
		status: string;
		currentTask: string;
		actionsTaken: number;
		parallelMode: boolean;
		permissionMode: string;
		screenshotPath: string | null;
		logLines: string[];
		startedAt: string | null;
		updatedAt: string;
	};

	type ParallelTask = {
		id: string;
		sessionId: string;
		title: string;
		description: string;
		status: string;
		priority: number;
		deviceTarget: string;
		progressPct: number;
		resultSummary: string;
		createdAt: string;
		updatedAt: string;
	};

	type AndroidDevice = {
		serial: string;
		model: string;
		status: string;
		apiLevel: string;
		pandaInstalled: boolean;
	};

	type PermissionRequest = {
		id: string;
		sessionId: string;
		actionType: string;
		description: string;
		riskLevel: string;
		status: string;
		requestedAt: string;
		resolvedAt: string | null;
		resolvedBy: string;
	};

	// ─── State ────────────────────────────────────────────────────────────────

	let activeTab = 'desktop'; // 'desktop' | 'android' | 'parallel' | 'permissions'
	let sessions: AgentSession[] = [];
	let parallelTasks: ParallelTask[] = [];
	let androidDevices: AndroidDevice[] = [];
	let pendingPermissions: PermissionRequest[] = [];
	let loading = false;
	let agentEnabled = true;

	// New session form
	let sessionName = 'My Agent';
	let sessionTask = 'Take a screenshot and describe the current desktop state';
	let sessionTarget = 'desktop';
	let sessionParallel = false;
	let sessionPermMode = 'ask';
	let startingSession = false;

	// New parallel task form
	let ptaskTitle = '';
	let ptaskDesc = '';
	let ptaskTarget = 'desktop';
	let ptaskPriority = 3;
	let addingTask = false;

	// Android
	let loadingDevices = false;
	let selectedDevice = '';
	let adbCommand = '';
	let adbArgs = '';
	let adbRunning = false;
	let adbResult = '';

	// Session view
	let selectedSession: AgentSession | null = null;

	// Safety kill switch
	let killSwitchActive = false;

	let pollInterval: ReturnType<typeof setInterval> | null = null;

	// ─── Actions ──────────────────────────────────────────────────────────────

	async function loadAll() {
		if (!isDesktopRuntime()) return;
		try {
			const [s, pt, pp] = await Promise.all([
				invokeTauri<AgentSession[]>('list_agent_sessions'),
				invokeTauri<ParallelTask[]>('list_parallel_tasks'),
				invokeTauri<PermissionRequest[]>('list_permission_requests', { status: 'pending' })
			]);
			sessions = s;
			parallelTasks = pt;
			pendingPermissions = pp;
		} catch (e) {
			console.error('Failed to load computer control data', e);
		}
	}

	async function loadAndroid() {
		loadingDevices = true;
		try {
			androidDevices = await invokeTauri<AndroidDevice[]>('list_android_devices');
		} catch (e) {
			androidDevices = [];
		} finally {
			loadingDevices = false;
		}
	}

	async function startSession() {
		if (!sessionTask.trim()) return;
		startingSession = true;
		try {
			const session = await invokeTauri<AgentSession>('start_agent_session', {
				req: {
					name: sessionName,
					task: sessionTask,
					target: sessionTarget,
					parallelMode: sessionParallel,
					permissionMode: sessionPermMode,
					model: null
				}
			});
			sessions = [session, ...sessions];
			selectedSession = session;

			// Log to Memory Spine
			await logApiUsage({
				providerId: 'computer-agent',
				providerName: 'Vy Computer Agent',
				model: 'vision-agent-v1',
				tokensIn: 0,
				tokensOut: 0,
				costUsd: 0,
				outputSummary: `Started agent session "${session.name}" on ${session.target}: ${session.currentTask}`,
				toolId: 'computer-agent',
				workflowId: session.id
			});
		} catch (e) {
			console.error('Failed to start session', e);
		} finally {
			startingSession = false;
		}
	}

	async function stopSession(id: string) {
		if (killSwitchActive) return;
		try {
			await invokeTauri('stop_agent_session', { sessionId: id });
			await loadAll();
		} catch (e) {
			console.error('Failed to stop session', e);
		}
	}

	async function executeAction(sessionId: string, actionType: string, target: string, params: Record<string, string> = {}) {
		if (killSwitchActive) {
			alert('Kill switch is active. Deactivate it to run agent actions.');
			return;
		}
		try {
			await invokeTauri('execute_agent_action', {
				req: { sessionId, actionType, targetElement: target, params }
			});
			await loadAll();
		} catch (e) {
			console.error('Failed to execute action', e);
		}
	}

	async function addParallelTask() {
		if (!ptaskTitle.trim()) return;
		addingTask = true;
		try {
			const task = await invokeTauri<ParallelTask>('create_parallel_task', {
				req: {
					title: ptaskTitle,
					description: ptaskDesc,
					deviceTarget: ptaskTarget,
					priority: ptaskPriority,
					sessionId: null
				}
			});
			parallelTasks = [task, ...parallelTasks];
			ptaskTitle = '';
			ptaskDesc = '';

			await createKaizenTask({
				title: `Parallel task queued: ${task.title}`,
				description: `Background task "${task.title}" queued for ${task.deviceTarget}.`,
				priority: 'normal',
				source: 'parallel-exec',
				providerId: 'computer-agent',
				usageLogId: task.id
			});
		} catch (e) {
			console.error('Failed to create parallel task', e);
		} finally {
			addingTask = false;
		}
	}

	async function advanceTask(taskId: string) {
		const task = parallelTasks.find((t) => t.id === taskId);
		if (!task) return;
		const nextStatus =
			task.status === 'queued' ? 'running' : task.status === 'running' ? 'completed' : task.status;
		const nextPct =
			task.status === 'queued' ? 25 : task.status === 'running' ? 100 : task.progressPct;
		try {
			const updated = await invokeTauri<ParallelTask>('update_parallel_task_status', {
				id: taskId,
				status: nextStatus,
				progressPct: nextPct,
				resultSummary:
					nextStatus === 'completed' ? 'Task completed successfully by background agent.' : null
			});
			parallelTasks = parallelTasks.map((t) => (t.id === taskId ? updated : t));
		} catch (e) {
			console.error(e);
		}
	}

	async function runAdbCommand() {
		if (!selectedDevice || !adbCommand) return;
		adbRunning = true;
		adbResult = '';
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('execute_adb_command', {
				req: {
					deviceSerial: selectedDevice,
					command: adbCommand,
					args: adbArgs
						.trim()
						.split(/\s+/)
						.filter((a) => a)
				}
			});
			adbResult = result.message;
		} catch (e) {
			adbResult = String(e);
		} finally {
			adbRunning = false;
		}
	}

	async function installPanda() {
		if (!selectedDevice) return;
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('install_panda_apk', {
				deviceSerial: selectedDevice
			});
			adbResult = result.message;
			await loadAndroid();
		} catch (e) {
			adbResult = String(e);
		}
	}

	async function resolvePermission(id: string, approved: boolean) {
		try {
			await invokeTauri('resolve_permission', {
				id,
				approved,
				resolvedBy: 'user'
			});
			await loadAll();
		} catch (e) {
			console.error(e);
		}
	}

	async function activateKillSwitch() {
		killSwitchActive = true;
		for (const s of sessions.filter((s) => s.status === 'running')) {
			await stopSession(s.id);
		}
	}

	onMount(async () => {
		await loadAll();
		if (activeTab === 'android') await loadAndroid();
		pollInterval = setInterval(loadAll, 5000);
	});

	onDestroy(() => {
		if (pollInterval) clearInterval(pollInterval);
	});

	$: pendingCount = pendingPermissions.length;
	$: runningCount = sessions.filter((s) => s.status === 'running').length;
	$: queuedCount = parallelTasks.filter((t) => t.status === 'queued').length;

	const targetOptions = [
		{ value: 'desktop', label: 'Desktop (Vy mode)' },
		{ value: 'android', label: 'Android (Panda/blurr)' },
		{ value: 'vps', label: 'VPS (Remote)' },
		{ value: 'rpi', label: 'Raspberry Pi' }
	];

	const permModeOptions = [
		{ value: 'ask', label: 'Ask before each action' },
		{ value: 'auto', label: 'Auto-approve (trust mode)' },
		{ value: 'block', label: 'Block all — review only' }
	];

	function riskColor(risk: string) {
		return risk === 'high'
			? 'text-rose-400 bg-rose-400/10 border-rose-400/30'
			: risk === 'medium'
				? 'text-amber-400 bg-amber-400/10 border-amber-400/30'
				: 'text-green-400 bg-green-400/10 border-green-400/30';
	}

	function statusColor(status: string) {
		return status === 'running'
			? 'text-cyan-400 bg-cyan-400/10'
			: status === 'completed'
				? 'text-green-400 bg-green-400/10'
				: status === 'error'
					? 'text-rose-400 bg-rose-400/10'
					: status === 'queued'
						? 'text-amber-400 bg-amber-400/10'
						: 'text-slate-400 bg-slate-400/10';
	}
</script>

<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 shadow-2xl shadow-violet-950/20 backdrop-blur">
		<div class="flex items-start justify-between gap-4 flex-wrap">
			<div class="max-w-2xl">
				<p class="text-sm uppercase tracking-[0.35em] text-violet-300/80">Computer Control</p>
				<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
					Vy + Panda Mode
				</h1>
				<p class="mt-4 text-base leading-7 text-slate-300">
					Vision-based desktop agent (suitedaces/computer-agent + Agent-S) + Android control (Panda/blurr).
					Take screenshots, understand any GUI, control mouse/keyboard in ANY app while you work.
				</p>
			</div>
			<div class="flex flex-col items-end gap-3">
				<label class="flex cursor-pointer items-center gap-3">
					<span class="text-sm text-slate-400">Enable Computer Control</span>
					<div
						role="switch"
						aria-checked={agentEnabled}
						tabindex="0"
						on:click={() => (agentEnabled = !agentEnabled)}
						on:keydown={(e) => e.key === 'Enter' && (agentEnabled = !agentEnabled)}
						class="relative h-6 w-11 rounded-full transition {agentEnabled
							? 'bg-violet-500'
							: 'bg-slate-700'} cursor-pointer"
					>
						<span
							class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {agentEnabled
								? 'translate-x-5'
								: 'translate-x-0'}"
						></span>
					</div>
				</label>
				{#if runningCount > 0}
					<button
						type="button"
						on:click={activateKillSwitch}
						class="rounded-full border border-rose-400/40 bg-rose-500/10 px-4 py-2 text-sm font-semibold text-rose-400 hover:bg-rose-500/20"
					>
						🛑 Kill Switch ({runningCount} running)
					</button>
				{/if}
			</div>
		</div>

		<!-- Stats -->
		<div class="mt-6 grid grid-cols-2 gap-3 sm:grid-cols-4">
			{#each [
				{ label: 'Active Sessions', value: runningCount, color: 'cyan' },
				{ label: 'Parallel Tasks', value: queuedCount, color: 'violet' },
				{ label: 'Pending Approvals', value: pendingCount, color: pendingCount > 0 ? 'amber' : 'slate' },
				{ label: 'Total Actions', value: sessions.reduce((a, s) => a + s.actionsTaken, 0), color: 'green' }
			] as stat}
				<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
					<p class="text-xs text-slate-500">{stat.label}</p>
					<p class="mt-2 text-2xl font-semibold text-white">{stat.value}</p>
				</div>
			{/each}
		</div>

		<!-- Safety Banner -->
		{#if killSwitchActive}
			<div class="mt-4 rounded-2xl border border-rose-400/30 bg-rose-500/10 p-4 flex items-center gap-3">
				<span class="text-rose-400 text-xl">🛑</span>
				<div>
					<p class="text-sm font-semibold text-rose-300">Kill Switch Active — All agents stopped</p>
					<button
						type="button"
						on:click={() => (killSwitchActive = false)}
						class="mt-1 text-xs text-rose-400 underline hover:text-rose-300"
					>
						Deactivate kill switch
					</button>
				</div>
			</div>
		{/if}

		{#if !agentEnabled}
			<div class="mt-4 rounded-2xl border border-amber-400/30 bg-amber-500/10 p-4">
				<p class="text-sm text-amber-300">⚠️ Computer Control is disabled. Toggle above to enable Vy + Panda mode.</p>
			</div>
		{/if}
	</div>

	<!-- Tabs -->
	<div class="flex gap-2 flex-wrap">
		{#each [
			{ id: 'desktop', label: 'Desktop (Vy)', icon: '🖥️' },
			{ id: 'parallel', label: 'Parallel Tasks', icon: '⚡', badge: queuedCount },
			{ id: 'android', label: 'Android (Panda)', icon: '📱' },
			{ id: 'permissions', label: 'Permissions', icon: '🔐', badge: pendingCount }
		] as tab}
			<button
				type="button"
				on:click={async () => {
					activeTab = tab.id;
					if (tab.id === 'android') await loadAndroid();
				}}
				class="relative rounded-full px-5 py-2.5 text-sm font-medium transition {activeTab === tab.id
					? 'bg-violet-500/20 text-violet-100 border border-violet-400/30'
					: 'text-slate-400 hover:text-white border border-white/8 bg-white/3'}"
			>
				{tab.icon} {tab.label}
				{#if tab.badge && tab.badge > 0}
					<span class="absolute -top-1.5 -right-1.5 flex h-4 w-4 items-center justify-center rounded-full bg-amber-400 text-[10px] font-bold text-slate-950">
						{tab.badge}
					</span>
				{/if}
			</button>
		{/each}
	</div>

	<!-- ─── Desktop Tab ────────────────────────────────────────────────────── -->
	{#if activeTab === 'desktop'}
		<div class="grid gap-6 lg:grid-cols-[1.5fr_1fr]">
			<!-- New Session -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white mb-1">Start Agent Session</h2>
				<p class="text-xs text-slate-500 mb-5">
					Powered by suitedaces/computer-agent + trycua/cua + simular-ai/Agent-S
				</p>

				<div class="space-y-4">
					<div>
						<label class="block text-xs text-slate-400 mb-1">Session Name</label>
						<input
							bind:value={sessionName}
							class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-violet-400/50 focus:outline-none"
							placeholder="e.g. Tax Filing Agent"
						/>
					</div>

					<div>
						<label class="block text-xs text-slate-400 mb-1">Task Description</label>
						<textarea
							bind:value={sessionTask}
							rows={3}
							class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-violet-400/50 focus:outline-none resize-none"
							placeholder="Describe what the agent should do..."
						></textarea>
					</div>

					<div class="grid grid-cols-2 gap-3">
						<div>
							<label class="block text-xs text-slate-400 mb-1">Target Device</label>
							<select
								bind:value={sessionTarget}
								class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white focus:outline-none"
							>
								{#each targetOptions as opt}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</select>
						</div>
						<div>
							<label class="block text-xs text-slate-400 mb-1">Permission Mode</label>
							<select
								bind:value={sessionPermMode}
								class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white focus:outline-none"
							>
								{#each permModeOptions as opt}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</select>
						</div>
					</div>

					<label class="flex items-center gap-3 cursor-pointer">
						<input type="checkbox" bind:checked={sessionParallel} class="rounded border-white/20 bg-slate-900 accent-violet-500" />
						<span class="text-sm text-slate-300">Enable parallel execution (run in background)</span>
					</label>

					<div class="rounded-2xl border border-amber-400/20 bg-amber-400/5 p-3">
						<p class="text-xs text-amber-300 font-medium">⚠️ Safety Warning</p>
						<p class="text-xs text-slate-400 mt-1">
							Computer agents can control your mouse, keyboard, and files. Always use "Ask before each action" mode for sensitive operations.
						</p>
					</div>

					<button
						type="button"
						on:click={startSession}
						disabled={startingSession || !agentEnabled || killSwitchActive || !sessionTask.trim()}
						class="w-full rounded-full bg-gradient-to-r from-violet-500 to-cyan-500 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-violet-500/30 disabled:opacity-50"
					>
						{startingSession ? '▶ Starting...' : '▶ Start Agent Session'}
					</button>
				</div>
			</div>

			<!-- Session List -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white mb-4">Agent Sessions</h2>
				<div class="space-y-3">
					{#if sessions.length === 0}
						<p class="text-sm text-slate-500">No sessions yet. Start your first agent above.</p>
					{:else}
						{#each sessions.slice(0, 8) as session}
							<div
								class="rounded-2xl border border-white/8 bg-white/3 p-4 cursor-pointer hover:border-white/15 transition"
								role="button"
								tabindex="0"
								on:click={() => (selectedSession = session)}
								on:keydown={(e) => e.key === 'Enter' && (selectedSession = session)}
							>
								<div class="flex items-center justify-between mb-2">
									<p class="text-sm font-medium text-white truncate max-w-[180px]">{session.name}</p>
									<span class="rounded-full px-2 py-0.5 text-xs font-medium {statusColor(session.status)}">
										{session.status}
									</span>
								</div>
								<p class="text-xs text-slate-500 truncate">{session.currentTask}</p>
								<div class="mt-2 flex items-center gap-3 text-xs text-slate-600">
									<span>🖥️ {session.target}</span>
									<span>⚡ {session.actionsTaken} actions</span>
									{#if session.parallelMode}
										<span class="text-violet-400">⫷ parallel</span>
									{/if}
								</div>
								{#if session.status === 'running'}
									<button
										type="button"
										on:click|stopPropagation={() => stopSession(session.id)}
										class="mt-2 rounded-lg border border-rose-400/30 px-3 py-1 text-xs text-rose-400 hover:bg-rose-400/10"
									>
										Stop
									</button>
								{/if}
							</div>
						{/each}
					{/if}
				</div>
			</div>
		</div>

		<!-- Selected Session Detail -->
		{#if selectedSession}
			<div class="rounded-3xl border border-violet-400/20 bg-violet-500/5 p-6 backdrop-blur">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold text-white">Session: {selectedSession.name}</h2>
					<button
						type="button"
						on:click={() => (selectedSession = null)}
						class="text-slate-500 hover:text-white text-sm"
					>
						✕ Close
					</button>
				</div>

				<p class="text-sm text-slate-400 mb-4">Task: {selectedSession.currentTask}</p>

				<!-- Quick Actions -->
				<div class="flex flex-wrap gap-2 mb-5">
					{#each [
						{ type: 'screenshot', label: '📸 Screenshot', target: 'desktop', params: {} as Record<string, string> },
						{ type: 'click', label: '🖱️ Click', target: 'Start button', params: {} as Record<string, string> },
						{ type: 'type', label: '⌨️ Type', target: 'active input', params: { text: 'Hello from AmitOS' } as Record<string, string> },
						{ type: 'scroll', label: '📜 Scroll Down', target: 'main content', params: { direction: 'down' } as Record<string, string> },
						{ type: 'key', label: '⌨️ Enter', target: '', params: { key: 'Return' } as Record<string, string> }
					] as action}
						<button
							type="button"
							on:click={() => executeAction(selectedSession!.id, action.type, action.target, action.params)}
							class="rounded-full border border-white/15 bg-white/5 px-4 py-2 text-xs text-white hover:bg-white/10"
						>
							{action.label}
						</button>
					{/each}
				</div>

				<!-- Session Log -->
				{#if selectedSession.logLines.length > 0}
					<div class="rounded-2xl bg-slate-950/80 p-4 font-mono text-xs space-y-1 max-h-48 overflow-y-auto border border-white/5">
						{#each selectedSession.logLines as line}
							<p class="text-slate-400">{line}</p>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	{/if}

	<!-- ─── Parallel Tasks Tab ─────────────────────────────────────────────── -->
	{#if activeTab === 'parallel'}
		<div class="grid gap-6 lg:grid-cols-[1.5fr_1fr]">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white mb-1">Parallel Task Queue</h2>
				<p class="text-xs text-slate-500 mb-5">
					Run heavy tasks in the background while you continue working. Each task runs on a separate agent thread.
				</p>

				<!-- Add task form -->
				<div class="space-y-3 mb-6 rounded-2xl border border-white/8 bg-white/3 p-4">
					<h3 class="text-sm font-semibold text-white">Add Background Task</h3>
					<input
						bind:value={ptaskTitle}
						class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-violet-400/50"
						placeholder="Task title (e.g. 'Fill Excel tax form')"
					/>
					<textarea
						bind:value={ptaskDesc}
						rows={2}
						class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-violet-400/50 resize-none"
						placeholder="Description..."
					></textarea>
					<div class="grid grid-cols-2 gap-3">
						<select bind:value={ptaskTarget} class="rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white focus:outline-none">
							<option value="desktop">Desktop</option>
							<option value="android">Android</option>
							<option value="vps">VPS</option>
							<option value="rpi">Raspberry Pi</option>
						</select>
						<select bind:value={ptaskPriority} class="rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white focus:outline-none">
							<option value={1}>P1 – Critical</option>
							<option value={2}>P2 – High</option>
							<option value={3}>P3 – Normal</option>
							<option value={5}>P5 – Low</option>
						</select>
					</div>
					<button
						type="button"
						on:click={addParallelTask}
						disabled={addingTask || !ptaskTitle.trim()}
						class="w-full rounded-full bg-gradient-to-r from-violet-500 to-cyan-500 px-4 py-2 text-sm font-semibold text-white disabled:opacity-50"
					>
						{addingTask ? 'Adding...' : '+ Add to Queue'}
					</button>
				</div>

				<!-- Task list -->
				<div class="space-y-3">
					{#if parallelTasks.length === 0}
						<p class="text-sm text-slate-500">No tasks queued. Add your first background task above.</p>
					{:else}
						{#each parallelTasks as task}
							<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
								<div class="flex items-start justify-between gap-2">
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-2 mb-1">
											<span class="rounded-full px-2 py-0.5 text-xs font-medium {statusColor(task.status)}">
												{task.status}
											</span>
											<span class="text-xs text-slate-500">{task.deviceTarget}</span>
											<span class="text-xs text-slate-600">P{task.priority}</span>
										</div>
										<p class="text-sm font-medium text-white">{task.title}</p>
										{#if task.description}
											<p class="text-xs text-slate-500 mt-1">{task.description}</p>
										{/if}
									</div>
									{#if task.status !== 'completed' && task.status !== 'failed'}
										<button
											type="button"
											on:click={() => advanceTask(task.id)}
											class="shrink-0 rounded-full border border-cyan-400/30 px-3 py-1 text-xs text-cyan-400 hover:bg-cyan-400/10"
										>
											{task.status === 'queued' ? '▶ Start' : '✓ Complete'}
										</button>
									{/if}
								</div>
								<!-- Progress bar -->
								{#if task.progressPct > 0}
									<div class="mt-3 h-1.5 rounded-full bg-slate-800 overflow-hidden">
										<div
											class="h-full rounded-full {task.status === 'completed' ? 'bg-green-400' : 'bg-violet-400'} transition-all"
											style="width: {task.progressPct}%"
										></div>
									</div>
								{/if}
								{#if task.resultSummary}
									<p class="mt-2 text-xs text-green-400">{task.resultSummary}</p>
								{/if}
							</div>
						{/each}
					{/if}
				</div>
			</div>

			<!-- Example Tasks -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-sm font-semibold text-white mb-4">Example Parallel Workflows</h2>
				<div class="space-y-3">
					{#each [
						{ title: 'Fill tax form in Excel', desc: 'Agent opens Excel, fills W2 data, saves PDF', target: 'desktop', icon: '📊' },
						{ title: 'Update Notion workspace', desc: 'Agent updates project status pages', target: 'desktop', icon: '📝' },
						{ title: 'Monitor phone messages', desc: 'Panda reads and replies to WhatsApp', target: 'android', icon: '💬' },
						{ title: 'Run overnight data pipeline', desc: 'VPS agent processes CSV batch jobs', target: 'vps', icon: '🔄' },
						{ title: 'RPi home automation', desc: 'RPi agent toggles smart home routines', target: 'rpi', icon: '🏠' }
					] as example}
						<button
							type="button"
							on:click={() => {
								ptaskTitle = example.title;
								ptaskDesc = example.desc;
								ptaskTarget = example.target;
							}}
							class="w-full rounded-2xl border border-white/8 bg-white/3 p-4 text-left hover:border-white/15 transition"
						>
							<div class="flex items-center gap-3">
								<span class="text-2xl">{example.icon}</span>
								<div>
									<p class="text-sm font-medium text-white">{example.title}</p>
									<p class="text-xs text-slate-500 mt-0.5">{example.desc}</p>
									<span class="mt-1 inline-block rounded-full border border-white/10 bg-white/5 px-2 py-0.5 text-[10px] text-slate-400">{example.target}</span>
								</div>
							</div>
						</button>
					{/each}
				</div>
			</div>
		</div>
	{/if}

	<!-- ─── Android / Panda Tab ────────────────────────────────────────────── -->
	{#if activeTab === 'android'}
		<div class="grid gap-6 lg:grid-cols-[1.5fr_1fr]">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white mb-1">Android — Panda/blurr Agent</h2>
				<p class="text-xs text-slate-500 mb-5">
					Based on Ayush0Chaudhary/blurr. Accessibility Service agent controls your phone exactly like a human.
				</p>

				<button
					type="button"
					on:click={loadAndroid}
					disabled={loadingDevices}
					class="mb-4 rounded-full border border-white/15 bg-white/5 px-4 py-2 text-sm text-white hover:bg-white/10"
				>
					{loadingDevices ? '⟳ Scanning...' : '🔍 Scan for Devices'}
				</button>

				{#if androidDevices.length > 0}
					<div class="space-y-3 mb-5">
						{#each androidDevices as device}
							<button
								type="button"
								on:click={() => (selectedDevice = device.serial)}
								class="w-full rounded-2xl border {selectedDevice === device.serial ? 'border-violet-400/40 bg-violet-500/10' : 'border-white/8 bg-white/3'} p-4 text-left hover:border-white/15 transition"
							>
								<div class="flex items-center justify-between">
									<div>
										<p class="text-sm font-medium text-white">{device.model}</p>
										<p class="text-xs text-slate-500 font-mono mt-1">{device.serial}</p>
										<p class="text-xs text-slate-500 mt-1">API {device.apiLevel} · {device.status}</p>
									</div>
									{#if device.pandaInstalled}
										<span class="rounded-full bg-green-400/10 border border-green-400/30 px-3 py-1 text-xs text-green-400">
											Panda ✓
										</span>
									{:else}
										<span class="rounded-full bg-amber-400/10 border border-amber-400/30 px-3 py-1 text-xs text-amber-400">
											No Panda
										</span>
									{/if}
								</div>
							</button>
						{/each}
					</div>
				{/if}

				{#if selectedDevice}
					<div class="space-y-3 rounded-2xl border border-violet-400/20 bg-violet-500/5 p-4">
						<h3 class="text-sm font-semibold text-white">Device: {selectedDevice}</h3>

						<button
							type="button"
							on:click={installPanda}
							class="w-full rounded-full bg-gradient-to-r from-violet-500 to-pink-500 px-4 py-2.5 text-sm font-semibold text-white"
						>
							📦 Install Panda Agent APK
						</button>

						<div class="space-y-2">
							<div class="grid grid-cols-[1fr_auto] gap-2">
								<input
									bind:value={adbCommand}
									class="rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none font-mono"
									placeholder="ADB command (e.g. shell)"
								/>
								<input
									bind:value={adbArgs}
									class="rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none font-mono"
									placeholder="args"
								/>
							</div>
							<button
								type="button"
								on:click={runAdbCommand}
								disabled={adbRunning || !adbCommand}
								class="w-full rounded-full border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm font-medium text-cyan-300 hover:bg-cyan-400/20 disabled:opacity-50"
							>
								{adbRunning ? 'Running...' : '▶ Run ADB Command'}
							</button>
						</div>

						<!-- Quick ADB actions -->
						<div class="flex flex-wrap gap-2">
							{#each [
								{ label: '📸 Screenshot', cmd: 'shell', args: 'screencap -p /sdcard/screen.png' },
								{ label: '🏠 Home', cmd: 'shell', args: 'input keyevent 3' },
								{ label: '◀ Back', cmd: 'shell', args: 'input keyevent 4' },
								{ label: '📋 Apps', cmd: 'shell', args: 'input keyevent 187' }
							] as qa}
								<button
									type="button"
									on:click={() => {
										adbCommand = qa.cmd;
										adbArgs = qa.args;
										runAdbCommand();
									}}
									class="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-slate-300 hover:bg-white/10"
								>
									{qa.label}
								</button>
							{/each}
						</div>

						{#if adbResult}
							<div class="rounded-xl bg-slate-950/80 p-3 font-mono text-xs text-green-400 max-h-32 overflow-y-auto border border-white/5">
								{adbResult}
							</div>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Panda Info -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur space-y-4">
				<h2 class="text-sm font-semibold text-white">Panda Setup Guide</h2>
				<div class="space-y-3 text-xs text-slate-400">
					{#each [
						{ step: '1', text: 'Enable Developer Options on your Android device (Settings → About → tap Build Number 7x)' },
						{ step: '2', text: 'Enable USB Debugging in Developer Options and connect via USB' },
						{ step: '3', text: 'Click "Scan for Devices" to detect your phone' },
						{ step: '4', text: 'Click "Install Panda Agent APK" to sideload the Panda/blurr Accessibility Service' },
						{ step: '5', text: 'On your phone, go to Settings → Accessibility → Panda Agent and enable it' },
						{ step: '6', text: 'Start an Android agent session from the Desktop tab to control your phone' }
					] as s}
						<div class="flex gap-3">
							<span class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-violet-500/20 text-[10px] font-bold text-violet-400">{s.step}</span>
							<p class="leading-5">{s.text}</p>
						</div>
					{/each}
				</div>

				<div class="rounded-2xl border border-cyan-400/20 bg-cyan-500/5 p-4">
					<p class="text-xs font-semibold text-cyan-300 mb-2">Panda/blurr Capabilities</p>
					<ul class="text-xs text-slate-400 space-y-1">
						<li>• Tap, swipe, type in any app</li>
						<li>• Read screen content via Accessibility</li>
						<li>• Open apps, navigate menus</li>
						<li>• Voice command relay from PC to phone</li>
						<li>• Remote permission approval from desktop</li>
					</ul>
				</div>
			</div>
		</div>
	{/if}

	<!-- ─── Permissions Tab ────────────────────────────────────────────────── -->
	{#if activeTab === 'permissions'}
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white mb-1">Remote Permission Requests</h2>
			<p class="text-xs text-slate-500 mb-5">
				When an agent needs approval, requests appear here. Approve or deny from desktop, mobile, or voice command.
			</p>

			{#if pendingPermissions.length === 0}
				<div class="rounded-2xl border border-green-400/20 bg-green-500/5 p-6 text-center">
					<p class="text-3xl mb-2">✓</p>
					<p class="text-sm text-green-300">No pending permission requests</p>
					<p class="text-xs text-slate-500 mt-1">All agent actions have been cleared</p>
				</div>
			{:else}
				<div class="space-y-4">
					{#each pendingPermissions as perm}
						<div class="rounded-2xl border {riskColor(perm.riskLevel)} p-5">
							<div class="flex items-start justify-between gap-4">
								<div class="flex-1">
									<div class="flex items-center gap-2 mb-2">
										<span class="rounded-full border px-2 py-0.5 text-xs font-medium {riskColor(perm.riskLevel)}">
											{perm.riskLevel} risk
										</span>
										<span class="text-xs text-slate-500 font-mono">{perm.actionType}</span>
									</div>
									<p class="text-sm font-medium text-white">{perm.description}</p>
									<p class="text-xs text-slate-500 mt-1">Session: {perm.sessionId.slice(0, 16)}...</p>
									<p class="text-xs text-slate-600 mt-0.5">{new Date(perm.requestedAt).toLocaleString()}</p>
								</div>
								<div class="flex gap-2 shrink-0">
									<button
										type="button"
										on:click={() => resolvePermission(perm.id, true)}
										class="rounded-full bg-green-500/20 border border-green-400/30 px-4 py-2 text-sm font-semibold text-green-300 hover:bg-green-500/30"
									>
										✓ Approve
									</button>
									<button
										type="button"
										on:click={() => resolvePermission(perm.id, false)}
										class="rounded-full bg-rose-500/20 border border-rose-400/30 px-4 py-2 text-sm font-semibold text-rose-300 hover:bg-rose-500/30"
									>
										✕ Deny
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}

			<!-- Demo: Create test permission request -->
			<div class="mt-6 rounded-2xl border border-white/8 bg-white/3 p-4">
				<h3 class="text-sm font-semibold text-white mb-3">Test Permission System</h3>
				<p class="text-xs text-slate-500 mb-3">Create a demo permission request to test the approval flow (including phone approval).</p>
				<button
					type="button"
					on:click={async () => {
						if (!isDesktopRuntime()) return;
						const session = sessions[0];
						if (!session) {
							alert('Start an agent session first to test permissions.');
							return;
						}
						await invokeTauri('request_permission', {
							req: {
								sessionId: session.id,
								actionType: 'file_write',
								description: 'Agent wants to write to ~/Documents/taxes-2025.xlsx with calculated deductions',
								riskLevel: 'high'
							}
						});
						await loadAll();
						activeTab = 'permissions';
					}}
					class="rounded-full border border-amber-400/30 bg-amber-400/10 px-4 py-2 text-sm text-amber-300 hover:bg-amber-400/20"
				>
					Create Test Permission Request
				</button>
			</div>
		</div>
	{/if}

	<!-- Foundation Credits -->
	<div class="rounded-3xl border border-white/8 bg-white/2 p-5">
		<p class="text-xs text-slate-600 mb-3">Open-source foundations</p>
		<div class="flex flex-wrap gap-4 text-xs text-slate-500">
			{#each [
				{ name: 'suitedaces/computer-agent', desc: 'Vision GUI agent' },
				{ name: 'trycua/cua', desc: 'Computer-use agent' },
				{ name: 'simular-ai/Agent-S', desc: 'GUI action planner' },
				{ name: 'Ayush0Chaudhary/blurr (Panda)', desc: 'Android Accessibility' },
				{ name: 'quantiota/RPi-AI-Agent-Host', desc: 'Raspberry Pi node' }
			] as lib}
				<div class="rounded-xl border border-white/8 bg-white/3 px-3 py-2">
					<p class="font-mono text-[10px] text-slate-400">{lib.name}</p>
					<p class="text-[10px] text-slate-600">{lib.desc}</p>
				</div>
			{/each}
		</div>
	</div>
</section>
