<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import Icon from '@iconify/svelte';
	import type { MediaListItem } from '$lib/types/item';
	import type { PlaybackState } from '$lib/types/playback';
import { getFeedById } from '$lib/state';
	import { restoreMainWindow } from '$lib/utils/tauri-window';
	import { nextEpisode, previousEpisode, skip } from '$lib/utils/player-controls';
	import { playbackState as globalPlaybackState } from '$lib/state/playback.svelte';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import { getCoverTheme } from '$lib/state/playback.svelte';
	import CoverThemeStyles from './player/CoverThemeStyles.svelte';
	import { useArtwork } from '$lib/hooks/useArtwork.svelte';
	import MiniPlayerCoverCard from './MiniPlayerCoverCard.svelte';

	let coverTheme = $derived(getCoverTheme());

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
	};

	let { item, imageUrl, playbackState }: Props = $props();

	const artwork = useArtwork(
		() => imageUrl,
		() => (item ? getFeedById(item.feedId)?.imageUrl : undefined)
	);

	function handleSkip(deltaSeconds: number) {
		skip(playbackState, item?.mediaEnclosure.durationSeconds, deltaSeconds);
	}

	const canSkipPrevious = $derived(globalPlaybackState.playbackHistory.length > 0);
	const canSkipNext = $derived(
		globalPlaybackState.manualQueue.length > 0 || globalPlaybackState.autoQueue.length > 0
	);

	onMount(() => {
		const miniWindow = getCurrentWebviewWindow();

		const unlisten = miniWindow.onCloseRequested(async (event) => {
			event.preventDefault();
			try {
				await restoreMainWindow();
			} catch {
				await miniWindow.destroy();
			}
		});

		return () => {
			unlisten.then((fn) => fn());
		};
	});

	useMediaSession(() => item, handleSkip, previousEpisode, nextEpisode);
</script>

<CoverThemeStyles />

<div class="flex aspect-square h-full w-full flex-col overflow-hidden bg-surface-shell">
	<div class="flex aspect-square h-full w-full flex-1 flex-col items-center justify-center">
		{#if item && playbackState}
			{#await artwork.artworkChoice}
				<MiniPlayerCoverCard
					{item}
					{playbackState}
					displayImageUrl={artwork.activeEpisodeUrl ?? artwork.activeFallbackUrl}
					overlayImageUrl={artwork.activeEpisodeUrl ?? artwork.activeFallbackUrl}
					{coverTheme}
					{canSkipPrevious}
					{canSkipNext}
					onArtworkError={artwork.handleError}
					onSkip={handleSkip}
					onPreviousEpisode={previousEpisode}
					onNextEpisode={nextEpisode}
				/>
			{:then selectedImageUrl}
				<MiniPlayerCoverCard
					{item}
					{playbackState}
					displayImageUrl={selectedImageUrl ??
						artwork.activeEpisodeUrl ??
						artwork.activeFallbackUrl}
					overlayImageUrl={selectedImageUrl ??
						artwork.activeEpisodeUrl ??
						artwork.activeFallbackUrl}
					{coverTheme}
					{canSkipPrevious}
					{canSkipNext}
					onArtworkError={artwork.handleError}
					onSkip={handleSkip}
					onPreviousEpisode={previousEpisode}
					onNextEpisode={nextEpisode}
				/>
			{:catch}
				<MiniPlayerCoverCard
					{item}
					{playbackState}
					displayImageUrl={artwork.activeEpisodeUrl ?? artwork.activeFallbackUrl}
					overlayImageUrl={artwork.activeEpisodeUrl ?? artwork.activeFallbackUrl}
					{coverTheme}
					{canSkipPrevious}
					{canSkipNext}
					onArtworkError={artwork.handleError}
					onSkip={handleSkip}
					onPreviousEpisode={previousEpisode}
					onNextEpisode={nextEpisode}
				/>
			{/await}
		{:else}
			<div class="flex flex-1 flex-col items-center justify-center text-fg-muted">
				<Icon icon="lucide:disc-3" class="mb-4 size-16" />
				<p class="text-sm">Nothing playing</p>
			</div>
		{/if}
	</div>
</div>
