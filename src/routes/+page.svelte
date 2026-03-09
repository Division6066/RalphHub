<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type Snapshot = {
		bun: { installed: boolean; version?: string | null };
		managedProjectCount: number;
		workflowRunCount: number;
		memoryEntryCount: number;
		kaizenTaskCount: number;
		todayTaskCount: number;
		apiKeyCount: number;
	};

	let snapshot: Snapshot | null = null;
	let loading = true;
	let isDesktop = false;

	const features = [
		{ icon: '☀️', title: 'Today Board', desc: 'ADHD-friendly daily focus — pick 3 tasks and ship them.', href: '/today', color: 'from-amber-500/20 to-orange-500/10', border: 'border-amber-400/20', badge: '' },
		{ icon: '♾️', title: 'Kaizen OS', desc: 'Continuous improvement across all life domains.', href: '/kaizen', color: 'from-violet-500/20 to-purple-500/10', border: 'border-violet-400/20', badge: '' },
		{ icon: '🧠', title: 'Memory Spine', desc: 'Capture and search everything you want to remember.', href: '/memory', color: 'from-cyan-500/20 to-blue-500/10', border: 'border-cyan-400/20', badge: '' },
		{ icon: '🖥️', title: 'Vy Agent', desc: 'Desktop AI that watches your screen and executes with approval.', href: '/vy', color: 'from-violet-600/20 to-indigo-500/10', border: 'border-violet-500/30', badge: 'NEW' },
		{ icon: '🐼', title: 'Panda Phone', desc: 'Use your phone to approve actions, capture voice and check today tasks.', href: '/panda', color: 'from-cyan-600/20 to-teal-500/10', border: 'border-cyan-500/30', badge: 'NEW' },
		{ icon: '🛠️', title: '35+ AI Tools', desc: 'One-click deploy of Perplexica, Aider, OpenHands, and more.', href: '/tools', color: 'from-emerald-500/20 to-teal-500/10', border: 'border-emerald-400/20', badge: '' },
		{ icon: '🔑', title: '50+ API Keys', desc: 'Universal key manager — all providers, zero friction.', href: '/settings', color: 'from-rose-500/20 to-pink-500/10', border: 'border-rose-400/20', badge: '' },
		{ icon: '🎙️', title: 'Voice Mode', desc: 'Say commands to control AmitOS hands-free.', href: '/voice', color: 'from-indigo-500/20 to-blue-500/10', border: 'border-indigo-400/20', badge: '' },
		{ icon: '⚡', title: 'Workflows', desc: 'Chain AI tools into overnight research and coding loops.', href: '/workflows', color: 'from-amber-600/20 to-yellow-500/10', border: 'border-amber-400/20', badge: '' },
	];

	onMount(async () => {
		isDesktop = isDesktopRuntime();
		if (!isDesktop) { loading = false; return; }
		try {
			snapshot = await invokeTauri<Snapshot>('get_dashboard_snapshot');
		} catch {}
		loading = false;
	});

	function stat(label: string, value: string | number, sub: string) {
		return { label, value: String(value), sub };
	}

	$: stats = snapshot
		? [
				stat('Today tasks', snapshot.todayTaskCount, 'Waiting for you'),
				stat('Total tasks', snapshot.kaizenTaskCount, 'Across all domains'),
				stat('Memories', snapshot.memoryEntryCount, 'Saved to spine'),
				stat('API providers', snapshot.apiKeyCount, 'Keys configured'),
				stat('Workflows', snapshot.workflowRunCount, 'Runs prepared'),
				stat('Bun', snapshot.bun.installed ? (snapshot.bun.version ?? '✓') : '✗', 'Package manager'),
			]
		: [];
</script>

<section class="space-y-5">
	<!-- Hero -->
	<div class="relative overflow-hidden rounded-2xl border border-white/8 bg-gradient-to-br from-violet-950/60 via-slate-950/80 to-cyan-950/40 p-8 shadow-2xl backdrop-blur">
		<div class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_30%_50%,rgba(139,92,246,0.15),transparent_60%)]"></div>
		<div class="relative">
			<div class="mb-4 inline-flex items-center gap-2 rounded-full border border-violet-400/20 bg-violet-400/10 px-3 py-1">
				<span class="h-1.5 w-1.5 rounded-full bg-violet-400 animate-pulse"></span>
				<span class="text-[11px] font-semibold uppercase tracking-widest text-violet-300">AmitOS v1.0 — Ready</span>
			</div>
			<h1 class="text-4xl font-bold tracking-tight text-white sm:text-5xl">
				Your Universal<br/>
				<span class="bg-gradient-to-r from-violet-400 to-cyan-400 bg-clip-text text-transparent">AI Operating System</span>
			</h1>
			<p class="mt-4 max-w-2xl text-base leading-7 text-slate-300">
				Today Board · Kaizen OS · Memory Spine · 35+ AI tools · 50+ API providers · Voice mode · Desktop + Mobile.
				Everything for ADHD-friendly deep work, shipped in one app.
			</p>
			<div class="mt-6 flex flex-wrap gap-3">
				<a href="/today" class="rounded-xl bg-gradient-to-r from-violet-500 to-cyan-500 px-5 py-2.5 text-sm font-bold text-white shadow-lg shadow-violet-500/30 transition hover:scale-105">
					☀️ Start Today
				</a>
				<a href="/kaizen" class="rounded-xl border border-white/12 bg-white/5 px-5 py-2.5 text-sm font-semibold text-white transition hover:bg-white/10">
					♾️ Kaizen Board
				</a>
				<a href="/tools" class="rounded-xl border border-white/12 bg-white/5 px-5 py-2.5 text-sm font-semibold text-white transition hover:bg-white/10">
					🛠️ Launch Tools
				</a>
			</div>
		</div>
	</div>

	<!-- Stats row -->
	{#if !loading && stats.length > 0}
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-6">
			{#each stats as s}
				<div class="rounded-xl border border-white/8 bg-slate-950/50 p-4 backdrop-blur">
					<p class="text-xs text-slate-500">{s.label}</p>
					<p class="mt-1.5 text-2xl font-bold text-white">{s.value}</p>
					<p class="mt-0.5 text-xs text-slate-500">{s.sub}</p>
				</div>
			{/each}
		</div>
	{:else if loading}
		<div class="flex items-center gap-2 rounded-xl border border-white/8 bg-slate-950/50 p-4 text-sm text-slate-400">
			<span class="animate-pulse">⬡</span>
			<span>Loading AmitOS runtime…</span>
		</div>
	{/if}

	<!-- Feature grid -->
	<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
		{#each features as f}
			<a
				href={f.href}
				class={`group relative overflow-hidden rounded-2xl border ${f.border} bg-gradient-to-br ${f.color} p-6 transition hover:scale-[1.02] hover:shadow-lg`}
			>
				{#if f.badge}
					<span class="absolute right-3 top-3 rounded-full bg-violet-500/30 border border-violet-400/40 px-2 py-0.5 text-[9px] font-bold uppercase tracking-wider text-violet-200">{f.badge}</span>
				{/if}
				<div class="mb-3 text-3xl">{f.icon}</div>
				<h3 class="text-base font-bold text-white">{f.title}</h3>
				<p class="mt-2 text-sm leading-6 text-slate-300">{f.desc}</p>
				<span class="absolute bottom-4 right-4 text-slate-600 transition group-hover:text-slate-400">→</span>
			</a>
		{/each}
	</div>

	<!-- Quick actions -->
	<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-6 backdrop-blur">
		<h2 class="mb-4 text-base font-bold text-white">Quick Actions</h2>
		<div class="flex flex-wrap gap-2">
			<a href="/today" class="rounded-xl border border-amber-400/25 bg-amber-400/10 px-4 py-2 text-sm font-medium text-amber-200 transition hover:bg-amber-400/20">☀️ Open Today Board</a>
			<a href="/kaizen" class="rounded-xl border border-violet-400/25 bg-violet-400/10 px-4 py-2 text-sm font-medium text-violet-200 transition hover:bg-violet-400/20">+ New Kaizen Task</a>
			<a href="/memory" class="rounded-xl border border-cyan-400/25 bg-cyan-400/10 px-4 py-2 text-sm font-medium text-cyan-200 transition hover:bg-cyan-400/20">🧠 Add Memory</a>
			<a href="/vy" class="rounded-xl border border-violet-500/30 bg-violet-500/10 px-4 py-2 text-sm font-medium text-violet-200 transition hover:bg-violet-500/20">🖥️ Start Vy Agent</a>
			<a href="/panda" class="rounded-xl border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-sm font-medium text-cyan-200 transition hover:bg-cyan-500/20">🐼 Panda Phone</a>
			<a href="/voice" class="rounded-xl border border-indigo-400/25 bg-indigo-400/10 px-4 py-2 text-sm font-medium text-indigo-200 transition hover:bg-indigo-400/20">🎙️ Voice Command</a>
			<a href="/settings" class="rounded-xl border border-rose-400/25 bg-rose-400/10 px-4 py-2 text-sm font-medium text-rose-200 transition hover:bg-rose-400/20">🔑 Add API Key</a>
			<a href="/workflows" class="rounded-xl border border-emerald-400/25 bg-emerald-400/10 px-4 py-2 text-sm font-medium text-emerald-200 transition hover:bg-emerald-400/20">⚡ New Workflow</a>
		</div>
	</div>

	{#if !isDesktop}
		<div class="rounded-xl border border-amber-400/20 bg-amber-400/8 p-4 text-sm text-amber-200">
			<strong>Running in browser mode.</strong> For full functionality including secure key storage, Kaizen persistence, and tool deployment — launch the AmitOS desktop app.
		</div>
	{/if}
</section>
