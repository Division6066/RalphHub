<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type PandaStatus = {
		connected: boolean;
		deviceName?: string;
		platform?: string;
		batteryLevel?: number;
		lastSeen?: string;
		pendingApprovals: number;
	};

	type PendingAction = {
		id: string;
		type: 'desktop-action' | 'voice-capture' | 'task-add' | 'memory-add' | 'workflow-start';
		description: string;
		detail: string;
		requestedAt: string;
		requestedFrom: string;
	};

	type RecentActivity = {
		id: string;
		icon: string;
		label: string;
		time: string;
		color: string;
	};

	let status: PandaStatus = {
		connected: false,
		pendingApprovals: 0
	};

	let pendingActions: PendingAction[] = [];
	let recentActivity: RecentActivity[] = [];
	let activeTab: 'remote' | 'approvals' | 'voice' | 'setup' = 'remote';
	let isDesktop = false;
	let loading = false;
	let statusMsg = '';
	let pairedDevice = '';
	let voicePermissionGranted = false;
	let voiceListening = false;
	let voiceCapture = '';

	const PANDA_FEATURES = [
		{ icon: '🖥️', label: 'Remote Desktop View', desc: 'See your AmitOS screen on your phone' },
		{ icon: '✅', label: 'One-tap Approvals', desc: 'Approve Vy actions from anywhere in the world' },
		{ icon: '🎙️', label: 'Voice Capture', desc: 'Speak tasks and memories — synced in real-time' },
		{ icon: '☀️', label: 'Today Board', desc: 'Check and complete today\'s tasks on the go' },
		{ icon: '🔔', label: 'Workflow Notifications', desc: 'Get notified when overnight runs complete' },
		{ icon: '⚡', label: 'Quick Task Add', desc: 'One-tap to add a Kaizen task from phone' },
	];

	const SETUP_STEPS = [
		{ num: '1', title: 'Enable Mobile Sync', desc: 'Click "Enable Sync" in the Mobile tab or here', icon: '📱' },
		{ num: '2', title: 'Scan QR Code', desc: 'Open AmitOS on your phone and scan the QR code', icon: '📷' },
		{ num: '3', title: 'Grant Voice Permission', desc: 'Allow microphone access for voice capture from phone', icon: '🎙️' },
		{ num: '4', title: 'Panda is Ready', desc: 'Your phone is now a remote control for AmitOS', icon: '🐼' },
	];

	onMount(async () => {
		isDesktop = isDesktopRuntime();

		// Simulate demo pending actions
		pendingActions = [
			{
				id: '1',
				type: 'desktop-action',
				description: 'Vy wants to open Terminal',
				detail: 'Vy is attempting: open /Applications/Utilities/Terminal.app',
				requestedAt: new Date(Date.now() - 12000).toLocaleTimeString(),
				requestedFrom: 'Vy Desktop Agent'
			},
			{
				id: '2',
				type: 'voice-capture',
				description: 'Voice memo from phone',
				detail: '"Remember to check the pricing page on the competitor site tomorrow morning"',
				requestedAt: new Date(Date.now() - 65000).toLocaleTimeString(),
				requestedFrom: 'Panda Mobile'
			}
		];

		recentActivity = [
			{ id: '1', icon: '✅', label: 'Approved: Open VS Code', time: '2 min ago', color: 'text-green-400' },
			{ id: '2', icon: '🎙️', label: 'Voice memo saved to Memory', time: '15 min ago', color: 'text-cyan-400' },
			{ id: '3', icon: '☀️', label: 'Completed: Send weekly report', time: '1 hr ago', color: 'text-violet-400' },
			{ id: '4', icon: '⚡', label: 'Workflow finished: Research run', time: '3 hr ago', color: 'text-amber-400' },
		];

		status.pendingApprovals = pendingActions.length;
	});

	function approveAction(id: string) {
		pendingActions = pendingActions.filter(a => a.id !== id);
		status.pendingApprovals = pendingActions.length;
		recentActivity = [
			{ id: Date.now().toString(), icon: '✅', label: 'Approved from phone', time: 'just now', color: 'text-green-400' },
			...recentActivity
		];
		statusMsg = 'Action approved.';
	}

	function rejectAction(id: string) {
		pendingActions = pendingActions.filter(a => a.id !== id);
		status.pendingApprovals = pendingActions.length;
		statusMsg = 'Action rejected.';
	}

	async function grantVoicePermission() {
		voicePermissionGranted = false;
		try {
			const stream = await (navigator.mediaDevices as any)?.getUserMedia({ audio: true });
			if (stream) {
				stream.getTracks().forEach((t: any) => t.stop());
				voicePermissionGranted = true;
				statusMsg = '✅ Microphone permission granted for Panda voice capture';
			}
		} catch {
			statusMsg = '⚠ Microphone permission denied. Please allow in browser settings.';
		}
	}

	async function startVoiceCapture() {
		voiceListening = true;
		voiceCapture = '';
		await new Promise(r => setTimeout(r, 1500));
		voiceCapture = '"Schedule team sync for Thursday at 2pm"';
		voiceListening = false;
	}

	$: actionTypeColor = (type: string) =>
		type === 'desktop-action' ? 'border-violet-400/20 bg-violet-500/5' :
		type === 'voice-capture' ? 'border-cyan-400/20 bg-cyan-500/5' :
		'border-white/10 bg-white/3';

	$: actionTypeLabel = (type: string) =>
		type === 'desktop-action' ? '🖥️ Desktop' :
		type === 'voice-capture' ? '🎙️ Voice' :
		type === 'task-add' ? '✅ Task' :
		type === 'memory-add' ? '🧠 Memory' : '⚡ Workflow';
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-cyan-400/20 bg-gradient-to-br from-cyan-900/20 to-slate-950/60 p-7 backdrop-blur">
		<div class="flex items-start justify-between gap-4 flex-wrap">
			<div>
				<div class="flex items-center gap-3 mb-2">
					<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-cyan-500 to-violet-400 text-2xl shadow-lg">🐼</div>
					<div>
						<p class="text-[10px] uppercase tracking-[0.35em] text-cyan-300/70 font-semibold">Phone Remote Control</p>
						<h1 class="text-2xl font-bold text-white">Panda</h1>
					</div>
					<span class="rounded-full border {status.connected ? 'border-green-400/30 bg-green-500/10 text-green-300' : 'border-slate-600/30 bg-slate-800/30 text-slate-400'} px-3 py-1 text-xs font-semibold">
						{status.connected ? '⬤ Connected' : '⬤ Not paired'}
					</span>
					{#if status.pendingApprovals > 0}
						<span class="rounded-full bg-rose-500 px-2.5 py-0.5 text-xs font-bold text-white animate-pulse">
							{status.pendingApprovals} pending
						</span>
					{/if}
				</div>
				<p class="text-sm leading-6 text-slate-300 max-w-xl">
					Use your phone as a remote control for AmitOS. Approve Vy actions, capture voice memos, check today's tasks — all from anywhere.
				</p>
			</div>
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex gap-2 rounded-xl border border-white/8 bg-slate-950/50 p-1.5 flex-wrap">
		{#each [['remote', '🖥️ Remote Control'], ['approvals', `✅ Approvals ${status.pendingApprovals > 0 ? `(${status.pendingApprovals})` : ''}`], ['voice', '🎙️ Voice Capture'], ['setup', '⚙️ Setup']] as [tab, label]}
			<button
				on:click={() => activeTab = tab as typeof activeTab}
				class="rounded-lg px-4 py-2 text-sm font-medium transition-all {activeTab === tab ? 'bg-cyan-500/20 text-cyan-100 border border-cyan-400/30' : 'text-slate-400 hover:text-slate-200'}"
			>
				{label}
			</button>
		{/each}
	</div>

	{#if activeTab === 'remote'}
		<!-- Remote Control Panel -->
		<div class="grid gap-5 lg:grid-cols-[1fr_280px]">
			<div class="space-y-4">
				<!-- Phone screen preview -->
				<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-5 backdrop-blur">
					<h2 class="mb-4 text-base font-semibold text-white">Phone Dashboard View</h2>
					<!-- Simulated phone UI -->
					<div class="mx-auto w-[280px] rounded-3xl border-4 border-slate-700 bg-slate-950 p-3 shadow-2xl">
						<div class="mb-2 flex items-center justify-between px-1">
							<span class="text-[9px] text-slate-500">9:41 AM</span>
							<div class="flex gap-1">
								<span class="text-[9px] text-slate-500">●●●</span>
							</div>
						</div>
						<div class="rounded-2xl bg-gradient-to-b from-slate-900 to-slate-950 p-3 space-y-2">
							<div class="flex items-center gap-2 mb-3">
								<div class="h-5 w-5 rounded-md bg-gradient-to-br from-violet-500 to-cyan-400 flex items-center justify-center text-[10px] font-bold text-white">A</div>
								<p class="text-[11px] font-bold text-white">AmitOS · Panda</p>
							</div>
							<div class="rounded-xl bg-cyan-500/10 border border-cyan-400/20 p-2.5">
								<p class="text-[9px] text-cyan-300 uppercase tracking-wider mb-1">Today</p>
								<div class="space-y-1">
									{#each ['Ship landing page copy', 'Review PR #42', 'Evening walk 30min'] as task}
										<div class="flex items-center gap-1.5">
											<div class="h-3 w-3 rounded-sm border border-slate-600"></div>
											<p class="text-[9px] text-slate-300">{task}</p>
										</div>
									{/each}
								</div>
							</div>
							{#if pendingActions.length > 0}
								<div class="rounded-xl bg-rose-500/10 border border-rose-400/20 p-2.5">
									<p class="text-[9px] text-rose-300 font-semibold">{pendingActions.length} actions need approval</p>
									<button class="mt-1.5 w-full rounded-lg bg-rose-500/20 py-1 text-[9px] font-semibold text-rose-200">Review →</button>
								</div>
							{/if}
							<div class="grid grid-cols-3 gap-1.5 pt-1">
								{#each [['☀️', 'Today'], ['🧠', 'Memory'], ['⚡', 'Workflows']] as [icon, label]}
									<button class="rounded-xl border border-white/8 bg-white/5 p-2 text-center">
										<p class="text-base">{icon}</p>
										<p class="text-[8px] text-slate-400 mt-0.5">{label}</p>
									</button>
								{/each}
							</div>
						</div>
					</div>
					<p class="mt-4 text-center text-xs text-slate-500">This is how AmitOS looks on your phone via Panda</p>
				</div>
			</div>

			<!-- Recent activity sidebar -->
			<div class="space-y-4">
				<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-5 backdrop-blur">
					<h2 class="mb-4 text-sm font-semibold text-white uppercase tracking-wider">Recent Activity</h2>
					<div class="space-y-3">
						{#each recentActivity as item}
							<div class="flex items-center gap-3">
								<span class="text-base {item.color}">{item.icon}</span>
								<div class="min-w-0">
									<p class="text-xs text-slate-300 truncate">{item.label}</p>
									<p class="text-[10px] text-slate-600">{item.time}</p>
								</div>
							</div>
						{/each}
					</div>
				</div>

				<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-5 backdrop-blur">
					<h2 class="mb-3 text-sm font-semibold text-white uppercase tracking-wider">Features</h2>
					<div class="space-y-3">
						{#each PANDA_FEATURES as feat}
							<div class="flex items-start gap-2.5">
								<span class="text-base leading-none mt-0.5">{feat.icon}</span>
								<div>
									<p class="text-xs font-medium text-white">{feat.label}</p>
									<p class="text-[10px] text-slate-500 mt-0.5 leading-4">{feat.desc}</p>
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>

	{:else if activeTab === 'approvals'}
		<!-- Pending Approvals -->
		<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
			<h2 class="mb-2 text-base font-semibold text-white">Pending Approvals</h2>
			<p class="text-sm text-slate-400 mb-5">These actions are waiting for your approval from the phone or here.</p>

			{#if pendingActions.length === 0}
				<div class="rounded-xl border border-white/8 bg-white/3 p-8 text-center">
					<p class="text-2xl mb-2">✅</p>
					<p class="text-sm text-slate-400">No pending approvals</p>
					<p class="text-xs text-slate-600 mt-1">All caught up!</p>
				</div>
			{:else}
				<div class="space-y-4">
					{#each pendingActions as action}
						<div class="rounded-2xl border {actionTypeColor(action.type)} p-5">
							<div class="flex items-start justify-between gap-4">
								<div class="min-w-0">
									<div class="flex items-center gap-2 mb-1">
										<span class="text-xs font-semibold rounded-full bg-slate-800/60 px-2 py-0.5 text-slate-400">{actionTypeLabel(action.type)}</span>
										<span class="text-xs text-slate-600">from {action.requestedFrom}</span>
									</div>
									<p class="text-sm font-semibold text-white mb-1">{action.description}</p>
									<p class="text-xs text-slate-400 font-mono bg-slate-900/60 rounded-lg px-3 py-2 mt-2">{action.detail}</p>
									<p class="text-xs text-slate-600 mt-2">Requested at {action.requestedAt}</p>
								</div>
							</div>
							<div class="mt-4 flex gap-2">
								<button
									on:click={() => approveAction(action.id)}
									class="rounded-xl bg-green-500/20 border border-green-500/30 px-5 py-2.5 text-sm font-semibold text-green-300 hover:bg-green-500/30 transition"
								>
									✅ Approve
								</button>
								<button
									on:click={() => rejectAction(action.id)}
									class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 text-sm font-semibold text-slate-400 hover:text-white transition"
								>
									✗ Reject
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}

			{#if statusMsg}
				<div class="mt-4 rounded-xl border border-cyan-400/20 bg-cyan-500/8 p-3 text-xs text-cyan-300">{statusMsg}</div>
			{/if}
		</div>

	{:else if activeTab === 'voice'}
		<!-- Voice Capture from Phone -->
		<div class="grid gap-5 lg:grid-cols-2">
			<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
				<h2 class="mb-2 text-base font-semibold text-white">Phone Voice Capture</h2>
				<p class="text-sm text-slate-400 mb-5 leading-6">
					Use your phone's microphone to quickly capture tasks and memories — without touching your keyboard.
				</p>

				{#if !voicePermissionGranted}
					<div class="rounded-xl border border-amber-400/20 bg-amber-500/8 p-4 mb-4">
						<p class="text-sm font-medium text-amber-300 mb-3">Microphone permission required</p>
						<button
							on:click={grantVoicePermission}
							class="rounded-xl bg-gradient-to-r from-cyan-500 to-violet-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg hover:opacity-90 transition"
						>
							🎙️ Grant Microphone Permission
						</button>
					</div>
				{:else}
					<div class="rounded-xl border border-green-400/20 bg-green-500/8 p-3 mb-4 text-xs text-green-300 font-medium">
						✅ Microphone permission active
					</div>
					<div class="space-y-4">
						<button
							on:click={startVoiceCapture}
							disabled={voiceListening}
							class="w-full rounded-2xl border-2 {voiceListening ? 'border-cyan-400 bg-cyan-500/10 animate-pulse' : 'border-white/15 bg-white/5 hover:bg-white/8'} py-8 text-center transition"
						>
							<p class="text-4xl mb-2">{voiceListening ? '🔴' : '🎙️'}</p>
							<p class="text-sm font-semibold text-white">{voiceListening ? 'Listening…' : 'Tap to Record'}</p>
							<p class="text-xs text-slate-500 mt-1">{voiceListening ? 'Speak your task or memory' : 'Voice captured instantly'}</p>
						</button>

						{#if voiceCapture}
							<div class="rounded-xl border border-cyan-400/20 bg-cyan-500/8 p-4">
								<p class="text-xs text-cyan-300 uppercase tracking-wider mb-2 font-semibold">Captured</p>
								<p class="text-sm text-white italic">"{voiceCapture}"</p>
								<div class="mt-3 flex gap-2">
									<button class="rounded-lg bg-green-500/20 border border-green-500/30 px-3 py-1.5 text-xs font-semibold text-green-300 hover:bg-green-500/30 transition">
										Save to Memory
									</button>
									<button class="rounded-lg bg-violet-500/20 border border-violet-500/30 px-3 py-1.5 text-xs font-semibold text-violet-300 hover:bg-violet-500/30 transition">
										Add as Task
									</button>
								</div>
							</div>
						{/if}
					</div>
				{/if}

				{#if statusMsg}
					<div class="mt-4 rounded-xl border border-cyan-400/20 bg-cyan-500/8 p-3 text-xs text-cyan-300">{statusMsg}</div>
				{/if}
			</div>

			<!-- Voice guide -->
			<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
				<h2 class="mb-4 text-base font-semibold text-white">What to Say</h2>
				<div class="space-y-3">
					{#each [
						{ example: '"Remember to..."', action: 'Saved to Memory Spine', icon: '🧠' },
						{ example: '"Add task to do..."', action: 'Added to Kaizen board', icon: '✅' },
						{ example: '"Schedule tomorrow..."', action: 'Added to Today board', icon: '☀️' },
						{ example: '"Note that..."', action: 'Saved as general note', icon: '📝' },
						{ example: '"Idea about..."', action: 'Saved to Learning domain', icon: '💡' },
					] as item}
						<div class="flex items-center gap-3 rounded-xl border border-white/8 bg-white/3 p-3">
							<span class="text-lg">{item.icon}</span>
							<div>
								<p class="text-xs text-slate-300 italic font-medium">{item.example}</p>
								<p class="text-[10px] text-slate-500 mt-0.5">→ {item.action}</p>
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>

	{:else}
		<!-- Setup Guide -->
		<div class="grid gap-5 lg:grid-cols-2">
			<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
				<h2 class="mb-5 text-base font-semibold text-white">Set Up Panda</h2>
				<div class="space-y-4">
					{#each SETUP_STEPS as step}
						<div class="flex items-start gap-4">
							<div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-cyan-500/15 text-xl">{step.icon}</div>
							<div class="pt-1.5">
								<div class="flex items-center gap-2">
									<span class="text-xs font-bold text-cyan-300">Step {step.num}</span>
								</div>
								<p class="text-sm font-medium text-white mt-0.5">{step.title}</p>
								<p class="text-xs text-slate-500 mt-0.5 leading-5">{step.desc}</p>
							</div>
						</div>
					{/each}
				</div>
				<div class="mt-6">
					<a href="/mobile" class="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-cyan-500 to-violet-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg hover:opacity-90 transition">
						📱 Go to Mobile Sync →
					</a>
				</div>
			</div>

			<div class="rounded-2xl border border-white/10 bg-slate-950/50 p-6 backdrop-blur">
				<h2 class="mb-4 text-base font-semibold text-white">Requirements</h2>
				<div class="space-y-3 text-sm text-slate-300 leading-6">
					<div class="flex items-start gap-3">
						<span class="text-green-400">✓</span>
						<p>AmitOS desktop app running on the same Wi-Fi network as your phone</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-green-400">✓</span>
						<p>Mobile Sync enabled (go to Mobile tab, click Enable Sync)</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-green-400">✓</span>
						<p>Microphone permission granted for voice capture</p>
					</div>
					<div class="flex items-start gap-3">
						<span class="text-amber-400">○</span>
						<p>Optional: AmitOS Android APK for a native app experience (download from Releases)</p>
					</div>
				</div>

				<div class="mt-5 rounded-xl border border-violet-400/20 bg-violet-500/8 p-4">
					<p class="text-sm font-semibold text-violet-200 mb-1">📱 Native Android App</p>
					<p class="text-xs text-slate-400 leading-5">
						Download the AmitOS APK from the GitHub Releases page for a full native experience with push notifications and background sync.
					</p>
					<a href="https://github.com/amitos/amitos/releases" target="_blank" rel="noopener"
						class="mt-2 inline-block text-xs text-violet-300 hover:text-violet-200 underline">
						Download APK →
					</a>
				</div>
			</div>
		</div>
	{/if}
</section>
