<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type VoiceConfig = {
		enabled: boolean;
		sttProvider: string;
		ttsProvider: string;
		sttModel: string;
		ttsVoice: string;
		whisperInstalled: boolean;
		piperInstalled: boolean;
		offlineFallback: boolean;
	};

	let config: VoiceConfig | null = null;
	let voices: string[] = [];
	let loading = true;
	let busy = false;
	let message = '';
	let globalVoiceEnabled = false;

	const sttModels = ['tiny', 'base', 'small', 'medium', 'large-v3'];

	onMount(async () => {
		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}
		await refresh();
		try {
			voices = await invokeTauri<string[]>('list_piper_voices');
		} catch {}
	});

	async function refresh() {
		loading = true;
		try {
			config = await invokeTauri<VoiceConfig>('check_voice_system');
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to check voice system.';
		} finally {
			loading = false;
		}
	}

	async function installVoice() {
		if (!config) return;
		busy = true;
		message = 'Installing voice system (faster-whisper + piper)...';
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('ensure_voice', {
				sttProvider: config.sttProvider,
				ttsProvider: 'piper'
			});
			message = result.message;
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Installation failed.';
		} finally {
			busy = false;
		}
	}

	function statusBadge(ok: boolean) {
		return ok
			? 'bg-green-400/15 text-green-300'
			: 'bg-slate-600/30 text-slate-400';
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Voice Mode</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Local STT + TTS — fully offline.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			faster-whisper for speech-to-text, Piper for text-to-speech. Every workflow and tool can accept
			voice input and speak responses. No cloud required.
		</p>
	</div>

	{#if message}
		<div class="rounded-3xl border border-cyan-400/20 bg-cyan-500/10 p-4 text-sm text-cyan-100">
			{message}
		</div>
	{/if}

	<!-- Global toggle -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-center justify-between">
			<div>
				<h2 class="text-lg font-semibold text-white">Global voice toggle</h2>
				<p class="mt-1 text-sm text-slate-400">When enabled, all workflows use local voice by default.</p>
			</div>
			<button
				on:click={() => (globalVoiceEnabled = !globalVoiceEnabled)}
				aria-label="Toggle global voice mode"
				class="relative inline-flex h-8 w-14 items-center rounded-full transition {globalVoiceEnabled ? 'bg-cyan-500' : 'bg-slate-700'}"
			>
				<span
					class="inline-block h-6 w-6 transform rounded-full bg-white shadow transition {globalVoiceEnabled ? 'translate-x-7' : 'translate-x-1'}"
				></span>
			</button>
		</div>
		{#if globalVoiceEnabled}
			<p class="mt-3 text-xs text-cyan-300">
				Voice mode active — all tools will accept mic input and speak responses via local Piper TTS.
			</p>
		{/if}
	</div>

	{#if loading}
		<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-6 text-sm text-slate-400 backdrop-blur">
			Checking voice system...
		</div>
	{:else if config}
		<div class="grid gap-4 xl:grid-cols-2">
			<!-- STT -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-center justify-between">
					<h2 class="text-lg font-semibold text-white">Speech-to-Text</h2>
					<span class="rounded-full px-3 py-1 text-xs font-medium {statusBadge(config.whisperInstalled)}">
						{config.whisperInstalled ? 'Ready' : 'Not installed'}
					</span>
				</div>
				<p class="mt-2 text-sm text-slate-400">faster-whisper — quantized Whisper models, runs on CPU.</p>

				<div class="mt-4 space-y-3">
					<div>
						<p class="text-xs text-slate-500">STT Provider</p>
						<p class="mt-1 text-sm text-white font-mono">{config.sttProvider}</p>
					</div>
					<div>
						<p class="text-xs text-slate-500">Model size</p>
						<p class="mt-1 text-sm text-white">{config.sttModel}</p>
						<p class="text-xs text-slate-500 mt-1">base ≈ 145MB | small ≈ 466MB | medium ≈ 1.5GB</p>
					</div>
				</div>

				{#if !config.whisperInstalled}
					<button
						on:click={installVoice}
						disabled={busy}
						class="mt-4 rounded-full bg-cyan-400/12 px-4 py-2 text-sm font-medium text-cyan-100 hover:bg-cyan-400/20 disabled:opacity-60"
					>
						{busy ? 'Installing...' : 'Install faster-whisper'}
					</button>
				{/if}
			</div>

			<!-- TTS -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-center justify-between">
					<h2 class="text-lg font-semibold text-white">Text-to-Speech</h2>
					<span class="rounded-full px-3 py-1 text-xs font-medium {statusBadge(config.piperInstalled)}">
						{config.piperInstalled ? 'Ready' : 'Not installed'}
					</span>
				</div>
				<p class="mt-2 text-sm text-slate-400">Piper TTS — fast, high-quality neural voices, fully offline.</p>

				<div class="mt-4 space-y-3">
					<div>
						<p class="text-xs text-slate-500">TTS Provider</p>
						<p class="mt-1 text-sm text-white font-mono">{config.ttsProvider}</p>
					</div>
					<div>
						<p class="text-xs text-slate-500">Selected voice</p>
						<p class="mt-1 text-sm text-white">{config.ttsVoice}</p>
					</div>
					{#if voices.length > 0}
						<div>
							<p class="text-xs text-slate-500">Available voices</p>
							<div class="mt-2 flex flex-wrap gap-2">
								{#each voices as voice}
									<span class="rounded-full border border-white/10 px-2 py-0.5 text-xs text-slate-300">{voice}</span>
								{/each}
							</div>
						</div>
					{/if}
				</div>

				{#if !config.piperInstalled}
					<button
						on:click={installVoice}
						disabled={busy}
						class="mt-4 rounded-full bg-violet-500/15 px-4 py-2 text-sm font-medium text-violet-200 hover:bg-violet-500/25 disabled:opacity-60"
					>
						{busy ? 'Installing...' : 'Install Piper TTS'}
					</button>
				{/if}
			</div>
		</div>

		<!-- Offline fallback indicator -->
		<div class="rounded-3xl border {config.offlineFallback ? 'border-green-400/20 bg-green-500/8' : 'border-white/10 bg-slate-950/45'} p-6 backdrop-blur">
			<div class="flex items-center gap-3">
				<div class="h-3 w-3 rounded-full {config.offlineFallback ? 'bg-green-400' : 'bg-slate-600'}"></div>
				<div>
					<p class="text-sm font-medium text-white">
						{config.offlineFallback ? 'Offline voice fallback active' : 'Offline fallback not ready'}
					</p>
					<p class="mt-1 text-xs text-slate-400">
						{config.offlineFallback
							? 'Both STT and TTS are installed. All voice flows work without internet.'
							: 'Install both faster-whisper and Piper to enable fully offline voice mode.'}
					</p>
				</div>
			</div>
		</div>

		<!-- Usage guide -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">How voice mode works</h2>
			<div class="mt-4 space-y-3 text-sm text-slate-400">
				<div class="flex gap-3">
					<span class="shrink-0 text-cyan-400">1.</span>
					<p>Enable the global toggle above. Any workflow that supports voice will show a mic button.</p>
				</div>
				<div class="flex gap-3">
					<span class="shrink-0 text-cyan-400">2.</span>
					<p>Click the mic — faster-whisper transcribes your speech and sends it as the prompt.</p>
				</div>
				<div class="flex gap-3">
					<span class="shrink-0 text-cyan-400">3.</span>
					<p>The response is spoken back by Piper TTS using your selected voice.</p>
				</div>
				<div class="flex gap-3">
					<span class="shrink-0 text-cyan-400">4.</span>
					<p>All audio processing happens locally — no data leaves your machine.</p>
				</div>
			</div>
		</div>
	{/if}
</section>
