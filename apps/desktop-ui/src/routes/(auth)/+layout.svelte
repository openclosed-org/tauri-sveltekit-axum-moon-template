<script lang="ts">
import '../app.css';
const { children } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-[var(--color-bg)]">
	{@render children()}
</div>
