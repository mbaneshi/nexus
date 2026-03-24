<script lang="ts">
	import { page } from '$app/state';

	const links = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/search', label: 'Search' },
		{ href: '/configs', label: 'Configs' }
	];
</script>

<nav class="border-b border-[var(--border)] bg-[var(--bg-card)]">
	<div class="max-w-7xl mx-auto px-4 flex items-center h-14 gap-8">
		<a href="/" class="text-[var(--accent)] font-bold text-lg tracking-tight">nexus</a>
		<div class="flex gap-1">
			{#each links as link}
				<a
					href={link.href}
					class="px-3 py-1.5 rounded text-sm transition-colors {page.url.pathname === link.href
						? 'bg-[var(--bg-hover)] text-white'
						: 'text-[var(--text-muted)] hover:text-white hover:bg-[var(--bg-hover)]'}"
				>
					{link.label}
				</a>
			{/each}
		</div>
	</div>
</nav>
