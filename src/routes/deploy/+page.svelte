<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

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

	type EnvEntry = {
		key: string;
		value: string;
	};

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

	let repoUrl = 'https://github.com/gsd-build/get-shit-done';
	let loading = false;
	let error = '';
	let status = 'Ready to deploy.';
	let lastResult: DeployResult | null = null;

	async function deployToPc() {
		loading = true;
		error = '';
		status = 'Ensuring Bun and cloning the repository...';

		try {
			await invoke('ensure_bun');
			const result = await invoke<DeployResult>('deploy_to_pc', { request: { url: repoUrl } });
			lastResult = result;
			status = result.message;

			const keys = await loadKeys();
			const entries = buildEnvEntries(keys);
			if (entries.length && window.confirm('Inject your saved central API keys into this workspace now?')) {
				await invoke('inject_keys', {
					request: {
						workspacePath: result.workspacePath,
						entries
					}
				});
				status = 'Repository deployed, keys injected, and workspace is ready.';
			}

			await invoke('open_in_code', {
				workspacePath: result.workspacePath,
				branch: result.branch
			});
		} catch (deployError) {
			error = deployError instanceof Error ? deployError.message : 'Deploy to PC failed.';
			status = 'Deploy failed.';
		} finally {
			loading = false;
		}
	}

	async function deployToColab() {
		loading = true;
		error = '';
		status = 'Generating Colab notebook...';

		try {
			const result = await invoke<DeployResult>('deploy_to_colab', { request: { url: repoUrl } });
			lastResult = result;
			status = result.notebookPath
				? `Colab notebook generated at ${result.notebookPath}`
				: result.message;
		} catch (deployError) {
			error = deployError instanceof Error ? deployError.message : 'Deploy to Colab failed.';
			status = 'Colab generation failed.';
		} finally {
			loading = false;
		}
	}

	function buildEnvEntries(keys: KeyMap): EnvEntry[] {
		return KEY_FIELDS.filter((field) => keys[field].trim()).map((field) => ({
			key: field,
			value: keys[field].trim()
		}));
	}
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
				bind:value={repoUrl}
				class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-4 text-sm text-white outline-none ring-0"
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
					on:click={deployToColab}
					disabled={loading}
					class="rounded-full border border-white/12 bg-white/5 px-6 py-3 text-sm font-semibold text-white"
					type="button"
				>
					Deploy to Colab
				</button>
				<a
					href="/settings"
					class="rounded-full border border-cyan-400/30 bg-cyan-400/10 px-6 py-3 text-sm font-semibold text-cyan-100"
				>
					Configure keys
				</a>
			</div>

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
							<p>Notebook: {lastResult.notebookPath}</p>
						{/if}
					</div>
				{/if}
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
