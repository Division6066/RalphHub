<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type BrowserAction = {
		id: string;
		actionType: string;
		target: string;
		url: string;
		details: string | null;
		status: string;
		screenshotPath: string | null;
		createdAt: string;
	};

	type BrowserSettings = {
		preferredBrowser: string;
		agentMode: string;
	};

	type EdgeProfileConfig = {
		detected: boolean;
		binaryPath: string | null;
		profileDir: string;
	};

	// ── state ────────────────────────────────────────────────────────────────
	let settings: BrowserSettings = { preferredBrowser: 'system', agentMode: 'permission' };
	let edgeConfig: EdgeProfileConfig = { detected: false, binaryPath: null, profileDir: '' };
	let actions: BrowserAction[] = [];
	let pendingActions: BrowserAction[] = [];

	let urlInput = 'https://google.com';
	let launchMode = 'permission';
	let launching = false;
	let launchStatus = '';

	// Permission modal state
	let modalAction: BrowserAction | null = null;
	let alwaysAllow = false;

	// Autonomous mode kill-switch
	let killSwitchActive = false;

	let pollTimer: ReturnType<typeof setInterval> | null = null;
	let loading = true;

	$: pendingActions = actions.filter((a) => a.status === 'pending');
	$: isAutonomous = settings.agentMode === 'autonomous' && !killSwitchActive;

	// Show first pending action in modal
	$: if (pendingActions.length > 0 && !modalAction) {
		modalAction = pendingActions[0];
	} else if (pendingActions.length === 0) {
		modalAction = null;
	}

	onMount(async () => {
		if (!isDesktopRuntime()) {
			loading = false;
			return;
		}
		await refresh();
		pollTimer = setInterval(refresh, 2500);
	});

	onDestroy(() => {
		if (pollTimer) clearInterval(pollTimer);
	});

	async function refresh() {
		try {
			const [s, e, a] = await Promise.all([
				invokeTauri<BrowserSettings>('get_browser_settings'),
				invokeTauri<EdgeProfileConfig>('get_edge_profile_config'),
				invokeTauri<BrowserAction[]>('list_browser_actions')
			]);
			settings = s;
			edgeConfig = e;
			actions = a;
		} catch (_) {
			// silent — poll will retry
		} finally {
			loading = false;
		}
	}

	async function launchUrl() {
		if (!isDesktopRuntime()) return;
		launching = true;
		launchStatus = '';
		try {
			const res = await invokeTauri<{ ok: boolean; message: string }>('launch_browser_with_profile', {
				url: urlInput,
				mode: launchMode
			});
			launchStatus = res.message;
			await refresh();
		} catch (err) {
			launchStatus = err instanceof Error ? err.message : 'Launch failed.';
		} finally {
			launching = false;
		}
	}

	async function approveAction(action: BrowserAction, approved: boolean) {
		if (!isDesktopRuntime()) return;
		try {
			await invokeTauri('approve_browser_action', {
				request: { actionId: action.id, approved, alwaysAllowSite: alwaysAllow }
			});
			modalAction = null;
			alwaysAllow = false;
			await refresh();
		} catch (err) {
			console.error(err);
		}
	}

	function activateKillSwitch() {
		killSwitchActive = true;
	}

	function formatTime(iso: string) {
		try {
			return new Date(iso).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' });
		} catch {
			return iso;
		}
	}

	const statusColor: Record<string, string> = {
		pending: 'text-amber-300 border-amber-400/30 bg-amber-400/10',
		approved: 'text-cyan-300 border-cyan-400/30 bg-cyan-400/10',
		executed: 'text-green-300 border-green-400/30 bg-green-400/10',
		denied: 'text-rose-300 border-rose-400/30 bg-rose-400/10',
		failed: 'text-rose-400 border-rose-400/30 bg-rose-400/10'
	};

	const actionIcon: Record<string, string> = {
		launch: '↗',
		navigate: '→',
		click: '⊙',
		type: '⌨',
		scrape: '⬇'
	};
</script>

<!-- ── Permission approval modal ──────────────────────────────────────────── -->
{#if modalAction && settings.agentMode === 'permission'}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-sm">
		<div class="w-full max-w-lg rounded-3xl border border-amber-400/30 bg-slate-950 p-8 shadow-2xl shadow-amber-900/20">
			<p class="text-xs uppercase tracking-[0.35em] text-amber-300/80">Permission required</p>
			<h2 class="mt-3 text-xl font-semibold text-white">Agent wants to perform a browser action</h2>

			<div class="mt-6 rounded-2xl border border-white/10 bg-slate-900/60 p-5 space-y-3">
				<div class="flex items-center gap-3">
					<span class="text-2xl">{actionIcon[modalAction.actionType] ?? '?'}</span>
					<div>
						<p class="text-sm font-medium text-white capitalize">{modalAction.actionType}</p>
						<p class="text-xs text-slate-400">{modalAction.target}</p>
					</div>
				</div>
				<p class="text-xs text-slate-500 break-all">{modalAction.url}</p>
				{#if modalAction.details}
					<p class="text-xs text-slate-400">Details: {modalAction.details}</p>
				{/if}
			</div>

			<label class="mt-4 flex items-center gap-3 text-sm text-slate-300">
				<input type="checkbox" bind:checked={alwaysAllow} class="rounded" />
				Always allow actions on this site
			</label>

			<div class="mt-6 flex gap-3">
				<button
					type="button"
					on:click={() => approveAction(modalAction!, true)}
					class="flex-1 rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20"
				>
					Approve
				</button>
				<button
					type="button"
					on:click={() => approveAction(modalAction!, false)}
					class="flex-1 rounded-full border border-rose-400/30 bg-rose-500/10 py-3 text-sm font-semibold text-rose-200"
				>
					Deny
				</button>
			</div>

			<p class="mt-4 text-center text-xs text-slate-500">
				{pendingActions.length} action{pendingActions.length === 1 ? '' : 's'} awaiting approval
			</p>
		</div>
	</div>
{/if}

<!-- ── Page ───────────────────────────────────────────────────────────────── -->
<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-cyan-400/20 bg-slate-950/55 p-8 shadow-2xl shadow-cyan-950/20 backdrop-blur">
		<div class="flex items-start justify-between gap-4">
			<div>
				<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Browser Agent</p>
				<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white">
					Real browser, full session.
				</h1>
				<p class="mt-4 max-w-3xl text-base leading-7 text-slate-300">
					Launch URLs in Microsoft Edge with your real profile, cookies, and extensions. Every action
					passes through the permission queue unless you enable autonomous mode.
				</p>
			</div>

			<!-- Mode badge + kill switch -->
			<div class="shrink-0 space-y-2 text-right">
				{#if isAutonomous}
					<div class="rounded-full border border-rose-400/40 bg-rose-500/15 px-4 py-2 text-sm font-semibold text-rose-200">
						⚡ Autonomous
					</div>
					<button
						type="button"
						on:click={activateKillSwitch}
						class="block w-full rounded-full border border-rose-500/50 bg-rose-600/20 px-4 py-2 text-xs font-semibold text-rose-100"
					>
						Kill switch — stop agent
					</button>
				{:else if killSwitchActive}
					<div class="rounded-full border border-slate-600 bg-slate-800 px-4 py-2 text-sm font-semibold text-slate-400">
						⏹ Agent stopped
					</div>
				{:else}
					<div class="rounded-full border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm font-medium text-cyan-100">
						🔒 Permission mode
					</div>
					{#if pendingActions.length > 0}
						<div class="rounded-full border border-amber-400/30 bg-amber-400/10 px-4 py-2 text-xs text-amber-200">
							{pendingActions.length} pending
						</div>
					{/if}
				{/if}
			</div>
		</div>
	</div>

	<!-- Autonomous mode warning banner -->
	{#if isAutonomous}
		<div class="rounded-3xl border border-rose-400/30 bg-rose-500/10 p-5 backdrop-blur">
			<p class="text-sm font-semibold text-rose-200">
				⚠ Autonomous mode active — the agent will execute browser actions without prompting you.
			</p>
			<p class="mt-2 text-xs text-rose-300/70">
				All actions are logged below for audit. Use the kill switch to halt the agent instantly.
			</p>
		</div>
	{/if}

	<div class="grid gap-4 lg:grid-cols-[1.4fr_1fr]">
		<!-- Launch panel -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white">Launch browser</h2>
			<p class="mt-2 text-sm text-slate-400">
				Open any URL in your real Edge session with all accounts and extensions intact.
			</p>

			<div class="mt-5 space-y-3">
				<input
					bind:value={urlInput}
					placeholder="https://..."
					class="w-full rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-sm text-white outline-none"
				/>

				<div class="flex gap-3">
					<label class="flex flex-1 cursor-pointer items-center gap-2 rounded-2xl border border-white/10 bg-white/3 px-4 py-3 text-sm">
						<input type="radio" bind:group={launchMode} value="permission" />
						<span class="text-slate-300">Permission</span>
					</label>
					<label class="flex flex-1 cursor-pointer items-center gap-2 rounded-2xl border border-white/10 bg-white/3 px-4 py-3 text-sm">
						<input type="radio" bind:group={launchMode} value="autonomous" />
						<span class="text-slate-300">Autonomous</span>
					</label>
				</div>

				<button
					type="button"
					on:click={launchUrl}
					disabled={launching || killSwitchActive}
					class="w-full rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20 disabled:cursor-not-allowed disabled:opacity-60"
				>
					{launching ? 'Opening...' : 'Open in Browser'}
				</button>

				{#if launchStatus}
					<p class="text-sm text-slate-300">{launchStatus}</p>
				{/if}
			</div>
		</div>

		<!-- Edge profile config -->
		<div class="space-y-4">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-lg font-semibold text-white">Edge profile</h2>
				{#if loading}
					<p class="mt-3 text-sm text-slate-500">Detecting...</p>
				{:else if edgeConfig.detected}
					<div class="mt-4 space-y-3">
						<div class="rounded-2xl border border-green-400/20 bg-green-500/8 p-3">
							<p class="text-xs font-medium text-green-300">Microsoft Edge detected</p>
							{#if edgeConfig.binaryPath}
								<p class="mt-1 break-all text-xs text-slate-500">{edgeConfig.binaryPath}</p>
							{/if}
						</div>
						{#if edgeConfig.profileDir}
							<div>
								<p class="text-xs text-slate-400">Profile directory</p>
								<p class="mt-1 break-all rounded-xl bg-slate-900/60 px-3 py-2 text-xs font-mono text-slate-300">
									{edgeConfig.profileDir}
								</p>
							</div>
						{/if}
						<p class="text-xs leading-5 text-slate-500">
							When preferred browser is set to Edge, agents launch with
							<code class="rounded bg-slate-800 px-1">--user-data-dir</code> pointing here — giving
							access to your real session, bookmarks, and extensions.
						</p>
					</div>
				{:else}
					<div class="mt-4 rounded-2xl border border-amber-400/20 bg-amber-500/8 p-4">
						<p class="text-xs text-amber-200">
							Edge not detected on this machine. URL launches will use the system default browser.
							Install Edge or change your preferred browser in Settings.
						</p>
					</div>
				{/if}
			</div>

			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<h2 class="text-sm font-semibold text-white">Playwright automation</h2>
				<p class="mt-3 text-xs leading-5 text-slate-400">
					Full automation uses a Bun subprocess running Playwright in persistent context mode,
					pointed at the Edge profile directory above. The agent can then navigate, click, type,
					and scrape any site in your real session.
				</p>
				<a
					href="/settings"
					class="mt-4 inline-block rounded-full border border-cyan-400/30 bg-cyan-400/8 px-4 py-2 text-xs font-medium text-cyan-100"
				>
					Configure browser mode →
				</a>
			</div>
		</div>
	</div>

	<!-- Pending approval queue (permission mode) -->
	{#if settings.agentMode === 'permission' && pendingActions.length > 0}
		<div class="rounded-3xl border border-amber-400/20 bg-amber-500/5 p-6 backdrop-blur">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-white">Pending approvals</h2>
				<span class="rounded-full border border-amber-400/30 bg-amber-400/10 px-3 py-1 text-xs text-amber-200">
					{pendingActions.length} waiting
				</span>
			</div>
			<div class="mt-5 space-y-3">
				{#each pendingActions as action}
					<div class="flex items-center gap-4 rounded-2xl border border-white/8 bg-white/3 p-4">
						<span class="text-2xl">{actionIcon[action.actionType] ?? '?'}</span>
						<div class="min-w-0 flex-1">
							<p class="text-sm font-medium text-white capitalize">{action.actionType}</p>
							<p class="truncate text-xs text-slate-400">{action.target}</p>
							<p class="truncate text-xs text-slate-500">{action.url}</p>
						</div>
						<div class="flex gap-2">
							<button
								type="button"
								on:click={() => approveAction(action, true)}
								class="rounded-full bg-cyan-400/15 px-3 py-1 text-xs font-medium text-cyan-100"
							>
								Approve
							</button>
							<button
								type="button"
								on:click={() => approveAction(action, false)}
								class="rounded-full bg-rose-500/10 px-3 py-1 text-xs font-medium text-rose-200"
							>
								Deny
							</button>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Audit log -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-semibold text-white">Action audit log</h2>
			<span class="text-sm text-slate-500">{actions.length} recorded</span>
		</div>

		<div class="mt-5 space-y-2">
			{#if !actions.length}
				<p class="text-sm text-slate-500">
					No browser actions yet. Launch a URL above or trigger an action from a workflow.
				</p>
			{:else}
				{#each actions.slice(0, 30) as action}
					<div class="grid grid-cols-[2rem_1fr_auto_auto] items-center gap-3 rounded-2xl border border-white/6 bg-white/2 px-4 py-3">
						<span class="text-center text-base">{actionIcon[action.actionType] ?? '?'}</span>
						<div class="min-w-0">
							<p class="truncate text-xs font-medium text-white">{action.target}</p>
							<p class="truncate text-xs text-slate-500">{action.url}</p>
						</div>
						<span class={`rounded-full border px-2 py-0.5 text-[0.65rem] font-medium ${statusColor[action.status] ?? 'text-slate-400'}`}>
							{action.status}
						</span>
						<span class="shrink-0 text-xs text-slate-600">{formatTime(action.createdAt)}</span>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
