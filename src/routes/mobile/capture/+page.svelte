<script lang="ts">
	import { mobileState, quickCapture, addTask, captureVoice } from '$lib/mobile/store.svelte.js';
	import { onDestroy } from 'svelte';

	type CaptureMode = 'text' | 'task' | 'voice';

	let mode = $state<CaptureMode>('text');
	let textContent = $state('');
	let taskTitle = $state('');
	let taskPriority = $state<'urgent' | 'high' | 'normal' | 'low'>('normal');
	let submitted = $state(false);
	let submitting = $state(false);

	// Voice
	let recognition: SpeechRecognition | null = null;
	let finalTranscript = $state('');
	let interimTranscript = $state('');

	const modeLabels: Record<CaptureMode, string> = {
		text: 'Note',
		task: 'Task',
		voice: 'Voice',
	};

	async function handleSubmit() {
		if (submitting) return;
		submitting = true;
		try {
			if (mode === 'text' && textContent.trim()) {
				await quickCapture(textContent.trim());
				textContent = '';
			} else if (mode === 'task' && taskTitle.trim()) {
				await addTask(taskTitle.trim(), taskPriority);
				taskTitle = '';
			} else if (mode === 'voice' && finalTranscript.trim()) {
				await captureVoice(finalTranscript.trim());
				finalTranscript = '';
			} else {
				submitting = false;
				return;
			}
			submitted = true;
			setTimeout(() => { submitted = false; }, 2000);
		} finally {
			submitting = false;
		}
	}

	function startVoice() {
		const SR = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
		if (!SR) {
			mobileState.voiceError = 'Speech recognition not supported on this device.';
			return;
		}
		recognition = new SR();
		recognition.continuous = true;
		recognition.interimResults = true;
		recognition.lang = 'en-US';

		recognition.onresult = (e: SpeechRecognitionEvent) => {
			let interim = '';
			let final = '';
			for (let i = e.resultIndex; i < e.results.length; i++) {
				if (e.results[i].isFinal) {
					final += e.results[i][0].transcript;
				} else {
					interim += e.results[i][0].transcript;
				}
			}
			finalTranscript += final;
			interimTranscript = interim;
		};
		recognition.onerror = (e: SpeechRecognitionErrorEvent) => {
			mobileState.voiceError = e.error;
			mobileState.voiceRecording = false;
		};
		recognition.onend = () => {
			mobileState.voiceRecording = false;
		};

		recognition.start();
		mobileState.voiceRecording = true;
		mobileState.voiceError = null;
	}

	function stopVoice() {
		recognition?.stop();
		mobileState.voiceRecording = false;
	}

	onDestroy(() => {
		recognition?.stop();
	});
</script>

<div class="space-y-5 py-2">
	<div>
		<h1 class="text-xl font-bold text-white">Quick Capture</h1>
		<p class="text-xs text-slate-500 mt-0.5">Everything goes to memory & syncs to desktop</p>
	</div>

	<!-- Mode tabs -->
	<div class="grid grid-cols-3 gap-1 bg-slate-900/60 rounded-2xl p-1 border border-white/8">
		{#each (['text', 'task', 'voice'] as CaptureMode[]) as m}
			<button
				onclick={() => { mode = m; mobileState.voiceError = null; }}
				class={`py-2 rounded-xl text-sm font-medium transition-all ${
					mode === m ? 'bg-cyan-400/15 text-cyan-300' : 'text-slate-500 hover:text-slate-300'
				}`}
			>{modeLabels[m]}</button>
		{/each}
	</div>

	<!-- Text capture -->
	{#if mode === 'text'}
		<div class="space-y-3">
			<textarea
				bind:value={textContent}
				placeholder="Capture a thought, idea, or note..."
				rows="6"
				class="w-full rounded-2xl border border-white/10 bg-slate-900/60 px-4 py-3 text-sm text-white placeholder:text-slate-600 focus:outline-none focus:border-cyan-400/50 resize-none"
			></textarea>
			<button
				onclick={handleSubmit}
				disabled={!textContent.trim() || submitting}
				class="w-full rounded-2xl py-3.5 font-medium text-sm transition-all
					{submitted ? 'bg-emerald-500/20 border-emerald-500/30 text-emerald-300 border' :
					 textContent.trim() ? 'bg-cyan-500/20 border border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/30' :
					 'bg-slate-800/50 border border-white/5 text-slate-600 cursor-not-allowed'}"
			>
				{submitted ? '✓ Captured!' : submitting ? 'Sending...' : 'Capture Note →'}
			</button>
		</div>

	<!-- Task capture -->
	{:else if mode === 'task'}
		<div class="space-y-3">
			<input
				bind:value={taskTitle}
				placeholder="What needs to be done?"
				class="w-full rounded-2xl border border-white/10 bg-slate-900/60 px-4 py-3.5 text-sm text-white placeholder:text-slate-600 focus:outline-none focus:border-cyan-400/50"
				onkeydown={(e) => e.key === 'Enter' && handleSubmit()}
			/>
			<div>
				<p class="text-xs text-slate-500 mb-2">Priority</p>
				<div class="grid grid-cols-4 gap-2">
					{#each [['urgent','red'], ['high','orange'], ['normal','cyan'], ['low','slate']] as [p, color]}
						<button
							onclick={() => taskPriority = p as typeof taskPriority}
							class={`py-2 rounded-xl border text-xs font-medium transition-all ${
								taskPriority === p
									? `bg-${color}-400/15 border-${color}-400/40 text-${color}-300`
									: 'border-white/8 text-slate-500'
							}`}
						>{p}</button>
					{/each}
				</div>
			</div>
			<button
				onclick={handleSubmit}
				disabled={!taskTitle.trim() || submitting}
				class="w-full rounded-2xl py-3.5 font-medium text-sm transition-all
					{submitted ? 'bg-emerald-500/20 border-emerald-500/30 text-emerald-300 border' :
					 taskTitle.trim() ? 'bg-cyan-500/20 border border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/30' :
					 'bg-slate-800/50 border border-white/5 text-slate-600 cursor-not-allowed'}"
			>
				{submitted ? '✓ Task Added!' : submitting ? 'Adding...' : 'Add Task →'}
			</button>
		</div>

	<!-- Voice capture -->
	{:else if mode === 'voice'}
		<div class="space-y-4">
			<div class="rounded-2xl border border-white/8 bg-slate-900/50 p-5 min-h-[120px] relative">
				{#if mobileState.voiceRecording}
					<div class="flex items-center gap-2 mb-2">
						<span class="w-2 h-2 rounded-full bg-red-400 animate-pulse"></span>
						<span class="text-xs text-red-300">Recording…</span>
					</div>
				{/if}
				<p class="text-sm text-white leading-relaxed">
					{finalTranscript}{#if interimTranscript}<span class="text-slate-500">{interimTranscript}</span>{/if}
				</p>
				{#if !finalTranscript && !mobileState.voiceRecording}
					<p class="text-slate-600 text-sm">Press record to start speaking…</p>
				{/if}
			</div>

			{#if mobileState.voiceError}
				<p class="text-xs text-red-400 text-center">{mobileState.voiceError}</p>
			{/if}

			<div class="flex gap-3">
				{#if !mobileState.voiceRecording}
					<button
						onclick={startVoice}
						class="flex-1 rounded-2xl bg-red-500/15 border border-red-500/30 text-red-300 py-4 flex flex-col items-center gap-1 hover:bg-red-500/25 transition-all active:scale-95"
					>
						<span class="text-2xl">⏺</span>
						<span class="text-xs">Record</span>
					</button>
				{:else}
					<button
						onclick={stopVoice}
						class="flex-1 rounded-2xl bg-red-500/30 border border-red-500/50 text-red-200 py-4 flex flex-col items-center gap-1 animate-pulse"
					>
						<span class="text-2xl">⏹</span>
						<span class="text-xs">Stop</span>
					</button>
				{/if}

				{#if finalTranscript}
					<button
						onclick={handleSubmit}
						disabled={submitting}
						class="flex-1 rounded-2xl py-4 font-medium text-sm transition-all
							{submitted ? 'bg-emerald-500/20 border-emerald-500/30 text-emerald-300 border' :
							 'bg-cyan-500/20 border border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/30'}"
					>
						<span class="text-2xl block">{submitted ? '✓' : '▲'}</span>
						<span class="text-xs">{submitted ? 'Saved!' : 'Save'}</span>
					</button>
				{/if}
			</div>

			<a
				href="/mobile/voice"
				class="block text-center text-xs text-cyan-400/60 hover:text-cyan-400 transition-colors"
			>Open full voice screen →</a>
		</div>
	{/if}

	<!-- Recent captures -->
	{#if mobileState.captures.length > 0}
		<section>
			<h2 class="text-xs uppercase tracking-widest text-slate-500 mb-3">Recent Captures</h2>
			<div class="space-y-2">
				{#each mobileState.captures.slice(0, 5) as cap}
					<div class="rounded-xl border border-white/5 bg-slate-900/30 px-4 py-3 flex items-start gap-3">
						<span class="text-slate-600 text-sm mt-0.5">{cap.type === 'voice' ? '⏺' : '≡'}</span>
						<p class="text-sm text-slate-400 flex-1 line-clamp-2">{cap.content}</p>
						{#if !cap.processed}
							<span class="text-xs text-amber-400/60 shrink-0">unprocessed</span>
						{/if}
					</div>
				{/each}
			</div>
		</section>
	{/if}
</div>
