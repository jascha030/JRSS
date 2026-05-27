<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { listen, type UnlistenFn as EventUnlistenFn } from '@tauri-apps/api/event';
	import Icon from '@iconify/svelte';
	import type { MediaListItem } from '$lib/types/item';
	import type { PlaybackState } from '$lib/types/playback';
	import { getFeedById, requestTogglePlayback } from '$lib/state';
	import { restoreMainWindow } from '$lib/utils/tauri-window';
	import {
		VOLUME_STEP,
		adjustVolume,
		nextEpisode,
		previousEpisode,
		skip
	} from '$lib/utils/player-controls';
	import { playbackSettings } from '$lib/state/settings.svelte';
	import { playbackState as globalPlaybackState } from '$lib/state/playback.svelte';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import { getCoverTheme } from '$lib/state/playback.svelte';
	import CoverThemeStyles from './player/CoverThemeStyles.svelte';
	import { pickBestArtworkUrl } from '$lib/utils/artwork';
	import MiniPlayerCoverCard from './MiniPlayerCoverCard.svelte';

	let coverTheme = $derived(getCoverTheme());

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
	};

	let { item, imageUrl, playbackState }: Props = $props();

	const brokenImageUrls = $state<Record<string, true>>({});
	const feedImageUrl = $derived(item ? getFeedById(item.feedId)?.imageUrl : undefined);
	const episodeImageUrl = $derived(imageUrl && !brokenImageUrls[imageUrl] ? imageUrl : undefined);
	const fallbackImageUrl = $derived(
		feedImageUrl && !brokenImageUrls[feedImageUrl] ? feedImageUrl : undefined
	);
	const previewImageUrl = $derived(episodeImageUrl ?? fallbackImageUrl);
	const artworkChoice = $derived(pickBestArtworkUrl(episodeImageUrl, fallbackImageUrl));

	function handleArtworkError(event: Event) {
		const target = event.currentTarget;
		if (!(target instanceof HTMLImageElement)) return;

		const failedUrl = target.currentSrc || target.src;
		if (!failedUrl) return;

		brokenImageUrls[failedUrl] = true;
	}

	let cardHeight = $state(0);
	let controlsHeight = $state(0);

	const effectiveCardHeight = $derived(cardHeight > 0 ? cardHeight : 320);
	const effectiveControlsHeight = $derived(controlsHeight > 0 ? controlsHeight : 150);

	/*
        This is the vertical transition area ABOVE the controls.
        It is intentionally big enough to avoid the hard line.
    */
	const controlsBlurFeather = $derived(Math.max(112, Math.min(280, effectiveCardHeight * 0.28)));

	/*
        Precomputed fractions.
        Do NOT do calc(var(--x) * 0.8) in CSS here; WebKit can ignore that.
    */
	const controlsBlurFeather85 = $derived(controlsBlurFeather * 0.85);
	const controlsBlurFeather70 = $derived(controlsBlurFeather * 0.7);
	const controlsBlurFeather55 = $derived(controlsBlurFeather * 0.55);
	const controlsBlurFeather40 = $derived(controlsBlurFeather * 0.4);
	const controlsBlurFeather25 = $derived(controlsBlurFeather * 0.25);
	const controlsBlurFeather12 = $derived(controlsBlurFeather * 0.12);

	function handleSkip(deltaSeconds: number) {
		skip(playbackState, item?.mediaEnclosure.durationSeconds, deltaSeconds);
	}

	function handleAdjustVolume(delta: number) {
		adjustVolume(playbackState, delta);
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

		let unlistenSkipForward: EventUnlistenFn | undefined;
		let unlistenSkipBackward: EventUnlistenFn | undefined;
		let unlistenNextEpisode: EventUnlistenFn | undefined;
		let unlistenPrevEpisode: EventUnlistenFn | undefined;
		let unlistenVolumeUp: EventUnlistenFn | undefined;
		let unlistenVolumeDown: EventUnlistenFn | undefined;
		let unlistenSettings: EventUnlistenFn | undefined;
		let unlistenToggleMiniPlayer: EventUnlistenFn | undefined;

		const setupListeners = async () => {
			unlistenSkipForward = await listen('menu-skip-forward', () => {
				if (item) handleSkip(playbackSettings.skipForwardSeconds);
			});

			unlistenSkipBackward = await listen('menu-skip-backward', () => {
				if (item) handleSkip(-playbackSettings.skipBackwardSeconds);
			});

			unlistenNextEpisode = await listen('menu-next-episode', () => {
				if (canSkipNext) nextEpisode();
			});

			unlistenPrevEpisode = await listen('menu-prev-episode', () => {
				if (canSkipPrevious) previousEpisode();
			});

			unlistenVolumeUp = await listen('menu-volume-up', () => {
				handleAdjustVolume(VOLUME_STEP);
			});

			unlistenVolumeDown = await listen('menu-volume-down', () => {
				handleAdjustVolume(-VOLUME_STEP);
			});

			unlistenSettings = await listen('menu-settings', async () => {
				try {
					await restoreMainWindow();
				} catch {
					await miniWindow.destroy();
				}
			});

			unlistenToggleMiniPlayer = await listen('menu-toggle-mini-player', async () => {
				try {
					await restoreMainWindow();
				} catch {
					await miniWindow.destroy();
				}
			});
		};

		void setupListeners();

		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
				return;
			}

			if (e.key === ' ') {
				e.preventDefault();

				if (item) {
					requestTogglePlayback();
				}
			}
		};

		document.addEventListener('keydown', handleKeyDown);

		return () => {
			unlisten.then((fn) => fn());
			document.removeEventListener('keydown', handleKeyDown);

			if (unlistenSkipForward) unlistenSkipForward();
			if (unlistenSkipBackward) unlistenSkipBackward();
			if (unlistenNextEpisode) unlistenNextEpisode();
			if (unlistenPrevEpisode) unlistenPrevEpisode();
			if (unlistenVolumeUp) unlistenVolumeUp();
			if (unlistenVolumeDown) unlistenVolumeDown();
			if (unlistenSettings) unlistenSettings();
			if (unlistenToggleMiniPlayer) unlistenToggleMiniPlayer();
		};
	});

	useMediaSession(() => item, handleSkip, previousEpisode, nextEpisode);
</script>

<CoverThemeStyles />

<div class="flex aspect-square h-full w-full flex-col overflow-hidden bg-surface-shell">
	<div class="flex aspect-square h-full w-full flex-1 flex-col items-center justify-center">
		{#if item && playbackState}
			{#await artworkChoice}
				<MiniPlayerCoverCard
					{item}
					{playbackState}
					displayImageUrl={previewImageUrl}
					overlayImageUrl={previewImageUrl}
					{coverTheme}
					bind:cardHeight
					bind:controlsHeight
					{effectiveControlsHeight}
					{controlsBlurFeather}
					{controlsBlurFeather85}
					{controlsBlurFeather70}
					{controlsBlurFeather55}
					{controlsBlurFeather40}
					{controlsBlurFeather25}
					{controlsBlurFeather12}
					{canSkipPrevious}
					{canSkipNext}
					onArtworkError={handleArtworkError}
					onSkip={handleSkip}
					onPreviousEpisode={previousEpisode}
					onNextEpisode={nextEpisode}
				/>
			{:then selectedImageUrl}
				<MiniPlayerCoverCard
					{item}
					{playbackState}
					displayImageUrl={selectedImageUrl}
					overlayImageUrl={selectedImageUrl}
					{coverTheme}
					bind:cardHeight
					bind:controlsHeight
					{effectiveControlsHeight}
					{controlsBlurFeather}
					{controlsBlurFeather85}
					{controlsBlurFeather70}
					{controlsBlurFeather55}
					{controlsBlurFeather40}
					{controlsBlurFeather25}
					{controlsBlurFeather12}
					{canSkipPrevious}
					{canSkipNext}
					onArtworkError={handleArtworkError}
					onSkip={handleSkip}
					onPreviousEpisode={previousEpisode}
					onNextEpisode={nextEpisode}
				/>
			{:catch}
				<MiniPlayerCoverCard
					{item}
					{playbackState}
					displayImageUrl={previewImageUrl}
					overlayImageUrl={previewImageUrl}
					{coverTheme}
					bind:cardHeight
					bind:controlsHeight
					{effectiveControlsHeight}
					{controlsBlurFeather}
					{controlsBlurFeather85}
					{controlsBlurFeather70}
					{controlsBlurFeather55}
					{controlsBlurFeather40}
					{controlsBlurFeather25}
					{controlsBlurFeather12}
					{canSkipPrevious}
					{canSkipNext}
					onArtworkError={handleArtworkError}
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
