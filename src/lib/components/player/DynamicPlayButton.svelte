<script lang="ts">
	import {
		getPlaybackPositionForItem,
		isAudioLoading,
		isAudioPlaying,
		isItemCurrentAudio,
		requestTogglePlayback,
		startPlaybackFromContext
	} from '$lib/stores/app.svelte';
	import type { MediaItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';
	import Icon from '@iconify/svelte';

	type ButtonSize = 'sm' | 'lg';

	type Props = {
		item: MediaItem;
		size?: ButtonSize;
		compact?: boolean;
	};

	let { item, size = 'lg', compact = false }: Props = $props();

	const isSmall = $derived(size === 'sm');
	const total = $derived(item.mediaEnclosure.durationSeconds ?? 0);
	const isCurrentItem = $derived(isItemCurrentAudio(item.id));
	const isStoreLoading = $derived(isCurrentItem && isAudioLoading());
	const isPlaying = $derived(isCurrentItem && isAudioPlaying());
	const playbackPosition = $derived(
		getPlaybackPositionForItem(item.id, item.playbackPositionSeconds)
	);
	const hasProgress = $derived(playbackPosition > 0);
	const isLoading = $derived(isStoreLoading);

	const progress = $derived(
		compact
			? formatDuration(total - playbackPosition)
			: hasProgress
				? `${formatDuration(playbackPosition)} / ${formatDuration(total)}`
				: formatDuration(total)
	);

	function handleAction() {
		if (isCurrentItem) {
			requestTogglePlayback();
		} else {
			startPlaybackFromContext(item);
		}
	}
</script>

<button
	type="button"
	class="preset-filled-accent btn rounded-xl font-semibold"
	class:btn-sm={isSmall}
	class:gap-1.5={compact}
	class:gap-2={!compact}
	onclick={handleAction}
	disabled={isLoading}
	aria-label={isCurrentItem && isPlaying ? 'Pause audio' : 'Play audio'}
	aria-pressed={isCurrentItem && isPlaying}
>
	{#if isLoading}
		<Icon icon="lucide:loader-2" class={isSmall ? 'size-4 animate-spin' : 'size-5 animate-spin'} />
	{:else if !isCurrentItem || !isPlaying}
		<Icon icon="heroicons:play-20-solid" class={isSmall ? 'size-4' : 'size-5'} />
	{:else}
		<Icon icon="heroicons:pause-20-solid" class={isSmall ? 'size-4' : 'size-5'} />
	{/if}

	{#if progress}
		<span class="tabular-nums">{progress}</span>
	{/if}
</button>
