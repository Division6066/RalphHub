<script lang="ts">
	const tools = [
		'get-shit-done',
		'vibe-kanban',
		'claudia',
		'claude-code-templates',
		'Perplexica',
		'llm-council',
		'autoresearch',
		'Open in Code',
		'Universal Ralph Loop'
	];
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Tools</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Built-in manifests for every RalphHub tool.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Each tool card will expose Bun-only install/launch, live status, logs, pause/resume, and
			automatic editor opening on the correct branch with `STATE.md`.
		</p>
	</div>

	<div class="grid gap-4 xl:grid-cols-3">
		{#each tools as tool}
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-center justify-between gap-4">
					<h2 class="text-lg font-semibold text-white">{tool}</h2>
					<span class="rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-medium text-amber-100">
						Pending
					</span>
				</div>

				<div class="mt-6 flex flex-wrap gap-2 text-sm">
					<button type="button" class="rounded-full bg-cyan-400/12 px-4 py-2 font-medium text-cyan-100">Launch</button>
					<button type="button" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Logs</button>
					<button type="button" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Pause</button>
					<button type="button" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Open in Code</button>
				</div>
			</div>
		{/each}
	</div>
</section>
