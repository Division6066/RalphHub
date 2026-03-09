<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import ModelSwitcher from '$lib/components/ModelSwitcher.svelte';
	import { loadProviders, enabledProvidersStore, activeModelStore, memoryStatsStore } from '$lib/utils/provider-registry';

	const navigation = [
		{ href: '/', label: 'Dashboard', icon: '⬡', badge: '' },
		{ href: '/today', label: 'Today Board', icon: '📅', badge: 'NEW' },
		{ href: '/kaizen', label: 'Kaizen Tasks', icon: '♾️', badge: '' },
		{ href: '/memory', label: 'Memory Spine', icon: '🧠', badge: '' },
		{ href: '/vy', label: 'Vy Agent', icon: '🖥️', badge: 'AI' },
		{ href: '/panda', label: 'Panda Phone', icon: '🐼', badge: 'PHONE' },
		{ href: '/parallel', label: 'Parallel Run', icon: '⚡', badge: 'NEW' },
		{ href: '/voice', label: 'Voice + Chat', icon: '🎙️', badge: '' },
		{ href: '/remote-nodes', label: 'VPS + RPi', icon: '☁️', badge: '' },
		{ href: '/mcp', label: 'MCP Browser', icon: '🔌', badge: '' },
		{ href: '/mobile', label: 'Mobile Sync', icon: '📱', badge: '' },
		{ href: '/tools', label: 'Tools', icon: '🛠️', badge: '' },
		{ href: '/workflows', label: 'Workflows', icon: '🔄', badge: '' },
		{ href: '/deploy', label: 'Deploy', icon: '🚀', badge: '' },
		{ href: '/settings', label: 'API Keys', icon: '🔑', badge: '' },
	];

	let { children } = $props();

	function isActive(href: string) {
		if (href === '/') return page.url.pathname === '/';
		return page.url.pathname.startsWith(href);
	}

	onMount(async () => {
		await loadProviders();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>AmitOS — Universal AI OS</title>
</svelte:head>

<div class="min-h-screen text-slate-100 amitos-bg">
	<div class="mx-auto flex min-h-screen max-w-[1600px] gap-5 px-3 py-3 lg:px-5">

		<!-- Sidebar -->
		<aside class="hidden w-60 shrink-0 rounded-2xl border border-white/8 bg-slate-950/75 p-5 shadow-2xl shadow-violet-950/30 backdrop-blur-xl lg:flex lg:flex-col">
			<!-- Brand -->
			<div class="mb-6">
				<div class="flex items-center gap-2.5 mb-3">
					<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-violet-500 to-cyan-400 text-sm font-bold text-white shadow-lg shadow-violet-500/30">A</div>
					<div>
						<p class="text-[10px] uppercase tracking-[0.3em] text-violet-300/70">Universal AI OS</p>
						<h1 class="text-base font-bold text-white leading-tight">AmitOS</h1>
					</div>
				</div>
				<p class="text-xs leading-5 text-slate-400">
					Deploy · Remember · Control — from one surface.
				</p>
			</div>

			<!-- Nav -->
			<nav class="space-y-0.5 flex-1 overflow-y-auto">
				{#each navigation as item}
					<a
						href={item.href}
						class={`nav-item flex items-center gap-3 rounded-xl px-3.5 py-2.5 text-sm font-medium transition-all duration-200 ${
							isActive(item.href)
								? 'bg-violet-500/20 text-violet-100 border border-violet-400/25 shadow-sm shadow-violet-900/50'
								: 'text-slate-400 hover:bg-white/5 hover:text-slate-200'
						}`}
					>
						<span class="text-base leading-none shrink-0">{item.icon}</span>
						<span class="truncate">{item.label}</span>
						{#if item.badge}
							<span class="ml-auto shrink-0 rounded-full px-1.5 py-0.5 text-[9px] font-bold
								{item.badge === 'NEW' ? 'bg-emerald-400/20 text-emerald-300' :
								 item.badge === 'AI' ? 'bg-violet-400/20 text-violet-300' :
								 item.badge === 'PHONE' ? 'bg-cyan-400/20 text-cyan-300' :
								 'bg-slate-700/80 text-slate-400'}">{item.badge}</span>
						{/if}
					</a>
				{/each}
			</nav>

			<!-- Model switcher -->
			<div class="mt-4 pt-4 border-t border-white/8">
				<p class="mb-2 text-[10px] uppercase tracking-wider text-slate-600">Active Model</p>
				<ModelSwitcher compact />
			</div>

			<!-- Status card -->
			<div class="mt-3 rounded-xl border border-cyan-400/15 bg-cyan-400/5 p-3 text-xs text-slate-400">
				<p class="font-semibold text-cyan-100 text-sm">
					{$enabledProvidersStore.length > 0 ? `${$enabledProvidersStore.length} providers active` : 'No providers active'}
				</p>
				{#if $activeModelStore}
					<p class="mt-1 truncate text-slate-500">Model: {$activeModelStore}</p>
				{:else}
					<a href="/settings" class="mt-1 block text-cyan-400 hover:text-cyan-300 underline underline-offset-2">Connect a provider →</a>
				{/if}
				{#if $memoryStatsStore}
					<p class="mt-1 text-slate-600">
						{$memoryStatsStore.totalEntries} memories · ${$memoryStatsStore.totalCostUsd.toFixed(4)}
					</p>
				{/if}
			</div>

			<!-- Footer -->
			<div class="mt-3 rounded-xl border border-white/5 bg-white/3 px-3 py-2 text-center">
				<p class="text-[10px] text-slate-600">AmitOS v1.0 · MIT License</p>
			</div>
		</aside>

		<!-- Mobile top bar -->
		<div class="fixed left-0 right-0 top-0 z-50 flex items-center justify-between border-b border-white/8 bg-slate-950/97 px-3 py-2 backdrop-blur-xl lg:hidden">
			<div class="flex items-center gap-2 shrink-0">
				<div class="flex h-7 w-7 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-cyan-400 text-xs font-bold text-white shadow-md">A</div>
				<span class="text-sm font-bold text-white">AmitOS</span>
			</div>
			<div class="flex gap-0.5 overflow-x-auto ml-2">
				{#each navigation.slice(0, 8) as item}
					<a
						href={item.href}
						class={`rounded-lg px-2 py-1.5 text-base transition-all ${
							isActive(item.href) ? 'bg-violet-500/25 text-violet-100' : 'text-slate-400 hover:text-white active:scale-95'
						}`}
						title={item.label}
						style="min-width: 36px; min-height: 36px; display: flex; align-items: center; justify-content: center;"
					>
						{item.icon}
					</a>
				{/each}
			</div>
		</div>

		<!-- Main content -->
		<main class="flex-1 min-w-0">
			<div class="pt-16 lg:pt-0">
				{@render children()}
			</div>
		</main>
	</div>
</div>
