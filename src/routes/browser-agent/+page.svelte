<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type BrowserSession = {
		id: string;
		url: string;
		status: string;
		backend: string;
		createdAt: string;
	};

	let sessions: BrowserSession[] = [];
	let loading = true;
	let busy = false;
	let message = '';
	let launchUrl = 'https://google.com';

	onMount(async () => {
		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}
		await refresh();
	});

	async function refresh() {
		loading = true;
		try {
			sessions = await invokeTauri<BrowserSession[]>('get_browser_sessions');
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to load sessions.';
		} finally {
			loading = false;
		}
	}

	async function connectMcp() {
		busy = true;
		message = 'Connecting Browser via MCP...';
		try {
			const session = await invokeTauri<BrowserSession>('connect_browser_mcp');
			message = `Connected: ${session.backend} at ${session.url}`;
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to connect MCP.';
		} finally {
			busy = false;
		}
	}

	async function ensurePlaywright() {
		busy = true;
		message = 'Installing Playwright MCP...';
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('ensure_playwright');
			message = result.message;
		} catch (e) {
			message = e instanceof Error ? e.message : 'Installation failed.';
		} finally {
			busy = false;
		}
	}

	async function launchBrowser() {
		busy = true;
		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('launch_browser_with_profile', {
				url: launchUrl,
				profileDir: null
			});
			message = result.message;
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to launch browser.';
		} finally {
			busy = false;
		}
	}

	async function disconnectSession(id: string) {
		try {
			await invokeTauri('disconnect_browser_mcp', { sessionId: id });
			await refresh();
		} catch (e) {
			message = e instanceof Error ? e.message : 'Failed to disconnect.';
		}
	}

	function statusColor(s: string) {
		switch (s) {
			case 'connected': return 'bg-green-400/15 text-green-300';
			case 'connecting': return 'bg-amber-400/15 text-amber-300';
			default: return 'bg-slate-600/30 text-slate-400';
		}
	}
</script>

<section class="space-y-6">
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Browser Agent</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">MCP browser automation over Edge.</h1>
		<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
			Connect via Playwright MCP for full browser control: click, type, scrape, navigate. Launches
			Edge with your logged-in profile. Preferred backend for all workflows.
		</p>
	</div>

	{#if message}
		<div class="rounded-3xl border border-cyan-400/20 bg-cyan-500/10 p-4 text-sm text-cyan-100">
			{message}
		</div>
	{/if}

	<!-- Quick actions -->
	<div class="grid gap-4 xl:grid-cols-3">
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Connect via MCP</h2>
			<p class="mt-2 text-sm text-slate-400">
				Installs Playwright MCP if needed and starts the MCP server. Gives RalphHub full browser control.
			</p>
			<button
				on:click={connectMcp}
				disabled={busy}
				class="mt-4 w-full rounded-full bg-gradient-to-r from-cyan-500/20 to-violet-500/20 px-4 py-3 text-sm font-semibold text-white hover:from-cyan-500/30 hover:to-violet-500/30 disabled:opacity-60"
			>
				{busy ? 'Connecting...' : 'Connect Browser via MCP'}
			</button>
		</div>

		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Launch Edge with profile</h2>
			<p class="mt-2 text-sm text-slate-400">
				Opens Edge with your default logged-in profile so agents access your accounts.
			</p>
			<input
				bind:value={launchUrl}
				placeholder="https://..."
				class="mt-4 w-full rounded-2xl border border-white/10 bg-slate-900/80 px-4 py-3 text-sm text-white outline-none"
			/>
			<button
				on:click={launchBrowser}
				disabled={busy}
				class="mt-3 w-full rounded-full bg-cyan-400/12 px-4 py-2 text-sm font-medium text-cyan-100 hover:bg-cyan-400/20 disabled:opacity-60"
			>
				Launch Edge
			</button>
		</div>

		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Install Playwright MCP</h2>
			<p class="mt-2 text-sm text-slate-400">
				Installs @playwright/mcp globally via Bun. Required for MCP browser control.
			</p>
			<div class="mt-4 rounded-xl bg-slate-900/80 px-4 py-3">
				<code class="text-xs text-slate-400">bun add -g @playwright/mcp</code>
			</div>
			<button
				on:click={ensurePlaywright}
				disabled={busy}
				class="mt-3 w-full rounded-full bg-violet-500/15 px-4 py-2 text-sm font-medium text-violet-200 hover:bg-violet-500/25 disabled:opacity-60"
			>
				Auto-install
			</button>
		</div>
	</div>

	<!-- Sessions -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-semibold text-white">Browser sessions</h2>
			<button
				on:click={refresh}
				class="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-400 hover:text-white"
			>
				Refresh
			</button>
		</div>

		<div class="mt-6 space-y-3">
			{#if loading}
				<p class="text-sm text-slate-500">Loading...</p>
			{:else if !sessions.length}
				<p class="text-sm text-slate-500">No active sessions. Click "Connect Browser via MCP" to start.</p>
			{:else}
				{#each sessions as session}
					<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
						<div class="flex items-center justify-between gap-4">
							<div>
								<div class="flex items-center gap-2">
									<span class="rounded-full px-2 py-0.5 text-xs {statusColor(session.status)}">{session.status}</span>
									<span class="text-xs text-slate-500 font-mono">{session.backend}</span>
								</div>
								<p class="mt-1 text-sm text-white">{session.url}</p>
								<p class="mt-1 text-xs text-slate-600">{new Date(session.createdAt).toLocaleString()}</p>
							</div>
							<button
								on:click={() => disconnectSession(session.id)}
								class="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-400 hover:text-red-400"
							>
								Disconnect
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</div>

	<!-- MCP protocol info -->
	<div class="rounded-3xl border border-violet-400/20 bg-violet-500/8 p-6 backdrop-blur">
		<h2 class="text-lg font-semibold text-white">How it works</h2>
		<div class="mt-4 space-y-3 text-sm text-slate-300">
			<p>1. Playwright MCP starts a local server on port 8931.</p>
			<p>2. RalphHub connects to it via the MCP protocol.</p>
			<p>3. Any workflow can send browser commands: navigate, click, type, screenshot, extract.</p>
			<p>4. Edge launches with your real logged-in profile — agents can interact with Gmail, Notion, GitHub, etc.</p>
			<p>5. If MCP extension is not installed, falls back to Playwright directly.</p>
		</div>
	</div>
</section>
