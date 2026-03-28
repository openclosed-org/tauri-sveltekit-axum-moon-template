<script lang="ts">
import { DropdownMenu as BitsDropdown } from 'bits-ui';
import type { Snippet } from 'svelte';

interface Props {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: Snippet;
}

// biome-ignore lint/style/useConst: Svelte bind requires mutable variable.
let { open = $bindable(false), children, ...restProps }: Props = $props();
</script>

<BitsDropdown.Root bind:open {...restProps}>
	{@render children()}
</BitsDropdown.Root>
