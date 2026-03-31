<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import '../app.css';
	import { setSession, initAuthListeners } from '$lib/stores/auth.svelte';
	import { handleOAuthCallback } from '$lib/ipc/auth';
	import Toast from '$lib/components/ui/Toast.svelte';

	const { children } = $props();

	// Error toast state
	let errorMessage = $state('');
	let toastTimeout: ReturnType<typeof setTimeout> | undefined;

	function showError(msg: string) {
		clearTimeout(toastTimeout);
		errorMessage = msg;
		toastTimeout = setTimeout(() => {
			errorMessage = '';
		}, 5000);
	}

	function dismissError() {
		clearTimeout(toastTimeout);
		errorMessage = '';
	}

	onMount(() => {
		// Listen for auth:expired events from refresh timer
		const cleanupAuth = initAuthListeners();

		// Listen for deep link URLs from tauri-plugin-deep-link
		const unlistenDeepLink = listen<string>('deep-link://new-url', async (event) => {
			const url = event.payload;
			if (url.includes('oauth/callback')) {
				try {
					const session = await handleOAuthCallback(url);
					setSession(session);
				} catch (e) {
					console.error('OAuth callback failed:', e);
				}
			}
		});

		// Listen for Rust panic events
		const unlistenPanic = listen<string>('app:panic', (event) => {
			showError(event.payload);
		});

		// Ctrl+Q → quit
		const handleKeydown = (e: KeyboardEvent) => {
			if ((e.ctrlKey || e.metaKey) && e.key === 'q') {
				e.preventDefault();
				invoke('quit_app');
			}
		};
		window.addEventListener('keydown', handleKeydown);

		return () => {
			cleanupAuth?.();
			unlistenDeepLink.then((fn) => fn());
			unlistenPanic.then((fn) => fn());
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

{@render children()}

{#if errorMessage}
	<div class="fixed top-4 right-4 z-50 max-w-md">
		<Toast variant="error">
			<div class="flex items-center justify-between gap-3 w-full">
				<span class="text-sm">{errorMessage}</span>
				<button
					onclick={dismissError}
					class="shrink-0 text-current opacity-60 hover:opacity-100 transition-opacity cursor-pointer"
				>
					✕
				</button>
			</div>
		</Toast>
	</div>
{/if}
