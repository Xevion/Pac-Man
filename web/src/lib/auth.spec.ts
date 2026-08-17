import { describe, it, expect } from 'vitest';
import { fetchProfile, fetchProviders, displayName, type Profile } from './auth';

describe('fetchProfile', () => {
	it('maps the API profile to the display shape', async () => {
		const apiProfile = {
			id: 42,
			email: 'a@example.com',
			providers: [
				{
					provider: 'discord',
					provider_user_id: '123',
					email: 'a@example.com',
					username: 'wakka',
					display_name: 'Wakka Wakka',
					avatar_url: 'https://example.com/a.png'
				}
			],
			created_at: '2026-01-01T00:00:00Z',
			updated_at: '2026-01-02T00:00:00Z'
		};
		const fakeFetch = (async () =>
			new Response(JSON.stringify(apiProfile), {
				status: 200,
				headers: { 'content-type': 'application/json' }
			})) as unknown as typeof fetch;

		const profile = await fetchProfile(fakeFetch);

		expect(profile?.id).toBe(42);
		expect(profile?.providers[0].displayName).toBe('Wakka Wakka');
		expect(profile?.providers[0].providerUserId).toBe('123');
	});

	it('returns null when logged out', async () => {
		const fakeFetch = (async () => new Response(null, { status: 401 })) as unknown as typeof fetch;

		const profile = await fetchProfile(fakeFetch);

		expect(profile).toBeNull();
	});

	it('throws on other failures', async () => {
		const fakeFetch = (async () =>
			new Response('nope', { status: 500 })) as unknown as typeof fetch;

		await expect(fetchProfile(fakeFetch)).rejects.toThrow();
	});
});

describe('fetchProviders', () => {
	it('returns the enabled providers list', async () => {
		const apiProviders = [{ id: 'discord', name: 'Discord', active: true }];
		const fakeFetch = (async () =>
			new Response(JSON.stringify(apiProviders), {
				status: 200,
				headers: { 'content-type': 'application/json' }
			})) as unknown as typeof fetch;

		const providers = await fetchProviders(fakeFetch);

		expect(providers).toEqual(apiProviders);
	});
});

describe('displayName', () => {
	it('prefers the first provider display name', () => {
		const profile: Profile = {
			id: 1,
			email: null,
			providers: [
				{
					provider: 'discord',
					providerUserId: '1',
					email: null,
					username: 'wakka',
					displayName: 'Wakka Wakka',
					avatarUrl: null
				}
			],
			createdAt: '',
			updatedAt: ''
		};
		expect(displayName(profile)).toBe('Wakka Wakka');
	});

	it('falls back to username, then a generic label', () => {
		const noDisplayName: Profile = {
			id: 2,
			email: null,
			providers: [
				{
					provider: 'github',
					providerUserId: '2',
					email: null,
					username: 'gh-user',
					displayName: null,
					avatarUrl: null
				}
			],
			createdAt: '',
			updatedAt: ''
		};
		expect(displayName(noDisplayName)).toBe('gh-user');

		const noProviders: Profile = {
			id: 3,
			email: null,
			providers: [],
			createdAt: '',
			updatedAt: ''
		};
		expect(displayName(noProviders)).toBe('Player 3');
	});
});
