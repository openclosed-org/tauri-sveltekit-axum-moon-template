import { goto } from '$app/navigation';
import type { UserProfile } from '$lib/generated/auth/UserProfile';
import { type AuthSession, clearAuthStore, getSession, logout, startOAuth } from '$lib/ipc/auth';
import { safeListen } from '$lib/ipc/bridge';

export const auth = $state({
  isAuthenticated: false,
  currentUser: null as UserProfile | null,
  authLoading: false,
  authError: null as string | null,
  tokenExpiresAt: 0,
});

/**
 * Check stored session on app load. Returns true if valid session found.
 * Per D-11: detect valid token on page load, redirect to /counter if found.
 */
export async function checkSession(): Promise<boolean> {
  try {
    const session = await getSession();
    if (session && session.tokens.expires_in > Date.now() / 1000) {
      auth.isAuthenticated = true;
      auth.currentUser = session.user;
      auth.tokenExpiresAt = session.tokens.expires_in;
      return true;
    }
    // Session expired — clear stale data
    if (session) {
      await clearAuthStore();
    }
    return false;
  } catch {
    return false;
  }
}

/**
 * Initiate Google OAuth login.
 * Per D-09: set authLoading for Lottie loading state in login page.
 */
export async function signInWithGoogle(): Promise<void> {
  auth.authLoading = true;
  auth.authError = null;
  try {
    await startOAuth();
    // Note: actual login completes when oauth-callback event fires
    // and handleOAuthCallback is called from the Tauri event handler.
    // Safety timeout: reset loading after 2 minutes if no callback received.
    setTimeout(() => {
      if (auth.authLoading && !auth.isAuthenticated) {
        auth.authLoading = false;
        auth.authError = 'Login timed out. Please try again.';
      }
    }, 120_000);
  } catch (e) {
    auth.authError = String(e);
    auth.authLoading = false;
  }
}

/**
 * Called by deep link callback handler after successful token exchange.
 */
export function setSession(session: AuthSession): void {
  auth.isAuthenticated = true;
  auth.currentUser = session.user;
  auth.tokenExpiresAt = session.tokens.expires_in;
  auth.authLoading = false;
  auth.authError = null;
}

/**
 * Reset auth loading state (e.g., when callback fails or is cancelled).
 */
export function resetAuthLoading(): void {
  auth.authLoading = false;
}

/**
 * Sign out: clear store, reset state, redirect to login.
 */
export async function signOut(): Promise<void> {
  try {
    await logout();
  } catch {
    // remote logout best-effort; local cleanup must still run
  } finally {
    await clearAuthStore();
  }

  auth.isAuthenticated = false;
  auth.currentUser = null;
  auth.tokenExpiresAt = 0;
  auth.authError = null;

  const fallbackRoute = '/login';
  let target: '/' | '/login' = fallbackRoute;
  const protectedPrefixes = ['/counter', '/admin', '/agent', '/settings'];

  if (typeof window !== 'undefined') {
    const referrer = document.referrer;
    if (referrer) {
      try {
        const referrerUrl = new URL(referrer);
        const isProtectedRoute = protectedPrefixes.some((prefix) =>
          referrerUrl.pathname.startsWith(prefix),
        );
        if (referrerUrl.origin === window.location.origin && !isProtectedRoute) {
          if (referrerUrl.pathname === '/') {
            target = '/';
          } else if (referrerUrl.pathname === '/login') {
            target = '/login';
          }
        }
      } catch {
        // fallback to /login
      }
    }
  }

  if (target === '/') {
    await goto('/');
    return;
  }

  await goto('/login');
}

/**
 * Mark auth as expired (called by refresh timer on failure).
 * Per D-07: silent expiry — clear tokens, next action triggers login redirect.
 */
export async function markExpired(): Promise<void> {
  await clearAuthStore();
  auth.isAuthenticated = false;
  auth.currentUser = null;
  auth.tokenExpiresAt = 0;
}

/**
 * Initialize auth event listeners. Call once from root +layout.svelte.
 * Listens for auth:expired event from Rust backend refresh timer.
 */
export function initAuthListeners(): () => void {
  let cleanup: (() => void) | undefined;

  safeListen('auth:expired', () => {
    auth.isAuthenticated = false;
    auth.currentUser = null;
    auth.tokenExpiresAt = 0;
    // Don't auto-redirect — let auth guard handle it on next navigation
  }).then((fn) => { cleanup = fn; });

  return () => { cleanup?.(); };
}
