<script lang="ts">
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import { KEY_FIELDS, loadKeys, type KeyMap } from '$lib/utils/secure-store';

	type DeployResult = {
		workspacePath: string;
		normalizedUrl: string;
		branch: string;
		message: string;
		statePath: string;
		envPath: string;
		notebookPath?: string | null;
	};

	type EnvEntry = { key: string; value: string };

	const deployTargets = [
		{
			title: 'Deploy to PC',
			description:
				'Clone a GitHub or Hugging Face repo, run Bun-only setup, inject central keys, and launch locally.'
		},
		{
			title: 'Deploy to Colab',
			description:
				'Generate a ready-to-run notebook with Bun bootstrap, env guidance, and Ralph loop startup cells. Colab opens automatically after generation.'
		}
	];

	let repoUrl = 'https://github.com/gsd-build/get-shit-done';
	let loading = false;
	let error = '';
	let status = 'Ready to deploy.';
	let lastResult: DeployResult | null = null;
	let colabLaunched = false;
	let colabLaunching = false;
	let startingExecution = false;
	let executionConfirmed = false;

	// ── Deploy to PC ──────────────────────────────────────────────────────────
	async function deployToPc() {
		if (!isDesktopRuntime()) {
			status = 'Deploy actions are available in the RalphHub desktop runtime.';
			return;
		}
		loading = true;
		error = '';
		status = 'Ensuring Bun and cloning repository...';

		try {
			await invokeTauri('ensure_bun');
			const result = await invokeTauri<DeployResult>('deploy_to_pc', {
				request: { url: repoUrl }
			});
			lastResult = result;
			status = result.message;

			const keys = await loadKeys();
			const entries = buildEnvEntries(keys);
			if (
				entries.length &&
				window.confirm('Inject your saved central API keys into this workspace now?')
			) {
				await invokeTauri('inject_keys', {
					request: { workspacePath: result.workspacePath, entries }
				});
				status = 'Repository deployed, keys injected, workspace ready.';
			}

			await invokeTauri('open_in_code', {
				workspacePath: result.workspacePath,
				branch: result.branch
			});
		} catch (err) {
			error = err instanceof Error ? err.message : 'Deploy to PC failed.';
			status = 'Deploy failed.';
		} finally {
			loading = false;
		}
	}

	// ── Deploy to Colab ───────────────────────────────────────────────────────
	async function deployToColab() {
		if (!isDesktopRuntime()) {
			status = 'Colab generation is available in the RalphHub desktop runtime.';
			return;
		}
		loading = true;
		colabLaunched = false;
		error = '';
		status = 'Generating Colab notebook...';

		try {
			const result = await invokeTauri<DeployResult>('deploy_to_colab', {
				request: { url: repoUrl }
			});
			lastResult = result;
			status = result.notebookPath
				? `Notebook generated at ${result.notebookPath}`
				: result.message;

			// Auto-open Colab in user's browser
			await openColab();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Colab generation failed.';
			status = 'Colab generation failed.';
		} finally {
			loading = false;
		}
	}

	// ── Open Colab ────────────────────────────────────────────────────────────
	async function openColab() {
		if (!isDesktopRuntime()) return;
		colabLaunching = true;
		try {
			const res = await invokeTauri<{ ok: boolean; message: string }>('open_colab_url');
			status = res.message;
			colabLaunched = true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to open Colab.';
		} finally {
			colabLaunching = false;
		}
	}

	// ── Start Execution ───────────────────────────────────────────────────────
	async function startColabExecution() {
		if (!isDesktopRuntime()) return;

		// Permission-mode confirmation
		if (!executionConfirmed) {
			const ok = window.confirm(
				'Start execution of the generated notebook in Colab?\n\n' +
					'RalphHub will open Colab in your browser. Run the notebook cells manually, ' +
					'or use Colab\'s "Run all" feature.\n\n' +
					'Note: Colab Pro / compute credits are required for GPU-accelerated runs. ' +
					'A quota warning will appear in the notebook if credits are insufficient.'
			);
			if (!ok) return;
			executionConfirmed = true;
		}

		startingExecution = true;
		error = '';
		try {
			const res = await invokeTauri<{ ok: boolean; message: string }>('open_colab_url');
			status = res.message + ' — use Runtime → Run all to execute all cells.';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to open Colab for execution.';
		} finally {
			startingExecution = false;
		}
	}

	function buildEnvEntries(keys: KeyMap): EnvEntry[] {
		return KEY_FIELDS.filter((f) => keys[f].trim()).map((f) => ({
			key: f,
			value: keys[f].trim()
		}));
	}
</script>

<section class="space-y-6">
	<!-- Header -->
	<div
		class="rounded-[2rem] border border-cyan-400/20 bg-slate-950/55 p-8 shadow-2xl shadow-cyan-950/20 backdrop-blur"
	>
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Deploy</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">
			Paste any repo. Launch from the UI.
		</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Bun-only deploys with env injection, sandbox detection, and automatic editor handoff. Colab
			notebooks open in your browser immediately after generation.
		</p>

		<div class="mt-8 rounded-[1.75rem] border border-white/10 bg-slate-900/70 p-4 sm:p-5">
			<label for="repo-url" class="mb-3 block text-sm font-medium text-slate-300">
				GitHub or Hugging Face URL
			</label>
			<input
				id="repo-url"
				type="text"
				bind:value={repoUrl}
				class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-4 text-sm text-white outline-none"
			/>

			<div class="mt-4 flex flex-wrap gap-3">
				<button
					type="button"
					on:click={deployToPc}
					disabled={loading}
					class="rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-6 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:cursor-not-allowed disabled:opacity-60"
				>
					{loading ? 'Working...' : 'Deploy to PC'}
				</button>
				<button
					type="button"
					on:click={deployToColab}
					disabled={loading}
					class="rounded-full border border-white/12 bg-white/5 px-6 py-3 text-sm font-semibold text-white disabled:opacity-60"
				>
					{loading ? 'Generating...' : 'Deploy to Colab'}
				</button>
				<a
					href="/settings"
					class="rounded-full border border-cyan-400/30 bg-cyan-400/10 px-6 py-3 text-sm font-semibold text-cyan-100"
				>
					Configure keys
				</a>
			</div>

			<!-- Status / result panel -->
			<div class="mt-4 rounded-2xl border border-white/10 bg-slate-950/50 p-4 text-sm text-slate-300">
				<p>{status}</p>
				{#if error}
					<p class="mt-2 text-rose-300">{error}</p>
				{/if}
				{#if lastResult}
					<div class="mt-3 space-y-1 text-xs text-slate-500">
						<p>Source: {lastResult.normalizedUrl}</p>
						{#if lastResult.workspacePath}
							<p>Workspace: {lastResult.workspacePath}</p>
						{/if}
						<p>State: {lastResult.statePath}</p>
						{#if lastResult.notebookPath}
							<p class="text-slate-300">
								Notebook: <span class="font-mono">{lastResult.notebookPath}</span>
							</p>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Colab actions panel (shown after Colab deployment) -->
	{#if lastResult?.notebookPath}
		<div class="rounded-3xl border border-violet-400/20 bg-violet-500/8 p-6 backdrop-blur">
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 class="text-lg font-semibold text-white">Colab notebook ready</h2>
					<p class="mt-2 text-sm text-slate-400">
						Your notebook has been generated. Upload it to Google Drive and open it in Colab, or use
						the buttons below.
					</p>
					<p class="mt-3 rounded-xl bg-slate-900/60 px-3 py-2 font-mono text-xs text-slate-300">
						{lastResult.notebookPath}
					</p>
				</div>
				{#if colabLaunched}
					<span
						class="shrink-0 rounded-full border border-green-400/30 bg-green-500/10 px-3 py-1 text-xs text-green-200"
					>
						Colab opened ✓
					</span>
				{/if}
			</div>

			<div class="mt-5 flex flex-wrap gap-3">
				<button
					type="button"
					on:click={openColab}
					disabled={colabLaunching}
					class="rounded-full border border-violet-400/30 bg-violet-500/15 px-5 py-3 text-sm font-semibold text-violet-100 disabled:opacity-60"
				>
					{colabLaunching ? 'Opening...' : 'Open Colab in browser'}
				</button>
				<button
					type="button"
					on:click={startColabExecution}
					disabled={startingExecution}
					class="rounded-full bg-gradient-to-r from-violet-500 to-cyan-400 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-violet-500/20 disabled:opacity-60"
				>
					{startingExecution ? 'Starting...' : 'Start Execution'}
				</button>
			</div>

			<div class="mt-4 rounded-2xl border border-amber-400/20 bg-amber-500/8 p-4 text-xs text-amber-200">
				<p class="font-medium">Before running the notebook:</p>
				<ul class="mt-2 space-y-1 text-amber-300/80">
					<li>• Upload the generated <code>.ipynb</code> file to Google Drive</li>
					<li>• Open it in Colab and fill in your API keys in Cell 3</li>
					<li>• Colab Pro / compute credits are required for GPU-accelerated runs</li>
					<li>• A quota warning will appear in the notebook output if credits are insufficient</li>
				</ul>
			</div>
		</div>
	{/if}

	<!-- Deploy target cards -->
	<div class="grid gap-4 lg:grid-cols-2">
		{#each deployTargets as target}
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">{target.title}</h2>
				<p class="mt-3 text-sm leading-6 text-slate-400">{target.description}</p>
			</div>
		{/each}
	</div>
</section>
