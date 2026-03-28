<script lang="ts">
import { Select as BitsSelect } from 'bits-ui';
import type { Snippet } from 'svelte';

interface Props {
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  name?: string;
  required?: boolean;
  open?: boolean;
  children: Snippet;
}

// biome-ignore lint/style/useConst: Svelte bind requires mutable variable.
let { value = $bindable(''), children, ...restProps }: Props = $props();
</script>

<BitsSelect.Root type="single" bind:value {...restProps}>
	{@render children()}
</BitsSelect.Root>
