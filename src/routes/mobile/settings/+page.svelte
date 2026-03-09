<script lang="ts">
	import { mobileState, setDesktopEndpoint, checkDesktopConnection } from '$lib/mobile/store.svelte.js';

	let endpoint = $state(mobileState.desktopEndpoint || '');
	let testing = $state(false);
	let testResult = $state<'success' | 'failed' | null>(null);
	let saved = $state(false);

	async function handleSave() {
		if (!endpoint.trim()) return;
		await setDesktopEndpoint(endpoint.trim());
		saved = true;
		setTimeout(() => saved = false, 2000);
	}

	async function handleTest() {
		if (!endpoint.trim()) return;
		testing = true;
		testResult = null;
		// Temporarily set endpoint for test
		const prev = mobileState.desktopEndpoint;
		mobileState.desktopEndpoint = endpoint.trim();
		const ok = await checkDesktopConnection();
		testResult = ok ? 'success' : 'failed';
		if (!ok) mobileState.desktopEndpoint = prev;
		testing = false;
	}

	const infoCells = [
		{ label: 'Device ID', value: () => 'Loading…', mono: true },
		{ label: 'Sync Protocol', value: () => 'AES-256-GCM + Vector Clocks', mono: false },
		{ label: 'Offline Queue', value: () => `${mobileState.pendingSyncCount} pending`, mono: true },
		{ label: 'Desktop', value: () => mobileState.desktopConnected ? 'Connected' : 'Disconnected', mono: false },
	];
</script>

<div class="space-y-5 py-2">
	<div>
		<h1 class="text-xl font-bold text-white">Mobile Settings</h1>
		<p class="text-xs text-slate-500 mt-0.5">Connect to your RalphHub desktop instance</p>
	</div>

	<!-- Desktop endpoint -->
	<div class="rounded-2xl border border-white/10 bg-slate-900/50 p-5 space-y-4">
		<div>
			<h2 class="text-sm font-semibold text-white">Desktop Sync Endpoint</h2>
			<p class="text-xs text-slate-500 mt-1">Scan the QR code in Settings → Mobile ↗ on desktop, or enter manually.</p>
		</div>

		<div class="space-y-2">
			<label class="text-xs text-slate-400">Endpoint URL</label>
			<input
				bind:value={endpoint}
				placeholder="http://192.168.1.x:7842"
				class="w-full rounded-xl border border-white/10 bg-slate-950/60 px-4 py-3 text-sm text-white font-mono placeholder:text-slate-600 focus:outline-none focus:border-cyan-400/50"
				onkeydown={(e) => e.key === 'Enter' && handleSave()}
			/>
		</div>

		{#if testResult === 'success'}
			<div class="flex items-center gap-2 text-xs text-emerald-400">
				<span>✓</span> Connected to desktop successfully
			</div>
		{:else if testResult === 'failed'}
			<div class="flex items-center gap-2 text-xs text-red-400">
				<span>✗</span> Cannot reach desktop — check IP and port
			</div>
		{/if}

		<div class="flex gap-2">
			<button
				onclick={handleTest}
				disabled={!endpoint.trim() || testing}
				class="flex-1 rounded-xl border border-white/10 bg-slate-800/50 text-slate-300 text-xs font-medium py-2.5 hover:bg-slate-800 disabled:opacity-50 transition-colors"
			>
				{testing ? 'Testing…' : 'Test Connection'}
			</button>
			<button
				onclick={handleSave}
				disabled={!endpoint.trim()}
				class={`flex-1 rounded-xl text-sm font-medium py-2.5 transition-all disabled:opacity-50 ${
					saved
						? 'bg-emerald-500/20 border border-emerald-500/30 text-emerald-300'
						: 'bg-cyan-500/20 border border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/30'
				}`}
			>
				{saved ? '✓ Saved' : 'Save Endpoint'}
			</button>
		</div>
	</div>

	<!-- Sync status -->
	<div class="rounded-2xl border border-white/8 bg-slate-900/30 p-5 space-y-3">
		<h2 class="text-sm font-semibold text-white">Sync Status</h2>

		<div class="space-y-2">
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Network</span>
				<span class={mobileState.online ? 'text-emerald-400' : 'text-red-400'}>
					{mobileState.online ? 'Online' : 'Offline'}
				</span>
			</div>
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Desktop</span>
				<span class={mobileState.desktopConnected ? 'text-emerald-400' : 'text-slate-600'}>
					{mobileState.desktopConnected ? 'Connected' : 'Disconnected'}
				</span>
			</div>
			<div class="flex items-center justify-between text-sm">
				<span class="text-slate-400">Pending Events</span>
				<span class={mobileState.pendingSyncCount > 0 ? 'text-amber-400 font-mono' : 'text-slate-600 font-mono'}>
					{mobileState.pendingSyncCount}
				</span>
			</div>
		</div>
	</div>

	<!-- About -->
	<div class="rounded-2xl border border-white/5 bg-slate-900/20 p-5 space-y-2 text-xs text-slate-600">
		<p class="text-slate-400 font-medium text-sm mb-3">About RalphHub Mobile</p>
		<p>Version 1.0.0 · Companion only</p>
		<p>Encryption: AES-256-GCM (Web Crypto API)</p>
		<p>Sync: Vector clocks + offline queue</p>
		<p>STT: Web Speech API (on-device)</p>
		<p>Requires: RalphHub desktop on same LAN</p>
		<p class="pt-2 text-slate-700">com.ralphhub.mobile</p>
	</div>
</div>
