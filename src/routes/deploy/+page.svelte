<script lang="ts">
	const deployTargets = [
		{
			title: 'Deploy to PC',
			description: 'Clone a GitHub or Hugging Face repo, run Bun-only setup, inject central keys, and launch it locally.'
		},
		{
			title: 'Deploy to Colab',
			description: 'Generate a ready-to-run notebook with Bun bootstrap, env guidance, and Ralph loop startup cells.'
		}
	];
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-cyan-400/20 bg-slate-950/55 p-8 shadow-2xl shadow-cyan-950/20 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Deploy</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">Paste any repo. Launch from the UI.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			This screen will become the main entry point for Bun-only deploys, env injection, sandbox
			detection, and automatic “Open in Code” handoff.
		</p>

		<div class="mt-8 rounded-[1.75rem] border border-white/10 bg-slate-900/70 p-4 sm:p-5">
			<label for="repo-url" class="mb-3 block text-sm font-medium text-slate-300">GitHub or Hugging Face URL</label>
			<input
				id="repo-url"
				type="text"
				value="https://github.com/gsd-build/get-shit-done"
				readonly
				class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-4 text-sm text-white outline-none ring-0"
			/>
			<div class="mt-4 flex flex-wrap gap-3">
				<a
					href="/settings"
					class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-6 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30"
				>
					Configure keys first
				</a>
				<button
					class="rounded-full border border-white/12 bg-white/5 px-6 py-3 text-sm font-semibold text-white"
					type="button"
				>
					Launch deploy flow
				</button>
			</div>
		</div>
	</div>

	<div class="grid gap-4 lg:grid-cols-2">
		{#each deployTargets as target}
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">{target.title}</h2>
				<p class="mt-3 text-sm leading-6 text-slate-400">{target.description}</p>
			</div>
		{/each}
	</div>
</section>
