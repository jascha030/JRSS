<script lang="ts">
	let { imageUrl, alt, letter }: { imageUrl?: string; alt: string; letter: string } = $props();

	let loaded = $state(false);

	function handleLoad() {
		loaded = true;
	}
</script>

<div class="relative aspect-square w-full overflow-hidden rounded-xl bg-surface-elevated shadow-sm">
	{#if imageUrl}
		<div class="absolute inset-0 animate-pulse bg-surface-elevated" class:hidden={loaded}></div>

		<img
			class:opacity-0={!loaded}
			class:opacity-100={loaded}
			class="size-full object-cover transition-opacity duration-500 ease-out"
			decoding="async"
			draggable="false"
			fetchpriority="low"
			loading="lazy"
			oncontextmenu={(e) => e.preventDefault()}
			onload={handleLoad}
			src={imageUrl}
			{alt}
		/>
	{:else}
		<span
			class="flex size-full items-center justify-center bg-linear-to-br from-primary-400 to-primary-600 text-2xl font-bold text-fg-inverse"
		>
			{letter}
		</span>
	{/if}
</div>
