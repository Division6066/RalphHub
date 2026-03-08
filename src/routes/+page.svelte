<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type DashboardSnapshot = {
		bun: { installed: boolean; version?: string | null };
		managedProjectCount: number;
		workflowRunCount: number;
		overnightLoopCount: number;
	};

	type ManagedProject = {
		id: string;
		slug: string;
		sourceUrl: string;
		status: string;
		branch: string;
		workspacePath: string;
	};

	let snapshot: DashboardSnapshot | null = null;
	let projects: ManagedProject[] = [];
	let loading = true;
	let error = '';

	$: quickStats = [
		{ label: 'Managed projects', value: String(snapshot?.managedProjectCount ?? 0), detail: 'Tracked in SQLite' },
		{ label: 'Package manager', value: snapshot?.bun.installed ? `Bun ${snapshot?.bun.version ?? ''}`.trim() : 'Missing', detail: 'No npm fallback' },
		{ label: 'Workflow runs', value: String(snapshot?.workflowRunCount ?? 0), detail: 'Prepared overnight chains' }
	];

	const milestones = [
		'Bootstrap Bun-only Tauri + SvelteKit shell',
		'Add Stronghold-backed central key manager',
		'Implement deploy, tool launch, and workflow orchestration'
	];

	onMount(async () => {
		try {
			const [dashboard, managedProjects] = await Promise.all([
				invoke<DashboardSnapshot>('get_dashboard_snapshot'),
				invoke<ManagedProject[]>('list_managed_projects')
			]);
			snapshot = dashboard;
			projects = managedProjects;
		} catch (loadError) {
			error = loadError instanceof Error ? loadError.message : 'Failed to load dashboard.';
		} finally {
			loading = false;
		}
	});
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 shadow-2xl shadow-cyan-950/20 backdrop-blur">
		<div class="max-w-3xl">
			<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Dashboard</p>
			<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
				One desktop surface for deploys, tools, and overnight Ralph loops.
			</h1>
			<p class="mt-4 text-base leading-7 text-slate-300 sm:text-lg">
				RalphHub manages external AI repos as Bun-only workspaces, keeps launch state durable,
				and opens every active project directly in your editor with `STATE.md` ready.
			</p>
		</div>

		<div class="mt-8 flex flex-wrap gap-3">
			<a
				href="/deploy"
				class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30"
			>
				Start a deploy
			</a>
			<a
				href="/workflows"
				class="rounded-full border border-white/12 bg-white/5 px-5 py-3 text-sm font-semibold text-white"
			>
				Compose a workflow
			</a>
		</div>
	</div>

	<div class="grid gap-4 xl:grid-cols-3">
		{#each quickStats as stat}
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<p class="text-sm text-slate-400">{stat.label}</p>
				<p class="mt-3 text-2xl font-semibold text-white">{stat.value}</p>
				<p class="mt-2 text-sm text-slate-500">{stat.detail}</p>
			</div>
		{/each}
	</div>

	<div class="grid gap-4 lg:grid-cols-[1.5fr_1fr]">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-white">Milestone tracker</h2>
				<span class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-xs font-medium text-cyan-100">
					Phase 1
				</span>
			</div>

			<div class="mt-6 space-y-4">
				{#each milestones as milestone, index}
					<div class="flex items-start gap-4 rounded-2xl border border-white/6 bg-white/3 p-4">
						<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-cyan-400/15 text-sm font-semibold text-cyan-100">
							{index + 1}
						</div>
						<p class="pt-1 text-sm leading-6 text-slate-300">{milestone}</p>
					</div>
				{/each}
			</div>
		</div>

		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Current status</h2>
			<p class="mt-4 text-sm leading-6 text-slate-400">
				{#if error}
					{error}
				{:else if loading}
					Loading RalphHub runtime state...
				{:else}
					Bun status, deploy state, and workflow counts are now coming from the Tauri backend.
				{/if}
			</p>
			<div class="mt-6 rounded-2xl border border-violet-400/20 bg-violet-500/10 p-4">
				<p class="text-xs uppercase tracking-[0.25em] text-violet-200/80">Target</p>
				<p class="mt-2 text-sm leading-6 text-slate-200">
					Cross-platform Tauri app with Windows local builds and CI-driven `.dmg` / `.AppImage`
					release artifacts.
				</p>
			</div>
		</div>
	</div>

	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-semibold text-white">Managed workspaces</h2>
			<span class="text-sm text-slate-500">{projects.length} tracked</span>
		</div>

		<div class="mt-6 space-y-3">
			{#if !projects.length}
				<p class="text-sm text-slate-500">No managed repos yet. Deploy one from the Deploy tab.</p>
			{:else}
				{#each projects as project}
					<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex items-center justify-between gap-4">
							<div>
								<p class="text-sm font-medium text-white">{project.slug}</p>
								<p class="mt-1 text-xs text-slate-500">{project.sourceUrl}</p>
							</div>
							<span class="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-xs text-cyan-100">
								{project.status}
							</span>
						</div>
						<p class="mt-2 text-xs text-slate-500">Branch: {project.branch}</p>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
