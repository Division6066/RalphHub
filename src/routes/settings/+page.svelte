<script lang="ts">
	const keys = [
		'ANTHROPIC_API_KEY',
		'OPENAI_API_KEY',
		'GROK_API_KEY',
		'GEMINI_API_KEY',
		'PERPLEXICA_KEYS'
	];
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Settings</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Central API key manager.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Secrets will be stored via Tauri Stronghold, then injected into managed workspaces only
			after explicit confirmation.
		</p>
	</div>

	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white">Key inventory</h2>
		<div class="mt-6 grid gap-4 md:grid-cols-2">
			{#each keys as key}
				<div class="rounded-2xl border border-white/10 bg-slate-950/70 p-4">
					<p class="text-sm font-medium text-white">{key}</p>
					<p class="mt-2 text-sm text-slate-500">Stronghold-backed storage placeholder</p>
				</div>
			{/each}
		</div>
	</div>
</section>
