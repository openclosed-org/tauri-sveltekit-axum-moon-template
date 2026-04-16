<script lang="ts">
import { onMount } from 'svelte';
import '../app.css';
import Toast from '$lib/components/ui/Toast.svelte';
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
  // Listen for Rust panic events (Tauri only)
  let unlistenPanicCleanup: (() => void) | undefined;
  if (isTauri()) {
    safeListen<string>('app:panic', (payload) => {
      showError(payload);
    }).then((cleanup) => {
      unlistenPanicCleanup = cleanup;
    });
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

  return () => {
    unlistenPanicCleanup?.();
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
