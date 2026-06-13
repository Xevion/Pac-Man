<script lang="ts">
	import { onMount } from 'svelte';
	import { IconTrophy, IconCalendar } from '@tabler/icons-svelte';
	import {
		fetchLeaderboard,
		type LeaderboardEntry,
		type LeaderboardPeriod
	} from '$lib/leaderboard';

	type LoadState =
		| { status: 'loading' }
		| { status: 'error'; message: string }
		| { status: 'loaded'; entries: LeaderboardEntry[] };

	let activeTab = $state<LeaderboardPeriod>('global');
	const cache = $state<Partial<Record<LeaderboardPeriod, LoadState>>>({});

	async function load(period: LeaderboardPeriod): Promise<void> {
		cache[period] = { status: 'loading' };
		try {
			const entries = await fetchLeaderboard(period);
			cache[period] = { status: 'loaded', entries };
		} catch {
			cache[period] = {
				status: 'error',
				message: 'Could not load the leaderboard. Try again in a moment.'
			};
		}
	}

	function selectTab(period: LeaderboardPeriod): void {
		activeTab = period;
		if (!cache[period]) load(period);
	}

	onMount(() => load(activeTab));

	let current = $derived(cache[activeTab]);

	function tabButtonClass(isActive: boolean): string {
		return `inline-flex items-center gap-1 px-3 py-1 rounded border ${
			isActive
				? 'border-yellow-400/40 text-yellow-300'
				: 'border-transparent text-gray-300 hover:text-yellow-200'
		}`;
	}
</script>

{#snippet leaderboardTable(data: LeaderboardEntry[])}
	<table class="w-full border-separate border-spacing-y-2">
		<tbody>
			{#each data as entry (entry.id)}
				<tr class="bg-black">
					<td class="py-2">
						<div class="flex items-center gap-2">
							{#if entry.avatar}
								<img
									src={entry.avatar}
									alt={entry.name}
									class="w-9 h-9 rounded-sm"
									loading="lazy"
								/>
							{:else}
								<div
									class="w-9 h-9 rounded-sm bg-yellow-400/10 flex items-center justify-center text-yellow-400/60"
								>
									{entry.rank}
								</div>
							{/if}
							<div class="flex flex-col">
								<span class="text-yellow-400 font-semibold text-lg">{entry.name}</span>
								<span class="text-xs text-gray-400">{entry.submittedAt}</span>
							</div>
						</div>
					</td>
					<td class="py-2">
						<span class="text-yellow-300 font-[600] text-lg">{entry.score.toLocaleString()}</span>
					</td>
					<td class="py-2">
						<span class="text-gray-300">{entry.duration}</span>
					</td>
					<td class="py-2">Level {entry.levelCount}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/snippet}

<div class="page-container">
	<div class="space-y-6">
		<div class="card">
			<div class="flex gap-2 border-b border-yellow-400/20 pb-2 mb-4">
				<button onclick={() => selectTab('global')} class={tabButtonClass(activeTab === 'global')}>
					<IconTrophy size={16} />
					Global
				</button>
				<button
					onclick={() => selectTab('monthly')}
					class={tabButtonClass(activeTab === 'monthly')}
				>
					<IconCalendar size={16} />
					Monthly
				</button>
			</div>

			{#if !current || current.status === 'loading'}
				<p class="py-8 text-center text-gray-400">Loading scores&hellip;</p>
			{:else if current.status === 'error'}
				<div class="py-8 text-center space-y-3">
					<p class="text-gray-300">{current.message}</p>
					<button
						onclick={() => load(activeTab)}
						class="inline-flex items-center px-3 py-1 rounded border border-yellow-400/40 text-yellow-300 hover:text-yellow-200"
					>
						Try again
					</button>
				</div>
			{:else if current.entries.length === 0}
				<p class="py-8 text-center text-gray-400">
					No scores yet. Be the first to claim a spot on the board.
				</p>
			{:else}
				{@render leaderboardTable(current.entries)}
			{/if}
		</div>
	</div>
</div>
