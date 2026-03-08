<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type ToolManifest = {
		id: string;
		name: string;
		repoUrl: string;
		description: string;
		status: string;
		openInCode: boolean;
		requiredKeys: string[];
		categories: string[];
		supportsOllama: boolean;
		supportsVoice: boolean;
		installMethod: string;
	};

	type ConnectResult = {
		toolId: string;
		ollamaOk: boolean;
		voiceOk: boolean;
		mcpOk: boolean;
		cursorOk: boolean;
		notes: string[];
	};

	type DeployResult = {
		workspacePath: string;
		branch: string;
		message: string;
	};

	let tools: ToolManifest[] = [];
	let loading = true;
	let busyTool = '';
	let testingTool = '';
	let status = '';
	let connectResults: Record<string, ConnectResult> = {};
	let searchQuery = '';
	let categoryFilter = '';
	let expandedTool = '';

	$: filteredTools = tools.filter((t) => {
		const matchesSearch =
			!searchQuery ||
			t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			t.description.toLowerCase().includes(searchQuery.toLowerCase());
		const matchesCategory =
			!categoryFilter || t.categories.includes(categoryFilter);
		return matchesSearch && matchesCategory;
	});

	$: allCategories = [...new Set(tools.flatMap((t) => t.categories))].sort();

	onMount(async () => {
		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}

		tools = await invokeTauri<ToolManifest[]>('list_builtin_tools');
		loading = false;
	});

	async function launchTool(tool: ToolManifest) {
		if (tool.repoUrl.startsWith('internal://')) {
			status = `${tool.name} is a built-in RalphHub capability.`;
			return;
		}

		busyTool = tool.id;
		status = `Deploying ${tool.name}...`;

		try {
			await invokeTauri('ensure_bun');
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
			status = result.message;
			try {
				await invokeTauri('open_in_code', {
					workspacePath: result.workspacePath,
					branch: result.branch
				});
				status += ' Opened in editor.';
			} catch {}
		} catch (err) {
			status = err instanceof Error ? err.message : `Failed to launch ${tool.name}.`;
		} finally {
			busyTool = '';
		}
	}

	async function connectAndTest(tool: ToolManifest) {
		testingTool = tool.id;
		try {
			const result = await invokeTauri<ConnectResult>('connect_and_test_tool', {
				toolId: tool.id
			});
			connectResults = { ...connectResults, [tool.id]: result };
		} catch (e) {
			status = e instanceof Error ? e.message : 'Test failed.';
		} finally {
			testingTool = '';
		}
	}

	async function launchInCursorDesktop(tool: ToolManifest) {
		try {
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
			await invokeTauri('launch_in_cursor_desktop', { workspacePath: result.workspacePath });
			status = `Opened ${tool.name} in Cursor Desktop.`;
		} catch (e) {
			status = e instanceof Error ? e.message : 'Failed to open in Cursor.';
		}
	}

	async function launchInCursorAgentWeb(tool: ToolManifest) {
		try {
			const result = await invokeTauri<DeployResult>('deploy_to_pc', { request: { url: tool.repoUrl } });
			await invokeTauri('launch_in_cursor_agent_web', { workspacePath: result.workspacePath });
			status = `Opened ${tool.name} in Cursor Agent Web.`;
		} catch (e) {
			status = e instanceof Error ? e.message : 'Failed to open in Cursor Agent Web.';
		}
	}

	async function launchCodex(tool: ToolManifest) {
		try {
			await invokeTauri('launch_google_codex', {
				prompt: `Work on the ${tool.name} project: ${tool.description}`
			});
			status = `Opened Codex for ${tool.name}.`;
		} catch (e) {
			status = e instanceof Error ? e.message : 'Failed to open Codex.';
		}
	}

	function methodBadge(method: string) {
		switch (method) {
			case 'bun': return 'bg-amber-400/15 text-amber-300';
			case 'pip': return 'bg-blue-400/15 text-blue-300';
			case 'cargo': return 'bg-orange-400/15 text-orange-300';
			case 'internal': return 'bg-slate-600/30 text-slate-400';
			default: return 'bg-slate-600/30 text-slate-400';
		}
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Tools</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">All wishlist repos — one-click launch.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Every tool auto-installs (Bun or pip), injects API keys from Stronghold, routes to local Ollama,
			writes reports to Memory Spine, and opens in Cursor with STATE.md ready.
		</p>
	</div>

	{#if status}
		<div class="rounded-3xl border border-cyan-400/20 bg-cyan-500/10 p-4 text-sm text-cyan-100">
			{status}
		</div>
	{/if}

	<!-- Filters -->
	<div class="flex flex-wrap gap-3">
		<input
			bind:value={searchQuery}
			placeholder="Search tools..."
			class="flex-1 min-w-48 rounded-full border border-white/10 bg-slate-950/60 px-4 py-2 text-sm text-white outline-none placeholder:text-slate-600"
		/>
		<select
			bind:value={categoryFilter}
			class="rounded-full border border-white/10 bg-slate-950/60 px-4 py-2 text-sm text-white outline-none"
		>
			<option value="">All categories</option>
			{#each allCategories as cat}
				<option value={cat}>{cat}</option>
			{/each}
		</select>
		<span class="rounded-full border border-white/10 bg-slate-950/40 px-4 py-2 text-sm text-slate-500">
			{filteredTools.length} tools
		</span>
	</div>

	{#if loading}
		<div class="rounded-3xl border border-white/10 bg-slate-950/40 p-6 text-sm text-slate-400 backdrop-blur">
			Loading tool manifests...
		</div>
	{:else}
		<div class="grid gap-4 xl:grid-cols-3">
			{#each filteredTools as tool}
				{@const testResult = connectResults[tool.id]}
				<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur flex flex-col">
					<div class="flex items-start justify-between gap-4">
						<div class="min-w-0 flex-1">
							<h2 class="text-base font-semibold text-white">{tool.name}</h2>
							<div class="mt-1.5 flex flex-wrap gap-1.5">
								<span class="rounded-full px-2 py-0.5 text-xs {methodBadge(tool.installMethod)}">{tool.installMethod}</span>
								{#each tool.categories as cat}
									<span class="rounded-full border border-white/8 px-2 py-0.5 text-xs text-slate-500">{cat}</span>
								{/each}
							</div>
						</div>
						<div class="flex flex-col items-end gap-1">
							{#if tool.supportsOllama}
								<span class="rounded-full bg-violet-400/15 px-2 py-0.5 text-xs text-violet-300">ollama</span>
							{/if}
							{#if tool.supportsVoice}
								<span class="rounded-full bg-cyan-400/15 px-2 py-0.5 text-xs text-cyan-300">voice</span>
							{/if}
						</div>
					</div>

					<p class="mt-3 text-sm leading-6 text-slate-400 flex-1">{tool.description}</p>

					{#if tool.requiredKeys.length}
						<div class="mt-3 flex flex-wrap gap-1">
							{#each tool.requiredKeys as key}
								<span class="rounded-full border border-amber-400/20 bg-amber-400/8 px-2 py-0.5 text-xs text-amber-300">{key}</span>
							{/each}
						</div>
					{/if}

					<!-- Connect & Test result -->
					{#if testResult}
						<div class="mt-3 rounded-2xl border border-white/8 bg-slate-900/50 p-3 text-xs space-y-1">
							<div class="flex gap-4 flex-wrap">
								<span class="{testResult.ollamaOk ? 'text-green-400' : 'text-slate-600'}">Ollama {testResult.ollamaOk ? '✓' : '✗'}</span>
								<span class="{testResult.voiceOk ? 'text-green-400' : 'text-slate-600'}">Voice {testResult.voiceOk ? '✓' : '✗'}</span>
								<span class="{testResult.mcpOk ? 'text-green-400' : 'text-slate-600'}">MCP {testResult.mcpOk ? '✓' : '✗'}</span>
								<span class="{testResult.cursorOk ? 'text-green-400' : 'text-slate-600'}">Cursor {testResult.cursorOk ? '✓' : '✗'}</span>
							</div>
							{#each testResult.notes as note}
								<p class="text-slate-500">{note}</p>
							{/each}
						</div>
					{/if}

					<!-- Action buttons -->
					<div class="mt-4 grid grid-cols-2 gap-2 text-xs">
						<button
							on:click={() => launchTool(tool)}
							disabled={busyTool === tool.id || tool.repoUrl.startsWith('internal://')}
							class="col-span-2 rounded-full bg-cyan-400/12 px-4 py-2.5 font-medium text-cyan-100 hover:bg-cyan-400/20 disabled:cursor-not-allowed disabled:opacity-50"
						>
							{busyTool === tool.id ? 'Deploying...' : 'Deploy + Launch'}
						</button>

						<button
							on:click={() => connectAndTest(tool)}
							disabled={testingTool === tool.id}
							class="rounded-full border border-white/10 px-3 py-2 font-medium text-white hover:bg-white/5 disabled:opacity-50"
						>
							{testingTool === tool.id ? 'Testing...' : 'Connect & Test'}
						</button>

						<button
							on:click={() => launchInCursorDesktop(tool)}
							disabled={tool.repoUrl.startsWith('internal://')}
							class="rounded-full border border-violet-400/20 bg-violet-500/8 px-3 py-2 font-medium text-violet-200 hover:bg-violet-500/15 disabled:opacity-40"
						>
							Cursor Desktop
						</button>

						<button
							on:click={() => launchInCursorAgentWeb(tool)}
							disabled={tool.repoUrl.startsWith('internal://')}
							class="rounded-full border border-white/10 px-3 py-2 font-medium text-slate-300 hover:bg-white/5 disabled:opacity-40"
						>
							Cursor Agent Web
						</button>

						<button
							on:click={() => launchCodex(tool)}
							class="rounded-full border border-amber-400/20 bg-amber-500/8 px-3 py-2 font-medium text-amber-200 hover:bg-amber-500/15"
						>
							Google Codex
						</button>
					</div>

					{#if !tool.repoUrl.startsWith('internal://')}
						<a
							href={tool.repoUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="mt-2 text-center text-xs text-slate-600 hover:text-slate-400"
						>
							{tool.repoUrl.replace('https://github.com/', 'github: ')} ↗
						</a>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</section>
