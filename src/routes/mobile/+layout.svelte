<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { initMobile, mobileState, checkDesktopConnection } from '$lib/mobile/store.svelte.js';

	let { children } = $props();

	const tabs = [
		{ id: 'home', href: '/mobile', label: 'Home', icon: '⊞' },
		{ id: 'capture', href: '/mobile/capture', label: 'Capture', icon: '＋' },
		{ id: 'approvals', href: '/mobile/approvals', label: 'Queue', icon: '◎' },
		{ id: 'agents', href: '/mobile/agents', label: 'Agents', icon: '◈' },
		{ id: 'settings', href: '/mobile/settings', label: 'Setup', icon: '⚙' },
	];

	let currentPath = $derived(page.url.pathname);

	onMount(async () => {
		await initMobile();
		checkDesktopConnection();
	});
</script>

<div class="mobile-shell min-h-screen bg-[#060816] text-slate-100 flex flex-col max-w-md mx-auto relative">
	<!-- Status bar -->
	<div class="flex items-center justify-between px-4 pt-3 pb-1 text-xs text-slate-500">
		<span>RalphHub</span>
		<div class="flex items-center gap-2">
			{#if mobileState.pendingSyncCount > 0}
				<span class="bg-amber-500/20 text-amber-300 rounded-full px-2 py-0.5">
					{mobileState.pendingSyncCount} pending
				</span>
			{/if}
			<span class={mobileState.online ? 'text-emerald-400' : 'text-red-400'}>
				{mobileState.online ? '● Online' : '○ Offline'}
			</span>
			<span class={mobileState.desktopConnected ? 'text-cyan-400' : 'text-slate-600'}>
				{mobileState.desktopConnected ? '⇄ Desktop' : '⇄ —'}
			</span>
		</div>
	</div>

	<!-- Page content -->
	<main class="flex-1 overflow-y-auto px-4 pb-24 pt-2">
		{@render children()}
	</main>

	<!-- Bottom tab bar -->
	<nav class="fixed bottom-0 left-1/2 -translate-x-1/2 w-full max-w-md bg-slate-950/95 border-t border-white/10 backdrop-blur-xl">
		<div class="flex items-center justify-around px-2 py-2">
			{#each tabs as tab}
				<a
					href={tab.href}
					class={`flex flex-col items-center gap-0.5 px-3 py-2 rounded-2xl transition-all text-xs font-medium ${
						currentPath === tab.href || (tab.href !== '/mobile' && currentPath.startsWith(tab.href))
							? 'bg-cyan-400/15 text-cyan-300'
							: 'text-slate-500 hover:text-slate-300'
					}`}
				>
					<span class="text-lg leading-none">{tab.icon}</span>
					<span>{tab.label}</span>
					{#if tab.id === 'approvals' && mobileState.approvals.filter(a => a.status === 'pending').length > 0}
						<span class="absolute top-1 right-1 w-2 h-2 rounded-full bg-amber-400"></span>
					{/if}
				</a>
			{/each}
		</div>
		<!-- iPhone safe area -->
		<div class="h-safe-area-inset-bottom"></div>
	</nav>
</div>
