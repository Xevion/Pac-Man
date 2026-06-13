/** A leaderboard row in display form, ready to render. */
export interface LeaderboardEntry {
	id: number;
	rank: number;
	name: string;
	score: number;
	duration: string;
	levelCount: number;
	submittedAt: string;
	avatar?: string;
}

/** Raw leaderboard row as returned by `GET /api/scores`. */
export interface ApiScoreEntry {
	rank: number;
	user_id: number;
	name: string | null;
	avatar: string | null;
	score: number;
	level_count: number;
	duration_ms: number | null;
	submitted_at: string;
}

export type LeaderboardPeriod = 'global' | 'monthly';

// Em dash placeholder for absent values.
const MISSING = '—';

/** Format a millisecond duration as `m:ss` (minutes are not capped at 60). */
export function formatDuration(ms: number | null): string {
	if (ms == null) return MISSING;
	const totalSeconds = Math.floor(ms / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function plural(value: number, unit: string): string {
	return `${value} ${unit}${value === 1 ? '' : 's'} ago`;
}

/** Human-friendly "time ago" string. `now` is injectable for deterministic tests. */
export function formatRelativeTime(iso: string, now: Date = new Date()): string {
	const seconds = Math.floor((now.getTime() - new Date(iso).getTime()) / 1000);
	if (seconds < 60) return 'just now';

	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return plural(minutes, 'minute');
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return plural(hours, 'hour');
	const days = Math.floor(hours / 24);
	if (days < 7) return plural(days, 'day');
	const weeks = Math.floor(days / 7);
	return plural(weeks, 'week');
}

/** Convert an API row to display form. `now` is injectable for deterministic tests. */
export function mapApiEntry(entry: ApiScoreEntry, now: Date = new Date()): LeaderboardEntry {
	return {
		id: entry.user_id,
		rank: entry.rank,
		name: entry.name ?? `Player ${entry.user_id}`,
		score: entry.score,
		duration: formatDuration(entry.duration_ms),
		levelCount: entry.level_count,
		submittedAt: formatRelativeTime(entry.submitted_at, now),
		avatar: entry.avatar ?? undefined
	};
}

/** Fetch and map the leaderboard for a period. `fetchFn` is injectable for tests. */
export async function fetchLeaderboard(
	period: LeaderboardPeriod,
	fetchFn: typeof fetch = fetch
): Promise<LeaderboardEntry[]> {
	const response = await fetchFn(`/api/scores?period=${period}`);
	if (!response.ok) {
		throw new Error(`leaderboard request failed: ${response.status}`);
	}
	const data: ApiScoreEntry[] = await response.json();
	return data.map((entry) => mapApiEntry(entry));
}
