<script lang="ts">
	import { mobileState, submitApproval } from '$lib/mobile/store.svelte.js';

	const priorityColor: Record<string, string> = {
		urgent: 'text-red-400 bg-red-400/10 border-red-400/30',
		high: 'text-orange-400 bg-orange-400/10 border-orange-400/30',
		normal: 'text-cyan-400 bg-cyan-400/10 border-cyan-400/30',
		low: 'text-slate-400 bg-slate-400/10 border-slate-400/30',
	};

	const statusIcon: Record<string, string> = {
		running: '⟳',
		waiting_approval: '⏸',
		queued: '○',
		success: '✓',
		failed: '✗',
	};

	const statusColor: Record<string, string> = {
		running: 'text-cyan-400',
		waiting_approval: 'text-amber-400',
		queued: 'text-slate-400',
		success: 'text-emerald-400',
		failed: 'text-red-400',
	};

	let topTasks = $derived(mobileState.tasks.slice(0, 3));
	let pendingApprovals = $derived(mobileState.approvals.filter((a) => a.status === 'pending'));
	let runningAgents = $derived(mobileState.agents.filter((a) => ['running', 'waiting_approval'].includes(a.status)));
	let today = $derived(new Date().toISOString().slice(0, 10));
	let todayHabits = $derived(mobileState.habits.map((h) => ({
		...h,
		done: h.completedDates.includes(today),
	})));
</script>

<!-- Home — mobile companion dashboard -->
<div class="space-y-6 py-2">

	<!-- Greeting -->
	<div>
		<p class="text-xs uppercase tracking-widest text-cyan-300/60">Ralph Mobile</p>
		<h1 class="text-2xl font-bold text-white mt-1">
			{new Date().getHours() < 12 ? 'Good morning' : new Date().getHours() < 18 ? 'Good afternoon' : 'Good evening'}
		</h1>
		<p class="text-sm text-slate-500 mt-0.5">{new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}</p>
	</div>

	<!-- Top 3 Tasks -->
	<section>
		<div class="flex items-center justify-between mb-3">
			<h2 class="text-sm font-semibold text-slate-300 uppercase tracking-wider">Top Tasks</h2>
			<a href="/mobile/tasks" class="text-xs text-cyan-400 hover:text-cyan-300">See all →</a>
		</div>
		<div class="space-y-2">
			{#each topTasks as task}
				<div class="rounded-2xl border border-white/8 bg-slate-900/60 p-4 flex items-start gap-3">
					<div class={`mt-0.5 rounded-full border px-2 py-0.5 text-xs font-medium shrink-0 ${priorityColor[task.priority]}`}>
						{task.priority}
					</div>
					<div class="flex-1 min-w-0">
						<p class="text-sm text-white font-medium truncate">{task.title}</p>
						{#if task.dueDate}
							<p class="text-xs text-slate-500 mt-0.5">Due {task.dueDate}</p>
						{/if}
					</div>
					<span class={`text-xs shrink-0 ${task.status === 'in_progress' ? 'text-cyan-400' : 'text-slate-600'}`}>
						{task.status === 'in_progress' ? '▶' : '○'}
					</span>
				</div>
			{:else}
				<p class="text-sm text-slate-600 text-center py-4">No tasks — capture something!</p>
			{/each}
		</div>
	</section>

	<!-- Habits today -->
	<section>
		<div class="flex items-center justify-between mb-3">
			<h2 class="text-sm font-semibold text-slate-300 uppercase tracking-wider">Habits Today</h2>
			<a href="/mobile/habits" class="text-xs text-cyan-400 hover:text-cyan-300">Manage →</a>
		</div>
		<div class="grid grid-cols-3 gap-2">
			{#each todayHabits as habit}
				<div class={`rounded-2xl border p-3 text-center transition-all ${habit.done ? 'border-emerald-400/30 bg-emerald-400/8' : 'border-white/8 bg-slate-900/40'}`}>
					<div class={`text-lg mb-1 ${habit.done ? 'opacity-100' : 'opacity-30'}`}>
						{habit.done ? '✓' : '○'}
					</div>
					<p class="text-xs text-slate-300 truncate">{habit.name}</p>
					<p class="text-xs text-slate-600 mt-0.5">{habit.streak}d streak</p>
				</div>
			{/each}
		</div>
	</section>

	<!-- Pending Approvals -->
	{#if pendingApprovals.length > 0}
		<section>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-sm font-semibold text-slate-300 uppercase tracking-wider">Approvals</h2>
				<span class="text-xs bg-amber-400/15 text-amber-300 rounded-full px-2 py-0.5">{pendingApprovals.length}</span>
			</div>
			<div class="space-y-2">
				{#each pendingApprovals.slice(0, 2) as approval}
					<div class={`rounded-2xl border p-4 ${approval.priority === 'urgent' ? 'border-red-400/30 bg-red-400/5' : 'border-amber-400/20 bg-amber-400/5'}`}>
						<div class="flex items-start justify-between mb-2">
							<div>
								<p class="text-xs font-medium text-amber-300">{approval.agentName}</p>
								<p class="text-sm text-white mt-0.5">{approval.action}</p>
							</div>
							{#if approval.priority === 'urgent'}
								<span class="text-xs bg-red-400/20 text-red-300 rounded-full px-2 py-0.5 shrink-0">Urgent</span>
							{/if}
						</div>
						<p class="text-xs text-slate-500 mb-3 line-clamp-2">{approval.context}</p>
						<div class="flex gap-2">
							<button
								onclick={() => submitApproval(approval.id, 'approved')}
								class="flex-1 rounded-xl bg-emerald-500/20 border border-emerald-500/30 text-emerald-300 text-xs font-medium py-2 hover:bg-emerald-500/30 transition-colors"
							>
								Approve
							</button>
							<button
								onclick={() => submitApproval(approval.id, 'rejected')}
								class="flex-1 rounded-xl bg-red-500/20 border border-red-500/30 text-red-300 text-xs font-medium py-2 hover:bg-red-500/30 transition-colors"
							>
								Reject
							</button>
						</div>
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<!-- Running Agents -->
	{#if runningAgents.length > 0}
		<section>
			<h2 class="text-sm font-semibold text-slate-300 uppercase tracking-wider mb-3">Running Agents</h2>
			<div class="space-y-2">
				{#each runningAgents as agent}
					<div class="rounded-2xl border border-white/8 bg-slate-900/40 px-4 py-3 flex items-center gap-3">
						<span class={`text-lg ${statusColor[agent.status]}`}>{statusIcon[agent.status]}</span>
						<div class="flex-1 min-w-0">
							<p class="text-sm text-white truncate">{agent.name}</p>
							<p class="text-xs text-slate-500">
								{agent.startedAt ? `Started ${new Date(agent.startedAt).toLocaleTimeString()}` : 'Queued'}
							</p>
						</div>
						<span class={`text-xs font-medium ${statusColor[agent.status]}`}>{agent.status}</span>
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<!-- Quick capture shortcut -->
	<a
		href="/mobile/capture"
		class="block rounded-3xl border border-cyan-400/25 bg-gradient-to-r from-cyan-400/10 to-violet-400/10 p-5 text-center hover:from-cyan-400/15 hover:to-violet-400/15 transition-all"
	>
		<span class="text-2xl">＋</span>
		<p class="text-sm font-medium text-cyan-100 mt-1">Quick Capture</p>
		<p class="text-xs text-slate-500 mt-0.5">Tap to capture a task, idea, or voice note</p>
	</a>
</div>
