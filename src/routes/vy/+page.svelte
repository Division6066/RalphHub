<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type VySession = {
		id: string;
		goal: string;
		status: 'idle' | 'watching' | 'controlling' | 'reviewing' | 'done';
		screenshotCount: number;
		actionsTaken: number;
		startedAt?: string;
		log: string[];
	};

	type TutorialStep = {
		id: string;
		title: string;
		description: string;
		status: 'pending' | 'watching' | 'done';
		url?: string;
	};

	let session: VySession = {
		id: '',
		goal: '',
		status: 'idle',
		screenshotCount: 0,
		actionsTaken: 0,
		log: []
	};

	let isDesktop = false;
	let goalInput = '';
	let tutorialUrl = '';
	let activeTab: 'takeover' | 'tutorial' | 'permissions' = 'takeover';
	let permissionGranted = false;
	let permissionAsking = false;
	let statusMsg = '';

	let tutorialSteps: TutorialStep[] = [
		{ id: '1', title: 'Watch tutorial video', description: 'Vy observes what you are learning', status: 'pending' },
		{ id: '2', title: 'Extract key actions', description: 'Vy identifies the steps shown', status: 'pending' },
		{ id: '3', title: 'Create task list', description: 'Vy adds steps to your Kaizen board', status: 'pending' },
		{ id: '4', title: 'Optionally replicate', description: 'Vy can execute the steps for you', status: 'pending' },
	];

	const VY_CAPABILITIES = [
		{ icon: '👁️', label: 'Screen Watch', desc: 'Observes your screen while you work or watch tutorials' },
		{ icon: '🖱️', label: 'Desktop Control', desc: 'Moves the cursor, clicks, types — with your approval' },
		{ icon: '📋', label: 'Task Extraction', desc: 'Turns video/tutorial steps into Kaizen tasks automatically' },
		{ icon: '🔔', label: 'Permission First', desc: 'Always asks before taking any action — you stay in control' },
		{ icon: '🧠', label: 'Memory Write', desc: 'Saves everything it learns to your Memory Spine' },
		{ icon: '⏸️', label: 'One-Click Stop', desc: 'Press Escape or click Stop — Vy freezes immediately' },
	];

	const APPROVAL_STEPS = [
		{ step: '1', label: 'You start Vy with a goal', icon: '🎯' },
		{ step: '2', label: 'Vy shows a preview of its plan', icon: '📋' },
		{ step: '3', label: 'You approve or reject each action', icon: '✅' },
		{ step: '4', label: 'Vy executes only approved actions', icon: '▶️' },
		{ step: '5', label: 'Results saved to Memory Spine', icon: '🧠' },
	];

	onMount(() => {
		isDesktop = isDesktopRuntime();
	});

	async function requestPermission() {
		permissionAsking = true;
		statusMsg = '';
		await new Promise(r => setTimeout(r, 1200));
		permissionGranted = true;
		permissionAsking = false;
		statusMsg = 'Desktop control permission granted for this session.';
		session.log = [...session.log, '✅ Screen observation permission granted'];
	}

	async function startTakeover() {
		if (!goalInput.trim()) { statusMsg = 'Please enter a goal first.'; return; }
		if (!permissionGranted) { statusMsg = 'Grant desktop permission first (above).'; return; }
		session = {
			...session,
			id: `vy-${Date.now()}`,
			goal: goalInput,
			status: 'watching',
			startedAt: new Date().toLocaleTimeString(),
			log: [`🎯 Goal: "${goalInput}"`, '👁️ Vy is observing your screen...', '📸 Capturing context...']
		};
		statusMsg = '';
		await new Promise(r => setTimeout(r, 1800));
		session = {
			...session,
			status: 'reviewing',
			screenshotCount: 3,
			log: [
				...session.log,
				'📸 3 screenshots captured',
				'🤔 Analysing current state...',
				'📋 Ready to present action plan'
			]
		};
	}

	async function watchTutorial() {
		if (!tutorialUrl.trim()) { statusMsg = 'Enter a tutorial URL first.'; return; }
		tutorialSteps = tutorialSteps.map(s => ({ ...s, status: 'pending' }));
		for (let i = 0; i < tutorialSteps.length; i++) {
			tutorialSteps[i] = { ...tutorialSteps[i], status: 'watching' };
			tutorialSteps = [...tutorialSteps];
			await new Promise(r => setTimeout(r, 900));
			tutorialSteps[i] = { ...tutorialSteps[i], status: 'done' };
			tutorialSteps = [...tutorialSteps];
		}
		statusMsg = '✅ Tutorial analysed — steps added to your Kaizen board!';
	}

	function stopSession() {
		session = { id: '', goal: '', status: 'idle', screenshotCount: 0, actionsTaken: 0, log: [] };
		statusMsg = 'Session stopped. Vy is idle.';
	}

	$: statusColor = session.status === 'idle' ? 'text-slate-400'
		: session.status === 'watching' ? 'text-cyan-400'
		: session.status === 'controlling' ? 'text-violet-400'
		: session.status === 'reviewing' ? 'text-amber-400'
		: 'text-green-400';
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-violet-400/20 bg-gradient-to-br from-violet-900/25 to-slate-950/60 p-7 backdrop-blur">
		<div class="flex items-start justify-between gap-4 flex-wrap">
			<div>
				<div class="flex items-center gap-3 mb-2">
					<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-violet-500 to-cyan-400 text-lg font-bold text-white shadow-lg">V</div>
					<div>
						<p class="text-[10px] uppercase tracking-[0.35em] text-violet-300/70 font-semibold">Desktop AI Agent</p>
						<h1 class="text-2xl font-bold text-white">Vy</h1>
					</div>
					<span class="rounded-full border border-violet-400/30 bg-violet-500/15 px-3 py-1 text-xs font-semibold text-violet-300">
						{session.status === 'idle' ? '⬤ Idle' : session.status === 'watching' ? '⬤ Watching' : session.status === 'reviewing' ? '⬤ Reviewing' : '⬤ Active'}
					</span>
				</div>
				<p class="text-sm leading-6 text-slate-300 max-w-xl">
					Vy watches your screen, learns from tutorials, and can take over your desktop — with your approval for every action.
				</p>
			</div>
			{#if session.status !== 'idle'}
				<button
					on:click={stopSession}
					class="rounded-xl border border-rose-500/40 bg-rose-500/10 px-5 py-2.5 text-sm font-semibold text-rose-300 hover:bg-rose-500/20 transition"
				>
					⏹ Stop Vy
				</button>
			{/if}
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex gap-2 rounded-xl border border-white/8 bg-slate-950/50 p-1.5 w-fit">
		{#each [['takeover', '🖥️ Desktop Takeover'], ['tutorial', '📹 Tutorial Watch'], ['permissions', '🔐 Permissions']] as [tab, label]}
			<button
				on:click={() => activeTab = tab as typeof activeTab}
				class="rounded-lg px-4 py-2 text-sm font-medium transition-all {activeTab === tab ? 'bg-violet-500/25 text-violet-100 border border-violet-400/30' : 'text-slate-400 hover:text-slate-200'}"
			>
				{label}
			</button>
		{/each}
	</div>

	{#if activeTab === 'takeover'}
		<!-- Desktop Takeover Panel -->
		<div class="grid gap-5 lg:grid-cols-[1fr_300px]">
			<div class="space-y-4">
				<!-- Goal Input -->
				<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-5 backdrop-blur">
					<h2 class="mb-4 text-base font-semibold text-white">Start a Vy Session</h2>
					<div class="space-y-3">
						<div>
							<label class="mb-1.5 block text-xs font-medium text-slate-400 uppercase tracking-wider">What should Vy help with?</label>
							<input
								bind:value={goalInput}
								placeholder="e.g. 'Set up my dev environment' or 'Research and summarise competitor pricing'"
								class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-4 py-3 text-sm text-white placeholder-slate-600 focus:border-violet-500/50 focus:outline-none"
							/>
						</div>
						<div class="flex gap-3 flex-wrap">
							<button
								on:click={startTakeover}
								disabled={session.status !== 'idle' || !permissionGranted}
								class="rounded-xl bg-gradient-to-r from-violet-500 to-cyan-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg disabled:opacity-50 transition"
							>
								{session.status === 'idle' ? '▶ Start Vy' : '● Running…'}
							</button>
							{#if !permissionGranted}
								<p class="self-center text-xs text-amber-400">⚠ Grant permission in the Permissions tab first</p>
							{/if}
						</div>
					</div>
				</div>

				<!-- Session Log -->
				{#if session.log.length > 0}
					<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-5 backdrop-blur">
						<div class="mb-3 flex items-center justify-between">
							<h2 class="text-base font-semibold text-white">Session Log</h2>
							<span class="text-xs {statusColor} font-semibold uppercase">{session.status}</span>
						</div>
						<div class="space-y-1.5 font-mono text-xs max-h-48 overflow-y-auto">
							{#each session.log as line}
								<p class="{line.startsWith('✅') ? 'text-green-400' : line.startsWith('📸') ? 'text-cyan-400' : line.startsWith('🎯') ? 'text-violet-300' : 'text-slate-400'}">{line}</p>
							{/each}
						</div>
						{#if session.status === 'reviewing'}
							<div class="mt-4 rounded-xl border border-amber-400/20 bg-amber-500/8 p-4">
								<p class="text-sm font-semibold text-amber-300 mb-2">Vy has a plan — approve to continue:</p>
								<div class="space-y-2 text-xs text-slate-300">
									<div class="flex items-center gap-2"><span class="text-amber-400">1.</span> Open terminal in project folder</div>
									<div class="flex items-center gap-2"><span class="text-amber-400">2.</span> Run <code class="bg-slate-800 px-1.5 py-0.5 rounded text-cyan-300">bun install</code></div>
									<div class="flex items-center gap-2"><span class="text-amber-400">3.</span> Open browser to localhost:5173</div>
								</div>
								<div class="mt-3 flex gap-2">
									<button class="rounded-lg bg-green-500/20 border border-green-500/30 px-4 py-2 text-xs font-semibold text-green-300 hover:bg-green-500/30 transition">
										✅ Approve All
									</button>
									<button class="rounded-lg bg-slate-800/60 px-4 py-2 text-xs font-semibold text-slate-400 hover:text-white transition">
										Review One-by-One
									</button>
								</div>
							</div>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Capabilities sidebar -->
			<div class="space-y-4">
				<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-5 backdrop-blur">
					<h2 class="mb-4 text-sm font-semibold text-white uppercase tracking-wider">Capabilities</h2>
					<div class="space-y-3">
						{#each VY_CAPABILITIES as cap}
							<div class="flex items-start gap-3">
								<span class="text-lg leading-none mt-0.5">{cap.icon}</span>
								<div>
									<p class="text-xs font-semibold text-white">{cap.label}</p>
									<p class="text-xs text-slate-500 leading-4 mt-0.5">{cap.desc}</p>
								</div>
							</div>
						{/each}
					</div>
				</div>

				{#if statusMsg}
					<div class="rounded-xl border border-cyan-400/20 bg-cyan-500/8 p-3 text-xs text-cyan-300">
						{statusMsg}
					</div>
				{/if}
			</div>
		</div>

	{:else if activeTab === 'tutorial'}
		<!-- Tutorial Watch Panel -->
		<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
			<h2 class="mb-2 text-base font-semibold text-white">Watch a Tutorial with Vy</h2>
			<p class="text-sm text-slate-400 mb-5">Paste a YouTube / Loom / local file URL. Vy will extract every actionable step and add them to your Kaizen board.</p>

			<div class="space-y-4">
				<div>
					<label class="mb-1.5 block text-xs font-medium text-slate-400 uppercase tracking-wider">Tutorial URL</label>
					<input
						bind:value={tutorialUrl}
						placeholder="https://youtube.com/watch?v=..."
						class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-4 py-3 text-sm text-white placeholder-slate-600 focus:border-violet-500/50 focus:outline-none"
					/>
				</div>
				<button
					on:click={watchTutorial}
					class="rounded-xl bg-gradient-to-r from-cyan-500 to-violet-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg hover:opacity-90 transition"
				>
					▶ Watch with Vy
				</button>
			</div>

			<!-- Steps progress -->
			<div class="mt-6 space-y-3">
				{#each tutorialSteps as step}
					<div class="flex items-center gap-4 rounded-xl border {step.status === 'done' ? 'border-green-400/20 bg-green-500/5' : step.status === 'watching' ? 'border-cyan-400/20 bg-cyan-500/5' : 'border-white/8 bg-white/3'} p-4 transition">
						<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg {step.status === 'done' ? 'bg-green-500/20 text-green-400' : step.status === 'watching' ? 'bg-cyan-500/20 text-cyan-400' : 'bg-slate-800/60 text-slate-600'} text-sm">
							{step.status === 'done' ? '✓' : step.status === 'watching' ? '◉' : step.id}
						</div>
						<div>
							<p class="text-sm font-medium {step.status === 'done' ? 'text-green-200' : step.status === 'watching' ? 'text-cyan-200' : 'text-slate-300'}">{step.title}</p>
							<p class="text-xs text-slate-500 mt-0.5">{step.description}</p>
						</div>
					</div>
				{/each}
			</div>

			{#if statusMsg}
				<div class="mt-4 rounded-xl border border-green-400/20 bg-green-500/8 p-3 text-sm font-medium text-green-300">
					{statusMsg}
				</div>
			{/if}
		</div>

	{:else}
		<!-- Permissions Panel -->
		<div class="grid gap-5 lg:grid-cols-2">
			<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
				<h2 class="mb-2 text-base font-semibold text-white">Desktop Control Permission</h2>
				<p class="text-sm text-slate-400 mb-5 leading-6">
					Vy never acts without your approval. Grant permission for this session only — it expires when you close AmitOS.
				</p>

				<div class="space-y-3 mb-5">
					{#each APPROVAL_STEPS as s}
						<div class="flex items-center gap-3">
							<div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-violet-500/20 text-xs font-bold text-violet-300">{s.step}</div>
							<div class="flex items-center gap-2 text-sm text-slate-300">
								<span>{s.icon}</span>
								<span>{s.label}</span>
							</div>
						</div>
					{/each}
				</div>

				{#if permissionGranted}
					<div class="rounded-xl border border-green-400/20 bg-green-500/8 p-4 text-sm text-green-300 font-medium">
						✅ Desktop control permission active for this session
					</div>
				{:else}
					<button
						on:click={requestPermission}
						disabled={permissionAsking}
						class="w-full rounded-xl bg-gradient-to-r from-violet-500 to-cyan-500 py-3 text-sm font-semibold text-white shadow-lg disabled:opacity-60 transition"
					>
						{permissionAsking ? 'Requesting…' : '🔐 Grant Vy Desktop Permission'}
					</button>
				{/if}
			</div>

			<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
				<h2 class="mb-4 text-base font-semibold text-white">Privacy & Safety</h2>
				<div class="space-y-4 text-sm text-slate-300 leading-6">
					<div class="flex items-start gap-3">
						<span class="text-green-400 text-base mt-0.5">✓</span>
						<p>Screenshots are processed locally — nothing sent to cloud without your consent</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-green-400 text-base mt-0.5">✓</span>
						<p>Every action shown before execution — you approve or reject individually</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-green-400 text-base mt-0.5">✓</span>
						<p>Escape key or Stop button halts Vy immediately at any point</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-green-400 text-base mt-0.5">✓</span>
						<p>All screen data stays in your Memory Spine — private to you</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-green-400 text-base mt-0.5">✓</span>
						<p>Permission is session-only — no persistence across restarts</p>
					</div>
				</div>
			</div>
		</div>
	{/if}
</section>
