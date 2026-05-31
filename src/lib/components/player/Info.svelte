<script lang="ts">
	import type { MediaListItem } from '$lib/types/item';
	import { useOverflowMarquee } from '$lib/hooks/useOverflowMarquee.svelte';
	import { Avatar } from '@skeletonlabs/skeleton-svelte';
	import { openAudioContextMenu } from '$lib/utils/tauri-menu';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		onNavigate?: () => void;
		onShowCover?: () => void;
		showCover?: boolean;
		compact?: boolean;
		class?: string;
	};

	let {
		item,
		imageUrl,
		onNavigate,
		onShowCover: onShowCover = undefined,
		showCover = true,
		compact = false,
		class: className = ''
	}: Props = $props();

	let titleViewportEl: HTMLDivElement | null = $state(null);
	let titleTextEl: HTMLSpanElement | null = $state(null);

	const titleMarquee = useOverflowMarquee({
		getViewportElement: () => titleViewportEl,
		getTextElement: () => titleTextEl,
		getText: () => item?.title ?? null
	});
</script>

{#if item}
	<div class={`flex min-w-0 items-center gap-3 ${className}`}>
		{#if showCover}
			<button
				class="shrink-0"
				type="button"
				onclick={onShowCover}
				oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
				aria-label="Show cover art"
			>
				<Avatar class={`w-auto rounded-md shadow-sm ${compact ? 'h-10' : 'h-12'}`}>
					{#if imageUrl}
						<Avatar.Image src={imageUrl} alt="" class="h-full max-h-12 w-full hover:object-cover" />
					{/if}
					<Avatar.Fallback
						class="grid size-12 h-full w-full place-items-center text-xs font-semibold"
					>
						♪
					</Avatar.Fallback>
				</Avatar>
			</button>
		{/if}

		<div class="min-w-0 flex-1">
			{#if !compact}
				<p class="text-[0.65rem] font-semibold tracking-widest text-fg-muted uppercase">
					Now playing
				</p>
			{/if}

			<button
				class="mt-1 block w-full text-left text-sm font-semibold text-fg transition-colors select-none hover:text-accent focus-visible:text-accent"
				type="button"
				onclick={onNavigate}
				oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
				onmouseenter={titleMarquee.pause}
				onmouseleave={titleMarquee.resume}
				onfocus={titleMarquee.pause}
				onblur={titleMarquee.resume}
			>
				<div bind:this={titleViewportEl} class="overflow-hidden">
					<span
						bind:this={titleTextEl}
						class={`block whitespace-nowrap will-change-transform ${!titleMarquee.isOverflowing || titleMarquee.reducedMotion ? 'truncate' : ''}`}
						style={`transform: translateX(-${titleMarquee.offset}px);`}
					>
						{item.title}
					</span>
				</div>
			</button>
		</div>
	</div>
{/if}

<style>
	@media (prefers-reduced-motion: reduce) {
		[style*='will-change: transform'] {
			transform: translateX(0) !important;
		}
	}
</style>
