<script lang="ts">
	import { mobileState, captureVoice } from '$lib/mobile/store.svelte.js';
	import { onDestroy } from 'svelte';

	let recognition: SpeechRecognition | null = null;
	let transcript = $state('');
	let interim = $state('');
	let saved = $state(false);
	let error = $state('');
	let waveformActive = $state(false);
	let history = $state<Array<{ text: string; at: string }>>([]);

	function startRecording() {
		const SR = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
		if (!SR) {
			error = 'Speech recognition not supported on this browser/device.';
			return;
		}

		error = '';
		recognition = new SR();
		recognition.continuous = true;
		recognition.interimResults = true;
		recognition.lang = 'en-US';

		recognition.onresult = (e: SpeechRecognitionEvent) => {
			let finalText = '';
			let interimText = '';
			for (let i = e.resultIndex; i < e.results.length; i++) {
				if (e.results[i].isFinal) finalText += e.results[i][0].transcript;
				else interimText += e.results[i][0].transcript;
			}
			transcript += finalText;
			interim = interimText;
		};
		recognition.onerror = (e: SpeechRecognitionErrorEvent) => {
			error = `Recognition error: ${e.error}`;
			mobileState.voiceRecording = false;
			waveformActive = false;
		};
		recognition.onend = () => {
			mobileState.voiceRecording = false;
			waveformActive = false;
		};

		recognition.start();
		mobileState.voiceRecording = true;
		waveformActive = true;
	}

	function stopRecording() {
		recognition?.stop();
		mobileState.voiceRecording = false;
		waveformActive = false;
	}

	async function saveCapture() {
		if (!transcript.trim()) return;
		await captureVoice(transcript.trim());
		history = [{ text: transcript.trim(), at: new Date().toISOString() }, ...history];
		transcript = '';
		saved = true;
		setTimeout(() => saved = false, 2500);
	}

	function clearTranscript() {
		transcript = '';
		interim = '';
	}

	onDestroy(() => {
		recognition?.stop();
	});
</script>

<div class="space-y-6 py-2">
	<div>
		<h1 class="text-xl font-bold text-white">Voice Capture</h1>
		<p class="text-xs text-slate-500 mt-0.5">Local STT — transcribed on-device, synced to memory spine</p>
	</div>

	<!-- Big record button -->
	<div class="flex flex-col items-center py-6">
		<div class="relative">
			{#if mobileState.voiceRecording}
				<div class="absolute inset-0 rounded-full bg-red-500/20 animate-ping scale-150"></div>
				<div class="absolute inset-0 rounded-full bg-red-500/10 animate-ping scale-125" style="animation-delay: 0.2s"></div>
			{/if}
			<button
				onclick={mobileState.voiceRecording ? stopRecording : startRecording}
				class={`relative w-28 h-28 rounded-full border-4 flex flex-col items-center justify-center gap-1.5 transition-all active:scale-95 ${
					mobileState.voiceRecording
						? 'border-red-400 bg-red-500/20 text-red-300'
						: 'border-slate-600 bg-slate-900/60 text-slate-400 hover:border-slate-400 hover:text-slate-300'
				}`}
			>
				<span class="text-3xl">{mobileState.voiceRecording ? '⏹' : '⏺'}</span>
				<span class="text-xs font-medium">{mobileState.voiceRecording ? 'Stop' : 'Record'}</span>
			</button>
		</div>

		{#if mobileState.voiceRecording}
			<div class="flex items-center gap-1 mt-5">
				{#each Array(7) as _, i}
					<div
						class="w-1 rounded-full bg-red-400 animate-bounce"
						style="height: {Math.random() * 20 + 8}px; animation-delay: {i * 0.1}s"
					></div>
				{/each}
			</div>
			<p class="text-sm text-red-300 mt-3 animate-pulse">Listening…</p>
		{:else}
			<p class="text-xs text-slate-600 mt-4">Press to start recording</p>
		{/if}
	</div>

	{#if error}
		<div class="rounded-xl bg-red-500/10 border border-red-500/20 px-4 py-3">
			<p class="text-xs text-red-400">{error}</p>
		</div>
	{/if}

	<!-- Transcript area -->
	{#if transcript || interim}
		<div class="rounded-2xl border border-white/10 bg-slate-900/60 p-4">
			<div class="flex items-center justify-between mb-3">
				<p class="text-xs text-slate-500 uppercase tracking-wider">Transcript</p>
				<button onclick={clearTranscript} class="text-xs text-slate-600 hover:text-slate-400">Clear</button>
			</div>
			<p class="text-sm text-white leading-relaxed">
				{transcript}<span class="text-slate-500">{interim}</span>
			</p>
		</div>

		{#if transcript.trim() && !mobileState.voiceRecording}
			<button
				onclick={saveCapture}
				class={`w-full rounded-2xl py-4 font-medium text-sm transition-all ${
					saved
						? 'bg-emerald-500/20 border border-emerald-500/30 text-emerald-300'
						: 'bg-cyan-500/20 border border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/30 active:scale-98'
				}`}
			>
				{saved ? '✓ Captured & Synced to Memory' : '▲ Save to Memory Spine'}
			</button>
		{/if}
	{/if}

	<!-- Voice history -->
	{#if history.length > 0}
		<section>
			<h2 class="text-xs uppercase tracking-widest text-slate-500 mb-3">This Session</h2>
			<div class="space-y-2">
				{#each history as item}
					<div class="rounded-xl border border-white/5 bg-slate-900/30 px-4 py-3">
						<p class="text-sm text-slate-300 leading-relaxed">{item.text}</p>
						<p class="text-xs text-slate-600 mt-1">{new Date(item.at).toLocaleTimeString()}</p>
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<!-- Info box -->
	<div class="rounded-2xl border border-white/5 bg-slate-900/20 p-4 text-xs text-slate-600 space-y-1.5">
		<p>⟡ Uses Web Speech API (on-device STT)</p>
		<p>⟡ Works offline — syncs when desktop reconnects</p>
		<p>⟡ Transcripts written to Memory Spine as raw events</p>
		<p>⟡ Auto-processed by Ralph on desktop</p>
	</div>
</div>
