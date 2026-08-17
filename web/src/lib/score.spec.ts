import { describe, it, expect } from 'vitest';
import { submitScore, submitPendingScoreIfAny, type ScoreStorage } from './score';

const PENDING_KEY = 'pacman:pendingScore';

function createStorage(): ScoreStorage {
	const store = new Map<string, string>();
	return {
		getItem: (key) => store.get(key) ?? null,
		setItem: (key, value) => void store.set(key, value),
		removeItem: (key) => void store.delete(key)
	};
}

describe('submitScore', () => {
	it('posts the score and clears any pending stash on success', async () => {
		let requestedUrl = '';
		let requestedBody: unknown = null;
		const storage = createStorage();
		storage.setItem(PENDING_KEY, '{"score":1,"levelCount":1,"durationMs":1}');

		const fakeFetch = (async (url: string, init?: RequestInit) => {
			requestedUrl = url;
			requestedBody = JSON.parse(init!.body as string);
			return new Response(null, { status: 201 });
		}) as unknown as typeof fetch;

		const result = await submitScore(12345, 3, 90000, fakeFetch, storage);

		expect(result).toBe('submitted');
		expect(requestedUrl).toBe('/api/scores');
		expect(requestedBody).toEqual({ score: 12345, level_count: 3, duration_ms: 90000 });
		expect(storage.getItem(PENDING_KEY)).toBeNull();
	});

	it('stashes the score and reports unauthenticated on 401', async () => {
		const storage = createStorage();
		const fakeFetch = (async () => new Response(null, { status: 401 })) as unknown as typeof fetch;

		const result = await submitScore(500, 1, 1000, fakeFetch, storage);

		expect(result).toBe('unauthenticated');
		expect(JSON.parse(storage.getItem(PENDING_KEY)!)).toEqual({
			score: 500,
			levelCount: 1,
			durationMs: 1000
		});
	});

	it('reports an error on other failures without touching the stash', async () => {
		const storage = createStorage();
		const fakeFetch = (async () => new Response(null, { status: 500 })) as unknown as typeof fetch;

		const result = await submitScore(500, 1, 1000, fakeFetch, storage);

		expect(result).toBe('error');
		expect(storage.getItem(PENDING_KEY)).toBeNull();
	});
});

describe('submitPendingScoreIfAny', () => {
	it('does nothing when no score is pending', async () => {
		const storage = createStorage();
		const fakeFetch = (async () => new Response(null, { status: 201 })) as unknown as typeof fetch;

		const result = await submitPendingScoreIfAny(fakeFetch, storage);

		expect(result).toBe('none');
	});

	it('resubmits a stashed score', async () => {
		const storage = createStorage();
		storage.setItem(PENDING_KEY, JSON.stringify({ score: 777, levelCount: 2, durationMs: 5000 }));
		let requestedBody: unknown = null;
		const fakeFetch = (async (_url: string, init?: RequestInit) => {
			requestedBody = JSON.parse(init!.body as string);
			return new Response(null, { status: 201 });
		}) as unknown as typeof fetch;

		const result = await submitPendingScoreIfAny(fakeFetch, storage);

		expect(result).toBe('submitted');
		expect(requestedBody).toEqual({ score: 777, level_count: 2, duration_ms: 5000 });
	});

	it('clears malformed stashed data and reports an error', async () => {
		const storage = createStorage();
		storage.setItem(PENDING_KEY, 'not json');
		const fakeFetch = (async () => new Response(null, { status: 201 })) as unknown as typeof fetch;

		const result = await submitPendingScoreIfAny(fakeFetch, storage);

		expect(result).toBe('error');
		expect(storage.getItem(PENDING_KEY)).toBeNull();
	});
});
