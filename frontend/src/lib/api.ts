const BASE = '/api';

async function get<T>(path: string): Promise<T> {
	const res = await fetch(`${BASE}${path}`);
	if (!res.ok) throw new Error(`API error: ${res.status}`);
	return res.json();
}

async function post<T>(path: string, body?: unknown): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: body ? JSON.stringify(body) : undefined
	});
	if (!res.ok) throw new Error(`API error: ${res.status}`);
	return res.json();
}

export interface Stats {
	total_files: number;
	total_dirs: number;
	total_size: number;
	config_tools: number;
	by_category: { category: string; file_count: number; total_size: number }[];
	last_scan: number | null;
}

export interface ConfigTool {
	name: string;
	config_dir: string;
	description: string;
	language: string;
}

export interface SearchResult {
	path: string;
	name: string;
	size: number;
	category: string;
	modified_at: number;
	rank: number;
}

export interface FileChange {
	path: string;
	change_type: string;
	detected_at: number;
	new_size: number | null;
}

export interface Snapshot {
	id: number;
	tool_id: number | null;
	tool_name: string | null;
	label: string | null;
	created_at: number;
	file_count: number;
}

export interface DiffEntry {
	path: string;
	status: string;
	old_size: number | null;
	new_size: number | null;
}

export const api = {
	health: () => get<{ status: string }>('/health'),
	stats: () => get<Stats>('/stats'),
	search: (q: string, category?: string, limit?: number) => {
		const params = new URLSearchParams({ q });
		if (category) params.set('category', category);
		if (limit) params.set('limit', String(limit));
		return get<{ count: number; results: SearchResult[] }>(`/search?${params}`);
	},
	configTools: () => get<ConfigTool[]>('/config/tools'),
	configSnapshots: () => get<Snapshot[]>('/config/snapshots'),
	configBackup: (tool?: string, label?: string) =>
		post<{ snapshot_id: number }>('/config/backup', { tool, label }),
	configRestore: (id: number) => post<{ restored_files: number }>(`/config/restore/${id}`),
	configDiff: (tool: string) =>
		get<{ tool: string; snapshot_id: number; changes: DiffEntry[] }>(`/config/${tool}/diff`),
	daemonStatus: () => get<{ running: boolean; pid?: number }>('/daemon/status'),
	changes: (limit?: number) => {
		const params = limit ? `?limit=${limit}` : '';
		return get<{ count: number; changes: FileChange[] }>(`/changes${params}`);
	}
};
