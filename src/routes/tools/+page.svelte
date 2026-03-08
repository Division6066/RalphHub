<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type ToolManifest = {
		id: string;
		name: string;
		repoUrl: string;
		description: string;
		status: string;
		openInCode: boolean;
	};

	type DeployResult = {
		workspacePath: string;
		branch: string;
		message: string;
	};

	let tools: ToolManifest[] = [];
	let loading = true;
	let busyTool = '';
	let status = '';

	onMount(async () => {
		tools = await invoke<ToolManifest[]>('list_builtin_tools');
		loading = false;
	});

	async function launchTool(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			status = `${tool.name} is an internal RalphHub capability and does not clone a repo.`;
			return;
		}

		busyTool = tool.id;
		status = `Launching ${tool.name}...`;

		try {
			await invoke('ensure_bun');
			const result = await invoke<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
			await invoke('open_in_code', {
				workspacePath: result.workspacePath,
				branch: result.branch
			});
			status = `${tool.name} is ready and opened in code.`;
		} catch (error) {
			status = error instanceof Error ? error.message : `Failed to launch ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}
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

	<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-4 text-sm text-slate-300 backdrop-blur">
		{#if loading}
			Loading tool manifests...
		{:else}
			{status || 'Choose a tool card to clone, initialize with Bun, and open in your editor.'}
		{/if}
	</div>

	<div class="grid gap-4 xl:grid-cols-3">
		{#each tools as tool}
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-center justify-between gap-4">
					<h2 class="text-lg font-semibold text-white">{tool.name}</h2>
					<span class="rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-medium text-amber-100">
						{tool.status}
					</span>
				</div>

				<p class="mt-4 text-sm leading-6 text-slate-400">{tool.description}</p>

				<div class="mt-6 flex flex-wrap gap-2 text-sm">
					<button
						type="button"
						on:click={() => launchTool(tool)}
						disabled={busyTool === tool.id}
						class="rounded-full bg-cyan-400/12 px-4 py-2 font-medium text-cyan-100 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{busyTool === tool.id ? 'Launching...' : 'Launch'}
					</button>
					<a href="/deploy" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Logs</a>
					<a href="/workflows" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Compose</a>
					<a href="/deploy" class="rounded-full border border-white/12 px-4 py-2 font-medium text-white">Open in Code</a>
				</div>
			</div>
		{/each}
	</div>
</section>
