<script lang="ts">
	import '../app.css';
	import Nav from '$lib/components/Nav.svelte';

	let { children } = $props();
</script>

<div class="min-h-screen flex flex-col">
	<Nav />
	<main class="flex-1 max-w-7xl mx-auto w-full px-4 py-6">
		{@render children()}
	</main>
</div>
