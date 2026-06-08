<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import Icon from '@iconify/svelte';
	import type { MediaListItem } from '$lib/types/item';
	import type { PlaybackState } from '$lib/types/playback';
	import { getFeedById, requestTogglePlayback } from '$lib/state';
	import { restoreMainWindow, MINI_WINDOW_LABEL } from '$lib/utils/tauri-window';
	import { nextEpisode, previousEpisode, skip } from '$lib/utils/player-controls';
	import { playbackState as globalPlaybackState } from '$lib/state/playback.svelte';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import { getCoverTheme } from '$lib/state/playback.svelte';
	import { invokeCommand } from '$lib/services/tauri';
	import CoverThemeStyles from './player/CoverThemeStyles.svelte';
	import { useArtwork } from '$lib/hooks/useArtwork.svelte';
	import MiniPlayerCoverCard from './MiniPlayerCoverCard.svelte';
	import Info from './player/Info.svelte';
	import Controls from './player/Controls.svelte';
	import SeekBar from './player/SeekBar.svelte';
	import Volume from './player/Volume.svelte';
	import IconButton from './ui/IconButton.svelte';
	import { Avatar } from '@skeletonlabs/skeleton-svelte';

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

	let isCompactMode = $state(false);

	async function toggleCompactMode() {
		isCompactMode = !isCompactMode;
		await invokeCommand('resize_mini_player', {
			label: MINI_WINDOW_LABEL,
			compact: isCompactMode
		});
	}

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

<div class="relative flex h-full w-full flex-col overflow-hidden bg-surface-shell">
	{#if isCompactMode}
		{#if item && playbackState}
			<div class="flex h-full w-full flex-col px-3">
				<div
					class="relative z-0 flex h-7 w-full items-center justify-end"
					data-tauri-drag-region
				></div>
				<div class="flext-row flex gap-4">
					<Avatar class="hidden aspect-square h-[94px] w-[94px] rounded-md shadow-sm sm:block">
						{#if imageUrl}
							<Avatar.Image
								src={artwork.activeEpisodeUrl ?? artwork.activeFallbackUrl}
								alt=""
								class="aspect-square h-[94px] w-[94px]"
							/>
						{/if}
						<Avatar.Fallback
							class="grid size-12 h-full w-full place-items-center text-xs font-semibold"
						>
							♪
						</Avatar.Fallback>
					</Avatar>
					<div class="min-w-0 grow">
						<div class="flex flex-row justify-between gap-2">
							<Info
								{item}
								imageUrl={artwork.activeEpisodeUrl ?? artwork.activeFallbackUrl}
								compact={true}
								showCover={false}
								class="mb-2 justify-center"
							/>

							<div class="min-w-0">
								<Volume showRange={true} volume={playbackState.volume} />
							</div>
						</div>

						<div class="w-full">
							<SeekBar
								{playbackState}
								durationSeconds={playbackState.durationSeconds ||
									item.mediaEnclosure.durationSeconds ||
									0}
							/>
						</div>

						<div class="mt-2 grid grid-cols-3">
							<div class="col-start-2 flex items-center justify-center gap-4">
								<Controls
									isPlaying={playbackState.isPlaying}
									onTogglePlayback={requestTogglePlayback}
									onSkip={handleSkip}
									onPreviousEpisode={canSkipPrevious ? previousEpisode : undefined}
									onNextEpisode={canSkipNext ? nextEpisode : undefined}
									{canSkipPrevious}
									{canSkipNext}
									size="sm"
									class="shrink-0"
								/>
							</div>

							<div class="flex w-full justify-end">
								<button
									class="preset-icon-subtle btn-icon size-5 shrink-0 rounded-xl"
									type="button"
									onclick={toggleCompactMode}
									aria-label="Expand"
								>
									<Icon icon="lucide:maximize" class="size-5" />
								</button>
							</div>
						</div>
					</div>
				</div>
				<!-- <div class="flex items-center gap-3"> -->
				<!---->
				<!---->

				<!---->
				<!-- 	<button -->
				<!-- 		class="preset-icon-subtle btn-icon size-5 shrink-0 rounded-xl" -->
				<!-- 		type="button" -->
				<!-- 		onclick={volumeControl.toggleMute} -->
				<!-- 		aria-label={volumeControl.isMuted ? 'Unmute' : 'Mute'} -->
				<!-- 		aria-pressed={volumeControl.isMuted} -->
				<!-- 	> -->
				<!-- 		{#if volumeControl.isMuted || volumeControl.effectiveVolume === 0} -->
				<!-- 			<Icon icon="lucide:volume-x" class="size-5" /> -->
				<!-- 		{:else if volumeControl.effectiveVolume < 0.5} -->
				<!-- 			<Icon icon="lucide:volume-1" class="size-5" /> -->
				<!-- 		{:else} -->
				<!-- 			<Icon icon="lucide:volume-2" class="size-5" /> -->
				<!-- 		{/if} -->
				<!-- 	</button> -->
				<!---->
				<!-- </div> -->
				<!---->
				<!-- <div class="mt-1"> -->
				<!-- 	<SeekBar -->
				<!-- 		{playbackState} -->
				<!-- 		durationSeconds={item.mediaEnclosure.durationSeconds ?? 0} -->
				<!-- 		showTimeLabels={false} -->
				<!-- 		class="min-w-0" -->
				<!-- 	/> -->
				<!-- </div> -->
			</div>
		{:else}
			<div class="flex h-full w-full items-center justify-between px-4 text-fg-muted">
				<Icon icon="lucide:disc-3" class="size-5" />
				<button
					class="preset-icon-subtle btn-icon size-5 shrink-0 rounded-xl"
					type="button"
					onclick={toggleCompactMode}
					aria-label="Expand"
				>
					<Icon icon="lucide:maximize" class="size-5" />
				</button>
			</div>
		{/if}
	{:else}
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
						onToggleCompactMode={toggleCompactMode}
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
						onToggleCompactMode={toggleCompactMode}
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
						onToggleCompactMode={toggleCompactMode}
					/>
				{/await}
			{:else}
				<div class="flex flex-1 flex-col items-center justify-center text-fg-muted">
					<Icon icon="lucide:disc-3" class="mb-4 size-16" />
					<p class="text-sm">Nothing playing</p>
				</div>
			{/if}
		</div>
	{/if}
</div>
