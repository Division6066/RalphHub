<script lang="ts">
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	const coreNav = [
		{ href: '/', label: 'Dashboard', icon: '⬡', group: 'amitos' },
		{ href: '/memory', label: 'Memory', icon: '🧠', group: 'amitos' },
		{ href: '/projects', label: 'Projects', icon: '🗂️', group: 'amitos' },
		{ href: '/tasks', label: 'Tasks', icon: '✅', group: 'amitos' },
		{ href: '/kanban', label: 'Kanban', icon: '📋', group: 'amitos' },
		{ href: '/inbox', label: 'Inbox', icon: '📥', group: 'amitos' },
		{ href: '/daily-log', label: 'Daily Log', icon: '📓', group: 'amitos' },
	];

	const integrationNav = [
		{ href: '/notion', label: 'Notion Sync', icon: '🔲', group: 'integration' },
	];

	const toolNav = [
		{ href: '/deploy', label: 'Deploy', icon: '🚀', group: 'ralph' },
		{ href: '/tools', label: 'Tools', icon: '🔧', group: 'ralph' },
		{ href: '/workflows', label: 'Workflows', icon: '⚙️', group: 'ralph' },
		{ href: '/settings', label: 'Settings', icon: '🔑', group: 'ralph' },
	];

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>AmitOS</title>
</svelte:head>

<div class="min-h-screen bg-[radial-gradient(circle_at_top,rgba(88,28,135,0.3),transparent_40%),radial-gradient(circle_at_right,rgba(34,211,238,0.15),transparent_35%),#04060f] text-slate-100">
	<div class="mx-auto flex min-h-screen max-w-[1600px] gap-0 px-0 py-0 lg:px-0">
		<!-- Sidebar -->
		<aside class="hidden w-56 shrink-0 border-r border-white/8 bg-slate-950/70 p-4 shadow-2xl shadow-cyan-950/20 backdrop-blur-xl lg:flex lg:flex-col">
			<!-- Brand -->
			<div class="mb-6 px-2">
				<div class="flex items-center gap-2">
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-cyan-400 to-purple-500 text-sm font-bold text-white shadow-lg shadow-cyan-500/30">A</div>
					<div>
						<p class="text-xs font-bold uppercase tracking-widest text-cyan-300">AmitOS</p>
						<p class="text-[10px] text-slate-500">Personal OS Layer</p>
					</div>
				</div>
			</div>

			<!-- AmitOS nav section -->
			<div class="mb-1">
				<p class="mb-1 px-2 text-[9px] font-semibold uppercase tracking-widest text-slate-600">Memory & Tasks</p>
				<nav class="space-y-0.5">
					{#each coreNav as item}
						<a
							href={item.href}
							class={`flex items-center gap-2.5 rounded-xl px-3 py-2 text-sm font-medium transition-all ${
								page.url.pathname === item.href
									? 'bg-cyan-400/15 text-cyan-100 shadow-md shadow-cyan-950/30 ring-1 ring-cyan-400/20'
									: 'text-slate-400 hover:bg-white/5 hover:text-white'
							}`}
						>
							<span class="text-base leading-none">{item.icon}</span>
							{item.label}
						</a>
					{/each}
				</nav>
			</div>

			<div class="my-2 border-t border-white/8"></div>

			<!-- Integrations -->
			<div class="mb-1">
				<p class="mb-1 px-2 text-[9px] font-semibold uppercase tracking-widest text-slate-600">Integrations</p>
				<nav class="space-y-0.5">
					{#each integrationNav as item}
						<a
							href={item.href}
							class={`flex items-center gap-2.5 rounded-xl px-3 py-2 text-sm font-medium transition-all ${
								page.url.pathname === item.href
									? 'bg-slate-400/15 text-slate-100 shadow-md ring-1 ring-slate-400/20'
									: 'text-slate-400 hover:bg-white/5 hover:text-white'
							}`}
						>
							<span class="text-base leading-none">{item.icon}</span>
							{item.label}
						</a>
					{/each}
				</nav>
			</div>

			<div class="my-2 border-t border-white/8"></div>

			<!-- RalphHub nav section -->
			<div class="mb-1">
				<p class="mb-1 px-2 text-[9px] font-semibold uppercase tracking-widest text-slate-600">RalphHub</p>
				<nav class="space-y-0.5">
					{#each toolNav as item}
						<a
							href={item.href}
							class={`flex items-center gap-2.5 rounded-xl px-3 py-2 text-sm font-medium transition-all ${
								page.url.pathname === item.href
									? 'bg-purple-400/15 text-purple-100 shadow-md ring-1 ring-purple-400/20'
									: 'text-slate-400 hover:bg-white/5 hover:text-white'
							}`}
						>
							<span class="text-base leading-none">{item.icon}</span>
							{item.label}
						</a>
					{/each}
				</nav>
			</div>

			<div class="mt-auto space-y-3">
				<div class="rounded-xl border border-cyan-400/15 bg-cyan-400/5 p-3 text-xs text-slate-300">
					<p class="font-semibold text-cyan-200">AmitOS Active</p>
					<p class="mt-1 text-slate-500">Memory Spine + Kaizen Tasks</p>
				</div>
			</div>
		</aside>

		<!-- Main content -->
		<main class="flex-1 min-h-screen overflow-auto">
			<!-- Mobile topbar -->
			<div class="flex items-center justify-between border-b border-white/8 bg-slate-950/60 px-4 py-3 backdrop-blur lg:hidden">
				<div class="flex items-center gap-2">
					<div class="flex h-7 w-7 items-center justify-center rounded-lg bg-gradient-to-br from-cyan-400 to-purple-500 text-xs font-bold text-white">A</div>
					<div>
						<p class="text-xs font-bold uppercase tracking-widest text-cyan-300">AmitOS</p>
					</div>
				</div>
				<div class="flex gap-2">
					<a href="/inbox" class="rounded-lg border border-cyan-400/30 bg-cyan-400/10 px-3 py-1.5 text-xs font-medium text-cyan-100">Inbox</a>
					<a href="/tasks" class="rounded-lg border border-purple-400/30 bg-purple-400/10 px-3 py-1.5 text-xs font-medium text-purple-100">Tasks</a>
				</div>
			</div>

			<div class="p-5 lg:p-6">
				{@render children()}
			</div>
		</main>
	</div>
</div>
