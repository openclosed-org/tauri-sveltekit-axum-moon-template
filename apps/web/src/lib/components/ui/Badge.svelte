<script lang="ts">
import { cn } from '$lib/utils/cn';
import type { Snippet } from 'svelte';

type Variant = 'default' | 'secondary' | 'success' | 'warning' | 'destructive';

interface Props {
  variant?: Variant;
  class?: string;
  children: Snippet;
}

const { variant = 'default', class: className, children }: Props = $props();

const variants: Record<Variant, string> = {
  default: 'bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200',
  secondary: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200',
  success: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  warning: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  destructive: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
};
</script>

<span
	class={cn(
		"inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium",
		variants[variant],
		className,
	)}
>
	{@render children()}
</span>
