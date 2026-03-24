<script lang="ts">
	let { label, value, sub = '' }: { label: string; value: string; sub?: string } = $props();
</script>

<div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-lg p-4">
	<div class="text-[var(--text-muted)] text-xs uppercase tracking-wider mb-1">{label}</div>
	<div class="text-2xl font-bold text-white">{value}</div>
	{#if sub}
		<div class="text-[var(--text-muted)] text-xs mt-1">{sub}</div>
	{/if}
</div>
