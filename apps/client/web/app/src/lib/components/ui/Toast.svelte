<script lang="ts">
import { cn } from '$lib/utils/cn';
import type { Snippet } from 'svelte';

type Variant = 'default' | 'success' | 'error' | 'warning';

interface Props {
  variant?: Variant;
  class?: string;
  children: Snippet;
}

const { variant = 'default', class: className, children }: Props = $props();

const variants: Record<Variant, string> = {
  default:
    'border-gray-200 bg-white text-gray-900 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-50',
  success:
    'border-green-200 bg-green-50 text-green-900 dark:border-green-800 dark:bg-green-950 dark:text-green-100',
  error:
    'border-red-200 bg-red-50 text-red-900 dark:border-red-800 dark:bg-red-950 dark:text-red-100',
  warning:
    'border-yellow-200 bg-yellow-50 text-yellow-900 dark:border-yellow-800 dark:bg-yellow-950 dark:text-yellow-100',
};
</script>

<div
	class={cn(
		"pointer-events-auto relative flex w-full items-center justify-between gap-4 rounded-lg border p-4 shadow-lg",
		variants[variant],
		className,
	)}
	role="alert"
>
	{@render children()}
</div>
