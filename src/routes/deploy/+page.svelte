<script lang="ts">
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import { loadKeys } from '$lib/utils/secure-store';

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

	const QUICK_REPOS = [
		{ label: '🔍 Perplexica', url: 'https://github.com/ItzCrazyKns/Perplexica' },
		{ label: '💻 OpenHands', url: 'https://github.com/All-Hands-AI/OpenHands' },
		{ label: '✏️ Aider', url: 'https://github.com/paul-gauthier/aider' },
		{ label: '🔥 Firecrawl', url: 'https://github.com/mendableai/firecrawl' },
		{ label: '🤖 Goose', url: 'https://github.com/block/goose' },
		{ label: '📊 LiteLLM', url: 'https://github.com/BerriAI/litellm' },
	];

	let repoUrl = 'https://github.com/ItzCrazyKns/Perplexica';
	let loading = false;
	let error = '';
	let status = 'Ready to deploy.';
	let lastResult: DeployResult | null = null;
	let injectKeys = true;

	async function deployToPc() {
		if (!isDesktopRuntime()) {
			status = 'Desktop runtime required to deploy tools.';
			return;
		}
		loading = true;
		error = '';
		status = 'Ensuring Bun and cloning repository…';
		try {
			await invokeTauri('ensure_bun');
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: repoUrl } });
			lastResult = result;
			status = result.message;
			if (injectKeys) {
				try {
					const keys = await loadKeys();
					const entries: EnvEntry[] = Object.entries(keys)
						.filter(([, v]) => v.trim())
						.map(([k, v]) => ({ key: k, value: v.trim() }));
					if (entries.length) {
						await invokeTauri('inject_keys', { request: { workspacePath: result.workspacePath, entries } });
						status = '✓ Deployed + keys injected. Opening in editor…';
					}
				} catch {}
			}
			await invokeTauri('open_in_code', { workspacePath: result.workspacePath, branch: result.branch });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			status = 'Deploy failed.';
		} finally {
			loading = false;
		}
	}

	async function deployToColab() {
		if (!isDesktopRuntime()) { status = 'Desktop runtime required.'; return; }
		loading = true;
		error = '';
		status = 'Generating Colab notebook…';
		try {
			const result = await invokeTauri<DeployResult>('deploy_to_colab', { request: { url: repoUrl } });
			lastResult = result;
			status = result.notebookPath ? `Notebook at: ${result.notebookPath}` : result.message;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-cyan-400/20 bg-gradient-to-br from-cyan-950/40 via-slate-950/80 to-blue-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-cyan-300/70">One-Click Deploy</p>
		<h1 class="mt-2 text-3xl font-bold text-white">🚀 Deploy</h1>
		<p class="mt-2 text-sm text-slate-400">Paste any GitHub repo. AmitOS clones it, installs with Bun, injects your API keys, and opens it in your editor.</p>
	</div>

	<!-- Quick repos -->
	<div>
		<p class="mb-2.5 text-xs font-semibold uppercase tracking-widest text-slate-500">Quick Deploy</p>
		<div class="flex flex-wrap gap-2">
			{#each QUICK_REPOS as repo}
				<button
					onclick={() => repoUrl = repo.url}
					class={`rounded-xl border px-3 py-2 text-xs font-medium transition ${repoUrl === repo.url ? 'border-cyan-400/30 bg-cyan-400/15 text-cyan-100' : 'border-white/8 bg-slate-950/50 text-slate-400 hover:text-white'}`}
				>
					{repo.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Deploy form -->
	<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-6 backdrop-blur">
		<div class="space-y-4">
			<div>
				<label class="mb-1.5 block text-xs font-medium text-slate-400">GitHub / Hugging Face URL</label>
				<input
					bind:value={repoUrl}
					type="text"
					placeholder="https://github.com/owner/repo"
					class="w-full rounded-xl border border-white/10 bg-slate-800 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400"
				/>
			</div>

			<label class="flex cursor-pointer items-center gap-3 rounded-xl border border-emerald-400/15 bg-emerald-950/15 px-4 py-3">
				<input type="checkbox" bind:checked={injectKeys} class="h-4 w-4 rounded" />
				<div>
					<p class="text-sm font-medium text-emerald-100">Auto-inject saved API keys</p>
					<p class="text-xs text-slate-400">Writes your configured keys to .env in the workspace</p>
				</div>
			</label>

			<div class="flex flex-wrap gap-3">
				<button
					onclick={deployToPc}
					disabled={loading || !repoUrl.trim()}
					class="rounded-xl bg-gradient-to-r from-cyan-400 to-violet-500 px-6 py-3 text-sm font-bold text-slate-950 shadow-lg disabled:opacity-60 transition hover:scale-105"
				>
					{loading ? '⏳ Working…' : '🚀 Deploy to PC'}
				</button>
				<button
					onclick={deployToColab}
					disabled={loading}
					class="rounded-xl border border-white/12 bg-white/5 px-6 py-3 text-sm font-semibold text-white transition hover:bg-white/10 disabled:opacity-60"
				>
					📓 Generate Colab
				</button>
				<a href="/settings" class="rounded-xl border border-cyan-400/25 bg-cyan-400/10 px-6 py-3 text-sm font-semibold text-cyan-200 transition hover:bg-cyan-400/20">
					🔑 Configure Keys
				</a>
			</div>

			<!-- Status -->
			<div class="rounded-xl border border-white/8 bg-white/3 p-4">
				<p class="text-sm text-slate-300">{status}</p>
				{#if error}
					<p class="mt-2 text-sm text-rose-300">{error}</p>
				{/if}
				{#if lastResult}
					<div class="mt-3 space-y-1 text-xs text-slate-500">
						<p>Source: {lastResult.normalizedUrl}</p>
						{#if lastResult.workspacePath}<p>Workspace: {lastResult.workspacePath}</p>{/if}
						{#if lastResult.envPath}<p>Env: {lastResult.envPath}</p>{/if}
						{#if lastResult.notebookPath}<p>Notebook: {lastResult.notebookPath}</p>{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Deploy targets info -->
	<div class="grid gap-4 sm:grid-cols-2">
		<div class="rounded-xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
			<h3 class="mb-3 text-sm font-bold text-white">🖥️ Deploy to PC</h3>
			<ul class="space-y-1.5 text-xs text-slate-400">
				<li>• Clones to <span class="font-mono">~/.amitos/repos/</span></li>
				<li>• Runs <span class="font-mono">bun install</span> automatically</li>
				<li>• Injects API keys from your vault</li>
				<li>• Opens in Cursor / VS Code with STATE.md</li>
			</ul>
		</div>
		<div class="rounded-xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
			<h3 class="mb-3 text-sm font-bold text-white">📓 Deploy to Colab</h3>
			<ul class="space-y-1.5 text-xs text-slate-400">
				<li>• Generates a ready-to-run Jupyter notebook</li>
				<li>• Bootstrap cells with Bun install + startup</li>
				<li>• Drop-in env guidance for API keys</li>
				<li>• Zero setup on the Colab side</li>
			</ul>
		</div>
	</div>
</section>
