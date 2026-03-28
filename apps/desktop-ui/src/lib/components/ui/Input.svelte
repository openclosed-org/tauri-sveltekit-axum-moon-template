<script lang="ts">
import { cn } from '$lib/utils/cn';

interface Props {
  class?: string;
  type?: string;
  placeholder?: string;
  value?: string;
  disabled?: boolean;
  id?: string;
  name?: string;
  oninput?: (e: Event) => void;
}

// biome-ignore lint/style/useConst: Svelte bind requires mutable variable.
let { class: className, value = $bindable(''), type = 'text', ...restProps }: Props = $props();
</script>

<input
	{type}
	class={cn(
		"flex h-9 w-full rounded-md border border-gray-300 bg-white px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-gray-400 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50 dark:placeholder:text-gray-500 dark:focus-visible:ring-primary-400",
		className,
	)}
	bind:value
	{...restProps}
/>
