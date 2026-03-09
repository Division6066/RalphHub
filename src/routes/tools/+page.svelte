<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type ToolManifest = {
		id: string;
		name: string;
		repoUrl: string;
		description: string;
		status: string;
		category: string;
		openInCode: boolean;
		needsSandbox: boolean;
		requiredKeys: string[];
		tags: string[];
	};

	type DeployResult = {
		workspacePath: string;
		branch: string;
		message: string;
	};

	const CATEGORY_META: Record<string, { label: string; icon: string; color: string }> = {
		research: { label: 'Research', icon: '🔍', color: 'violet' },
		coding: { label: 'Coding Agents', icon: '💻', color: 'cyan' },
		tasks: { label: 'Tasks', icon: '📋', color: 'amber' },
		'multi-model': { label: 'Multi-Model', icon: '🧠', color: 'purple' },
		browser: { label: 'Browser / Web', icon: '🌐', color: 'blue' },
		memory: { label: 'Memory', icon: '💾', color: 'emerald' },
		voice: { label: 'Voice', icon: '🎙️', color: 'rose' },
		mcp: { label: 'MCP Servers', icon: '⚡', color: 'orange' },
		design: { label: 'Design / UI', icon: '🎨', color: 'pink' },
		data: { label: 'Data & Analytics', icon: '📊', color: 'teal' },
		internal: { label: 'Internal', icon: '🔧', color: 'slate' },
	};

	let tools: ToolManifest[] = [];
	let loading = true;
	let busyTool = '';
	let status = '';
	let searchQuery = '';
	let selectedCategory = 'all';
	let isDesktop = false;

	onMount(async () => {
		isDesktop = isDesktopRuntime();
		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		loading = false;
	});

	async function launchTool(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			status = `${tool.name} is an internal AmitOS capability.`;
			return;
		}
		if (!isDesktop) { status = 'Desktop runtime required to deploy tools.'; return; }

		busyTool = tool.id;
		status = `Deploying ${tool.name}…`;
		try {
			await invokeTauri('ensure_bun');
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
			await invokeTauri('open_in_code', { workspacePath: result.workspacePath, branch: result.branch });
			status = `✓ ${tool.name} deployed and opened in editor.`;
		} catch (error) {
			status = error instanceof Error ? error.message : `Failed to launch ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}

	$: categories = ['all', ...new Set(tools.map((t) => t.category))];

	$: filteredTools = tools.filter((t) => {
		const matchCat = selectedCategory === 'all' || t.category === selectedCategory;
		const matchSearch = !searchQuery || t.name.toLowerCase().includes(searchQuery.toLowerCase()) || t.description.toLowerCase().includes(searchQuery.toLowerCase()) || t.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));
		return matchCat && matchSearch;
	});

	$: groupedTools = (() => {
		const g: Record<string, ToolManifest[]> = {};
		for (const t of filteredTools) {
			if (!g[t.category]) g[t.category] = [];
			g[t.category].push(t);
		}
		return g;
	})();

	function colorClass(color: string) {
		const map: Record<string, string> = {
			violet: 'border-violet-400/20 bg-violet-400/8',
			cyan: 'border-cyan-400/20 bg-cyan-400/8',
			amber: 'border-amber-400/20 bg-amber-400/8',
			purple: 'border-purple-400/20 bg-purple-400/8',
			blue: 'border-blue-400/20 bg-blue-400/8',
			emerald: 'border-emerald-400/20 bg-emerald-400/8',
			rose: 'border-rose-400/20 bg-rose-400/8',
			orange: 'border-orange-400/20 bg-orange-400/8',
			pink: 'border-pink-400/20 bg-pink-400/8',
			teal: 'border-teal-400/20 bg-teal-400/8',
			slate: 'border-slate-400/20 bg-slate-400/8',
		};
		return map[color] ?? 'border-white/10 bg-white/5';
	}
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-white/10 bg-gradient-to-br from-slate-950/80 via-violet-950/30 to-slate-950/80 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-slate-300/60">Tool Catalog</p>
		<h1 class="mt-2 text-3xl font-bold text-white">🛠️ 35+ AI Tools</h1>
		<p class="mt-2 text-sm text-slate-400">One-click deploy. Perplexica, Aider, OpenHands, Playwright MCP, Firecrawl, and more.</p>
		<div class="mt-4">
			<p class="text-sm text-slate-400">{status}</p>
		</div>
	</div>

	<!-- Search + filter -->
	<div class="flex flex-wrap gap-3">
		<div class="relative flex-1 min-w-48">
			<span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-500">🔍</span>
			<input bind:value={searchQuery} placeholder="Search tools…" class="w-full rounded-xl border border-white/10 bg-slate-950/60 pl-10 pr-4 py-2.5 text-sm text-white outline-none focus:border-violet-400 backdrop-blur" />
		</div>
		<div class="flex flex-wrap gap-1.5">
			<button onclick={() => selectedCategory = 'all'} class={`rounded-xl border px-3 py-2 text-xs font-medium transition ${selectedCategory === 'all' ? 'border-violet-400/30 bg-violet-400/15 text-violet-100' : 'border-white/8 bg-slate-950/50 text-slate-400 hover:text-white'}`}>All ({tools.length})</button>
			{#each categories.filter(c => c !== 'all') as cat}
				{@const meta = CATEGORY_META[cat] ?? { label: cat, icon: '⚙️', color: 'slate' }}
				<button onclick={() => selectedCategory = cat} class={`rounded-xl border px-3 py-2 text-xs font-medium transition ${selectedCategory === cat ? 'border-violet-400/30 bg-violet-400/15 text-violet-100' : 'border-white/8 bg-slate-950/50 text-slate-400 hover:text-white'}`}>
					{meta.icon} {meta.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Tool groups -->
	{#if loading}
		<div class="py-10 text-center text-sm text-slate-400">Loading tools…</div>
	{:else}
		{#each Object.entries(groupedTools) as [category, toolGroup]}
			{@const meta = CATEGORY_META[category] ?? { label: category, icon: '⚙️', color: 'slate' }}
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					<span class="text-lg">{meta.icon}</span>
					<h2 class="text-sm font-bold text-white">{meta.label}</h2>
					<span class="text-xs text-slate-500">({toolGroup.length})</span>
				</div>
				<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
					{#each toolGroup as tool}
						<div class={`group flex flex-col rounded-xl border p-4 transition hover:scale-[1.01] ${colorClass(meta.color)}`}>
							<div class="flex items-start justify-between gap-3">
								<div class="flex-1 min-w-0">
									<p class="text-sm font-bold text-white">{tool.name}</p>
									{#if tool.needsSandbox}
										<span class="mt-1 inline-block rounded-full border border-amber-400/20 bg-amber-400/10 px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-amber-300">Sandbox</span>
									{/if}
								</div>
								<span class="shrink-0 rounded-full border border-white/10 bg-white/5 px-2 py-0.5 text-[10px] text-slate-400">
									{tool.status}
								</span>
							</div>
							<p class="mt-2 flex-1 text-xs leading-5 text-slate-400">{tool.description}</p>

							{#if tool.tags.length > 0}
								<div class="mt-2.5 flex flex-wrap gap-1">
									{#each tool.tags.slice(0, 4) as tag}
										<span class="rounded-full bg-white/5 px-2 py-0.5 text-[10px] text-slate-500">{tag}</span>
									{/each}
								</div>
							{/if}

							{#if tool.requiredKeys.length > 0}
								<p class="mt-2 text-[10px] text-slate-600">Needs: {tool.requiredKeys.slice(0, 2).join(', ')}{tool.requiredKeys.length > 2 ? ` +${tool.requiredKeys.length - 2}` : ''}</p>
							{/if}

							<div class="mt-4 flex flex-wrap gap-2">
								{#if !tool.repoUrl.startsWith('internal://')}
									<button
										onclick={() => launchTool(tool)}
										disabled={busyTool === tool.id}
										class="rounded-xl bg-white/10 px-4 py-2 text-xs font-semibold text-white transition hover:bg-white/20 disabled:cursor-not-allowed disabled:opacity-60"
									>
										{busyTool === tool.id ? 'Deploying…' : '🚀 Deploy'}
									</button>
									<a href={tool.repoUrl} target="_blank" rel="noopener" class="rounded-xl border border-white/10 bg-white/3 px-4 py-2 text-xs font-medium text-slate-400 transition hover:text-white">
										GitHub ↗
									</a>
								{:else}
									<a href={tool.id === 'voice-command' ? '/voice' : tool.id === 'memory-spine' ? '/memory' : '/'} class="rounded-xl bg-white/10 px-4 py-2 text-xs font-semibold text-white transition hover:bg-white/20">
										Open →
									</a>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/each}
	{/if}
</section>
