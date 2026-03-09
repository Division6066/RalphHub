<script lang="ts">
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	const navigation = [
		{ href: '/', label: 'Dashboard', icon: '⬡', badge: '' },
		{ href: '/today', label: 'Today', icon: '☀️', badge: 'NOW' },
		{ href: '/kaizen', label: 'Kaizen', icon: '♾️', badge: '' },
		{ href: '/memory', label: 'Memory', icon: '🧠', badge: '' },
		{ href: '/vy', label: 'Vy Agent', icon: '🖥️', badge: 'AI' },
		{ href: '/panda', label: 'Panda', icon: '🐼', badge: 'PHONE' },
		{ href: '/tools', label: 'Tools', icon: '🛠️', badge: '' },
		{ href: '/workflows', label: 'Workflows', icon: '⚡', badge: '' },
		{ href: '/deploy', label: 'Deploy', icon: '🚀', badge: '' },
		{ href: '/voice', label: 'Voice', icon: '🎙️', badge: '' },
		{ href: '/mcp', label: 'MCP', icon: '🔌', badge: '' },
		{ href: '/mobile', label: 'Mobile Sync', icon: '📱', badge: '' },
		{ href: '/settings', label: 'API Keys', icon: '🔑', badge: '' }
	];

	let { children } = $props();

	function isActive(href: string) {
		if (href === '/') return page.url.pathname === '/';
		return page.url.pathname.startsWith(href);
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>AmitOS</title>
</svelte:head>

<div class="min-h-screen text-slate-100" style="background: radial-gradient(circle at top left, rgba(124,58,237,0.18) 0%, transparent 45%), radial-gradient(circle at bottom right, rgba(6,182,212,0.12) 0%, transparent 40%), #050a14;">
	<div class="mx-auto flex min-h-screen max-w-[1600px] gap-5 px-3 py-3 lg:px-5">

		<!-- Sidebar -->
		<aside class="hidden w-60 shrink-0 rounded-2xl border border-white/8 bg-slate-950/70 p-5 shadow-2xl shadow-violet-950/30 backdrop-blur lg:flex lg:flex-col">
			<div class="mb-6">
				<div class="flex items-center gap-2.5 mb-3">
					<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-violet-500 to-cyan-400 text-sm font-bold text-white shadow-lg">A</div>
					<div>
						<p class="text-[10px] uppercase tracking-[0.3em] text-violet-300/70">Universal AI OS</p>
						<h1 class="text-base font-bold text-white leading-tight">AmitOS</h1>
					</div>
				</div>
				<p class="text-xs leading-5 text-slate-400">
					Deploy, research, remember — all from one surface.
				</p>
			</div>

			<nav class="space-y-1 flex-1">
				{#each navigation as item}
					<a
						href={item.href}
						class={`flex items-center gap-3 rounded-xl px-3.5 py-2.5 text-sm font-medium transition-all ${
							isActive(item.href)
								? 'bg-violet-500/20 text-violet-100 shadow-sm border border-violet-400/20'
								: 'text-slate-400 hover:bg-white/5 hover:text-slate-200'
						}`}
					>
						<span class="text-base leading-none">{item.icon}</span>
						<span>{item.label}</span>
						{#if item.badge}
							<span class="ml-auto rounded-full {item.badge === 'NOW' ? 'bg-cyan-400/20 text-cyan-300' : item.badge === 'AI' ? 'bg-violet-400/20 text-violet-300' : item.badge === 'PHONE' ? 'bg-cyan-400/15 text-cyan-400' : 'bg-slate-700 text-slate-400'} px-1.5 py-0.5 text-[9px] font-bold">{item.badge}</span>
						{/if}
					</a>
				{/each}
			</nav>

			<div class="mt-4 rounded-xl border border-violet-500/20 bg-violet-500/8 p-3">
				<p class="text-[10px] uppercase tracking-[0.25em] text-violet-300/80 font-semibold">AmitOS v1.0</p>
				<p class="mt-1.5 text-xs leading-5 text-slate-400">
					Full Kaizen OS + Memory + Voice + 50+ API providers.
				</p>
			</div>
		</aside>

		<!-- Mobile header -->
		<div class="fixed left-0 right-0 top-0 z-50 flex items-center justify-between rounded-b-xl border-b border-white/8 bg-slate-950/95 px-4 py-3 backdrop-blur lg:hidden">
			<div class="flex items-center gap-2">
				<div class="flex h-7 w-7 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-cyan-400 text-xs font-bold text-white shadow-md">A</div>
				<span class="text-sm font-bold text-white">AmitOS</span>
			</div>
			<div class="flex gap-1">
				{#each navigation.slice(0, 6) as item}
					<a
						href={item.href}
						class={`rounded-lg px-2 py-1.5 text-base transition ${
							isActive(item.href) ? 'bg-violet-500/25 text-violet-100' : 'text-slate-400 hover:text-white'
						}`}
						title={item.label}
					>
						{item.icon}
					</a>
				{/each}
			</div>
		</div>

		<!-- Main content -->
		<main class="flex-1 min-w-0 pt-0 lg:pt-0">
			<div class="pt-14 lg:pt-0">
				{@render children()}
			</div>
		</main>
	</div>
</div>
