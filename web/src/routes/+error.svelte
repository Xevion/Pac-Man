<script lang="ts">
	import { page } from '$app/stores';
</script>

<div class="flex flex-col items-center justify-center min-h-[50vh] text-center px-4">
	<h1 class="text-4xl font-bold mb-4">
		{$page.status === 404 ? 'Page Not Found' : 'Internal Error'}
	</h1>
	<p class="text-gray-400">
		{$page.status === 404 ? 'This page could not be found.' : 'Something went wrong.'}
	</p>
</div>
