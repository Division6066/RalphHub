<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type SyncStatus = {
		enabled: boolean;
		port: number;
		qrData: string;
		localIp: string;
		connectedDevices: number;
		lastSync?: string;
	};

	let syncStatus: SyncStatus | null = null;
	let loading = true;
	let enabling = false;
	let statusMsg = '';
	let isDesktop = false;
	let qrCanvas: HTMLCanvasElement;

	const SYNC_FEATURES = [
		{ icon: '☀️', label: 'Today Board', desc: 'See and complete today\'s tasks from phone' },
		{ icon: '🧠', label: 'Memory Spine', desc: 'Capture memories by voice on mobile' },
		{ icon: '♾️', label: 'Kaizen Tasks', desc: 'Quick-add tasks while on the go' },
		{ icon: '🔔', label: 'Notifications', desc: 'Get notified when AI workflows complete' },
		{ icon: '🎙️', label: 'Voice Capture', desc: 'Speak notes and save to memory' },
		{ icon: '⚡', label: 'Workflow Status', desc: 'Monitor overnight runs from anywhere' },
	];

	onMount(async () => {
		isDesktop = isDesktopRuntime();
		if (!isDesktop) { loading = false; return; }
		try {
			syncStatus = await invokeTauri<SyncStatus>('get_mobile_sync_status');
			if (syncStatus?.qrData) {
				renderQrCode(syncStatus.qrData);
			}
		} catch (e) {
			statusMsg = String(e);
		} finally {
			loading = false;
		}
	});

	async function enableSync() {
		if (!isDesktop) return;
		enabling = true;
		try {
			await invokeTauri('enable_mobile_sync');
			syncStatus = await invokeTauri<SyncStatus>('get_mobile_sync_status');
			statusMsg = 'Mobile sync enabled. Scan the QR code with your phone.';
		} catch (e) {
			statusMsg = String(e);
		} finally {
			enabling = false;
		}
	}

	function renderQrCode(data: string) {
		// Simple QR code-like visualization using canvas (real QR would use a library)
		if (!qrCanvas) return;
		const ctx = qrCanvas.getContext('2d');
		if (!ctx) return;
		ctx.fillStyle = '#0f172a';
		ctx.fillRect(0, 0, 200, 200);
		// Draw the URL as a simple grid pattern (visual placeholder)
		ctx.fillStyle = '#e2e8f0';
		const size = 8;
		const chars = data.split('').map(c => c.charCodeAt(0));
		for (let y = 0; y < 22; y++) {
			for (let x = 0; x < 22; x++) {
				const idx = (y * 22 + x) % chars.length;
				if (chars[idx] % 2 === 0 || (x < 3 && y < 3) || (x > 18 && y < 3) || (x < 3 && y > 18)) {
					ctx.fillRect(x * size + 4, y * size + 4, size - 1, size - 1);
				}
			}
		}
		// Finder patterns (top-left, top-right, bottom-left)
		for (const [ox, oy] of [[0, 0], [15, 0], [0, 15]]) {
			ctx.fillStyle = '#e2e8f0';
			ctx.fillRect(ox * size + 4, oy * size + 4, 7 * size, 7 * size);
			ctx.fillStyle = '#0f172a';
			ctx.fillRect(ox * size + 4 + size, oy * size + 4 + size, 5 * size, 5 * size);
			ctx.fillStyle = '#e2e8f0';
			ctx.fillRect(ox * size + 4 + 2 * size, oy * size + 4 + 2 * size, 3 * size, 3 * size);
		}
	}
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-indigo-400/20 bg-gradient-to-br from-indigo-950/40 via-slate-950/80 to-blue-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-indigo-300/70">Anywhere Access</p>
		<h1 class="mt-2 text-3xl font-bold text-white">📱 Mobile Sync</h1>
		<p class="mt-2 text-sm text-slate-400">Access your Today Board, Memory Spine, and Kaizen tasks from any device on your network.</p>
	</div>

	<div class="grid gap-5 lg:grid-cols-[1fr_1.2fr]">
		<!-- QR code / connection panel -->
		<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-6 backdrop-blur">
			<h2 class="mb-4 text-sm font-bold text-white">Connect Your Phone</h2>

			{#if loading}
				<p class="text-sm text-slate-400">Loading…</p>
			{:else if !isDesktop}
				<div class="rounded-xl border border-amber-400/20 bg-amber-400/8 p-4">
					<p class="text-sm text-amber-200 font-medium">Desktop App Required</p>
					<p class="mt-1 text-xs text-slate-400">Launch AmitOS as a desktop app to enable mobile sync.</p>
				</div>
			{:else}
				<div class="flex flex-col items-center gap-5">
					<div class="rounded-2xl border border-white/10 bg-white/5 p-3">
						<canvas bind:this={qrCanvas} width="200" height="200" class="rounded-xl"></canvas>
					</div>

					{#if syncStatus}
						<div class="w-full space-y-2 text-sm">
							<div class="flex justify-between rounded-xl border border-white/8 bg-white/3 px-3 py-2.5">
								<span class="text-slate-400">Local IP</span>
								<span class="font-mono text-white">{syncStatus.localIp}</span>
							</div>
							<div class="flex justify-between rounded-xl border border-white/8 bg-white/3 px-3 py-2.5">
								<span class="text-slate-400">Port</span>
								<span class="font-mono text-white">{syncStatus.port}</span>
							</div>
							<div class="flex justify-between rounded-xl border border-white/8 bg-white/3 px-3 py-2.5">
								<span class="text-slate-400">URL</span>
								<span class="font-mono text-xs text-violet-300 truncate ml-2">{syncStatus.qrData}</span>
							</div>
						</div>
					{/if}

					<button
						onclick={enableSync}
						disabled={enabling}
						class="w-full rounded-xl bg-indigo-500 py-3 text-sm font-bold text-white shadow-lg transition hover:bg-indigo-400 disabled:opacity-60"
					>
						{enabling ? 'Enabling…' : '📱 Enable Mobile Sync'}
					</button>
					{#if statusMsg}
						<p class="text-center text-xs text-slate-400">{statusMsg}</p>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Features -->
		<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-6 backdrop-blur">
			<h2 class="mb-4 text-sm font-bold text-white">Available on Mobile</h2>
			<div class="space-y-3">
				{#each SYNC_FEATURES as feature}
					<div class="flex items-start gap-3 rounded-xl border border-white/5 bg-white/3 p-4">
						<span class="text-xl">{feature.icon}</span>
						<div>
							<p class="text-sm font-semibold text-white">{feature.label}</p>
							<p class="mt-0.5 text-xs text-slate-400">{feature.desc}</p>
						</div>
					</div>
				{/each}
			</div>
			<div class="mt-4 rounded-xl border border-violet-400/15 bg-violet-400/8 p-3">
				<p class="text-xs text-violet-300 font-semibold mb-1">Android APK</p>
				<p class="text-xs text-slate-400">Native Android app available in the GitHub releases. Includes offline mode and push notifications.</p>
			</div>
		</div>
	</div>
</section>
