import type { TokenPair } from '$lib/generated/auth/TokenPair';
import type { UserProfile } from '$lib/generated/auth/UserProfile';
import { isTauri, safeInvoke } from '$lib/ipc/bridge';

/**
 * Full auth session combining generated TokenPair + UserProfile.
 * This is a frontend composition type — not defined in contracts
 * because it aggregates multiple contract types for client-side use.
 */
export interface AuthSession {
  tokens: TokenPair;
  id_token: string;
  user: UserProfile;
}

export async function startOAuth(): Promise<void> {
  return safeInvoke('start_oauth') as Promise<void>;
}

export async function handleOAuthCallback(url: string): Promise<AuthSession> {
  return safeInvoke('handle_oauth_callback', { url }) as Promise<AuthSession>;
}

export async function getSession(): Promise<AuthSession | null> {
  if (isTauri()) {
    return safeInvoke('get_session') as Promise<AuthSession | null>;
  }
  // Web mode: read id_token from localStorage
  const idToken = localStorage.getItem('auth_id_token');
  if (!idToken) return null;
  return {
    tokens: { access_token: '', refresh_token: '', expires_in: 0 },
    id_token: idToken,
    user: { sub: '', email: '', name: '', picture: '' },
  } as unknown as AuthSession;
}

export async function logout(): Promise<void> {
  return safeInvoke('logout') as Promise<void>;
}

export async function clearAuthStore(): Promise<void> {
  if (isTauri()) {
    const { Store } = await import('@tauri-apps/plugin-store');
    const store = await Store.load('auth.json');
    await store.delete('tokens');
    await store.delete('id_token');
    await store.delete('user');
  } else {
    // Web mode: clear localStorage keys
    localStorage.removeItem('tokens');
    localStorage.removeItem('id_token');
    localStorage.removeItem('user');
    localStorage.removeItem('auth_id_token');
  }
}
