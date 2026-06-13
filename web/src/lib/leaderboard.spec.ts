import { describe, it, expect } from 'vitest';
import {
	formatDuration,
	formatRelativeTime,
	mapApiEntry,
	fetchLeaderboard,
	type ApiScoreEntry
} from './leaderboard';

describe('formatDuration', () => {
	it('formats whole minutes and zero-pads seconds', () => {
		expect(formatDuration(60000)).toBe('1:00');
		expect(formatDuration(125000)).toBe('2:05');
		expect(formatDuration(2732000)).toBe('45:32');
	});

	it('renders a placeholder when the duration is missing', () => {
		expect(formatDuration(null)).toBe('—');
	});
});

describe('formatRelativeTime', () => {
	const now = new Date('2026-06-13T12:00:00Z');

	it('describes recent times in hours', () => {
		expect(formatRelativeTime('2026-06-13T10:00:00Z', now)).toBe('2 hours ago');
	});

	it('uses singular units when appropriate', () => {
		expect(formatRelativeTime('2026-06-12T12:00:00Z', now)).toBe('1 day ago');
	});

	it('collapses very recent times to "just now"', () => {
		expect(formatRelativeTime('2026-06-13T11:59:30Z', now)).toBe('just now');
	});
});

describe('mapApiEntry', () => {
	it('maps API fields to the display shape', () => {
		const api: ApiScoreEntry = {
			rank: 1,
			user_id: 42,
			name: 'Wakka Wakka',
			avatar: 'https://example.com/a.png',
			score: 12345,
			level_count: 3,
			duration_ms: 60000,
			submitted_at: '2026-06-13T10:00:00Z'
		};

		const entry = mapApiEntry(api, new Date('2026-06-13T12:00:00Z'));

		expect(entry.id).toBe(42);
		expect(entry.rank).toBe(1);
		expect(entry.name).toBe('Wakka Wakka');
		expect(entry.score).toBe(12345);
		expect(entry.levelCount).toBe(3);
		expect(entry.duration).toBe('1:00');
		expect(entry.submittedAt).toBe('2 hours ago');
		expect(entry.avatar).toBe('https://example.com/a.png');
	});

	it('falls back to a placeholder name when the account has none', () => {
		const api: ApiScoreEntry = {
			rank: 2,
			user_id: 7,
			name: null,
			avatar: null,
			score: 100,
			level_count: 1,
			duration_ms: null,
			submitted_at: '2026-06-13T12:00:00Z'
		};

		const entry = mapApiEntry(api, new Date('2026-06-13T12:00:00Z'));

		expect(entry.name).toBe('Player 7');
		expect(entry.avatar).toBeUndefined();
	});
});

describe('fetchLeaderboard', () => {
	it('requests the period and maps the response', async () => {
		const apiData: ApiScoreEntry[] = [
			{
				rank: 1,
				user_id: 1,
				name: 'Top',
				avatar: null,
				score: 9999,
				level_count: 5,
				duration_ms: 90000,
				submitted_at: '2026-06-13T11:00:00Z'
			}
		];
		let requestedUrl = '';
		const fakeFetch = (async (url: string) => {
			requestedUrl = url;
			return new Response(JSON.stringify(apiData), {
				status: 200,
				headers: { 'content-type': 'application/json' }
			});
		}) as unknown as typeof fetch;

		const entries = await fetchLeaderboard('monthly', fakeFetch);

		expect(requestedUrl).toContain('period=monthly');
		expect(entries).toHaveLength(1);
		expect(entries[0].name).toBe('Top');
		expect(entries[0].duration).toBe('1:30');
	});

	it('throws on a non-ok response', async () => {
		const fakeFetch = (async () =>
			new Response('nope', { status: 503 })) as unknown as typeof fetch;

		await expect(fetchLeaderboard('global', fakeFetch)).rejects.toThrow();
	});
});
