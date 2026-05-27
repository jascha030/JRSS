<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from '@iconify/svelte';
	import type { PageTransitionPhase } from '$lib/hooks/usePageTransition.svelte';

	type Props = {
		phase: PageTransitionPhase;
		children: Snippet;
	};

	let { phase, children }: Props = $props();

	const isIdle = $derived(phase === 'idle');
	const isFadingOut = $derived(phase === 'fading-out');
	const isSpinner = $derived(phase === 'spinner');
	const isFadingIn = $derived(phase === 'fading-in');
</script>

<div class="relative h-full w-full">
	<div
		class="h-full w-full transition-[opacity,filter] duration-180 ease-out motion-reduce:transition-none"
		class:opacity-100={isIdle || isFadingIn}
		class:opacity-0={isFadingOut || isSpinner}
		class:blur-none={isIdle || isFadingIn}
		class:blur-[2px]={isFadingOut || isSpinner}
	>
		{@render children()}
	</div>

	{#if isSpinner || isFadingIn}
		<div
			class="pointer-events-none absolute inset-0 flex items-center justify-center transition-opacity duration-200 ease-out motion-reduce:transition-none"
			class:opacity-0={isFadingIn}
			class:opacity-100={isSpinner}
		>
			<div
				class="flex flex-col items-center gap-3 rounded-2xl bg-surface-glass-heavy px-8 py-6 shadow-xl backdrop-blur-xl"
			>
				<Icon icon="lucide:loader-2" class="size-8 animate-spin text-fg-muted" />
				<span class="text-sm font-medium text-fg-muted">Loading</span>
			</div>
		</div>
	{/if}
</div>
