<script lang="ts">
	const flow = [
		'Perplexica researches',
		'llm-council votes',
		'get-shit-done executes',
		'vibe-kanban updates board',
		'claudia consolidates memory'
	];
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-violet-400/20 bg-slate-950/55 p-8 shadow-2xl shadow-violet-950/20 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-violet-200/80">Workflows</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Compose a real multi-tool Ralph loop.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			The final composer will persist a combined manifest, write `STATE.md`, and run a durable
			overnight chain in the background.
		</p>
	</div>

	<div class="grid gap-4 lg:grid-cols-[1.25fr_0.9fr]">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Example chain</h2>
			<div class="mt-6 space-y-3">
				{#each flow as step, index}
					<div class="flex items-center gap-4 rounded-2xl border border-white/6 bg-white/3 p-4">
						<div class="flex h-8 w-8 items-center justify-center rounded-full bg-violet-400/15 text-sm font-semibold text-violet-100">
							{index + 1}
						</div>
						<p class="text-sm text-slate-300">{step}</p>
					</div>
				{/each}
			</div>
		</div>

		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Composer goals</h2>
			<ul class="mt-4 space-y-3 text-sm leading-6 text-slate-400">
				<li>Persist combined workflow config</li>
				<li>Generate `STATE.md` per run</li>
				<li>Support pause, resume, retries, and logs</li>
				<li>Feed Open in Code automatically for the active workspace</li>
			</ul>
		</div>
	</div>
</section>
