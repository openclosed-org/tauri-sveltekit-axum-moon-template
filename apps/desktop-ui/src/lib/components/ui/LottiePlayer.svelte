<script lang="ts">
import { cn } from '$lib/utils/cn';
import { DotLottieSvelte } from '@lottiefiles/dotlottie-svelte';
import type { DotLottie } from '@lottiefiles/dotlottie-svelte';

interface Props {
  src: string;
  loop?: boolean;
  autoplay?: boolean;
  speed?: number;
  width?: string | number;
  height?: string | number;
  class?: string;
  dotLottieRefCallback?: (ref: DotLottie) => void;
}

const {
  src,
  loop = true,
  autoplay = true,
  speed = 1,
  width = '100%',
  height = '100%',
  class: className,
  dotLottieRefCallback,
}: Props = $props();
</script>

<div class={cn("inline-block", className)} style="width: {width}; height: {height};">
	<DotLottieSvelte
		{src}
		{loop}
		{autoplay}
		{speed}
		{dotLottieRefCallback}
	/>
</div>
