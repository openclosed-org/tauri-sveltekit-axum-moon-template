<script lang="ts">
import { goto } from '$app/navigation';

// Redirect to login page (Phase 6 will add auth-aware routing)
goto('/login');
</script>

<div class="flex min-h-screen items-center justify-center bg-[var(--color-bg)]">
	<a href="/login" class="text-sm text-primary-600 hover:underline">
		Go to Login
	</a>
</div>
