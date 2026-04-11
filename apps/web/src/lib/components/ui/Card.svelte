<script lang="ts">
import { cn } from '$lib/utils/cn';
import type { Snippet } from 'svelte';

interface Props {
  class?: string;
  children: Snippet;
}

const { class: className, children }: Props = $props();
</script>

<div
	class={cn(
		"rounded-lg border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-900",
		className,
	)}
>
	{@render children()}
</div>
