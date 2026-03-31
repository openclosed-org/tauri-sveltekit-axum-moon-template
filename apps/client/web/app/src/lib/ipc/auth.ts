import { invoke } from '@tauri-apps/api/core';

export interface UserProfile {
	email: string;
	name: string;
	picture: string;
	sub: string;
}

export interface AuthSession {
	access_token: string;
	refresh_token: string;
	id_token: string;
	expires_at: number;
	user: UserProfile;
}

export async function startOAuth(): Promise<void> {
	return invoke('start_oauth');
}

export async function handleOAuthCallback(url: string): Promise<AuthSession> {
	return invoke('handle_oauth_callback', { url });
}

export async function getSession(): Promise<AuthSession | null> {
	return invoke('get_session');
}

export async function clearAuthStore(): Promise<void> {
	const { Store } = await import('@tauri-apps/plugin-store');
	const store = await Store.load('auth.json');
	await store.delete('access_token');
	await store.delete('refresh_token');
	await store.delete('id_token');
	await store.delete('expires_at');
	await store.delete('user');
}
