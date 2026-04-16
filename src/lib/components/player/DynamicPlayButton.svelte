<script lang="ts">
	import {
		isAudioPlaying,
		isItemCurrentAudio,
		requestTogglePlayback,
		startPlaybackFromContext
	} from '$lib/stores/app.svelte';
	import type { FeedItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';
	import { Pause, Play } from '@lucide/svelte';

	type ButtonSize = 'sm' | 'lg';

	type Props = {
		item: FeedItem;
		size?: ButtonSize;
		compact?: boolean;
	};

	let { item, size = 'lg', compact = false }: Props = $props();

	const isSmall = $derived(size === 'sm');
	const hasProgress = $derived(item.playbackPositionSeconds > 0);

	const progress = $derived(
		hasProgress
			? formatDuration(item.playbackPositionSeconds) +
					(item.mediaEnclosure?.durationSeconds && !compact
						? ` / ${formatDuration(item.mediaEnclosure.durationSeconds)}`
						: '')
			: undefined
	);

	function handleClick() {
		if (isItemCurrentAudio(item.id)) {
			requestTogglePlayback();
		} else {
			startPlaybackFromContext(item);
		}
	}
</script>

<button class="btn-primary flex flex-row rounded-xl align-middle"
class:px-4={!isSmall}
class:py-2={!isSmall}
class:px-2={isSmall}
class:py-1={isSmall}

onclick={handleClick}>
	{#if !isItemCurrentAudio(item.id) || (isItemCurrentAudio(item.id) && !isAudioPlaying())}
		<Play class={isSmall ? 'size-4' : 'size-5'} />
	{:else}
		<Pause class={isSmall ? 'size-4' : 'size-5'} />
	{/if}

	{#if progress}
		<span class="ml-2 tabular-nums" class:text-xs={isSmall} class:text-sm={!isSmall}
			>{progress}</span
		>
	{/if}
</button>
