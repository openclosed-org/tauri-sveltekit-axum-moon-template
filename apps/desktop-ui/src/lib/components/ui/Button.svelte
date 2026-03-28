<script lang="ts">
import { cn } from '$lib/utils/cn';
import { Button } from 'bits-ui';
import type { Snippet } from 'svelte';

type Variant = 'primary' | 'secondary' | 'ghost' | 'destructive';
type Size = 'sm' | 'md' | 'lg';

interface Props {
  variant?: Variant;
  size?: Size;
  class?: string;
  disabled?: boolean;
  href?: string;
  onclick?: () => void;
  icon?: Snippet;
  iconEnd?: Snippet;
  children: Snippet;
}

const {
  variant = 'primary',
  size = 'md',
  class: className,
  icon,
  iconEnd,
  children,
  ...restProps
}: Props = $props();

const variants: Record<Variant, string> = {
  primary: 'bg-primary-600 text-white hover:bg-primary-700 focus-visible:ring-primary-500',
  secondary:
    'border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700',
  ghost: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800',
  destructive: 'bg-red-600 text-white hover:bg-red-700 focus-visible:ring-red-500',
};

const sizes: Record<Size, string> = {
  sm: 'h-8 px-3 text-sm',
  md: 'h-9 px-4 text-sm',
  lg: 'h-10 px-5 text-base',
};

const iconSizes: Record<Size, number> = {
  sm: 14,
  md: 16,
  lg: 18,
};
</script>

<Button.Root
	class={cn(
		"inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
		variants[variant],
		sizes[size],
		className,
	)}
	{...restProps}
>
	{#if icon}
		<span class="shrink-0">
			{@render icon()}
		</span>
	{/if}
	{@render children()}
	{#if iconEnd}
		<span class="shrink-0">
			{@render iconEnd()}
		</span>
	{/if}
</Button.Root>
