<script lang="ts">
	import { onMount } from 'svelte';
	import QRCode from 'qrcode';
	import { getMobileServerInfo, type MobileServerInfo } from '$lib/utils/mobile-server.js';

	let serverInfo = $state<MobileServerInfo | null>(null);
	let qrDataUrl = $state<string | null>(null);
	let loading = $state(true);
	let error = $state('');

	// The production APK download URL (set after build)
	const APK_URL = '/downloads/ralphhub-mobile.apk';
	const APK_VERSION = '1.0.0';

	onMount(async () => {
		try {
			serverInfo = await getMobileServerInfo();

			if (serverInfo.qrUrl) {
				qrDataUrl = await QRCode.toDataURL(serverInfo.qrUrl, {
					width: 280,
					margin: 2,
					color: { dark: '#f1f5f9', light: '#0f172a' },
					errorCorrectionLevel: 'M',
				});
			}

			// Also generate QR for APK download
			const origin = window.location.origin;
			const apkFullUrl = `${origin}${APK_URL}`;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load server info';
		} finally {
			loading = false;
		}
	});

	async function generateQrForUrl(url: string): Promise<string> {
		return QRCode.toDataURL(url, {
			width: 260,
			margin: 2,
			color: { dark: '#f1f5f9', light: '#0f172a' },
			errorCorrectionLevel: 'M',
		});
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Mobile Companion</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Connect your phone.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Download the RalphHub Mobile APK or scan the QR code to connect your Android device
			directly to this desktop instance. No account required — local-first, AES-256 encrypted.
		</p>
	</div>

	<div class="grid gap-6 xl:grid-cols-2">
		<!-- APK Download -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white mb-1">Install APK</h2>
			<p class="text-sm text-slate-400 mb-6">
				Android 8.0+ required. Enable "Install from unknown sources" in Settings → Security.
			</p>

			<div class="space-y-4">
				<div class="rounded-2xl border border-cyan-400/15 bg-cyan-400/5 p-5">
					<div class="flex items-center gap-3 mb-3">
						<div class="w-10 h-10 rounded-xl bg-cyan-400/15 border border-cyan-400/25 flex items-center justify-center text-cyan-300 font-bold">
							A
						</div>
						<div>
							<p class="text-sm font-medium text-white">RalphHub Mobile</p>
							<p class="text-xs text-slate-500">v{APK_VERSION} · Android 8.0+</p>
						</div>
					</div>
					<a
						href={APK_URL}
						download="ralphhub-mobile.apk"
						class="block w-full text-center py-3.5 rounded-2xl bg-gradient-to-r from-cyan-500/20 to-violet-500/20 border border-cyan-400/30 text-cyan-100 font-medium text-sm hover:from-cyan-500/30 hover:to-violet-500/30 transition-all"
					>
						↓ Download APK ({APK_VERSION})
					</a>
				</div>

				<!-- Install steps -->
				<div class="space-y-2">
					{#each [
						['1', 'Download the APK above', 'cyan'],
						['2', 'Open Downloads on your phone', 'violet'],
						['3', 'Tap the file to install', 'emerald'],
						['4', 'Scan the sync QR code →', 'amber'],
					] as [num, step, color]}
						<div class="flex items-start gap-3 text-sm">
							<span class={`w-6 h-6 rounded-full bg-${color}-400/15 text-${color}-300 flex items-center justify-center text-xs font-bold shrink-0`}>{num}</span>
							<span class="text-slate-400 pt-0.5">{step}</span>
						</div>
					{/each}
				</div>
			</div>
		</div>

		<!-- Desktop sync QR -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white mb-1">Connect to Desktop</h2>
			<p class="text-sm text-slate-400 mb-6">
				Scan this QR code on your phone after installing to connect it to this RalphHub instance.
				Both devices must be on the same Wi-Fi network.
			</p>

			{#if loading}
				<div class="h-64 flex items-center justify-center text-slate-500">
					<p class="text-sm">Loading server info…</p>
				</div>
			{:else if qrDataUrl && serverInfo?.running}
				<div class="flex flex-col items-center gap-4">
					<div class="rounded-2xl border border-white/10 p-3 bg-slate-900">
						<img src={qrDataUrl} alt="Desktop sync QR code" class="w-56 h-56" />
					</div>
					<div class="text-center">
						<p class="text-xs text-slate-500 mb-1">Sync endpoint</p>
						<code class="text-sm text-cyan-300 bg-slate-900 rounded-lg px-3 py-1.5 border border-white/5 block">
							{serverInfo?.qrUrl}
						</code>
					</div>
					<div class="flex items-center gap-2 text-xs">
						<span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
						<span class="text-emerald-400">Server running on port {serverInfo?.port}</span>
					</div>
				</div>
			{:else}
				<div class="h-48 flex flex-col items-center justify-center gap-3 text-slate-500">
					<p class="text-4xl">◈</p>
					<p class="text-sm text-center">
						{#if !serverInfo?.running}
							Mobile sync server is starting up… <br />
							<span class="text-xs">Desktop app required for local sync</span>
						{:else}
							Connected to {serverInfo?.localIp}:{serverInfo?.port}
						{/if}
					</p>
					{#if serverInfo?.qrUrl}
						<code class="text-xs text-slate-400 bg-slate-900 rounded-lg px-3 py-1.5">
							{serverInfo.qrUrl}
						</code>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	<!-- Architecture notes -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white mb-4">Companion Architecture</h2>
		<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4 text-sm">
			{#each [
				{ icon: '⊞', title: 'Thin companion', desc: 'Capture, approvals, monitoring only. No full parity with desktop.' },
				{ icon: '◈', title: 'Local-first sync', desc: 'AES-256 encrypted. Works offline. Syncs when desktop is reachable.' },
				{ icon: '≡', title: 'Memory Spine', desc: 'Every action writes to the single source of truth. Nothing is lost.' },
				{ icon: '◉', title: 'Kaizen integration', desc: 'Rejected approvals and errors auto-generate improvement tasks.' },
			] as card}
				<div class="rounded-2xl border border-white/8 bg-slate-900/30 p-4">
					<p class="text-xl mb-2 text-cyan-400">{card.icon}</p>
					<p class="font-medium text-white mb-1">{card.title}</p>
					<p class="text-slate-500 leading-5">{card.desc}</p>
				</div>
			{/each}
		</div>
	</div>
</section>
