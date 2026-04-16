<script lang="ts">
import { Button } from '$lib/components';
import { Minus, Plus, RotateCcw } from '@jis3r/icons';
import { onMount } from 'svelte';

let count = $state(0);
let loading = $state(false);
let errorMessage = $state('');

interface TauriWindow extends Window {
  __TAURI__?: {
    core: {
      invoke(cmd: string): Promise<number>;
    };
  };
}

async function invokeCommand(cmd: string) {
  const tauriApi = (window as unknown as TauriWindow).__TAURI__;
  if (!tauriApi) {
    throw new Error('Tauri API is not available');
  }
  return tauriApi.core.invoke(cmd);
}

async function loadValue() {
  loading = true;
  errorMessage = '';
  try {
    count = await invokeCommand('counter_get_value');
  } catch (error) {
    console.error('Failed to load persisted counter value', error);
    count = 0;
    errorMessage = 'Failed to load persisted counter value';
  }
  loading = false;
}

async function increment() {
  try {
    count = await invokeCommand('counter_increment');
    errorMessage = '';
  } catch (error) {
    console.error('Failed to increment counter', error);
    errorMessage = 'Failed to increment counter';
  }
}

async function decrement() {
  try {
    count = await invokeCommand('counter_decrement');
    errorMessage = '';
  } catch (error) {
    console.error('Failed to decrement counter', error);
    errorMessage = 'Failed to decrement counter';
  }
}

async function reset() {
  try {
    count = await invokeCommand('counter_reset');
    errorMessage = '';
  } catch (error) {
    console.error('Failed to reset counter', error);
    errorMessage = 'Failed to reset counter';
  }
}

onMount(() => {
  loadValue();
});
</script>

<div class="flex min-h-screen flex-col items-center justify-center gap-8 bg-[var(--color-bg)] px-4">
	<!-- Counter Display -->
	<div class="font-mono text-8xl sm:text-9xl text-[var(--color-text)] tabular-nums py-8 select-none">
		{count}
	</div>

	{#if errorMessage}
		<div class="rounded-md border border-red-300 bg-red-50 px-4 py-2 text-sm text-red-700" role="alert">
			{errorMessage}
		</div>
	{/if}

	<!-- Controls -->
	<div class="flex flex-row items-center gap-4">
		<Button
			variant="secondary"
			size="lg"
			class="h-12 w-12"
			onclick={decrement}
		>
			{#snippet icon()}
				<Minus class="h-5 w-5" />
			{/snippet}
			<span class="sr-only">Decrement counter</span>
		</Button>

		<Button
			variant="primary"
			size="lg"
			class="h-12 w-12"
			onclick={increment}
		>
			{#snippet icon()}
				<Plus class="h-5 w-5" />
			{/snippet}
			<span class="sr-only">Increment counter</span>
		</Button>
	</div>

	<!-- Reset -->
	<Button
		variant="ghost"
		size="md"
		onclick={reset}
	>
		{#snippet icon()}
			<RotateCcw class="h-4 w-4" />
		{/snippet}
		Reset
	</Button>
</div>
