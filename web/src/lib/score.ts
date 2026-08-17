/** Minimal storage interface `score.ts` needs, so tests can inject an in-memory fake
 * instead of depending on a real `localStorage` global. */
export interface ScoreStorage {
	getItem(key: string): string | null;
	setItem(key: string, value: string): void;
	removeItem(key: string): void;
}

const PENDING_SCORE_KEY = 'pacman:pendingScore';

interface PendingScore {
	score: number;
	levelCount: number;
	durationMs: number;
}

export type SubmitScoreResult = 'submitted' | 'unauthenticated' | 'error';

/** Submits a finished run's score. On `401` (no session), stashes it in `storage` so
 * `submitPendingScoreIfAny` can retry after the player logs in. */
export async function submitScore(
	score: number,
	levelCount: number,
	durationMs: number,
	fetchFn: typeof fetch = fetch,
	storage: ScoreStorage = localStorage
): Promise<SubmitScoreResult> {
	const response = await fetchFn('/api/scores', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ score, level_count: levelCount, duration_ms: durationMs })
	});

	if (response.status === 401) {
		const pending: PendingScore = { score, levelCount, durationMs };
		storage.setItem(PENDING_SCORE_KEY, JSON.stringify(pending));
		return 'unauthenticated';
	}

	if (!response.ok) {
		return 'error';
	}

	storage.removeItem(PENDING_SCORE_KEY);
	return 'submitted';
}

/** Resubmits a score stashed before an OAuth login redirect, if one exists. A no-op
 * (returns `'none'`) when nothing is pending. */
export async function submitPendingScoreIfAny(
	fetchFn: typeof fetch = fetch,
	storage: ScoreStorage = localStorage
): Promise<SubmitScoreResult | 'none'> {
	const raw = storage.getItem(PENDING_SCORE_KEY);
	if (!raw) {
		return 'none';
	}

	let pending: PendingScore;
	try {
		pending = JSON.parse(raw);
	} catch {
		storage.removeItem(PENDING_SCORE_KEY);
		return 'error';
	}

	return submitScore(pending.score, pending.levelCount, pending.durationMs, fetchFn, storage);
}
