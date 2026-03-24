export function formatSize(bytes: number): string {
	if (bytes >= 1e12) return (bytes / 1e12).toFixed(1) + ' TB';
	if (bytes >= 1e9) return (bytes / 1e9).toFixed(1) + ' GB';
	if (bytes >= 1e6) return (bytes / 1e6).toFixed(1) + ' MB';
	if (bytes >= 1e3) return (bytes / 1e3).toFixed(1) + ' KB';
	return bytes + ' B';
}

export function formatTime(ts: number): string {
	return new Date(ts * 1000).toLocaleString();
}

export function shortenPath(path: string): string {
	const home = '/Users/'; // Will be replaced with actual home detection
	const idx = path.indexOf('/', home.length);
	if (path.startsWith(home) && idx > 0) {
		return '~' + path.slice(idx);
	}
	return path;
}
