<script lang="ts">
const {
  src = '',
  loop = true,
  autoplay = true,
}: { src?: string; loop?: boolean; autoplay?: boolean } = $props();
</script>

<div data-testid="lottie-mock" data-src={src}></div>
