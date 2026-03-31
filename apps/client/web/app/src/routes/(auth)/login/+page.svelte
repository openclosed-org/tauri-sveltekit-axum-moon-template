<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button, Input, LottiePlayer } from '$lib/components';
	import { auth, signInWithGoogle, checkSession } from '$lib/stores/auth.svelte';

	// D-11: Redirect to /counter if already authenticated
	onMount(async () => {
		const hasSession = await checkSession();
		if (hasSession) {
			goto('/counter');
		}
	});

	// Reactive redirect when auth state changes (e.g., after callback)
	$effect(() => {
		if (auth.isAuthenticated) {
			goto('/counter');
		}
	});
</script>

<!-- D-12: Background decoration animation -->
<div class="fixed inset-0 -z-10 overflow-hidden opacity-20">
	<LottiePlayer
		src="/animations/background.json"
		loop={true}
		autoplay={true}
		width="100%"
		height="100%"
	/>
</div>

<div class="flex min-h-screen w-full flex-col items-center justify-center px-4">
	<div class="w-full max-w-[400px] space-y-8">
		<!-- Logo + Branding -->
		<div class="flex flex-col items-center space-y-3">
			<div class="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary-600 text-white">
				<span class="text-2xl font-bold">T</span>
			</div>
			<div class="space-y-1 text-center">
				<h1 class="text-2xl font-semibold text-[var(--color-text)]">Tauri App</h1>
				<p class="text-sm text-[var(--color-text-muted)]">Welcome back</p>
			</div>
		</div>

		<!-- Login Actions -->
		<div class="space-y-4">
			{#if auth.authLoading}
				<!-- D-09: Loading state — Lottie spinner replaces button -->
				<div class="flex items-center justify-center gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-elevated)] px-4 py-3">
					<div class="h-8 w-8">
						<LottiePlayer
							src="/animations/loading.json"
							loop={true}
							autoplay={true}
							width="32px"
							height="32px"
						/>
					</div>
					<span class="text-sm text-[var(--color-text-muted)]">Opening browser...</span>
				</div>
			{:else}
				<!-- D-09: Idle state — Google sign-in button -->
				<Button
					variant="primary"
					size="lg"
					class="w-full"
					onclick={signInWithGoogle}
				>
					<svg class="mr-2 h-5 w-5" viewBox="0 0 24 24">
						<path fill="currentColor" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/>
						<path fill="currentColor" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
						<path fill="currentColor" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
						<path fill="currentColor" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
					</svg>
					Sign in with Google
				</Button>
			{/if}

			<!-- D-10: Error state — inline error message -->
			{#if auth.authError}
				<div class="rounded-md bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-800 px-4 py-3">
					<p class="text-sm text-red-700 dark:text-red-400">{auth.authError}</p>
				</div>
			{/if}

			<div class="relative">
				<div class="absolute inset-0 flex items-center">
					<span class="w-full border-t border-[var(--color-border)]"></span>
				</div>
				<div class="relative flex justify-center text-xs">
					<span class="bg-[var(--color-bg)] px-2 text-[var(--color-text-muted)]">or</span>
				</div>
			</div>

			<Input
				type="email"
				placeholder="Email (coming soon)"
				disabled
				class="w-full"
			/>
		</div>

		<p class="text-center text-xs text-[var(--color-text-muted)]">
			By signing in, you agree to our Terms of Service.
		</p>
	</div>
</div>
