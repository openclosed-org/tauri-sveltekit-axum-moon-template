<script lang="ts">
import { Button } from '$lib/components';
import { Plus, Minus, RotateCcw } from '@jis3r/icons';

let count = $state(0);
</script>

<div class="flex min-h-screen flex-col items-center justify-center gap-8 bg-[var(--color-bg)] px-4">
	<!-- Counter Display -->
	<div class="font-mono text-8xl sm:text-9xl text-[var(--color-text)] tabular-nums py-8 select-none">
		{count}
	</div>

	<!-- Controls -->
	<div class="flex flex-row items-center gap-4">
		<Button
			variant="secondary"
			size="lg"
			class="h-12 w-12"
			onclick={() => count--}
		>
			{#snippet icon()}
				<Minus class="h-5 w-5" />
			{/snippet}
			{''}
		</Button>

		<Button
			variant="primary"
			size="lg"
			class="h-12 w-12"
			onclick={() => count++}
		>
			{#snippet icon()}
				<Plus class="h-5 w-5" />
			{/snippet}
			{''}
		</Button>
	</div>

	<!-- Reset -->
	<Button
		variant="ghost"
		size="md"
		onclick={() => (count = 0)}
	>
		{#snippet icon()}
			<RotateCcw class="h-4 w-4" />
		{/snippet}
		Reset
	</Button>
</div>
