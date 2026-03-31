<script lang="ts">
	import { page } from '$app/state';
	import { Button } from '$lib/components';

	// SvelteKit passes handleError return value as page.error
	// Our handleError returns { message: string, errorId: string }
	const error = $derived(page.error as { message?: string; errorId?: string } | null);
	const status = $derived(page.status);
</script>

<svelte:head>
	<title>{status} — Error</title>
</svelte:head>

<div class="flex min-h-screen w-full flex-col items-center justify-center px-4">
	<div class="w-full max-w-[480px] space-y-6 text-center">
		<!-- Status code -->
		<div class="text-6xl font-bold text-[var(--color-text-muted)]">{status}</div>

		<!-- Error message -->
		<div class="space-y-2">
			<h1 class="text-xl font-semibold text-[var(--color-text)]">
				{#if status === 404}
					Page not found
				{:else}
					Something went wrong
				{/if}
			</h1>

			{#if error?.message && status !== 404}
				<p class="text-sm text-[var(--color-text-muted)]">{error.message}</p>
			{/if}

			<!-- Error tracking ID (for bug reports) -->
			{#if error?.errorId}
				<p class="font-mono text-xs text-[var(--color-text-muted)]">
					Error ID: {error.errorId}
				</p>
			{/if}
		</div>

		<!-- Actions -->
		<div class="flex items-center justify-center gap-3">
			<Button variant="secondary" onclick={() => history.back()}>Go back</Button>
			<Button variant="primary" onclick={() => (window.location.href = '/')}>Home</Button>
		</div>
	</div>
</div>
