import { describe, it, expect } from 'vitest';
import { pushLeaderboardToGame, type PacmanModule } from './pacman';

describe('pushLeaderboardToGame', () => {
	it('clears then pushes each row in rank order via ccall', () => {
		const calls: unknown[][] = [];
		const module = {
			ccall: (...args: unknown[]) => {
				calls.push(args);
				return null;
			}
		} as unknown as PacmanModule;

		pushLeaderboardToGame(
			[
				{ name: 'ACE', score: 100 },
				{ name: 'BOB', score: 50 }
			],
			module
		);

		expect(calls).toEqual([
			['pacman_leaderboard_clear', null, [], []],
			['pacman_leaderboard_push', null, ['string', 'number'], ['ACE', 100]],
			['pacman_leaderboard_push', null, ['string', 'number'], ['BOB', 50]]
		]);
	});

	it('does nothing when the module has no ccall (not yet loaded, or built without it)', () => {
		expect(() => pushLeaderboardToGame([{ name: 'X', score: 1 }], undefined)).not.toThrow();
	});
});
