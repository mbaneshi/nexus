<script lang="ts">
	import { api, type SearchResult } from '$lib/api';
	import { formatSize } from '$lib/format';

	let query = $state('');
	let category = $state('');
	let results: SearchResult[] = $state([]);
	let count = $state(0);
	let loading = $state(false);
	let searched = $state(false);

	let debounceTimer: ReturnType<typeof setTimeout>;

	function onInput() {
		clearTimeout(debounceTimer);
		if (query.trim().length < 2) return;
		debounceTimer = setTimeout(doSearch, 300);
	}

	async function doSearch() {
		if (!query.trim()) return;
		loading = true;
		searched = true;
		try {
			const res = await api.search(query, category || undefined, 50);
			results = res.results;
			count = res.count;
		} catch {
			results = [];
			count = 0;
		}
		loading = false;
	}

	const categories = ['', 'config', 'code', 'document', 'image', 'video', 'audio', 'archive', 'cache', 'project', 'download'];
</script>

<div class="space-y-4">
	<h1 class="text-xl font-bold">Search</h1>

	<div class="flex gap-3">
		<input
			type="text"
			bind:value={query}
			oninput={onInput}
			onkeydown={(e) => e.key === 'Enter' && doSearch()}
			placeholder="Search indexed files..."
			class="flex-1 bg-[var(--bg-card)] border border-[var(--border)] rounded px-3 py-2 text-sm text-white placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)]"
		/>
		<select
			bind:value={category}
			onchange={doSearch}
			class="bg-[var(--bg-card)] border border-[var(--border)] rounded px-3 py-2 text-sm text-white focus:outline-none"
		>
			{#each categories as cat}
				<option value={cat}>{cat || 'All categories'}</option>
			{/each}
		</select>
	</div>

	{#if loading}
		<div class="text-[var(--text-muted)] text-sm">Searching...</div>
	{:else if results.length > 0}
		<div class="text-xs text-[var(--text-muted)] mb-2">{count} results</div>
		<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg overflow-hidden">
			<table class="w-full text-sm">
				<thead>
					<tr class="text-[var(--text-muted)] text-xs uppercase tracking-wider border-b border-[var(--border)]">
						<th class="text-left px-4 py-2">Name</th>
						<th class="text-left px-4 py-2">Category</th>
						<th class="text-right px-4 py-2">Size</th>
						<th class="text-left px-4 py-2">Path</th>
					</tr>
				</thead>
				<tbody>
					{#each results as result}
						<tr class="border-b border-[var(--border)] hover:bg-[var(--bg-hover)]">
							<td class="px-4 py-2 font-medium">{result.name}</td>
							<td class="px-4 py-2 capitalize text-[var(--text-muted)]">{result.category}</td>
							<td class="px-4 py-2 text-right text-[var(--text-muted)]">{formatSize(result.size)}</td>
							<td class="px-4 py-2 text-[var(--text-muted)] truncate max-w-md">{result.path}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else if searched}
		<div class="text-[var(--text-muted)] text-sm">No results found.</div>
	{:else}
		<div class="text-[var(--text-muted)] text-sm">Type a query to search your indexed files.</div>
	{/if}
</div>
