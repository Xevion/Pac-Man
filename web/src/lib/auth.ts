/** An enabled OAuth login provider, as returned by `GET /api/auth/providers`. */
export interface AuthProvider {
	id: string;
	name: string;
	active: boolean;
}

/** One linked OAuth provider on a profile. */
export interface ProfileProvider {
	provider: string;
	providerUserId: string;
	email: string | null;
	username: string | null;
	displayName: string | null;
	avatarUrl: string | null;
}

/** The current session's profile, as returned by `GET /api/profile`. */
export interface Profile {
	id: number;
	email: string | null;
	providers: ProfileProvider[];
	createdAt: string;
	updatedAt: string;
}

interface ApiProfileProvider {
	provider: string;
	provider_user_id: string;
	email: string | null;
	username: string | null;
	display_name: string | null;
	avatar_url: string | null;
}

interface ApiProfile {
	id: number;
	email: string | null;
	providers: ApiProfileProvider[];
	created_at: string;
	updated_at: string;
}

/** Fetches the current session's profile. Returns `null` when logged out (401). */
export async function fetchProfile(fetchFn: typeof fetch = fetch): Promise<Profile | null> {
	const response = await fetchFn('/api/profile');
	if (response.status === 401) {
		return null;
	}
	if (!response.ok) {
		throw new Error(`profile request failed: ${response.status}`);
	}
	const data: ApiProfile = await response.json();
	return {
		id: data.id,
		email: data.email,
		providers: data.providers.map((p) => ({
			provider: p.provider,
			providerUserId: p.provider_user_id,
			email: p.email,
			username: p.username,
			displayName: p.display_name,
			avatarUrl: p.avatar_url
		})),
		createdAt: data.created_at,
		updatedAt: data.updated_at
	};
}

/** Fetches the enabled OAuth providers for building login links. */
export async function fetchProviders(fetchFn: typeof fetch = fetch): Promise<AuthProvider[]> {
	const response = await fetchFn('/api/auth/providers');
	if (!response.ok) {
		throw new Error(`providers request failed: ${response.status}`);
	}
	return response.json();
}

/** Best-effort display name: the first linked provider's display name or username, else a generic fallback. */
export function displayName(profile: Profile): string {
	const provider = profile.providers[0];
	return provider?.displayName ?? provider?.username ?? `Player ${profile.id}`;
}
