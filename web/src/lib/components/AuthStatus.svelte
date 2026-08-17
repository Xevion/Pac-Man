<script lang="ts">
	import { onMount } from 'svelte';
	import {
		fetchProfile,
		fetchProviders,
		displayName,
		type Profile,
		type AuthProvider
	} from '$lib/auth';

	let profile = $state<Profile | null>(null);
	let providers = $state<AuthProvider[]>([]);
	let loaded = $state(false);

	onMount(async () => {
		const [fetchedProfile, fetchedProviders] = await Promise.all([
			fetchProfile().catch(() => null),
			fetchProviders().catch(() => [])
		]);
		profile = fetchedProfile;
		providers = fetchedProviders;
		loaded = true;
	});
</script>

{#if loaded}
	<div class="flex items-center gap-3 text-sm">
		{#if profile}
			<span class="text-gray-300">{displayName(profile)}</span>
			<a
				href="/api/logout"
				class="text-gray-500 hover:text-gray-300 transition-colors duration-200"
			>
				Log out
			</a>
		{:else}
			{#each providers.filter((p) => p.active) as provider (provider.id)}
				<a
					href={`/api/auth/${provider.id}`}
					class="text-gray-500 hover:text-gray-300 transition-colors duration-200"
				>
					Log in with {provider.name}
				</a>
			{/each}
		{/if}
	</div>
{/if}
