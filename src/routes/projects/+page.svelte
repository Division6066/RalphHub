<script lang="ts">
	import { onMount } from 'svelte';
	import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

	interface KaizenProject {
		id: string;
		title: string;
		domain: string;
		description: string;
		status: string;
		taskCount: number;
		createdAt: string;
		updatedAt: string;
	}

	let projects = $state<KaizenProject[]>([]);
	let loading = $state(true);
	let error = $state('');
	let creating = $state(false);
	let msg = $state('');
	let showForm = $state(false);

	let newTitle = $state('');
	let newDomain = $state('work');
	let newDescription = $state('');

	const domainEmoji: Record<string, string> = {
		work: '💼', health: '🏃', learning: '📚', personal: '🏠', system: '⚙️',
	};

	const statusColor: Record<string, string> = {
		active: 'bg-emerald-500/20 text-emerald-300',
		paused: 'bg-amber-500/20 text-amber-300',
		complete: 'bg-slate-500/20 text-slate-300',
		archived: 'bg-slate-700/20 text-slate-500',
	};

	async function load() {
		loading = true;
		error = '';
		try {
			if (!isDesktopRuntime()) {
				projects = [
					{ id: '1', title: 'AmitOS Build', domain: 'work', description: 'Build the personal OS layer on top of RalphHub', status: 'active', taskCount: 8, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
					{ id: '2', title: 'Health Protocol Q1', domain: 'health', description: '90-day movement + recovery protocol', status: 'active', taskCount: 3, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
				];
				loading = false;
				return;
			}
			projects = await invokeTauri<KaizenProject[]>('list_kaizen_projects');
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function createProject() {
		if (!newTitle.trim()) { msg = 'Title required.'; return; }
		creating = true;
		msg = '';
		try {
			if (isDesktopRuntime()) {
				await invokeTauri('create_kaizen_project', {
					title: newTitle, domain: newDomain, description: newDescription,
				});
			}
			msg = '✓ Project created';
			newTitle = ''; newDescription = ''; showForm = false;
			setTimeout(() => { msg = ''; }, 2500);
			await load();
		} catch (e) {
			msg = '✗ ' + String(e);
		} finally {
			creating = false;
		}
	}

	onMount(load);
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-white">🗂️ Projects</h1>
			<p class="mt-1 text-sm text-slate-400">Projects → Milestones → Tasks. Group your work by initiative.</p>
		</div>
		<div class="flex gap-2">
			<button onclick={() => (showForm = !showForm)} class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-4 py-2 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400">+ New Project</button>
			<button onclick={load} class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300 hover:bg-white/10">↻</button>
		</div>
	</div>

	{#if error}
		<div class="rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">{error}</div>
	{/if}

	{#if showForm}
		<div class="rounded-2xl border border-white/8 bg-slate-900/60 p-5">
			<h2 class="mb-4 text-base font-bold text-white">New Project</h2>
			<div class="grid gap-3 sm:grid-cols-2">
				<div class="sm:col-span-2">
					<input bind:value={newTitle} type="text" placeholder="Project title" class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40" />
				</div>
				<div>
					<select bind:value={newDomain} class="w-full rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white outline-none focus:border-cyan-400/40">
						{#each ['work', 'health', 'learning', 'personal', 'system'] as d}
							<option value={d}>{domainEmoji[d]} {d}</option>
						{/each}
					</select>
				</div>
				<div class="sm:col-span-2">
					<textarea bind:value={newDescription} rows={2} placeholder="Description (optional)" class="w-full resize-none rounded-xl border border-white/10 bg-slate-800/60 px-4 py-3 text-sm text-white placeholder-slate-600 outline-none focus:border-cyan-400/40"></textarea>
				</div>
			</div>
			<div class="mt-3 flex items-center gap-3">
				<button onclick={createProject} disabled={creating || !newTitle.trim()} class="rounded-xl bg-gradient-to-r from-cyan-500 to-cyan-600 px-5 py-2.5 text-sm font-bold text-white shadow-lg shadow-cyan-500/20 hover:from-cyan-400 disabled:opacity-40">
					{creating ? 'Creating…' : '🗂️ Create'}
				</button>
				<button onclick={() => (showForm = false)} class="rounded-xl border border-white/10 px-4 py-2.5 text-sm text-slate-400 hover:text-white">Cancel</button>
				{#if msg}<p class="text-sm {msg.startsWith('✓') ? 'text-emerald-400' : 'text-red-400'}">{msg}</p>{/if}
			</div>
		</div>
	{/if}

	{#if loading}
		<div class="py-12 text-center text-slate-500">Loading projects…</div>
	{:else if projects.length === 0}
		<div class="rounded-2xl border border-dashed border-white/10 py-16 text-center">
			<p class="text-4xl">🗂️</p>
			<p class="mt-3 text-slate-400">No projects yet.</p>
			<button onclick={() => (showForm = true)} class="mt-4 rounded-xl bg-cyan-500/20 px-5 py-2.5 text-sm font-medium text-cyan-300 hover:bg-cyan-500/30">+ Create first project</button>
		</div>
	{:else}
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each projects as project}
				<div class="group rounded-2xl border border-white/8 bg-slate-900/50 p-5 transition hover:border-cyan-400/20 hover:bg-slate-800/60">
					<div class="flex items-start justify-between gap-2">
						<div class="flex items-center gap-2">
							<span class="text-xl">{domainEmoji[project.domain] ?? '📌'}</span>
							<h3 class="font-bold text-white">{project.title}</h3>
						</div>
						<span class="shrink-0 rounded-full px-2.5 py-1 text-[10px] font-medium {statusColor[project.status] ?? ''}">{project.status}</span>
					</div>
					{#if project.description}
						<p class="mt-2 text-sm text-slate-400 line-clamp-2">{project.description}</p>
					{/if}
					<div class="mt-4 flex items-center justify-between">
						<span class="text-sm font-medium text-slate-500">{project.taskCount} active tasks</span>
						<a href="/tasks" class="rounded-xl border border-cyan-400/20 bg-cyan-400/8 px-3 py-1.5 text-xs font-medium text-cyan-300 opacity-0 transition hover:bg-cyan-400/15 group-hover:opacity-100">View Tasks →</a>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
