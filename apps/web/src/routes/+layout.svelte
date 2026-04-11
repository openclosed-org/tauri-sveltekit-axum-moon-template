<script lang="ts">
import { onMount } from 'svelte';
import '../app.css';
import Toast from '$lib/components/ui/Toast.svelte';
import { handleOAuthCallback } from '$lib/ipc/auth';
import { auth, initAuthListeners, setSession } from '$lib/stores/auth.svelte';
import { isTauri, safeInvoke, safeListen } from '$lib/ipc/bridge';

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
  // Listen for auth:expired events from refresh timer (Tauri only)
  const cleanupAuth = isTauri() ? initAuthListeners() : undefined;

  // Listen for OAuth callback from Rust TCP listener (Tauri only)
  let unlistenOAuthCleanup: (() => void) | undefined;
  if (isTauri()) {
    safeListen<string>('oauth-callback', async (payload) => {
      console.log('[auth] Received oauth-callback event:', payload);
      const url = payload;
      if (url.includes('oauth/callback')) {
        try {
          const session = await handleOAuthCallback(url);
          console.log('[auth] OAuth callback succeeded:', session.user.email);
          setSession(session);
        } catch (e) {
          console.error('[auth] OAuth callback failed:', e);
          auth.authLoading = false;
          auth.authError = String(e);
        }
      }
    }).then((cleanup) => { unlistenOAuthCleanup = cleanup; });
  }

  // Listen for Rust panic events (Tauri only)
  let unlistenPanicCleanup: (() => void) | undefined;
  if (isTauri()) {
    safeListen<string>('app:panic', (payload) => {
      showError(payload);
    }).then((cleanup) => { unlistenPanicCleanup = cleanup; });
  }

  // Ctrl+Q → quit (Tauri only)
  const handleKeydown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'q') {
      e.preventDefault();
      if (isTauri()) {
        safeInvoke('quit_app');
      }
    }
  };
  window.addEventListener('keydown', handleKeydown);

  // MCP plugin listeners (Tauri only)
  let mcpCleanup: (() => void) | undefined;
  if (isTauri()) {
    import('tauri-plugin-mcp').then(({ setupPluginListeners, cleanupPluginListeners }) => {
      setupPluginListeners().catch((e) => {
        console.error('[mcp] setupPluginListeners() failed:', e);
      });
      mcpCleanup = cleanupPluginListeners;
    }).catch((e) => {
      console.warn('[mcp] failed to initialize plugin listeners:', e);
    });
  }

  return () => {
    cleanupAuth?.();
    unlistenOAuthCleanup?.();
    unlistenPanicCleanup?.();
    window.removeEventListener('keydown', handleKeydown);
    mcpCleanup?.();
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
