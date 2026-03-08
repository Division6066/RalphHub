<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type ToolManifest = {
		id: string;
		name: string;
		repoUrl: string;
		description: string;
		categories: string[];
	};

	type DeployResult = {
		workspacePath: string;
		branch: string;
		message: string;
	};

	let tools: ToolManifest[] = [];
	let loading = true;
	let busy = '';
	let message = '';
	let codexPrompt = '';

	onMount(async () => {
		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}
		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		loading = false;
	});

	async function openCursorDesktop(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			message = `${tool.name} is a built-in capability — no workspace needed.`;
			return;
		}
		busy = tool.id;
		try {
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
			await invokeTauri('launch_in_cursor_desktop', { workspacePath: result.workspacePath });
			message = `Opened ${tool.name} in Cursor Desktop.`;
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed.';
		} finally {
			busy = '';
		}
	}

	async function openCursorAgentWeb(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			await invokeTauri('launch_in_cursor_agent_web', { workspacePath: '.' });
		} else {
			busy = tool.id;
			try {
				const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
				await invokeTauri('launch_in_cursor_agent_web', { workspacePath: result.workspacePath });
				message = `Opened ${tool.name} in Cursor Agent Web.`;
			} catch (e) {
				message = e instanceof Error ? e.message : 'Failed.';
			} finally {
				busy = '';
			}
		}
	}

	async function openCodex(tool: ToolManifest) {
		await invokeTauri('launch_google_codex', {
			prompt: codexPrompt || `Work on ${tool.name}: ${tool.description}`
		});
		message = `Codex opened for ${tool.name}.`;
	}

	async function launchCursorDesktopDirect() {
		await invokeTauri('launch_in_cursor_desktop', { workspacePath: '.' });
	}

	async function launchCursorAgentWebDirect() {
		await invokeTauri('launch_in_cursor_agent_web', { workspacePath: '.' });
	}

	async function launchCodexDirect() {
		await invokeTauri('launch_google_codex', { prompt: codexPrompt || null });
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Launch</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">One-click launch — Cursor, Agent Web, Codex.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Open any tool or workspace instantly in Cursor Desktop, Cursor Agent Web, or Google Codex.
			All launches include full Memory Spine context.
		</p>
	</div>

	{#if message}
		<div class="rounded-3xl border border-cyan-400/20 bg-cyan-500/10 p-4 text-sm text-cyan-100">
			{message}
		</div>
	{/if}

	<!-- Quick launch buttons -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white">Quick launch</h2>
		<p class="mt-2 text-sm text-slate-400">Launch RalphHub workspace directly in your preferred editor or AI agent.</p>

		<div class="mt-6 grid gap-4 sm:grid-cols-3">
			<button
				on:click={launchCursorDesktopDirect}
				class="rounded-3xl border border-violet-400/20 bg-violet-500/10 p-6 text-left hover:bg-violet-500/20 transition"
			>
				<p class="text-lg font-semibold text-white">Cursor Desktop</p>
				<p class="mt-2 text-sm text-slate-400">Opens in Cursor IDE with STATE.md focused and Memory Spine context.</p>
				<p class="mt-4 rounded-full bg-violet-500/20 px-4 py-2 text-sm font-medium text-violet-200 text-center">
					Open in Cursor Desktop →
				</p>
			</button>

			<button
				on:click={launchCursorAgentWebDirect}
				class="rounded-3xl border border-cyan-400/20 bg-cyan-500/10 p-6 text-left hover:bg-cyan-500/20 transition"
			>
				<p class="text-lg font-semibold text-white">Cursor Agent Web</p>
				<p class="mt-2 text-sm text-slate-400">Opens Cursor Agent Web with workspace context for autonomous coding.</p>
				<p class="mt-4 rounded-full bg-cyan-500/20 px-4 py-2 text-sm font-medium text-cyan-200 text-center">
					Open in Cursor Agent Web →
				</p>
			</button>

			<div class="rounded-3xl border border-amber-400/20 bg-amber-500/10 p-6">
				<p class="text-lg font-semibold text-white">Google Codex</p>
				<p class="mt-2 text-sm text-slate-400">Opens Google Codex with an optional pre-filled prompt.</p>
				<input
					bind:value={codexPrompt}
					placeholder="Optional Codex prompt..."
					class="mt-4 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-2 text-sm text-white outline-none placeholder:text-slate-600"
				/>
				<button
					on:click={launchCodexDirect}
					class="mt-3 w-full rounded-full bg-amber-500/20 px-4 py-2 text-sm font-medium text-amber-200 text-center hover:bg-amber-500/30"
				>
					Open Google Codex →
				</button>
			</div>
		</div>
	</div>

	<!-- Per-tool launch -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white">Launch per tool</h2>
		<p class="mt-2 text-sm text-slate-400">Deploy and open any tool workspace in your preferred environment.</p>

		<div class="mt-6 space-y-2">
			{#if loading}
				<p class="text-sm text-slate-500">Loading tools...</p>
			{:else}
				{#each tools.filter((t) => !t.repoUrl.startsWith('internal://')) as tool}
					<div class="flex items-center gap-3 rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex-1 min-w-0">
							<p class="text-sm font-medium text-white">{tool.name}</p>
							<p class="mt-0.5 text-xs text-slate-500 truncate">{tool.repoUrl.replace('https://github.com/', '')}</p>
						</div>
						<div class="flex gap-2">
							<button
								on:click={() => openCursorDesktop(tool)}
								disabled={busy === tool.id}
								class="rounded-full bg-violet-500/12 px-3 py-1.5 text-xs font-medium text-violet-200 hover:bg-violet-500/20 disabled:opacity-50"
							>
								Cursor
							</button>
							<button
								on:click={() => openCursorAgentWeb(tool)}
								disabled={busy === tool.id}
								class="rounded-full bg-cyan-400/10 px-3 py-1.5 text-xs font-medium text-cyan-200 hover:bg-cyan-400/20 disabled:opacity-50"
							>
								Agent Web
							</button>
							<button
								on:click={() => openCodex(tool)}
								class="rounded-full bg-amber-400/10 px-3 py-1.5 text-xs font-medium text-amber-200 hover:bg-amber-400/20"
							>
								Codex
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
