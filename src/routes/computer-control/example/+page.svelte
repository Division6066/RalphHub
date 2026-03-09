<script lang="ts">
	/**
	 * Tutorial + Excel Taxes — The Classic Vy Parallel Workflow
	 *
	 * Demonstrates: you watch a YouTube tutorial while the agent:
	 * 1. Opens Excel and fills in your taxes
	 * 2. Updates Notion project tracker
	 * 3. Logs everything to Memory Spine + creates Kaizen tasks
	 */
	import { onMount, onDestroy } from 'svelte';
	import {
		loadCcSettings,
		saveCcSettings,
		startParallelWorkflow,
		startAgentTask,
		loadAgentTasks,
		activateKillSwitch,
		ccSettingsStore,
		ccTasksStore,
		ccKillSwitchStore,
		ccRunningTasksStore,
		type AgentTask,
		type ParallelWorkflow
	} from '$lib/utils/computer-control';
	import { logApiUsage, createKaizenTask, activeProviderIdStore, activeModelStore } from '$lib/utils/provider-registry';
	import PermissionModal from '$lib/components/PermissionModal.svelte';

	// ─── Workflow Configuration ───────────────────────────────────────────────
	let phase: 'setup' | 'running' | 'complete' = 'setup';
	let activeWorkflow: ParallelWorkflow | null = null;
	let tasks: AgentTask[] = [];
	let launching = false;
	let launchError = '';

	// Configurable fields
	let tutorialUrl = 'https://youtube.com/watch?v=...';
	let excelFile = 'C:\\Users\\Me\\Documents\\Taxes2024.xlsx';
	let notionPageUrl = 'https://notion.so/...';
	let taxYear = '2024';
	let agentMode: 'supervised' | 'autonomous' = 'supervised';

	let pollInterval: ReturnType<typeof setInterval>;

	onMount(async () => {
		await loadCcSettings();
		await loadAgentTasks();
	});

	onDestroy(() => clearInterval(pollInterval));

	$: runningTasks = $ccTasksStore.filter(t => t.status === 'running' || t.status === 'queued');
	$: completedTasks = $ccTasksStore.filter(t => t.status === 'completed');
	$: allDone = activeWorkflow && runningTasks.length === 0 && completedTasks.length >= 2;

	async function launchWorkflow() {
		launching = true;
		launchError = '';

		try {
			// Ensure computer control is enabled
			if (!$ccSettingsStore?.enabled) {
				const settings = { ...$ccSettingsStore!, enabled: true, mode: agentMode, allowBackgroundTasks: true };
				await saveCcSettings(settings);
			}

			// Define the background tasks
			const backgroundGoals = [
				`EXCEL TAXES TASK: Open Excel. File path: ${excelFile}. Navigate to the "${taxYear} Tax Return" sheet (or create it if missing). Fill in the following: B4=Revenue (use the last 3 months average from Sheet1.D10:D12), B8=Expenses (sum of Sheet2.E5:E20), B12=Net Income (B4-B8). Format all numbers as currency. Calculate totals. Add today's date in cell A1. Save the file. Take a screenshot as evidence.`,
				`NOTION UPDATE TASK: Open Notion. Go to this page: ${notionPageUrl}. Find the "Tax Status" property or section. Update it to "Filing in Progress - ${taxYear}". Add a note: "Agent filled in Excel on ${new Date().toLocaleDateString()}. Revenue, expenses, and net income calculated. File saved." Add today's date. Close Notion.`,
				`EVIDENCE TASK: Take a final screenshot of the desktop showing both tasks are complete. Write a summary report: what was done, which cells were filled, what the total tax liability appears to be. Save the report to Memory Spine and create a Kaizen completion task.`
			];

			const wf = await startParallelWorkflow(
				`Tutorial + ${taxYear} Taxes`,
				`User watching tutorial: ${tutorialUrl}`,
				backgroundGoals
			);
			activeWorkflow = wf;
			phase = 'running';

			// Log to Memory Spine
			if ($activeProviderIdStore && $activeModelStore) {
				await logApiUsage({
					providerId: $activeProviderIdStore,
					providerName: 'computer-control',
					model: $activeModelStore,
					tokensIn: 0,
					tokensOut: 0,
					costUsd: 0,
					outputSummary: `Parallel workflow launched: Tutorial + ${taxYear} Excel Taxes + Notion update`,
					toolId: 'tutorial-excel-workflow',
					workflowId: wf.id
				});
			}

			// Create master Kaizen task
			await createKaizenTask({
				title: `[CC Parallel] ${taxYear} Tax Filing via Agent`,
				description: `Agent filing taxes in Excel while user watches tutorial.\nExcel: ${excelFile}\nNotion: ${notionPageUrl}`,
				priority: 'urgent',
				source: 'tutorial-excel-workflow',
				providerId: $activeProviderIdStore,
				usageLogId: ''
			});

			// Poll for task updates
			pollInterval = setInterval(async () => {
				await loadAgentTasks();
				if (allDone) {
					phase = 'complete';
					clearInterval(pollInterval);
				}
			}, 1500);

		} catch (e) {
			launchError = e instanceof Error ? e.message : 'Failed to launch workflow';
			launching = false;
		}
	}

	const STEP_ICONS = ['📸', '🔍', '📋', '⌨️', '📸', '🔍', '⌨️', '✅', '⌨️', '📊'];
</script>

<PermissionModal />

<section class="space-y-6">

	<!-- Header -->
	<div class="rounded-[2rem] border border-amber-400/20 bg-gradient-to-br from-amber-950/40 to-violet-950/40 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-amber-300/80">Example Workflow</p>
		<h1 class="mt-3 text-4xl font-semibold tracking-tight text-white">
			Watch Tutorial + Do Taxes in Excel
		</h1>
		<p class="mt-3 max-w-2xl text-base leading-7 text-slate-300">
			The classic Vy parallel workflow: you relax and watch a tutorial, the agent simultaneously fills in your Excel tax spreadsheet and updates Notion — all in the background.
		</p>
		<div class="mt-4 flex flex-wrap gap-3 text-xs">
			{#each ['Vy-style computer control', 'Background execution', 'Memory Spine logging', 'Kaizen task creation', 'Excel + Notion'] as tag}
				<span class="rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-amber-200">{tag}</span>
			{/each}
		</div>
	</div>

	<!-- Kill Switch (always visible when running) -->
	{#if phase === 'running' && !$ccKillSwitchStore}
		<div class="rounded-3xl border border-rose-400/30 bg-rose-950/20 p-4 flex items-center justify-between">
			<div>
				<p class="text-sm font-semibold text-rose-200">Agent is running in the background</p>
				<p class="text-xs text-slate-400">Use the kill switch to immediately stop all actions</p>
			</div>
			<button
				type="button"
				on:click={activateKillSwitch}
				class="rounded-full border border-rose-400 bg-rose-500/20 px-5 py-2 text-sm font-bold text-rose-300 hover:bg-rose-500/40 transition-colors"
			>
				⛔ STOP ALL
			</button>
		</div>
	{/if}

	{#if $ccKillSwitchStore}
		<div class="rounded-3xl border border-rose-400/40 bg-rose-500/15 p-5 text-center">
			<p class="text-lg font-bold text-rose-300">⛔ KILL SWITCH ACTIVE — All agent tasks halted</p>
			<a href="/computer-control" class="mt-2 inline-block text-sm text-rose-400 underline">Go to Computer Control to resume →</a>
		</div>
	{/if}

	<!-- ═══ SETUP PHASE ════════════════════════════════════════════════════════ -->
	{#if phase === 'setup'}
		<div class="grid gap-5 lg:grid-cols-2">

			<!-- Configuration -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur space-y-4">
				<h2 class="text-base font-semibold text-white">Configure Your Workflow</h2>

				<div>
					<label class="block text-xs font-medium text-slate-400 mb-1.5">Tutorial URL (what you'll be watching)</label>
					<input
						type="url"
						bind:value={tutorialUrl}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-amber-400/40"
						placeholder="https://youtube.com/..."
					/>
				</div>

				<div>
					<label class="block text-xs font-medium text-slate-400 mb-1.5">Excel Tax File Path</label>
					<input
						type="text"
						bind:value={excelFile}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white font-mono outline-none focus:border-amber-400/40"
						placeholder="C:\Users\Me\Documents\Taxes2024.xlsx"
					/>
				</div>

				<div>
					<label class="block text-xs font-medium text-slate-400 mb-1.5">Notion Page URL</label>
					<input
						type="url"
						bind:value={notionPageUrl}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-amber-400/40"
						placeholder="https://notion.so/..."
					/>
				</div>

				<div>
					<label class="block text-xs font-medium text-slate-400 mb-1.5">Tax Year</label>
					<input
						type="text"
						bind:value={taxYear}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none focus:border-amber-400/40"
						placeholder="2024"
					/>
				</div>

				<div>
					<label class="block text-xs font-medium text-slate-400 mb-2">Agent Mode</label>
					<div class="grid grid-cols-2 gap-2">
						{#each [{ value: 'supervised', label: '👁 Supervised', desc: 'Ask before each action' }, { value: 'autonomous', label: '🤖 Autonomous', desc: 'Pre-approved goals only' }] as m}
							<button
								type="button"
								on:click={() => (agentMode = m.value as 'supervised' | 'autonomous')}
								class="rounded-2xl border p-3 text-left transition-colors
									{agentMode === m.value
									? 'border-amber-400/40 bg-amber-400/15 text-amber-200'
									: 'border-white/10 bg-slate-900/50 text-slate-400'}"
							>
								<p class="text-sm font-medium">{m.label}</p>
								<p class="text-xs mt-0.5 opacity-70">{m.desc}</p>
							</button>
						{/each}
					</div>
				</div>

				{#if !$ccSettingsStore?.enabled}
					<div class="rounded-2xl border border-amber-400/20 bg-amber-400/5 p-3">
						<p class="text-xs text-amber-300">⚠️ Computer Control is currently disabled. It will be auto-enabled when you launch.</p>
					</div>
				{/if}

				<button
					type="button"
					on:click={launchWorkflow}
					disabled={launching}
					class="w-full rounded-full bg-gradient-to-r from-amber-500 to-violet-500 py-4 text-sm font-bold text-white shadow-lg shadow-amber-950/40 hover:shadow-amber-950/60 transition-shadow disabled:opacity-60"
				>
					{launching ? '🚀 Launching parallel workflow…' : '🚀 Start: I\'ll Watch the Tutorial, Agent Does the Rest'}
				</button>

				{#if launchError}
					<p class="text-sm text-rose-400">{launchError}</p>
				{/if}
			</div>

			<!-- How It Works -->
			<div class="space-y-4">
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
					<h3 class="text-sm font-semibold text-white mb-4">How This Works</h3>
					<div class="space-y-3">
						{#each [
							{ icon: '👤', title: 'You (Foreground)', desc: 'Open YouTube, navigate to your tutorial URL, and start watching. You\'re fully in control of your screen.' },
							{ icon: '🤖', title: 'Agent (Background 1)', desc: 'Silently opens Excel, navigates to your tax spreadsheet, fills in revenue/expense/net income cells from your data, saves.' },
							{ icon: '📋', title: 'Agent (Background 2)', desc: 'Opens Notion, finds your project tracker, updates the tax status field with today\'s date and progress.' },
							{ icon: '📊', title: 'Agent (Background 3)', desc: 'Takes screenshots as evidence, writes a report to Memory Spine, creates a Kaizen completion task.' }
						] as item}
							<div class="flex gap-3">
								<span class="text-xl flex-shrink-0">{item.icon}</span>
								<div>
									<p class="text-sm font-medium text-white">{item.title}</p>
									<p class="text-xs text-slate-500 mt-0.5">{item.desc}</p>
								</div>
							</div>
						{/each}
					</div>
				</div>

				<div class="rounded-3xl border border-rose-400/20 bg-rose-950/15 p-5">
					<h3 class="text-sm font-semibold text-rose-200 mb-2">⚠️ Safety Reminders</h3>
					<ul class="space-y-1.5 text-xs text-slate-400">
						<li>• In supervised mode, a modal appears before each action</li>
						<li>• Kill switch is always visible at the top of this page</li>
						<li>• Agent cannot make financial transactions — only fills spreadsheet cells</li>
						<li>• All actions are logged to Memory Spine for audit</li>
						<li>• You can watch your agent work in the Tasks tab</li>
					</ul>
				</div>
			</div>
		</div>

	<!-- ═══ RUNNING PHASE ══════════════════════════════════════════════════════ -->
	{:else if phase === 'running'}
		<div class="space-y-5">

			<!-- Progress Overview -->
			<div class="grid gap-4 sm:grid-cols-3">
				{#each [
					{ icon: '👤', label: 'You', desc: 'Watching tutorial', status: 'active', color: 'border-blue-400/30 bg-blue-400/10' },
					{ icon: '🤖', label: 'Excel Agent', desc: runningTasks[0]?.currentStep ?? 'Queued', status: runningTasks[0]?.status ?? 'queued', color: 'border-amber-400/30 bg-amber-400/10' },
					{ icon: '📋', label: 'Notion Agent', desc: runningTasks[1]?.currentStep ?? 'Queued', status: runningTasks[1]?.status ?? 'queued', color: 'border-violet-400/30 bg-violet-400/10' }
				] as agent}
					<div class="rounded-3xl border {agent.color} p-5">
						<div class="flex items-center gap-3 mb-2">
							<span class="text-2xl">{agent.icon}</span>
							<div>
								<p class="text-sm font-semibold text-white">{agent.label}</p>
								<p class="text-xs text-slate-400">{agent.status}</p>
							</div>
						</div>
						<p class="text-xs text-slate-500">{agent.desc}</p>
					</div>
				{/each}
			</div>

			<!-- Live Task Progress -->
			{#if $ccTasksStore.length > 0}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
					<h3 class="text-sm font-semibold text-white mb-4">Live Agent Progress</h3>
					<div class="space-y-4">
						{#each $ccTasksStore as task (task.id)}
							<div class="rounded-2xl border border-white/8 bg-slate-900/40 p-4">
								<div class="flex items-center justify-between mb-2">
									<p class="text-sm font-medium text-white">{task.title}</p>
									<span class="rounded-full px-2 py-0.5 text-xs
										{task.status === 'completed' ? 'bg-green-500/20 text-green-300' :
										task.status === 'running' ? 'bg-cyan-500/20 text-cyan-300' :
										task.status === 'killed' ? 'bg-rose-500/20 text-rose-300' :
										'bg-slate-700 text-slate-400'}">
										{task.status}
									</span>
								</div>

								{#if task.status === 'running'}
									<div class="mb-2">
										<div class="flex justify-between text-xs text-slate-500 mb-1">
											<span>{task.currentStep}</span>
											<span>{task.progressPct.toFixed(0)}%</span>
										</div>
										<div class="h-1.5 w-full rounded-full bg-slate-800">
											<div
												class="h-1.5 rounded-full bg-gradient-to-r from-amber-500 to-violet-500 transition-all duration-700"
												style="width: {task.progressPct}%"
											></div>
										</div>
									</div>
								{/if}

								<!-- Log (last 3 entries) -->
								{#if task.log.length > 0}
									<div class="space-y-0.5">
										{#each task.log.slice(-3) as entry}
											<p class="text-xs font-mono text-slate-600">{entry}</p>
										{/each}
									</div>
								{/if}

								{#if task.screenshotB64}
									<img
										src="data:image/png;base64,{task.screenshotB64}"
										alt="Agent screenshot"
										class="mt-3 w-full max-h-40 object-contain rounded-xl border border-white/10"
									/>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>

	<!-- ═══ COMPLETE PHASE ════════════════════════════════════════════════════ -->
	{:else if phase === 'complete'}
		<div class="rounded-3xl border border-green-400/30 bg-green-950/20 p-8 text-center backdrop-blur space-y-4">
			<div class="text-5xl">✅</div>
			<h2 class="text-2xl font-bold text-green-200">Workflow Complete!</h2>
			<p class="text-slate-300">
				Your {taxYear} taxes have been filled in Excel and Notion has been updated — all while you watched your tutorial.
			</p>
			<div class="flex justify-center gap-4 mt-4">
				<a href="/computer-control" class="rounded-full bg-gradient-to-r from-green-500 to-cyan-500 px-6 py-3 text-sm font-semibold text-white shadow-lg">
					View Full Report
				</a>
				<button
					type="button"
					on:click={() => { phase = 'setup'; activeWorkflow = null; }}
					class="rounded-full border border-white/10 px-6 py-3 text-sm text-slate-400 hover:text-white"
				>
					Run Another Workflow
				</button>
			</div>
		</div>
	{/if}

	<!-- Back link -->
	<div class="flex items-center gap-4 text-sm">
		<a href="/computer-control" class="text-violet-400 hover:text-violet-300">← Back to Computer Control</a>
		<a href="/workflows" class="text-slate-500 hover:text-slate-300">Workflow Composer →</a>
	</div>
</section>
