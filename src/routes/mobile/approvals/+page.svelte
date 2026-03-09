<script lang="ts">
	import { mobileState, submitApproval } from '$lib/mobile/store.svelte.js';

	let filter = $state<'pending' | 'resolved' | 'all'>('pending');

	let filtered = $derived(
		filter === 'all'
			? mobileState.approvals
			: filter === 'pending'
				? mobileState.approvals.filter((a) => a.status === 'pending')
				: mobileState.approvals.filter((a) => a.status !== 'pending')
	);

	let pendingCount = $derived(mobileState.approvals.filter(a => a.status === 'pending').length);

	const priorityStyle: Record<string, string> = {
		urgent: 'border-red-400/30 bg-red-400/5',
		normal: 'border-amber-400/20 bg-amber-400/5',
	};
</script>

<div class="space-y-5 py-2">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-xl font-bold text-white">Approval Queue</h1>
			<p class="text-xs text-slate-500 mt-0.5">Agent actions waiting for your review</p>
		</div>
		{#if pendingCount > 0}
			<span class="bg-amber-400/15 text-amber-300 text-sm font-medium rounded-full px-3 py-1">
				{pendingCount} pending
			</span>
		{/if}
	</div>

	<!-- Filter tabs -->
	<div class="flex gap-2">
		{#each [['pending', 'Pending'], ['resolved', 'Resolved'], ['all', 'All']] as [f, label]}
			<button
				onclick={() => filter = f as typeof filter}
				class={`px-3 py-1.5 rounded-full text-xs font-medium border transition-all ${
					filter === f ? 'bg-cyan-400/15 text-cyan-300 border-cyan-400/30' : 'border-white/8 text-slate-500'
				}`}
			>{label}</button>
		{/each}
	</div>

	<!-- Approval cards -->
	<div class="space-y-3">
		{#each filtered as approval}
			<div class={`rounded-2xl border p-4 ${priorityStyle[approval.priority]}`}>
				<!-- Header -->
				<div class="flex items-start justify-between mb-3">
					<div class="flex items-center gap-2">
						<div class="w-8 h-8 rounded-xl bg-slate-800 border border-white/10 flex items-center justify-center text-sm">
							{approval.agentName.charAt(0)}
						</div>
						<div>
							<p class="text-xs text-slate-400">{approval.agentName}</p>
							<p class="text-sm font-medium text-white">{approval.action}</p>
						</div>
					</div>
					<div class="flex flex-col items-end gap-1">
						{#if approval.priority === 'urgent'}
							<span class="text-xs bg-red-400/20 text-red-300 rounded-full px-2 py-0.5">Urgent</span>
						{/if}
						{#if approval.status !== 'pending'}
							<span class={`text-xs rounded-full px-2 py-0.5 ${approval.status === 'approved' ? 'bg-emerald-400/15 text-emerald-300' : 'bg-red-400/15 text-red-300'}`}>
								{approval.status}
							</span>
						{/if}
					</div>
				</div>

				<!-- Context -->
				<div class="rounded-xl bg-slate-950/40 p-3 mb-3">
					<p class="text-xs text-slate-400 leading-relaxed">{approval.context}</p>
				</div>

				<!-- Timestamp -->
				<p class="text-xs text-slate-600 mb-3">
					{new Date(approval.createdAt).toLocaleString()}
					{#if approval.resolvedBy} · resolved by {approval.resolvedBy}{/if}
				</p>

				<!-- Actions -->
				{#if approval.status === 'pending'}
					<div class="grid grid-cols-2 gap-2">
						<button
							onclick={() => submitApproval(approval.id, 'rejected')}
							class="py-3 rounded-xl bg-red-500/15 border border-red-500/25 text-red-300 text-sm font-medium hover:bg-red-500/25 transition-colors active:scale-95"
						>
							✗ Reject
						</button>
						<button
							onclick={() => submitApproval(approval.id, 'approved')}
							class="py-3 rounded-xl bg-emerald-500/15 border border-emerald-500/25 text-emerald-300 text-sm font-medium hover:bg-emerald-500/25 transition-colors active:scale-95"
						>
							✓ Approve
						</button>
					</div>
				{/if}
			</div>
		{:else}
			<div class="text-center py-16 text-slate-600">
				<p class="text-4xl mb-3">◎</p>
				<p class="text-sm">
					{filter === 'pending' ? 'No pending approvals — agents are running smoothly' : 'Nothing here yet'}
				</p>
			</div>
		{/each}
	</div>
</div>
