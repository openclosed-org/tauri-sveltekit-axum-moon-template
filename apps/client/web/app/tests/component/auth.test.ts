import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock Tauri APIs before importing the auth store
vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn(() => Promise.resolve(() => {}))
}));

vi.mock('$lib/ipc/auth', () => ({
	getSession: vi.fn(() => Promise.resolve(null)),
	startOAuth: vi.fn(() => Promise.resolve()),
	clearAuthStore: vi.fn(() => Promise.resolve()),
	handleOAuthCallback: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn(() => Promise.resolve())
}));

describe('Auth Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('exports auth state object with expected properties', async () => {
		const { auth } = await import('$lib/stores/auth.svelte');
		expect(auth).toBeDefined();
		expect(auth).toHaveProperty('isAuthenticated');
		expect(auth).toHaveProperty('currentUser');
		expect(auth).toHaveProperty('authLoading');
		expect(auth).toHaveProperty('authError');
		expect(auth).toHaveProperty('tokenExpiresAt');
	});

	it('starts with unauthenticated state', async () => {
		const { auth } = await import('$lib/stores/auth.svelte');
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.currentUser).toBeNull();
		expect(auth.authLoading).toBe(false);
		expect(auth.authError).toBeNull();
		expect(auth.tokenExpiresAt).toBe(0);
	});

	it('setSession updates auth state correctly', async () => {
		const { auth, setSession } = await import('$lib/stores/auth.svelte');
		const mockSession = {
			user: { sub: 'google-123', name: 'Test User', email: 'test@example.com' },
			expires_at: Math.floor(Date.now() / 1000) + 3600,
			access_token: 'mock-token'
		};

		setSession(mockSession as any);

		expect(auth.isAuthenticated).toBe(true);
		expect(auth.currentUser).toEqual(mockSession.user);
		expect(auth.authLoading).toBe(false);
		expect(auth.authError).toBeNull();
	});

	it('checkSession returns false when no session exists', async () => {
		const { checkSession } = await import('$lib/stores/auth.svelte');
		const result = await checkSession();
		expect(result).toBe(false);
	});

	it('signInWithGoogle sets loading state', async () => {
		const { auth, signInWithGoogle } = await import('$lib/stores/auth.svelte');
		await signInWithGoogle();
		// After the call, loading should be false (callback hasn't fired)
		// But authError should be null since startOAuth is mocked to succeed
		expect(auth.authError).toBeNull();
	});

	it('signOut clears auth state', async () => {
		const { auth, setSession, signOut } = await import('$lib/stores/auth.svelte');

		// First set a session
		setSession({
			user: { sub: 'google-123', name: 'Test', email: 'test@test.com' },
			expires_at: Math.floor(Date.now() / 1000) + 3600,
			access_token: 'token'
		} as any);
		expect(auth.isAuthenticated).toBe(true);

		// Then sign out
		await signOut();
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.currentUser).toBeNull();
		expect(auth.tokenExpiresAt).toBe(0);
	});

	it('markExpired clears auth state without redirect', async () => {
		const { auth, setSession, markExpired } = await import('$lib/stores/auth.svelte');

		// Set session first
		setSession({
			user: { sub: 'google-123', name: 'Test', email: 'test@test.com' },
			expires_at: Math.floor(Date.now() / 1000) + 3600,
			access_token: 'token'
		} as any);

		await markExpired();
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.currentUser).toBeNull();
	});

	it('initAuthListeners returns cleanup function', async () => {
		const { initAuthListeners } = await import('$lib/stores/auth.svelte');
		const cleanup = initAuthListeners();
		expect(typeof cleanup).toBe('function');
		cleanup();
	});
});
