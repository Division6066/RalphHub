<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		loadCcSettings,
		saveCcSettings,
		activateKillSwitch,
		deactivateKillSwitch,
		takeScreenshot,
		executeAction,
		startAgentTask,
		stopAgentTask,
		loadAgentTasks,
		startParallelWorkflow,
		loadWorkflows,
		getPandaStatus,
		ccSettingsStore,
		ccTasksStore,
		ccWorkflowsStore,
		ccKillSwitchStore,
		ccScreenshotStore,
		ccRunningTasksStore,
		ccLastScreenshotTimeStore,
		type ComputerControlSettings,
		type AgentTask,
		type ParallelWorkflow
	} from '$lib/utils/computer-control';
	import PermissionModal from '$lib/components/PermissionModal.svelte';
	import { loadKaizenTasks, kaizenTasksStore } from '$lib/utils/provider-registry';

	// ─── Tab State ───────────────────────────────────────────────────────────────
	type Tab = 'dashboard' | 'desktop' | 'android' | 'tasks' | 'workflows';
	let activeTab: Tab = 'dashboard';

	// ─── State ────────────────────────────────────────────────────────────────────
	let loading = true;
	let error = '';
	let pandaStatus: Record<string, unknown> = {};
	let screenshotLoading = false;
	let savingSettings = false;
	let settingsStatus = '';

	// ─── Task Creation ────────────────────────────────────────────────────────────
	let newTaskTitle = '';
	let newTaskGoal = '';
	let newTaskMode: 'supervised' | 'autonomous' = 'supervised';
	let creatingTask = false;
	let taskStatus = '';

	// ─── Action Tester ────────────────────────────────────────────────────────────
	let actionKind = 'screenshot';
	let actionX = '';
	let actionY = '';
	let actionText = '';
	let actionResult = '';
	let executingAction = false;

	// ─── Parallel Workflow ────────────────────────────────────────────────────────
	let wfName = '';
	let wfForeground = '';
	let wfBackgroundGoals = 'Do my taxes in Excel\nUpdate Notion project tracker';
	let creatingWorkflow = false;
	let workflowStatus = '';

	// ─── Kill Switch ──────────────────────────────────────────────────────────────
	let killConfirm = false;

	// ─── Polling ──────────────────────────────────────────────────────────────────
	let pollInterval: ReturnType<typeof setInterval>;

	onMount(async () => {
		try {
			await Promise.all([
				loadCcSettings(),
				loadAgentTasks(),
				loadWorkflows(),
				loadKaizenTasks()
			]);
			pandaStatus = await getPandaStatus();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load computer control';
		} finally {
			loading = false;
		}

		// Poll running tasks every 2 seconds
		pollInterval = setInterval(async () => {
			if ($ccRunningTasksStore.length > 0) {
				await loadAgentTasks();
			}
		}, 2000);
	});

	onDestroy(() => clearInterval(pollInterval));

	async function handleTakeScreenshot() {
		screenshotLoading = true;
		try {
			await takeScreenshot();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Screenshot failed';
		} finally {
			screenshotLoading = false;
		}
	}

	async function handleExecuteAction() {
		executingAction = true;
		actionResult = '';
		try {
			const result = await executeAction({
				kind: actionKind as any,
				x: actionX ? parseInt(actionX) : undefined,
				y: actionY ? parseInt(actionY) : undefined,
				text: actionText || undefined,
				description: `Manual action: ${actionKind}`
			});
			actionResult = result.ok ? `✓ ${result.message}` : `✗ ${result.message}`;
		} catch (e) {
			actionResult = `✗ ${e instanceof Error ? e.message : 'Action failed'}`;
		} finally {
			executingAction = false;
		}
	}

	async function handleStartTask() {
		if (!newTaskTitle || !newTaskGoal) return;
		creatingTask = true;
		taskStatus = '';
		try {
			await startAgentTask({
				title: newTaskTitle,
				description: `Agent task: ${newTaskTitle}`,
				goal: newTaskGoal,
				mode: newTaskMode
			});
			taskStatus = `✓ Task "${newTaskTitle}" started`;
			newTaskTitle = '';
			newTaskGoal = '';
		} catch (e) {
			taskStatus = `✗ ${e instanceof Error ? e.message : 'Failed'}`;
		} finally {
			creatingTask = false;
		}
	}

	async function handleStopTask(taskId: string) {
		try {
			await stopAgentTask(taskId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to stop task';
		}
	}

	async function handleStartParallelWorkflow() {
		if (!wfName || !wfForeground) return;
		creatingWorkflow = true;
		workflowStatus = '';
		try {
			const goals = wfBackgroundGoals
				.split('\n')
				.map((g) => g.trim())
				.filter(Boolean);
			await startParallelWorkflow(wfName, wfForeground, goals);
			workflowStatus = `✓ Parallel workflow "${wfName}" started with ${goals.length} background tasks`;
		} catch (e) {
			workflowStatus = `✗ ${e instanceof Error ? e.message : 'Failed'}`;
		} finally {
			creatingWorkflow = false;
		}
	}

	async function handleKillSwitch() {
		if (!killConfirm) {
			killConfirm = true;
			setTimeout(() => (killConfirm = false), 5000);
			return;
		}
		await activateKillSwitch();
		killConfirm = false;
	}

	async function handleResumeKillSwitch() {
		await deactivateKillSwitch();
	}

	async function handleSaveSettings() {
		if (!$ccSettingsStore) return;
		savingSettings = true;
		settingsStatus = '';
		try {
			await saveCcSettings($ccSettingsStore);
			settingsStatus = '✓ Settings saved';
		} catch (e) {
			settingsStatus = `✗ ${e instanceof Error ? e.message : 'Save failed'}`;
		} finally {
			savingSettings = false;
		}
	}

	const STATUS_COLORS: Record<string, string> = {
		queued: 'bg-slate-600 text-slate-300',
		running: 'bg-cyan-500/20 text-cyan-300',
		completed: 'bg-green-500/20 text-green-300',
		failed: 'bg-rose-500/20 text-rose-300',
		killed: 'bg-orange-500/20 text-orange-300',
		paused: 'bg-yellow-500/20 text-yellow-300'
	};
</script>

<!-- Permission Modal (global) -->
<PermissionModal />

<section class="space-y-6">

	<!-- ═══ Header ═══════════════════════════════════════════════════════════════ -->
	<div class="rounded-[2rem] border border-violet-400/20 bg-gradient-to-br from-violet-950/60 to-slate-950/60 p-8 backdrop-blur">
		<div class="flex items-start justify-between gap-6 flex-wrap">
			<div>
				<p class="text-sm uppercase tracking-[0.35em] text-violet-300/80">AmitOS</p>
				<h1 class="mt-3 text-4xl font-semibold tracking-tight text-white">
					Vy-Style Computer Control
				</h1>
				<p class="mt-3 max-w-2xl text-base leading-7 text-slate-300">
					Full desktop + Android takeover. Agent takes screenshots, moves your mouse, types in any app, and runs in the background while you do other things.
				</p>
				<div class="mt-4 flex flex-wrap gap-3 text-sm">
					<span class="rounded-full border border-violet-400/20 bg-violet-400/10 px-3 py-1 text-violet-200">
						suitedaces/computer-agent
					</span>
					<span class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-cyan-200">
						simular-ai/Agent-S
					</span>
					<span class="rounded-full border border-blue-400/20 bg-blue-400/10 px-3 py-1 text-blue-200">
						trycua/cua
					</span>
					<span class="rounded-full border border-green-400/20 bg-green-400/10 px-3 py-1 text-green-200">
						🐼 Panda (blurr)
					</span>
				</div>
			</div>

			<!-- Kill Switch -->
			<div class="flex flex-col items-end gap-3">
				{#if $ccKillSwitchStore}
					<div class="rounded-2xl border border-rose-400/40 bg-rose-500/20 px-5 py-3 text-center">
						<p class="text-sm font-bold text-rose-300">⛔ KILL SWITCH ACTIVE</p>
						<p class="text-xs text-rose-400 mt-1">All agent tasks halted</p>
					</div>
					<button
						type="button"
						on:click={handleResumeKillSwitch}
						class="rounded-full border border-green-400/30 bg-green-500/10 px-5 py-2 text-sm font-semibold text-green-300 hover:bg-green-500/20 transition-colors"
					>
						▶ Resume Operations
					</button>
				{:else}
					<button
						type="button"
						on:click={handleKillSwitch}
						class="rounded-full border px-5 py-3 text-sm font-bold transition-all
							{killConfirm
							? 'border-rose-400 bg-rose-500 text-white animate-pulse'
							: 'border-rose-400/30 bg-rose-500/10 text-rose-400 hover:bg-rose-500/20'}"
					>
						{killConfirm ? '⛔ CONFIRM KILL ALL' : '⛔ Kill Switch'}
					</button>
					{#if killConfirm}
						<p class="text-xs text-rose-400">Click again to confirm — halts ALL agent tasks</p>
					{/if}
				{/if}

				<!-- Status pills -->
				<div class="flex gap-2 text-xs">
					<span class="rounded-full {$ccSettingsStore?.enabled ? 'bg-green-500/20 text-green-300 border border-green-400/30' : 'bg-slate-700 text-slate-400'} px-3 py-1">
						{$ccSettingsStore?.enabled ? '✓ Enabled' : '○ Disabled'}
					</span>
					<span class="rounded-full bg-slate-800 text-slate-400 px-3 py-1 border border-white/10">
						{$ccSettingsStore?.mode ?? 'supervised'}
					</span>
					<span class="rounded-full bg-slate-800 text-slate-400 px-3 py-1 border border-white/10">
						{$ccRunningTasksStore.length} running
					</span>
				</div>
			</div>
		</div>
	</div>

	<!-- Safety Warning -->
	{#if !$ccSettingsStore?.enabled}
		<div class="rounded-3xl border border-amber-400/30 bg-amber-400/5 p-6">
			<div class="flex gap-4">
				<span class="text-2xl flex-shrink-0">⚠️</span>
				<div>
					<h3 class="font-semibold text-amber-200">Computer Control is disabled</h3>
					<p class="mt-1 text-sm text-slate-300">
						Enable in the Settings tab below. In supervised mode, every action requires your approval. In autonomous mode, the agent acts independently toward pre-approved goals. A kill switch is always available.
					</p>
				</div>
			</div>
		</div>
	{/if}

	<!-- ═══ Tabs ═══════════════════════════════════════════════════════════════ -->
	<div class="flex gap-1 rounded-2xl border border-white/10 bg-slate-950/40 p-1 backdrop-blur overflow-x-auto">
		{#each [
			{ id: 'dashboard', label: '📊 Dashboard' },
			{ id: 'desktop', label: '🖥️ Desktop Agent' },
			{ id: 'android', label: '🐼 Android Panda' },
			{ id: 'tasks', label: `⚡ Tasks ${$ccRunningTasksStore.length > 0 ? `(${$ccRunningTasksStore.length})` : ''}` },
			{ id: 'workflows', label: '🔀 Parallel Workflows' },
		] as tab}
			<button
				type="button"
				on:click={() => (activeTab = tab.id as Tab)}
				class="flex-shrink-0 rounded-xl px-4 py-2.5 text-sm font-medium transition-colors
					{activeTab === tab.id
					? 'bg-white/10 text-white'
					: 'text-slate-500 hover:text-slate-300'}"
			>
				{tab.label}
			</button>
		{/each}
	</div>

	{#if loading}
		<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-12 text-center text-sm text-slate-500">
			Loading computer control…
		</div>
	{:else if error}
		<div class="rounded-3xl border border-rose-400/20 bg-rose-500/10 p-6 text-sm text-rose-300">{error}</div>

	<!-- ═══ DASHBOARD TAB ═══════════════════════════════════════════════════════ -->
	{:else if activeTab === 'dashboard'}
		<div class="space-y-5">

			<!-- Quick Stats -->
			<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
				{#each [
					{ label: 'Running Tasks', value: String($ccRunningTasksStore.length), color: 'text-cyan-300', sub: 'agent tasks active' },
					{ label: 'Total Tasks', value: String($ccTasksStore.length), color: 'text-violet-300', sub: 'all time' },
					{ label: 'Completed', value: String($ccTasksStore.filter(t => t.status === 'completed').length), color: 'text-green-300', sub: 'successfully' },
					{ label: 'Mode', value: $ccSettingsStore?.mode ?? 'supervised', color: 'text-amber-300', sub: 'current mode' }
				] as stat}
					<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
						<p class="text-xs text-slate-500">{stat.label}</p>
						<p class="mt-2 text-2xl font-semibold {stat.color}">{stat.value}</p>
						<p class="mt-1 text-xs text-slate-600">{stat.sub}</p>
					</div>
				{/each}
			</div>

			<!-- Quick Start: Parallel Workflow Example -->
			<div class="rounded-3xl border border-violet-400/20 bg-violet-950/20 p-6 backdrop-blur">
				<h3 class="text-base font-semibold text-violet-200 mb-2">
					🚀 Quick Start: Watch Tutorial + Do Taxes in Excel
				</h3>
				<p class="text-sm text-slate-400 mb-4">
					Classic Vy parallel workflow — you watch YouTube, agent does your taxes in Excel and updates Notion simultaneously.
				</p>
				<button
					type="button"
					on:click={async () => {
						activeTab = 'workflows';
						wfName = 'Tutorial + Excel Taxes';
						wfForeground = 'Watch tutorial video';
						wfBackgroundGoals = 'Open Excel, navigate to taxes spreadsheet, fill in Q4 revenue figures from memory, calculate totals, save file\nOpen Notion, find the project tracker page, update status to "Tax filing in progress", add today\'s date';
					}}
					class="rounded-2xl bg-gradient-to-r from-violet-500 to-cyan-500 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-violet-950/30"
				>
					Configure This Workflow →
				</button>
				<a
					href="/computer-control/example"
					class="inline-block rounded-2xl border border-violet-400/30 px-5 py-3 text-sm font-medium text-violet-300 hover:bg-violet-400/10 transition-colors"
				>
					Or use the guided demo →
				</a>
			</div>

			<!-- Live Screenshot Preview -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
				<div class="flex items-center justify-between mb-4">
					<h3 class="text-sm font-semibold text-white">Live Screen View</h3>
					<div class="flex items-center gap-3">
						{#if $ccLastScreenshotTimeStore}
							<span class="text-xs text-slate-500">Last: {new Date($ccLastScreenshotTimeStore).toLocaleTimeString()}</span>
						{/if}
						<button
							type="button"
							on:click={handleTakeScreenshot}
							disabled={screenshotLoading || !$ccSettingsStore?.enabled}
							class="rounded-xl bg-slate-800 px-3 py-1.5 text-xs font-medium text-slate-300 hover:bg-slate-700 disabled:opacity-50 transition-colors"
						>
							{screenshotLoading ? 'Capturing…' : '📸 Capture'}
						</button>
					</div>
				</div>
				{#if $ccScreenshotStore}
					<img
						src="data:image/png;base64,{$ccScreenshotStore}"
						alt="Screen capture"
						class="w-full rounded-2xl border border-white/10 object-contain max-h-96"
					/>
				{:else}
					<div class="flex h-48 items-center justify-center rounded-2xl border border-dashed border-white/10 text-sm text-slate-600">
						{$ccSettingsStore?.enabled ? 'Press Capture to take a screenshot' : 'Enable Computer Control to use screen capture'}
					</div>
				{/if}
			</div>

			<!-- Recent Tasks -->
			{#if $ccTasksStore.length > 0}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
					<h3 class="text-sm font-semibold text-white mb-3">Recent Agent Tasks</h3>
					<div class="space-y-2">
						{#each $ccTasksStore.slice(0, 5) as task}
							<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-4">
								<div class="flex items-center gap-3 flex-wrap">
									<span class="text-sm font-medium text-white flex-1 min-w-0 truncate">{task.title}</span>
									<span class="rounded-full px-2 py-0.5 text-xs font-medium {STATUS_COLORS[task.status] ?? 'bg-slate-700 text-slate-400'}">
										{task.status}
									</span>
									{#if task.status === 'running'}
										<span class="text-xs text-cyan-400">{task.progressPct.toFixed(0)}%</span>
									{/if}
								</div>
								{#if task.status === 'running'}
									<div class="mt-2 h-1.5 w-full rounded-full bg-slate-800">
										<div
											class="h-1.5 rounded-full bg-gradient-to-r from-cyan-500 to-violet-500 transition-all duration-500"
											style="width: {task.progressPct}%"
										></div>
									</div>
									<p class="mt-1.5 text-xs text-slate-500">{task.currentStep}</p>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>

	<!-- ═══ DESKTOP AGENT TAB ══════════════════════════════════════════════════ -->
	{:else if activeTab === 'desktop'}
		<div class="space-y-5">

			<!-- Connect Desktop -->
			<div class="rounded-3xl border border-cyan-400/20 bg-cyan-950/15 p-6 backdrop-blur">
				<div class="flex items-start gap-4">
					<span class="text-3xl">🖥️</span>
					<div class="flex-1">
						<h3 class="font-semibold text-cyan-200">Desktop Agent</h3>
						<p class="mt-1 text-sm text-slate-400">
							Powered by suitedaces/computer-agent + simular-ai/Agent-S vision loop. Uses xdotool (Linux) or PowerShell (Windows) for mouse/keyboard control.
						</p>
						<div class="mt-3 flex gap-3">
							{#if $ccSettingsStore?.enabled}
								<span class="rounded-full bg-green-500/20 border border-green-400/30 px-3 py-1 text-xs text-green-300">
									✓ Connected
								</span>
							{:else}
								<button
									type="button"
									on:click={() => {
										if ($ccSettingsStore) {
											$ccSettingsStore.enabled = true;
											saveCcSettings($ccSettingsStore);
										}
									}}
									class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-2 text-sm font-semibold text-slate-950 shadow-lg"
								>
									One-Click Connect Desktop Control
								</button>
							{/if}
						</div>
					</div>
				</div>
			</div>

			<!-- Settings -->
			{#if $ccSettingsStore}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur space-y-5">
					<h3 class="text-sm font-semibold text-white">Desktop Agent Settings</h3>

					<div class="grid gap-4 sm:grid-cols-2">
						<div class="flex items-center justify-between rounded-2xl border border-white/10 bg-slate-900/50 p-4">
							<div>
								<p class="text-sm font-medium text-white">Computer Control</p>
								<p class="text-xs text-slate-500">Global enable/disable</p>
							</div>
							<button
								type="button"
								on:click={() => { $ccSettingsStore!.enabled = !$ccSettingsStore!.enabled; }}
								class="relative h-6 w-11 rounded-full transition-colors {$ccSettingsStore.enabled ? 'bg-cyan-500' : 'bg-slate-700'}"
							>
								<span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {$ccSettingsStore.enabled ? 'translate-x-5' : ''}"></span>
							</button>
						</div>

						<div class="flex items-center justify-between rounded-2xl border border-white/10 bg-slate-900/50 p-4">
							<div>
								<p class="text-sm font-medium text-white">Background Tasks</p>
								<p class="text-xs text-slate-500">Run while you do other things</p>
							</div>
							<button
								type="button"
								on:click={() => { $ccSettingsStore!.allowBackgroundTasks = !$ccSettingsStore!.allowBackgroundTasks; }}
								class="relative h-6 w-11 rounded-full transition-colors {$ccSettingsStore.allowBackgroundTasks ? 'bg-violet-500' : 'bg-slate-700'}"
							>
								<span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {$ccSettingsStore.allowBackgroundTasks ? 'translate-x-5' : ''}"></span>
							</button>
						</div>

						<div class="flex items-center justify-between rounded-2xl border border-white/10 bg-slate-900/50 p-4">
							<div>
								<p class="text-sm font-medium text-white">Require Confirmation</p>
								<p class="text-xs text-slate-500">Prompt before each action</p>
							</div>
							<button
								type="button"
								on:click={() => { $ccSettingsStore!.requireConfirmation = !$ccSettingsStore!.requireConfirmation; }}
								class="relative h-6 w-11 rounded-full transition-colors {$ccSettingsStore.requireConfirmation ? 'bg-amber-500' : 'bg-slate-700'}"
							>
								<span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {$ccSettingsStore.requireConfirmation ? 'translate-x-5' : ''}"></span>
							</button>
						</div>

						<div class="rounded-2xl border border-white/10 bg-slate-900/50 p-4">
							<p class="text-sm font-medium text-white mb-2">Agent Mode</p>
							<div class="flex gap-2">
								{#each ['supervised', 'autonomous'] as mode}
									<button
										type="button"
										on:click={() => { $ccSettingsStore!.mode = mode as 'supervised' | 'autonomous'; }}
										class="flex-1 rounded-xl px-3 py-2 text-xs font-medium transition-colors
											{$ccSettingsStore.mode === mode
											? (mode === 'supervised' ? 'bg-amber-400/20 text-amber-200 border border-amber-400/30' : 'bg-violet-400/20 text-violet-200 border border-violet-400/30')
											: 'bg-slate-800 text-slate-500 border border-transparent'}"
									>
										{mode === 'supervised' ? '👁 Supervised' : '🤖 Autonomous'}
									</button>
								{/each}
							</div>
						</div>
					</div>

					<div class="flex items-center gap-3">
						<button
							type="button"
							on:click={handleSaveSettings}
							disabled={savingSettings}
							class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-2.5 text-sm font-semibold text-slate-950 shadow disabled:opacity-60"
						>
							{savingSettings ? 'Saving…' : 'Save Settings'}
						</button>
						{#if settingsStatus}
							<p class="text-sm {settingsStatus.startsWith('✓') ? 'text-green-400' : 'text-rose-400'}">{settingsStatus}</p>
						{/if}
					</div>
				</div>
			{/if}

			<!-- Action Tester -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h3 class="text-sm font-semibold text-white mb-4">Action Tester</h3>
				<div class="grid gap-3 sm:grid-cols-2">
					<div>
						<label class="block text-xs font-medium text-slate-400 mb-1.5">Action Kind</label>
						<select
							bind:value={actionKind}
							class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
						>
							{#each ['screenshot', 'mouse_click', 'mouse_move', 'mouse_right_click', 'mouse_double_click', 'type_text', 'key_press', 'key_combo', 'open_app', 'close_app', 'shell', 'wait'] as k}
								<option value={k}>{k}</option>
							{/each}
						</select>
					</div>
					{#if ['mouse_click', 'mouse_move', 'mouse_right_click', 'mouse_double_click', 'mouse_scroll'].includes(actionKind)}
						<div class="grid grid-cols-2 gap-2">
							<div>
								<label class="block text-xs font-medium text-slate-400 mb-1.5">X</label>
								<input type="number" bind:value={actionX} placeholder="0" class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none" />
							</div>
							<div>
								<label class="block text-xs font-medium text-slate-400 mb-1.5">Y</label>
								<input type="number" bind:value={actionY} placeholder="0" class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none" />
							</div>
						</div>
					{/if}
					{#if ['type_text', 'key_press', 'key_combo', 'open_app', 'close_app', 'shell'].includes(actionKind)}
						<div class="sm:col-span-2">
							<label class="block text-xs font-medium text-slate-400 mb-1.5">
								{actionKind === 'type_text' ? 'Text to type' : actionKind === 'shell' ? 'Command' : actionKind.includes('app') ? 'App name' : 'Keys (comma-separated)'}
							</label>
							<input type="text" bind:value={actionText} placeholder="e.g. Hello World" class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none" />
						</div>
					{/if}
				</div>

				<div class="mt-4 flex items-center gap-3">
					<button
						type="button"
						on:click={handleExecuteAction}
						disabled={executingAction || !$ccSettingsStore?.enabled}
						class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-2.5 text-sm font-semibold text-slate-950 shadow disabled:opacity-60"
					>
						{executingAction ? 'Executing…' : '▶ Execute Action'}
					</button>
					{#if actionResult}
						<p class="text-sm {actionResult.startsWith('✓') ? 'text-green-400' : 'text-rose-400'}">{actionResult}</p>
					{/if}
				</div>
			</div>
		</div>

	<!-- ═══ ANDROID PANDA TAB ══════════════════════════════════════════════════ -->
	{:else if activeTab === 'android'}
		<div class="space-y-5">

			<!-- Connect Panda -->
			<div class="rounded-3xl border border-green-400/20 bg-green-950/15 p-6 backdrop-blur">
				<div class="flex items-start gap-4">
					<span class="text-3xl">🐼</span>
					<div class="flex-1">
						<h3 class="font-semibold text-green-200">Panda Android Agent</h3>
						<p class="mt-1 text-sm text-slate-400">
							Powered by Ayush0Chaudhary/blurr AccessibilityService. Full control over any Android app — tap, scroll, type, navigate.
						</p>

						<div class="mt-4 grid gap-3 sm:grid-cols-2">
							<div class="rounded-2xl border border-white/10 bg-slate-900/60 p-4">
								<p class="text-xs font-semibold text-slate-400 mb-2">Status</p>
								<div class="flex items-center gap-2">
									<span class="h-2 w-2 rounded-full {pandaStatus?.connected ? 'bg-green-400' : 'bg-slate-600'}"></span>
									<span class="text-sm text-white">{pandaStatus?.connected ? 'Connected' : 'Not connected'}</span>
								</div>
								<p class="mt-1 text-xs text-slate-500">Version: {pandaStatus?.version ?? '—'}</p>
							</div>
							<div class="rounded-2xl border border-white/10 bg-slate-900/60 p-4">
								<p class="text-xs font-semibold text-slate-400 mb-2">Bridge Port</p>
								<p class="text-lg font-mono text-cyan-300">:7799</p>
								<p class="text-xs text-slate-500">TCP bridge on Android device</p>
							</div>
						</div>

						<button
							type="button"
							on:click={async () => {
								if ($ccSettingsStore) {
									$ccSettingsStore.androidPandaEnabled = true;
									await saveCcSettings($ccSettingsStore);
								}
							}}
							class="mt-4 rounded-full bg-gradient-to-r from-green-500 to-cyan-500 px-5 py-2 text-sm font-semibold text-white shadow-lg shadow-green-950/30"
						>
							Connect Android Panda
						</button>
					</div>
				</div>
			</div>

			<!-- Setup Guide -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h3 class="text-sm font-semibold text-white mb-4">Setup Guide</h3>
				<ol class="space-y-3">
					{#each [
						{ step: '1', title: 'Enable Unknown Sources', desc: 'Android Settings → Security → Install unknown apps → Allow' },
						{ step: '2', title: 'Install AmitOS-Panda.apk', desc: 'Download and install the Panda APK on your Android device' },
						{ step: '3', title: 'Enable Accessibility Service', desc: 'Settings → Accessibility → Installed Services → Panda Agent → Enable' },
						{ step: '4', title: 'Connect via ADB', desc: 'adb forward tcp:7799 tcp:7799 (or use same WiFi network)' },
						{ step: '5', title: 'Connect here', desc: 'Press "Connect Android Panda" above — the agent will start responding' }
					] as item}
						<div class="flex gap-4">
							<span class="flex-shrink-0 h-7 w-7 rounded-full bg-green-500/20 text-green-300 text-xs font-semibold flex items-center justify-center">
								{item.step}
							</span>
							<div>
								<p class="text-sm font-medium text-white">{item.title}</p>
								<p class="text-xs text-slate-500 mt-0.5">{item.desc}</p>
							</div>
						</div>
					{/each}
				</ol>
			</div>

			<!-- Capabilities -->
			{#if pandaStatus?.capabilities}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
					<h3 class="text-sm font-semibold text-white mb-3">Capabilities</h3>
					<div class="flex flex-wrap gap-2">
						{#each String(pandaStatus.capabilities).split(',') as cap}
							<span class="rounded-full border border-green-400/20 bg-green-400/10 px-3 py-1 text-xs text-green-300">{cap.trim()}</span>
						{/each}
					</div>
				</div>
			{/if}
		</div>

	<!-- ═══ TASKS TAB ══════════════════════════════════════════════════════════ -->
	{:else if activeTab === 'tasks'}
		<div class="space-y-5">

			<!-- Create Task -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur space-y-4">
				<h3 class="text-sm font-semibold text-white">Launch Agent Task</h3>

				<div class="grid gap-4 sm:grid-cols-2">
					<div>
						<label class="block text-xs font-medium text-slate-400 mb-1.5">Task Title *</label>
						<input
							type="text"
							bind:value={newTaskTitle}
							placeholder="e.g. Fill in Q4 taxes in Excel"
							class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40"
						/>
					</div>
					<div>
						<label class="block text-xs font-medium text-slate-400 mb-1.5">Mode</label>
						<div class="flex gap-2">
							{#each ['supervised', 'autonomous'] as m}
								<button
									type="button"
									on:click={() => (newTaskMode = m as 'supervised' | 'autonomous')}
									class="flex-1 rounded-xl px-3 py-3 text-xs font-medium border transition-colors
										{newTaskMode === m
										? (m === 'supervised' ? 'bg-amber-400/20 text-amber-200 border-amber-400/30' : 'bg-violet-400/20 text-violet-200 border-violet-400/30')
										: 'bg-slate-800 text-slate-500 border-transparent'}"
								>
									{m === 'supervised' ? '👁 Supervised' : '🤖 Autonomous'}
								</button>
							{/each}
						</div>
					</div>
					<div class="sm:col-span-2">
						<label class="block text-xs font-medium text-slate-400 mb-1.5">Goal / Instructions *</label>
						<textarea
							bind:value={newTaskGoal}
							placeholder="Describe exactly what the agent should do. Be specific: which app, which cells, what data..."
							rows="3"
							class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40 resize-none"
						></textarea>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<button
						type="button"
						on:click={handleStartTask}
						disabled={creatingTask || !newTaskTitle || !newTaskGoal || !$ccSettingsStore?.enabled}
						class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg disabled:opacity-60"
					>
						{creatingTask ? 'Launching…' : '🚀 Launch Task'}
					</button>
					{#if !$ccSettingsStore?.enabled}
						<p class="text-xs text-amber-400">Enable Computer Control first</p>
					{/if}
					{#if taskStatus}
						<p class="text-sm {taskStatus.startsWith('✓') ? 'text-green-400' : 'text-rose-400'}">{taskStatus}</p>
					{/if}
				</div>
			</div>

			<!-- Task List -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
				<div class="flex items-center justify-between mb-4">
					<h3 class="text-sm font-semibold text-white">Agent Tasks</h3>
					<span class="text-xs text-slate-500">{$ccTasksStore.length} total</span>
				</div>

				{#if $ccTasksStore.length === 0}
					<p class="text-sm text-slate-500 text-center py-8">No tasks yet. Create one above.</p>
				{:else}
					<div class="space-y-3">
						{#each $ccTasksStore as task (task.id)}
							<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-4">
								<div class="flex items-start justify-between gap-3">
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-2 flex-wrap">
											<span class="text-sm font-medium text-white truncate">{task.title}</span>
											<span class="rounded-full px-2 py-0.5 text-xs font-medium {STATUS_COLORS[task.status]}">
												{task.status}
											</span>
											<span class="text-xs text-slate-600">{task.mode}</span>
										</div>
										<p class="mt-1 text-xs text-slate-500 line-clamp-1">{task.goal}</p>

										{#if task.status === 'running'}
											<div class="mt-2 space-y-1">
												<div class="flex justify-between text-xs text-slate-500">
													<span>{task.currentStep}</span>
													<span>{task.progressPct.toFixed(0)}%</span>
												</div>
												<div class="h-1.5 w-full rounded-full bg-slate-800">
													<div
														class="h-1.5 rounded-full bg-gradient-to-r from-cyan-500 to-violet-500 transition-all duration-500"
														style="width: {task.progressPct}%"
													></div>
												</div>
											</div>
										{/if}

										<!-- Last log entry -->
										{#if task.log.length > 0}
											<p class="mt-1.5 text-xs font-mono text-slate-600 line-clamp-1">
												{task.log[task.log.length - 1]}
											</p>
										{/if}

										{#if task.kaizenTaskId}
											<p class="mt-1 text-xs text-violet-400">📌 Kaizen: {task.kaizenTaskId.slice(0, 8)}…</p>
										{/if}
									</div>

									<div class="flex flex-col gap-2 flex-shrink-0">
										{#if task.status === 'running' || task.status === 'queued'}
											<button
												type="button"
												on:click={() => handleStopTask(task.id)}
												class="rounded-xl bg-rose-500/15 border border-rose-400/20 px-3 py-1.5 text-xs text-rose-300 hover:bg-rose-500/25"
											>
												Stop
											</button>
										{/if}
										{#if task.screenshotB64}
											<img
												src="data:image/png;base64,{task.screenshotB64}"
												alt="Agent screenshot"
												class="w-20 rounded-lg border border-white/10"
											/>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>

	<!-- ═══ PARALLEL WORKFLOWS TAB ══════════════════════════════════════════════ -->
	{:else if activeTab === 'workflows'}
		<div class="space-y-5">

			<!-- Create Workflow -->
			<div class="rounded-3xl border border-violet-400/20 bg-violet-950/15 p-6 backdrop-blur space-y-4">
				<h3 class="text-sm font-semibold text-violet-200">Create Parallel Workflow</h3>
				<p class="text-xs text-slate-400">
					You do the foreground task (e.g. watch a tutorial), the agent handles all background tasks simultaneously.
				</p>

				<div class="space-y-3">
					<div>
						<label class="block text-xs font-medium text-slate-400 mb-1.5">Workflow Name *</label>
						<input
							type="text"
							bind:value={wfName}
							placeholder="e.g. Tutorial + Excel Taxes"
							class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-violet-400/40"
						/>
					</div>
					<div>
						<label class="block text-xs font-medium text-slate-400 mb-1.5">What YOU are doing (foreground) *</label>
						<input
							type="text"
							bind:value={wfForeground}
							placeholder="e.g. Watch YouTube tutorial on tax filing"
							class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-violet-400/40"
						/>
					</div>
					<div>
						<label class="block text-xs font-medium text-slate-400 mb-1.5">
							What the AGENT should do (background — one goal per line) *
						</label>
						<textarea
							bind:value={wfBackgroundGoals}
							rows="4"
							class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white font-mono outline-none focus:border-violet-400/40 resize-none"
							placeholder="Open Excel, navigate to Sheet3 (taxes), fill in revenue figures...&#10;Open Notion, find project tracker, mark taxes as 'in progress'..."
						></textarea>
						<p class="mt-1 text-xs text-slate-600">Each line = one background agent task</p>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<button
						type="button"
						on:click={handleStartParallelWorkflow}
						disabled={creatingWorkflow || !wfName || !wfForeground || !$ccSettingsStore?.enabled || !$ccSettingsStore?.allowBackgroundTasks}
						class="rounded-full bg-gradient-to-r from-violet-500 to-cyan-500 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-violet-950/30 disabled:opacity-60"
					>
						{creatingWorkflow ? 'Launching…' : '🚀 Start Parallel Workflow'}
					</button>
					{#if !$ccSettingsStore?.allowBackgroundTasks}
						<p class="text-xs text-amber-400">Enable Background Tasks in Desktop tab first</p>
					{/if}
					{#if workflowStatus}
						<p class="text-sm {workflowStatus.startsWith('✓') ? 'text-green-400' : 'text-rose-400'}">{workflowStatus}</p>
					{/if}
				</div>
			</div>

			<!-- Workflow List -->
			{#if $ccWorkflowsStore.length > 0}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
					<h3 class="text-sm font-semibold text-white mb-4">Active Workflows</h3>
					<div class="space-y-3">
						{#each $ccWorkflowsStore as wf (wf.id)}
							<div class="rounded-2xl border border-violet-400/15 bg-slate-900/40 p-4">
								<div class="flex items-center justify-between gap-3">
									<div>
										<p class="text-sm font-semibold text-white">{wf.name}</p>
										<p class="text-xs text-slate-500 mt-0.5">
											Foreground: {wf.foregroundTask}
										</p>
										<p class="text-xs text-slate-600 mt-0.5">
											{wf.backgroundTasks.length} background tasks
										</p>
									</div>
									<span class="rounded-full px-2 py-0.5 text-xs bg-cyan-500/20 text-cyan-300">
										{wf.status}
									</span>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Example Workflow Card -->
			<div class="rounded-3xl border border-amber-400/20 bg-amber-950/10 p-5 backdrop-blur">
				<h3 class="text-sm font-semibold text-amber-200 mb-2">📖 Example: The Classic Vy Use Case</h3>
				<div class="space-y-2 text-sm text-slate-400">
					<p><strong class="text-white">You:</strong> Watch "How to file Q4 taxes" tutorial on YouTube</p>
					<p><strong class="text-white">Agent (background 1):</strong> Opens Excel → navigates to tax spreadsheet → enters revenue data from last month → calculates totals → saves</p>
					<p><strong class="text-white">Agent (background 2):</strong> Opens Notion → finds project tracker → updates tax status → adds notes → closes</p>
					<p><strong class="text-white">Agent (background 3):</strong> Takes screenshots at each step → logs evidence to Memory Spine → creates Kaizen completion task</p>
				</div>
			</div>
		</div>
	{/if}
</section>
