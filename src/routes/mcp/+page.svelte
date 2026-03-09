<script lang="ts">
	import { onMount } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type McpServer = {
		id: string;
		name: string;
		command: string;
		description: string;
		status: 'stopped' | 'running' | 'error';
		port?: number;
		requiredKeys: string[];
		icon: string;
	};

	const MCP_SERVERS: McpServer[] = [
		{
			id: 'playwright',
			name: 'Playwright MCP',
			command: 'npx @playwright/mcp@latest',
			description: 'Browser automation MCP. Control any webpage, fill forms, capture screenshots with AI.',
			status: 'stopped',
			port: 3000,
			requiredKeys: [],
			icon: '🌐'
		},
		{
			id: 'filesystem',
			name: 'Filesystem MCP',
			command: 'npx @modelcontextprotocol/server-filesystem@latest',
			description: 'Read and write files safely with full AI agent control.',
			status: 'stopped',
			port: 3001,
			requiredKeys: [],
			icon: '📁'
		},
		{
			id: 'github',
			name: 'GitHub MCP',
			command: 'npx @modelcontextprotocol/server-github@latest',
			description: 'Manage repos, PRs, issues, and code reviews with AI.',
			status: 'stopped',
			port: 3002,
			requiredKeys: ['GITHUB_TOKEN'],
			icon: '🐙'
		},
		{
			id: 'brave-search',
			name: 'Brave Search MCP',
			command: 'npx @modelcontextprotocol/server-brave-search@latest',
			description: 'Real-time privacy-first web search for AI agents.',
			status: 'stopped',
			port: 3003,
			requiredKeys: ['BRAVE_API_KEY'],
			icon: '🔍'
		},
		{
			id: 'sequential-thinking',
			name: 'Sequential Thinking',
			command: 'npx @modelcontextprotocol/server-sequential-thinking@latest',
			description: 'Structured chain-of-thought reasoning for complex problems.',
			status: 'stopped',
			port: 3004,
			requiredKeys: [],
			icon: '🧩'
		},
		{
			id: 'sqlite',
			name: 'SQLite MCP',
			command: 'npx @modelcontextprotocol/server-sqlite@latest',
			description: 'Query and manipulate SQLite databases with AI agents.',
			status: 'stopped',
			port: 3005,
			requiredKeys: [],
			icon: '🗄️'
		},
		{
			id: 'firecrawl',
			name: 'Firecrawl MCP',
			command: 'npx firecrawl-mcp@latest',
			description: 'Turn any website into LLM-ready markdown. Scrape, crawl, extract structured data.',
			status: 'stopped',
			port: 3006,
			requiredKeys: ['FIRECRAWL_API_KEY'],
			icon: '🔥'
		},
		{
			id: 'memory',
			name: 'Memory MCP',
			command: 'npx @modelcontextprotocol/server-memory@latest',
			description: 'Persistent knowledge graph memory for AI agents.',
			status: 'stopped',
			port: 3007,
			requiredKeys: [],
			icon: '🧠'
		},
	];

	let servers = MCP_SERVERS.map((s) => ({ ...s }));
	let selectedServer: (typeof servers)[0] | null = null;
	let mcpConfig = '';
	let copyStatus = '';

	function toggleServer(id: string) {
		servers = servers.map((s) => {
			if (s.id === id) {
				return { ...s, status: s.status === 'running' ? 'stopped' : 'running' };
			}
			return s;
		});
		updateConfig();
	}

	function updateConfig() {
		const running = servers.filter((s) => s.status === 'running');
		const config = {
			mcpServers: Object.fromEntries(
				running.map((s) => [
					s.id,
					{
						command: s.command.split(' ')[0],
						args: s.command.split(' ').slice(1),
						env: Object.fromEntries(s.requiredKeys.map((k) => [k, `\${${k}}`]))
					}
				])
			)
		};
		mcpConfig = JSON.stringify(config, null, 2);
	}

	async function copyConfig() {
		await navigator.clipboard.writeText(mcpConfig);
		copyStatus = 'Copied!';
		setTimeout(() => (copyStatus = ''), 2000);
	}

	onMount(() => {
		updateConfig();
	});

	$: runningCount = servers.filter((s) => s.status === 'running').length;
</script>

<section class="space-y-5">
	<!-- Header -->
	<div class="rounded-2xl border border-orange-400/20 bg-gradient-to-br from-orange-950/40 via-slate-950/80 to-amber-950/30 p-7 backdrop-blur">
		<p class="text-xs uppercase tracking-[0.3em] text-orange-300/70">Model Context Protocol</p>
		<h1 class="mt-2 text-3xl font-bold text-white">⚡ MCP Browser</h1>
		<p class="mt-2 text-sm text-slate-400">
			Toggle MCP servers on/off. Generate a ready-to-paste Claude Desktop config. Playwright, Firecrawl, GitHub, and 5 more.
		</p>
		<div class="mt-4 flex items-center gap-3">
			<div class="flex items-center gap-2 rounded-xl border border-emerald-400/20 bg-emerald-400/10 px-3 py-2">
				<span class="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
				<span class="text-sm font-semibold text-emerald-300">{runningCount} servers active</span>
			</div>
		</div>
	</div>

	<div class="grid gap-5 lg:grid-cols-[1.2fr_1fr]">
		<!-- Server grid -->
		<div class="space-y-3">
			<h2 class="text-sm font-bold text-slate-300">Available MCP Servers</h2>
			<div class="grid gap-2.5 sm:grid-cols-2">
				{#each servers as server}
					<div class={`group rounded-xl border p-4 transition cursor-pointer ${server.status === 'running' ? 'border-emerald-400/25 bg-emerald-950/20' : 'border-white/8 bg-slate-950/40 hover:border-white/20'}`}>
						<div class="flex items-start justify-between gap-2">
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-2">
									<span class="text-base">{server.icon}</span>
									<p class="text-sm font-semibold text-white truncate">{server.name}</p>
								</div>
								<p class="mt-1.5 text-xs text-slate-400 leading-5 line-clamp-2">{server.description}</p>
								{#if server.requiredKeys.length > 0}
									<p class="mt-1.5 text-[10px] text-slate-600">Needs: {server.requiredKeys.join(', ')}</p>
								{/if}
								<p class="mt-1.5 text-[10px] font-mono text-slate-600 truncate">{server.command}</p>
							</div>

							<!-- Toggle -->
							<button
								onclick={() => toggleServer(server.id)}
								class={`shrink-0 mt-0.5 rounded-full transition ${server.status === 'running' ? 'text-emerald-400 hover:text-emerald-300' : 'text-slate-600 hover:text-slate-400'}`}
								title={server.status === 'running' ? 'Click to disable' : 'Click to enable'}
							>
								<div class={`h-9 w-9 rounded-full border-2 flex items-center justify-center text-sm transition ${server.status === 'running' ? 'border-emerald-400 bg-emerald-400/15' : 'border-slate-700 bg-white/5'}`}>
									{server.status === 'running' ? '✓' : '+'}
								</div>
							</button>
						</div>
					</div>
				{/each}
			</div>
		</div>

		<!-- Config output -->
		<div class="rounded-2xl border border-white/8 bg-slate-950/50 p-5 backdrop-blur">
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-sm font-bold text-white">Claude Desktop Config</h2>
				<button
					onclick={copyConfig}
					disabled={!mcpConfig || runningCount === 0}
					class="rounded-xl border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-white/10 disabled:opacity-40"
				>
					{copyStatus || '📋 Copy'}
				</button>
			</div>

			{#if runningCount === 0}
				<div class="rounded-xl border border-dashed border-white/10 p-6 text-center">
					<p class="text-sm text-slate-500">Toggle servers on the left to generate config.</p>
				</div>
			{:else}
				<pre class="overflow-x-auto rounded-xl border border-white/8 bg-slate-900 p-4 text-xs text-slate-300 max-h-80 overflow-y-auto leading-5">{mcpConfig}</pre>
			{/if}

			<div class="mt-4 space-y-2 text-xs text-slate-500">
				<p class="font-semibold text-slate-400">How to install:</p>
				<p>1. Copy the config above</p>
				<p>2. Open <span class="font-mono bg-white/8 px-1 rounded">~/.config/claude/claude_desktop_config.json</span></p>
				<p>3. Paste under the <span class="font-mono bg-white/8 px-1 rounded">"mcpServers"</span> key</p>
				<p>4. Restart Claude Desktop</p>
			</div>

			<div class="mt-4 rounded-xl border border-orange-400/15 bg-orange-400/8 p-3">
				<p class="text-xs font-semibold text-orange-300 mb-1">Playwright MCP Browser Control</p>
				<p class="text-xs text-slate-400">Enable Playwright to let AI agents control browsers: navigate pages, fill forms, click buttons, capture screenshots — all from chat.</p>
			</div>
		</div>
	</div>
</section>
