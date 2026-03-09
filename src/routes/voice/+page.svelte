<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';

	type RecognitionResult = {
		transcript: string;
		confidence: number;
	};

	let isListening = false;
	let transcript = '';
	let interimTranscript = '';
	let lastCommand = '';
	let commandResult = '';
	let commandHistory: Array<{ command: string; result: string; time: string }> = [];
	let recognition: any = null;
	let supported = false;
	let statusMsg = '';

	const COMMANDS: Array<{ pattern: RegExp; action: string; desc: string }> = [
		{ pattern: /open today|today board|what's today/i, action: 'navigate:/today', desc: 'Open Today Board' },
		{ pattern: /kaizen|tasks|task board/i, action: 'navigate:/kaizen', desc: 'Open Kaizen Board' },
		{ pattern: /memory|remember|capture/i, action: 'navigate:/memory', desc: 'Open Memory Spine' },
		{ pattern: /tools|deploy tool/i, action: 'navigate:/tools', desc: 'Open Tools' },
		{ pattern: /workflow|workflows/i, action: 'navigate:/workflows', desc: 'Open Workflows' },
		{ pattern: /settings|api keys|keys/i, action: 'navigate:/settings', desc: 'Open API Keys' },
		{ pattern: /deploy|deployment/i, action: 'navigate:/deploy', desc: 'Open Deploy' },
		{ pattern: /home|dashboard/i, action: 'navigate:/', desc: 'Go to Dashboard' },
		{ pattern: /add task (.+)/i, action: 'add-task', desc: 'Add a task' },
		{ pattern: /remember (.+)/i, action: 'add-memory', desc: 'Add a memory' },
	];

	const HOTKEYS = [
		{ key: 'Space', label: 'Start / Stop listening' },
		{ key: 'Escape', label: 'Cancel' },
	];

	onMount(() => {
		supported = 'SpeechRecognition' in window || 'webkitSpeechRecognition' in window;
		if (supported) {
			initRecognition();
		}
	});

	onDestroy(() => {
		stopListening();
	});

	function initRecognition() {
		const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
		recognition = new SpeechRecognition();
		recognition.continuous = true;
		recognition.interimResults = true;
		recognition.lang = 'en-US';

		recognition.onresult = (event: any) => {
			let interim = '';
			let final = '';
			for (let i = event.resultIndex; i < event.results.length; i++) {
				const result = event.results[i];
				if (result.isFinal) {
					final += result[0].transcript;
				} else {
					interim += result[0].transcript;
				}
			}
			if (final) {
				transcript = final.trim();
				interimTranscript = '';
				processCommand(transcript);
			} else {
				interimTranscript = interim;
			}
		};

		recognition.onerror = (event: any) => {
			statusMsg = `Voice error: ${event.error}`;
			isListening = false;
		};

		recognition.onend = () => {
			if (isListening) {
				try { recognition.start(); } catch {}
			}
		};
	}

	function toggleListening() {
		if (isListening) {
			stopListening();
		} else {
			startListening();
		}
	}

	function startListening() {
		if (!recognition) return;
		try {
			recognition.start();
			isListening = true;
			statusMsg = '';
			transcript = '';
			interimTranscript = '';
		} catch (e) {
			statusMsg = 'Could not start voice recognition.';
		}
	}

	function stopListening() {
		if (recognition) {
			try { recognition.stop(); } catch {}
		}
		isListening = false;
	}

	async function processCommand(text: string) {
		lastCommand = text;
		let result = `Heard: "${text}" — no matching command.`;

		for (const cmd of COMMANDS) {
			const match = text.match(cmd.pattern);
			if (match) {
				if (cmd.action.startsWith('navigate:')) {
					const path = cmd.action.replace('navigate:', '');
					result = `Navigating to ${path}…`;
					setTimeout(() => goto(path), 500);
				} else if (cmd.action === 'add-task') {
					const taskTitle = match[1];
					result = `Creating task: "${taskTitle}"`;
					// In a real integration: call create_kaizen_task
				} else if (cmd.action === 'add-memory') {
					const memContent = match[1];
					result = `Saving memory: "${memContent}"`;
					// In a real integration: call create_memory_entry
				}
				break;
			}
		}

		commandResult = result;
		commandHistory = [
			{ command: text, result, time: new Date().toLocaleTimeString() },
			...commandHistory.slice(0, 9)
		];
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.code === 'Space' && e.target === document.body) {
			e.preventDefault();
			toggleListening();
		}
		if (e.code === 'Escape') {
			stopListening();
		}
	}
</script>

<svelte:document onkeydown={handleKeyDown} />

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-indigo-400/20 bg-gradient-to-br from-indigo-950/50 via-slate-950/80 to-blue-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-indigo-300/70">Hands-Free Control</p>
		<h1 class="mt-2 text-3xl font-bold text-white">🎙️ Voice Mode</h1>
		<p class="mt-2 text-sm text-slate-400">Say commands to navigate AmitOS, add tasks, and capture memories.</p>
	</div>

	<!-- Main voice button -->
	<div class="flex flex-col items-center gap-5 rounded-2xl border border-white/8 bg-slate-950/50 p-10 backdrop-blur">
		{#if !supported}
			<div class="text-center">
				<p class="text-4xl mb-3">🚫</p>
				<p class="text-base font-semibold text-white mb-2">Voice Recognition Not Supported</p>
				<p class="text-sm text-slate-400">Please use Chrome, Edge, or Safari for voice commands.</p>
			</div>
		{:else}
			<!-- Microphone button -->
			<button
				onclick={toggleListening}
				class={`relative flex h-28 w-28 items-center justify-center rounded-full text-4xl shadow-2xl transition-all duration-300 ${
					isListening
						? 'bg-rose-500/20 border-2 border-rose-400 text-rose-100 scale-110'
						: 'bg-indigo-500/20 border-2 border-indigo-400 text-indigo-100 hover:scale-105'
				}`}
			>
				{isListening ? '🎙️' : '🎤'}
				{#if isListening}
					<span class="absolute inset-0 animate-ping rounded-full border-2 border-rose-400 opacity-30"></span>
				{/if}
			</button>

			<div class="text-center">
				<p class="text-lg font-bold text-white">{isListening ? 'Listening…' : 'Click to speak'}</p>
				<p class="text-sm text-slate-400">or press Space</p>
			</div>

			<!-- Live transcript -->
			{#if isListening || interimTranscript}
				<div class="w-full max-w-md rounded-xl border border-indigo-400/20 bg-indigo-950/30 p-4 text-center">
					<p class="text-sm text-slate-400">
						{#if interimTranscript}
							<span class="text-slate-300 italic">"{interimTranscript}"</span>
						{:else}
							<span class="text-slate-500">Waiting for speech…</span>
						{/if}
					</p>
				</div>
			{/if}

			{#if commandResult}
				<div class="w-full max-w-md rounded-xl border border-emerald-400/20 bg-emerald-950/20 p-4 text-center">
					<p class="text-sm text-emerald-300">{commandResult}</p>
				</div>
			{/if}
		{/if}
	</div>

	{#if supported}
		<div class="grid gap-4 sm:grid-cols-2">
			<!-- Available commands -->
			<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
				<h2 class="mb-4 text-sm font-bold text-white">Available Commands</h2>
				<div class="space-y-2">
					{#each COMMANDS as cmd}
						<div class="flex items-center gap-3 rounded-xl border border-white/5 bg-white/3 px-3 py-2.5">
							<span class="text-xs text-slate-400 font-mono flex-1">"{cmd.desc.toLowerCase()}"</span>
							<span class="text-xs text-violet-400">{cmd.action.replace('navigate:', '→ ')}</span>
						</div>
					{/each}
				</div>
			</div>

			<!-- Command history -->
			<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
				<h2 class="mb-4 text-sm font-bold text-white">Command History</h2>
				{#if commandHistory.length === 0}
					<p class="text-sm text-slate-500">No commands yet. Say something!</p>
				{:else}
					<div class="space-y-2">
						{#each commandHistory as item}
							<div class="rounded-xl border border-white/5 bg-white/3 p-3">
								<p class="text-xs font-medium text-white">"{item.command}"</p>
								<p class="mt-1 text-xs text-slate-500">{item.result}</p>
								<p class="mt-0.5 text-[10px] text-slate-600">{item.time}</p>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>

		<!-- Keyboard shortcuts -->
		<div class="rounded-xl border border-white/8 bg-slate-950/50 p-4 backdrop-blur">
			<p class="mb-3 text-xs font-bold uppercase tracking-widest text-slate-500">Keyboard Shortcuts</p>
			<div class="flex flex-wrap gap-3">
				{#each HOTKEYS as hk}
					<div class="flex items-center gap-2">
						<kbd class="rounded-lg border border-white/15 bg-white/8 px-2.5 py-1 text-xs font-mono text-white">{hk.key}</kbd>
						<span class="text-xs text-slate-400">{hk.label}</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if statusMsg}
		<div class="rounded-xl border border-rose-400/20 bg-rose-950/20 p-3 text-xs text-rose-300">{statusMsg}</div>
	{/if}
</section>
