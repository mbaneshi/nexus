<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Stats, type FileChange } from '$lib/api';
	import { formatSize, formatTime } from '$lib/format';
	import StatCard from '$lib/components/StatCard.svelte';

	let stats: Stats | null = $state(null);
	let changes: FileChange[] = $state([]);
	let daemon: { running: boolean; pid?: number } | null = $state(null);
	let error: string | null = $state(null);

	onMount(async () => {
		try {
			[stats, daemon] = await Promise.all([api.stats(), api.daemonStatus()]);
			const res = await api.changes(20);
			changes = res.changes;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		}
	});
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-xl font-bold">Dashboard</h1>
		{#if daemon}
			<span class="text-xs px-2 py-1 rounded {daemon.running ? 'bg-green-900/30 text-[var(--green)]' : 'bg-red-900/30 text-[var(--red)]'}">
				daemon: {daemon.running ? `running (PID ${daemon.pid})` : 'stopped'}
			</span>
		{/if}
	</div>

	{#if error}
		<div class="bg-red-900/20 border border-red-800 rounded p-3 text-[var(--red)] text-sm">
			{error}
		</div>
	{:else if !stats}
		<div class="text-[var(--text-muted)]">Loading...</div>
	{:else}
		<!-- Stats cards -->
		<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
			<StatCard label="Files" value={stats.total_files.toLocaleString()} />
			<StatCard label="Directories" value={stats.total_dirs.toLocaleString()} />
			<StatCard label="Total Size" value={formatSize(stats.total_size)} />
			<StatCard label="Config Tools" value={String(stats.config_tools)} />
		</div>

		<!-- Category breakdown -->
		{#if stats.by_category.length > 0}
			<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4">
				<h2 class="text-sm font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3">Categories</h2>
				<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
					{#each stats.by_category as cat}
						{@const pct = stats.total_size > 0 ? ((cat.total_size / stats.total_size) * 100).toFixed(1) : '0'}
						<div class="flex justify-between items-center px-3 py-2 rounded bg-[var(--bg)] text-sm">
							<span class="capitalize">{cat.category}</span>
							<span class="text-[var(--text-muted)]">{cat.file_count.toLocaleString()} &middot; {formatSize(cat.total_size)} &middot; {pct}%</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Last scan -->
		{#if stats.last_scan}
			<div class="text-xs text-[var(--text-muted)]">
				Last scan: {formatTime(stats.last_scan)}
			</div>
		{/if}

		<!-- Recent changes -->
		{#if changes.length > 0}
			<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4">
				<h2 class="text-sm font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3">Recent Changes</h2>
				<div class="space-y-1 max-h-64 overflow-y-auto">
					{#each changes as change}
						<div class="flex items-center gap-3 text-sm px-2 py-1 rounded hover:bg-[var(--bg-hover)]">
							<span class="w-16 text-[var(--text-muted)] text-xs">{new Date(change.detected_at * 1000).toLocaleTimeString()}</span>
							<span class="w-16 text-xs font-medium {
								change.change_type === 'created' ? 'text-[var(--green)]' :
								change.change_type === 'deleted' ? 'text-[var(--red)]' :
								'text-[var(--yellow)]'
							}">{change.change_type}</span>
							<span class="truncate text-[var(--text-muted)]">{change.path}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>
