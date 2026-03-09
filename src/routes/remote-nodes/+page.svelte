<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
	import { createKaizenTask } from '$lib/utils/provider-registry';

	// ─── Types ────────────────────────────────────────────────────────────────

	type RemoteNode = {
		id: string;
		nodeName: string;
		nodeType: string;
		host: string;
		port: number;
		username: string;
		status: string;
		lastPing: string | null;
		agentVersion: string;
		createdAt: string;
	};

	// ─── State ────────────────────────────────────────────────────────────────

	let nodes: RemoteNode[] = [];
	let loading = false;
	let deploying = false;
	let deployResult = '';
	let deployError = false;

	// Form
	let nodeType = 'vps';
	let nodeName = '';
	let host = '';
	let port = 22;
	let username = 'ubuntu';
	let sshKeyPath = '';

	// ─── Actions ──────────────────────────────────────────────────────────────

	async function loadNodes() {
		if (!isDesktopRuntime()) return;
		loading = true;
		try {
			nodes = await invokeTauri<RemoteNode[]>('list_remote_nodes');
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
		}
	}

	async function deployNode() {
		if (!nodeName.trim() || !host.trim()) return;
		deploying = true;
		deployResult = '';
		deployError = false;

		try {
			const result = await invokeTauri<{ ok: boolean; message: string }>('deploy_remote_node', {
				req: {
					nodeType,
					host,
					port,
					username,
					sshKeyPath: sshKeyPath || null,
					nodeName
				}
			});
			deployResult = result.message;
			deployError = !result.ok;
			await loadNodes();

			if (result.ok) {
				await createKaizenTask({
					title: `Remote node deployed: ${nodeName}`,
					description: `${nodeType.toUpperCase()} node "${nodeName}" at ${host}:${port} deployment initiated.`,
					priority: 'normal',
					source: 'remote-nodes',
					providerId: 'remote-node',
					usageLogId: ''
				});
			}
		} catch (e) {
			deployResult = String(e);
			deployError = true;
		} finally {
			deploying = false;
		}
	}

	function nodeTypeIcon(type: string) {
		return type === 'rpi' ? '🍓' : type === 'vps' ? '☁️' : '🖥️';
	}

	function statusColor(status: string) {
		return status === 'online'
			? 'text-green-400 bg-green-400/10 border-green-400/30'
			: status === 'deploying'
				? 'text-amber-400 bg-amber-400/10 border-amber-400/30'
				: 'text-slate-400 bg-slate-400/10 border-slate-400/20';
	}

	onMount(loadNodes);
</script>

<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 shadow-2xl shadow-cyan-950/20 backdrop-blur">
		<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Remote Nodes</p>
		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
			VPS + Raspberry Pi
		</h1>
		<p class="mt-4 text-base leading-7 text-slate-300 max-w-2xl">
			One-click deploy of headless AmitOS agent nodes to any VPS or Raspberry Pi. Full sync with Memory Spine and parallel task queue. Control from phone or desktop.
		</p>
	</div>

	<div class="grid gap-6 lg:grid-cols-[1.5fr_1fr]">
		<!-- Deploy Form -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
			<h2 class="text-lg font-semibold text-white mb-1">Deploy New Node</h2>
			<p class="text-xs text-slate-500 mb-5">
				One-click deployment — generates a bash script that installs Python, Bun, and the WebSocket agent daemon on any SSH-accessible server.
			</p>

			<div class="space-y-4">
				<!-- Node type -->
				<div>
					<label class="block text-xs text-slate-400 mb-2">Node Type</label>
					<div class="grid grid-cols-3 gap-2">
						{#each [
							{ value: 'vps', label: 'VPS', icon: '☁️', desc: 'Cloud server' },
							{ value: 'rpi', label: 'Raspberry Pi', icon: '🍓', desc: 'ARM board' },
							{ value: 'desktop', label: 'Desktop', icon: '🖥️', desc: 'Local machine' }
						] as type}
							<button
								type="button"
								on:click={() => (nodeType = type.value)}
								class="rounded-2xl border p-3 text-center transition {nodeType === type.value
									? 'border-cyan-400/40 bg-cyan-500/10 text-white'
									: 'border-white/8 bg-white/3 text-slate-400 hover:border-white/15'}"
							>
								<div class="text-2xl">{type.icon}</div>
								<div class="text-xs font-medium mt-1">{type.label}</div>
								<div class="text-[10px] text-slate-600">{type.desc}</div>
							</button>
						{/each}
					</div>
				</div>

				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="block text-xs text-slate-400 mb-1">Node Name</label>
						<input
							bind:value={nodeName}
							class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-400/50 focus:outline-none"
							placeholder="e.g. my-vps-1"
						/>
					</div>
					<div>
						<label class="block text-xs text-slate-400 mb-1">Username</label>
						<input
							bind:value={username}
							class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-400/50 focus:outline-none"
							placeholder="ubuntu"
						/>
					</div>
				</div>

				<div class="grid grid-cols-[1fr_auto] gap-3">
					<div>
						<label class="block text-xs text-slate-400 mb-1">Host / IP Address</label>
						<input
							bind:value={host}
							class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-400/50 focus:outline-none font-mono"
							placeholder="192.168.1.100 or server.example.com"
						/>
					</div>
					<div>
						<label class="block text-xs text-slate-400 mb-1">SSH Port</label>
						<input
							bind:value={port}
							type="number"
							min={1}
							max={65535}
							class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white focus:border-cyan-400/50 focus:outline-none font-mono"
						/>
					</div>
				</div>

				<div>
					<label class="block text-xs text-slate-400 mb-1">SSH Key Path (optional)</label>
					<input
						bind:value={sshKeyPath}
						class="w-full rounded-xl border border-white/10 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-400/50 focus:outline-none font-mono"
						placeholder="~/.ssh/id_rsa"
					/>
				</div>

				<!-- What gets installed -->
				<div class="rounded-2xl border border-cyan-400/20 bg-cyan-500/5 p-4">
					<p class="text-xs font-semibold text-cyan-300 mb-2">What gets deployed:</p>
					<ul class="text-xs text-slate-400 space-y-1">
						<li>• Python 3 + virtual environment with pyautogui, PIL, anthropic</li>
						<li>• Bun runtime for TypeScript agents</li>
						<li>• RalphHub WebSocket agent daemon (port 7788)</li>
						<li>• systemd service (auto-start on boot)</li>
						{#if nodeType === 'rpi'}
							<li>• ARM-optimized builds (aarch64)</li>
							<li>• GPIO + camera access for physical control</li>
						{/if}
					</ul>
				</div>

				<button
					type="button"
					on:click={deployNode}
					disabled={deploying || !nodeName.trim() || !host.trim()}
					class="w-full rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 px-5 py-3 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/30 disabled:opacity-50"
				>
					{deploying ? '⟳ Deploying...' : `🚀 Deploy ${nodeType.toUpperCase()} Node`}
				</button>
			</div>

			{#if deployResult}
				<div class="mt-4 rounded-2xl border {deployError ? 'border-rose-400/30 bg-rose-500/10' : 'border-green-400/20 bg-green-500/5'} p-4">
					<p class="text-xs {deployError ? 'text-rose-400' : 'text-green-300'} font-mono whitespace-pre-wrap break-words">
						{deployResult}
					</p>
				</div>
			{/if}
		</div>

		<!-- Nodes List + Info -->
		<div class="space-y-4">
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold text-white">Deployed Nodes</h2>
					<button
						type="button"
						on:click={loadNodes}
						disabled={loading}
						class="text-sm text-cyan-400 hover:text-cyan-300"
					>
						{loading ? '⟳' : '↻ Refresh'}
					</button>
				</div>

				{#if nodes.length === 0}
					<p class="text-sm text-slate-500">No nodes deployed yet. Use the form to add your first VPS or RPi.</p>
				{:else}
					<div class="space-y-3">
						{#each nodes as node}
							<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
								<div class="flex items-start justify-between gap-2">
									<div>
										<div class="flex items-center gap-2 mb-1">
											<span class="text-lg">{nodeTypeIcon(node.nodeType)}</span>
											<p class="text-sm font-medium text-white">{node.nodeName}</p>
											<span class="rounded-full border px-2 py-0.5 text-xs font-medium {statusColor(node.status)}">
												{node.status}
											</span>
										</div>
										<p class="text-xs text-slate-500 font-mono">{node.username}@{node.host}:{node.port}</p>
										{#if node.lastPing}
											<p class="text-xs text-slate-600 mt-0.5">Last ping: {new Date(node.lastPing).toLocaleString()}</p>
										{/if}
									</div>
									<div class="text-xs text-slate-600">v{node.agentVersion}</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Quick Deploy Presets -->
			<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-5 backdrop-blur">
				<h2 class="text-sm font-semibold text-white mb-3">Quick Presets</h2>
				<div class="space-y-2">
					{#each [
						{ name: 'Digital Ocean Droplet', type: 'vps', user: 'root', desc: 'Ubuntu 22.04 LTS' },
						{ name: 'AWS EC2 t2.micro', type: 'vps', user: 'ubuntu', desc: 'Amazon Linux / Ubuntu' },
						{ name: 'Google Cloud Run', type: 'vps', user: 'root', desc: 'GCE VM instance' },
						{ name: 'Raspberry Pi 4', type: 'rpi', user: 'pi', desc: 'Raspberry Pi OS (64-bit)' },
						{ name: 'Raspberry Pi Zero 2W', type: 'rpi', user: 'pi', desc: 'Headless minimal' }
					] as preset}
						<button
							type="button"
							on:click={() => {
								nodeType = preset.type;
								nodeName = preset.name.toLowerCase().replace(/\s+/g, '-');
								username = preset.user;
							}}
							class="w-full rounded-xl border border-white/8 bg-white/2 p-3 text-left hover:border-white/15 transition"
						>
							<div class="flex items-center gap-2">
								<span>{nodeTypeIcon(preset.type)}</span>
								<div>
									<p class="text-xs font-medium text-white">{preset.name}</p>
									<p class="text-[10px] text-slate-600">{preset.desc} · user: {preset.user}</p>
								</div>
							</div>
						</button>
					{/each}
				</div>
			</div>
		</div>
	</div>

	<!-- Deployment Architecture -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-sm font-semibold text-white mb-4">Deployment Architecture</h2>
		<div class="grid gap-4 sm:grid-cols-3">
			{#each [
				{
					title: 'Desktop (Vy Mode)',
					icon: '🖥️',
					points: ['Vision-based GUI control', 'Mouse + keyboard automation', 'Screenshot + analyze any app', 'Excel, browser, IDE support']
				},
				{
					title: 'VPS Node',
					icon: '☁️',
					points: ['24/7 background loops', 'Batch data processing', 'API integrations', 'WebSocket sync to desktop']
				},
				{
					title: 'Raspberry Pi Node',
					icon: '🍓',
					points: ['Physical device control', 'Home automation', 'ARM-optimized binaries', 'GPIO + camera access']
				}
			] as arch}
				<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
					<div class="flex items-center gap-2 mb-3">
						<span class="text-2xl">{arch.icon}</span>
						<h3 class="text-sm font-semibold text-white">{arch.title}</h3>
					</div>
					<ul class="text-xs text-slate-400 space-y-1">
						{#each arch.points as point}
							<li>• {point}</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>

		<div class="mt-4 rounded-2xl border border-violet-400/20 bg-violet-500/5 p-4">
			<p class="text-xs font-semibold text-violet-300 mb-2">Sync Architecture</p>
			<p class="text-xs text-slate-400">
				All nodes connect to the central RalphHub via WebSocket (port 7788). Actions, permission requests, and task results are synced in real-time to the Memory Spine and Kaizen task queue. Approve any remote action from your phone via the Voice + Chat interface.
			</p>
		</div>
	</div>
</section>
