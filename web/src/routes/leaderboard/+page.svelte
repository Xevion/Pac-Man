<script lang="ts">
	import { IconTrophy, IconCalendar } from '@tabler/icons-svelte';
	import { mockGlobalData, mockMonthlyData, type LeaderboardEntry } from '$lib/leaderboard';

	let activeTab = $state<'global' | 'monthly'>('global');

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
							<img
								src={entry.avatar}
								alt={entry.name}
								class="w-9 h-9 rounded-sm"
								loading="lazy"
							/>
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
				<button onclick={() => (activeTab = 'global')} class={tabButtonClass(activeTab === 'global')}>
					<IconTrophy size={16} />
					Global
				</button>
				<button
					onclick={() => (activeTab = 'monthly')}
					class={tabButtonClass(activeTab === 'monthly')}
				>
					<IconCalendar size={16} />
					Monthly
				</button>
			</div>

			{#if activeTab === 'global'}
				{@render leaderboardTable(mockGlobalData)}
			{:else}
				{@render leaderboardTable(mockMonthlyData)}
			{/if}
		</div>
	</div>
</div>
