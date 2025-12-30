<script lang="ts">
	import type { ComponentType } from 'svelte';

	let {
		href,
		icon,
		label,
		active = false,
		size = 18
	}: {
		href: string;
		icon: ComponentType;
		label: string;
		active?: boolean;
		size?: number;
	} = $props();

	function navLinkClass(active: boolean): string {
		return `flex items-center gap-1.5 tracking-wide transition-colors duration-200 ${
			active ? 'text-white' : 'text-gray-500 hover:text-gray-300'
		}`;
	}
</script>

<a {href} class={navLinkClass(active)}>
	<icon {size}></icon>
	<span>{label}</span>
</a>
