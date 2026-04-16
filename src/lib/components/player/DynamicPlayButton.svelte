<script lang="ts">
	import {
		isAudioPlaying,
		isItemCurrentAudio,
		requestTogglePlayback,
		startPlaybackFromContext
	} from '$lib/stores/app.svelte';
	import type { MediaItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';
	import { Pause, Play } from '@lucide/svelte';

	type ButtonSize = 'sm' | 'lg';

	type Props = {
		item: MediaItem;
		size?: ButtonSize;
		compact?: boolean;
	};

	let { item, size = 'lg', compact = false }: Props = $props();

	const isSmall = $derived(size === 'sm');
	const hasProgress = $derived(item.playbackPositionSeconds > 0);
	const total = $derived(item.mediaEnclosure.durationSeconds ?? 0);

	const progress = $derived(
		compact
			? formatDuration(total - item.playbackPositionSeconds)
			: hasProgress
				? `${formatDuration(item.playbackPositionSeconds)} / ${formatDuration(total)}`
				: formatDuration(total)
	);

	function handleClick() {
		if (isItemCurrentAudio(item.id)) {
			requestTogglePlayback();
		} else {
			startPlaybackFromContext(item);
		}
	}
</script>

<button
	class="btn-primary flex flex-row rounded-xl align-middle"
	class:btn-sm={isSmall}
	onclick={handleClick}
>
	{#if !isItemCurrentAudio(item.id) || (isItemCurrentAudio(item.id) && !isAudioPlaying())}
		<Play class={isSmall ? 'size-4' : 'size-5'} />
	{:else}
		<Pause class={isSmall ? 'size-4' : 'size-5'} />
	{/if}

	{#if progress}
		<span class="ml-2 tabular-nums">{progress}</span>
	{/if}
</button>
