<script lang="ts">
	import { mobileState, captureVoice } from '$lib/mobile/store.svelte.js';
	import { syncEngine } from '$lib/sync/sync-engine.js';
	import { memorySpine } from '$lib/memory/memory-spine.js';

	let today = $derived(new Date().toISOString().slice(0, 10));

	async function checkHabit(habitId: string) {
		await syncEngine.checkHabit(habitId);
		await memorySpine.write({
			tier: 'raw',
			eventType: 'habit.check',
			payload: { habitId, date: today },
			source: 'mobile',
		});
		const habit = mobileState.habits.find((h) => h.id === habitId);
		if (habit && !habit.completedDates.includes(today)) {
			habit.completedDates = [...habit.completedDates, today];
			habit.streak += 1;
		}
	}

	function streakColor(streak: number): string {
		if (streak >= 14) return 'text-amber-400';
		if (streak >= 7) return 'text-cyan-400';
		if (streak >= 3) return 'text-emerald-400';
		return 'text-slate-500';
	}
</script>

<div class="space-y-5 py-2">
	<div>
		<h1 class="text-xl font-bold text-white">Habits</h1>
		<p class="text-xs text-slate-500 mt-0.5">{new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' })}</p>
	</div>

	<!-- Today's progress bar -->
	{#if true}
		{@const done = mobileState.habits.filter(h => h.completedDates.includes(today)).length}
		{@const total = mobileState.habits.length}
		<div>
			<div class="flex items-center justify-between mb-2">
				<span class="text-xs text-slate-400">Today's progress</span>
				<span class="text-xs font-medium text-cyan-300">{done}/{total}</span>
			</div>
			<div class="h-2 bg-slate-800 rounded-full overflow-hidden">
				<div
					class="h-full bg-gradient-to-r from-cyan-400 to-violet-400 rounded-full transition-all duration-500"
					style="width: {total > 0 ? (done / total) * 100 : 0}%"
				></div>
			</div>
		</div>
	{/if}

	<!-- Habit cards -->
	<div class="space-y-3">
		{#each mobileState.habits as habit}
			{@const isDone = habit.completedDates.includes(today)}
			<div class={`rounded-2xl border p-4 transition-all ${isDone ? 'border-emerald-400/30 bg-emerald-400/5' : 'border-white/8 bg-slate-900/50'}`}>
				<div class="flex items-center gap-4">
					<button
						onclick={() => !isDone && checkHabit(habit.id)}
						class={`w-12 h-12 rounded-2xl border-2 flex items-center justify-center text-xl transition-all shrink-0 ${
							isDone
								? 'bg-emerald-400/20 border-emerald-400 text-emerald-400'
								: 'border-slate-600 text-slate-600 hover:border-slate-400 hover:text-slate-400 active:scale-95'
						}`}
					>{isDone ? '✓' : '○'}</button>

					<div class="flex-1 min-w-0">
						<div class="flex items-center justify-between">
							<p class="text-sm font-medium text-white">{habit.name}</p>
							<span class="text-xs text-slate-600">{habit.frequency}</span>
						</div>
						<div class="flex items-center gap-2 mt-1">
							<span class={`text-sm font-bold ${streakColor(habit.streak)}`}>{habit.streak}</span>
							<span class="text-xs text-slate-500">day streak</span>
							{#if habit.streak >= 7}
								<span class="text-xs">🔥</span>
							{/if}
						</div>
					</div>
				</div>

				<!-- Last 7 days visual -->
				<div class="flex gap-1.5 mt-3">
					{#each Array.from({length: 7}, (_, i) => {
						const d = new Date();
						d.setDate(d.getDate() - (6 - i));
						return d.toISOString().slice(0, 10);
					}) as dateStr}
						<div
							class={`flex-1 h-1.5 rounded-full ${habit.completedDates.includes(dateStr) ? 'opacity-100' : 'opacity-20'}`}
							style="background-color: {habit.color}"
						></div>
					{/each}
				</div>
			</div>
		{:else}
			<div class="text-center py-12 text-slate-600">
				<p class="text-3xl mb-2">○</p>
				<p class="text-sm">No habits tracked yet</p>
			</div>
		{/each}
	</div>
</div>
