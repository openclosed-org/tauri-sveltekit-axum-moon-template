<script lang="ts">
import { cn } from '$lib/utils/cn';
import { Switch } from 'bits-ui';

interface Props {
  checked?: boolean;
  class?: string;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
}

// biome-ignore lint/style/useConst: Svelte bind requires mutable variable.
let { checked = $bindable(false), class: className, ...restProps }: Props = $props();
</script>

<Switch.Root
	bind:checked
	class={cn(
		"peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary-600 data-[state=unchecked]:bg-gray-200 dark:data-[state=unchecked]:bg-gray-700",
		className,
	)}
	{...restProps}
>
	<Switch.Thumb
		class="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
	/>
</Switch.Root>
