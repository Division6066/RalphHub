<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	type ToolProcessStatus = {
		toolId: string;
		name: string;
		status: string;
		pid?: number;
		startedAt?: string;
	};

	let runningTools: ToolProcessStatus[] = [];
	let interval: ReturnType<typeof setInterval> | null = null;

	const VY_PANDA_TOOLS = ['diffusionstudio-agent', 'superpowers'];

	onMount(() => {
		if (!isDesktopRuntime()) return;
		poll();
		interval = setInterval(poll, 5000);
	});

	onDestroy(() => {
		if (interval) clearInterval(interval);
	});

	async function poll() {
		try {
			const all = await invokeTauri<ToolProcessStatus[]>('list_running_tools');
			runningTools = all.filter((t) => VY_PANDA_TOOLS.includes(t.toolId));
		} catch { /* ignore */ }
	}

	$: isActive = runningTools.some((t) => t.status === 'running');
	$: diffusionRunning = runningTools.find((t) => t.toolId === 'diffusionstudio-agent')?.status === 'running';
	$: superpowersRunning = runningTools.find((t) => t.toolId === 'superpowers')?.status === 'running';
</script>

<div class="rounded-2xl border {isActive ? 'border-green-400/20 bg-green-400/5' : 'border-white/8 bg-white/2'} p-3 text-xs transition-colors">
	<div class="flex items-center gap-2 mb-2">
		<span class="text-base">🐼</span>
		<span class="font-semibold text-white">Vy/Panda</span>
		<span class="rounded-full px-1.5 py-0.5 text-xs {isActive ? 'bg-green-500/20 text-green-300' : 'bg-slate-700 text-slate-400'}">
			{isActive ? 'active' : 'standby'}
		</span>
	</div>
	<div class="space-y-1">
		<div class="flex items-center gap-2">
			<span class="{superpowersRunning ? 'text-violet-400' : 'text-slate-600'}">⚡</span>
			<span class="{superpowersRunning ? 'text-slate-300' : 'text-slate-600'}">Superpowers {superpowersRunning ? '— running' : '— idle'}</span>
		</div>
		<div class="flex items-center gap-2">
			<span class="{diffusionRunning ? 'text-pink-400' : 'text-slate-600'}">🎬</span>
			<span class="{diffusionRunning ? 'text-slate-300' : 'text-slate-600'}">Video agent {diffusionRunning ? '— background' : '— idle'}</span>
		</div>
	</div>
	{#if isActive}
		<a href="/parallel" class="mt-2 block text-cyan-400 hover:text-cyan-300">View parallel →</a>
	{:else}
		<a href="/parallel" class="mt-2 block text-slate-600 hover:text-slate-400">Launch parallel →</a>
	{/if}
</div>
