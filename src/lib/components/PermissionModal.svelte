<script lang="ts">
	import { ccPermissionPendingStore, grantPermission, denyPermission } from '$lib/utils/computer-control';

	const RISK_COLORS: Record<string, string> = {
		low: 'text-green-400 border-green-400/30 bg-green-400/10',
		medium: 'text-amber-400 border-amber-400/30 bg-amber-400/10',
		high: 'text-rose-400 border-rose-400/30 bg-rose-400/10'
	};

	const RISK_ICONS: Record<string, string> = {
		low: '✅',
		medium: '⚠️',
		high: '🚨'
	};
</script>

{#if $ccPermissionPendingStore}
	<!-- Backdrop -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
		<div class="relative w-full max-w-md rounded-3xl border border-white/10 bg-slate-950 p-8 shadow-2xl shadow-rose-950/30">

			<!-- Header -->
			<div class="mb-6 flex items-start gap-4">
				<div class="flex-shrink-0 rounded-2xl bg-amber-400/15 p-3">
					<span class="text-2xl">🖥️</span>
				</div>
				<div>
					<h2 class="text-lg font-semibold text-white">Agent Action Request</h2>
					<p class="mt-1 text-sm text-slate-400">
						AmitOS wants to perform an action on your computer.
					</p>
				</div>
			</div>

			<!-- Action Details -->
			<div class="mb-5 rounded-2xl border border-white/10 bg-slate-900/60 p-4 space-y-3">
				<div class="flex items-center justify-between">
					<span class="text-xs uppercase tracking-wider text-slate-500">Action</span>
					<span class="rounded-lg bg-slate-800 px-2.5 py-1 text-xs font-mono text-cyan-300">
						{$ccPermissionPendingStore.action.kind}
					</span>
				</div>

				{#if $ccPermissionPendingStore.action.description}
					<div>
						<span class="text-xs uppercase tracking-wider text-slate-500">Description</span>
						<p class="mt-1 text-sm text-white">{$ccPermissionPendingStore.action.description}</p>
					</div>
				{/if}

				<div>
					<span class="text-xs uppercase tracking-wider text-slate-500">Reason</span>
					<p class="mt-1 text-sm text-slate-300">{$ccPermissionPendingStore.reason}</p>
				</div>

				{#if $ccPermissionPendingStore.action.x !== undefined}
					<div class="flex gap-4 text-xs text-slate-500">
						<span>x: {$ccPermissionPendingStore.action.x}</span>
						<span>y: {$ccPermissionPendingStore.action.y}</span>
					</div>
				{/if}

				{#if $ccPermissionPendingStore.action.text}
					<div>
						<span class="text-xs uppercase tracking-wider text-slate-500">Text</span>
						<p class="mt-1 rounded-lg bg-slate-800 px-3 py-2 text-xs font-mono text-slate-300">
							"{$ccPermissionPendingStore.action.text}"
						</p>
					</div>
				{/if}
			</div>

			<!-- Risk Level -->
			<div class="mb-6 flex items-center gap-3">
				<span class="text-sm text-slate-400">Risk Level:</span>
				<span class="rounded-full border px-3 py-1 text-xs font-semibold {RISK_COLORS[$ccPermissionPendingStore.riskLevel]}">
					{RISK_ICONS[$ccPermissionPendingStore.riskLevel]}
					{$ccPermissionPendingStore.riskLevel.toUpperCase()}
				</span>
			</div>

			<!-- Buttons -->
			<div class="flex gap-3">
				<button
					type="button"
					on:click={denyPermission}
					class="flex-1 rounded-2xl border border-rose-400/30 bg-rose-500/10 px-4 py-3 text-sm font-semibold text-rose-300 hover:bg-rose-500/20 transition-colors"
				>
					✕ Deny
				</button>
				<button
					type="button"
					on:click={grantPermission}
					class="flex-1 rounded-2xl bg-gradient-to-r from-cyan-500 to-violet-500 px-4 py-3 text-sm font-semibold text-white shadow-lg shadow-cyan-950/30 hover:shadow-cyan-950/50 transition-shadow"
				>
					✓ Allow
				</button>
			</div>

			<p class="mt-4 text-center text-xs text-slate-600">
				You can disable per-action prompts in Settings → Computer Control → Mode: Autonomous
			</p>
		</div>
	</div>
{/if}
