<script lang="ts">
	let { src = '', loop = true, autoplay = true }: { src?: string; loop?: boolean; autoplay?: boolean } = $props();
</script>

<div data-testid="lottie-mock" data-src={src}></div>
