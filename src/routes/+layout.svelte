<script lang="ts">
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	const navigation = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/deploy', label: 'Deploy' },
		{ href: '/tools', label: 'Tools' },
		{ href: '/workflows', label: 'Workflows' },
		{ href: '/ollama', label: 'Ollama' },
		{ href: '/voice', label: 'Voice' },
		{ href: '/memory', label: 'Memory' },
		{ href: '/kaizen', label: 'Kaizen' },
		{ href: '/browser-agent', label: 'Browser Agent' },
		{ href: '/launch', label: 'Launch' },
		{ href: '/settings', label: 'Settings' }
	];

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>RalphHub</title>
</svelte:head>

<div class="min-h-screen bg-[radial-gradient(circle_at_top,rgba(88,28,135,0.25),transparent_35%),radial-gradient(circle_at_right,rgba(34,211,238,0.12),transparent_30%),#060816] text-slate-100">
	<div class="mx-auto flex min-h-screen max-w-7xl gap-6 px-4 py-4 lg:px-6">
		<aside class="hidden w-64 shrink-0 rounded-3xl border border-white/10 bg-slate-950/60 p-5 shadow-2xl shadow-cyan-950/20 backdrop-blur lg:flex lg:flex-col">
			<div class="mb-8">
				<p class="text-xs uppercase tracking-[0.35em] text-cyan-300/80">RalphHub</p>
				<h1 class="mt-3 text-2xl font-semibold text-white">Central AI workstation</h1>
				<p class="mt-3 text-sm leading-6 text-slate-400">
					Bun-only orchestration for deploys, tools, workflows, and overnight Ralph loops.
				</p>
			</div>

			<nav class="space-y-2">
				{#each navigation as item}
					<a
						href={item.href}
						class={`flex items-center rounded-2xl px-4 py-3 text-sm font-medium transition ${
							page.url.pathname === item.href
								? 'bg-cyan-400/15 text-cyan-100 shadow-lg shadow-cyan-950/30'
								: 'text-slate-400 hover:bg-white/5 hover:text-white'
						}`}
					>
						{item.label}
					</a>
				{/each}
			</nav>

			<div class="mt-auto rounded-2xl border border-cyan-400/20 bg-cyan-400/8 p-4 text-sm text-slate-300">
				<p class="font-medium text-cyan-100">RalphHub v2 — Fully wired</p>
				<p class="mt-2 leading-6 text-slate-400">
					All wishlist tools, Ollama local models, voice mode, MCP browser, Memory Spine + Kaizen.
				</p>
			</div>
		</aside>

		<main class="flex-1">
			<div class="mb-4 flex items-center justify-between rounded-3xl border border-white/10 bg-slate-950/40 px-4 py-3 backdrop-blur lg:hidden">
				<div>
					<p class="text-xs uppercase tracking-[0.35em] text-cyan-300/80">RalphHub</p>
					<p class="text-sm text-slate-400">AI workstation</p>
				</div>
				<a
					href="/deploy"
					class="rounded-full border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm font-medium text-cyan-100"
				>
					Deploy
				</a>
			</div>

			{@render children()}
		</main>
	</div>
</div>
