<script lang="ts">
	const quickStats = [
		{ label: 'Managed tools', value: '9', detail: 'Built-in launch manifests' },
		{ label: 'Package manager', value: 'Bun only', detail: 'No npm fallback' },
		{ label: 'Workflow mode', value: 'Overnight', detail: 'Crash-resilient loop state' }
	];

	const milestones = [
		'Bootstrap Bun-only Tauri + SvelteKit shell',
		'Add Stronghold-backed central key manager',
		'Implement deploy, tool launch, and workflow orchestration'
	];
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
				The app shell is live. Secure key storage, managed repo workspaces, deploy execution,
				and workflow chaining will land in the next passes.
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
</section>
