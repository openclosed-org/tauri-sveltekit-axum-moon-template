<script lang="ts">
import { Button, Input } from '$lib/components';
</script>

<div class="flex min-h-screen w-full flex-col items-center justify-center px-4">
	<div class="w-full max-w-[400px] space-y-8">
		<!-- Logo + Branding -->
		<div class="flex flex-col items-center space-y-3">
			<div class="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary-600 text-white">
				<span class="text-2xl font-bold">T</span>
			</div>
			<div class="space-y-1 text-center">
				<h1 class="text-2xl font-semibold text-[var(--color-text)]">Tauri App</h1>
				<p class="text-sm text-[var(--color-text-muted)]">欢迎使用</p>
			</div>
		</div>

		<!-- Login Actions -->
		<div class="space-y-4">
			<Button variant="primary" size="lg" class="w-full">
				Sign in with Google
			</Button>

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
