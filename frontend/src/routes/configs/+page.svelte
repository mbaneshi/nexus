<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type ConfigTool, type Snapshot } from '$lib/api';

	let tools: ConfigTool[] = $state([]);
	let snapshots: Snapshot[] = $state([]);
	let loading = $state(true);
	let backupStatus: string | null = $state(null);

	onMount(async () => {
		try {
			[tools, snapshots] = await Promise.all([api.configTools(), api.configSnapshots()]);
		} catch {
			// empty state
		}
		loading = false;
	});

	async function backupTool(name: string) {
		backupStatus = `Backing up ${name}...`;
		try {
			const res = await api.configBackup(name);
			backupStatus = `Snapshot #${res.snapshot_id} created for ${name}`;
			snapshots = await api.configSnapshots();
		} catch (e) {
			backupStatus = e instanceof Error ? e.message : 'Backup failed';
		}
		setTimeout(() => (backupStatus = null), 3000);
	}

	async function backupAll() {
		backupStatus = 'Backing up all configs...';
		try {
			const res = await api.configBackup();
			backupStatus = `Snapshot #${res.snapshot_id} created`;
			snapshots = await api.configSnapshots();
		} catch (e) {
			backupStatus = e instanceof Error ? e.message : 'Backup failed';
		}
		setTimeout(() => (backupStatus = null), 3000);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-xl font-bold">Config Tools</h1>
		<button
			onclick={backupAll}
			class="bg-[var(--accent)] hover:bg-sky-400 text-black text-sm font-medium px-3 py-1.5 rounded transition-colors"
		>
			Backup All
		</button>
	</div>

	{#if backupStatus}
		<div class="bg-sky-900/20 border border-sky-800 rounded p-2 text-[var(--accent)] text-sm">
			{backupStatus}
		</div>
	{/if}

	{#if loading}
		<div class="text-[var(--text-muted)]">Loading...</div>
	{:else if tools.length === 0}
		<div class="text-[var(--text-muted)]">No config tools discovered. Run <code class="bg-[var(--bg-card)] px-1 rounded">nexus config list</code> first.</div>
	{:else}
		<!-- Tools grid -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
			{#each tools as tool}
				<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4 hover:border-[var(--accent)] transition-colors">
					<div class="flex items-center justify-between mb-2">
						<h3 class="font-semibold">{tool.name}</h3>
						<span class="text-xs bg-[var(--bg)] px-2 py-0.5 rounded text-[var(--text-muted)]">{tool.language}</span>
					</div>
					<p class="text-xs text-[var(--text-muted)] mb-2">{tool.description}</p>
					<div class="text-xs text-[var(--text-muted)] truncate mb-3">{tool.config_dir}</div>
					<button
						onclick={() => backupTool(tool.name)}
						class="text-xs text-[var(--accent)] hover:underline"
					>
						Backup
					</button>
				</div>
			{/each}
		</div>

		<!-- Snapshots -->
		{#if snapshots.length > 0}
			<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4">
				<h2 class="text-sm font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-3">Recent Snapshots</h2>
				<div class="space-y-1">
					{#each snapshots.slice(0, 20) as snap}
						<div class="flex items-center justify-between text-sm px-2 py-1 rounded hover:bg-[var(--bg-hover)]">
							<div class="flex items-center gap-3">
								<span class="text-[var(--text-muted)]">#{snap.id}</span>
								<span>{snap.tool_name ?? 'all'}</span>
								{#if snap.label}
									<span class="text-xs text-[var(--text-muted)]">({snap.label})</span>
								{/if}
							</div>
							<div class="flex items-center gap-3 text-[var(--text-muted)] text-xs">
								<span>{snap.file_count} files</span>
								<span>{new Date(snap.created_at * 1000).toLocaleString()}</span>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>
