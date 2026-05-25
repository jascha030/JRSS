<script lang="ts">
	import type { MediaListItem } from '$lib/types/item';
	import type { PlaybackState } from '$lib/types/playback';
	import type { Snippet } from 'svelte';

	import {
		VOLUME_STEP,
		adjustVolume,
		nextEpisode,
		previousEpisode,
		skip,
		togglePlayback
	} from '$lib/utils/player-controls';
	import { playbackSettings } from '$lib/state/settings.svelte';
	import { playbackState as globalPlaybackState } from '$lib/state/playback.svelte';
	import { useMenuShortcuts } from '$lib/hooks/useMenuShortcuts.svelte';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import Info from './Info.svelte';
	import Controls from './Controls.svelte';
	import Volume from './Volume.svelte';
	import SeekBar from './SeekBar.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
		onNavigateToItem: () => void;
		onShowCover?: () => void;
		controls?: Snippet;
	};

	let { item, imageUrl, playbackState, onNavigateToItem, onShowCover, controls }: Props = $props();

	function durationForPlayer(): number {
		if (playbackState && playbackState.durationSeconds > 0) {
			return playbackState.durationSeconds;
		}
		return item?.mediaEnclosure.durationSeconds ?? 0;
	}

	function handleSkip(deltaSeconds: number) {
		skip(playbackState, item?.mediaEnclosure.durationSeconds, deltaSeconds);
	}

	function handleTogglePlayback() {
		togglePlayback();
	}

	function handleAdjustVolume(delta: number) {
		adjustVolume(playbackState, delta);
	}

	const canSkipPrevious = $derived(globalPlaybackState.playbackHistory.length > 0);
	const canSkipNext = $derived(
		globalPlaybackState.manualQueue.length > 0 || globalPlaybackState.autoQueue.length > 0
	);

	useMenuShortcuts([
		{
			event: 'menu-play-pause',
			handler: () => {
				if (item) handleTogglePlayback();
			}
		},
		{
			event: 'menu-skip-forward',
			handler: () => {
				if (item) handleSkip(playbackSettings.skipForwardSeconds);
			}
		},
		{
			event: 'menu-skip-backward',
			handler: () => {
				if (item) handleSkip(-playbackSettings.skipBackwardSeconds);
			}
		},
		{
			event: 'menu-next-episode',
			handler: () => {
				if (canSkipNext) nextEpisode();
			}
		},
		{
			event: 'menu-prev-episode',
			handler: () => {
				if (canSkipPrevious) previousEpisode();
			}
		},
		{ event: 'menu-volume-up', handler: () => handleAdjustVolume(VOLUME_STEP) },
		{ event: 'menu-volume-down', handler: () => handleAdjustVolume(-VOLUME_STEP) },
		{
			event: 'menu-go-to-feed',
			handler: () => {
				if (item) onNavigateToItem();
			}
		}
	]);

	useMediaSession(() => item, handleSkip, previousEpisode, nextEpisode);
</script>

{#if item && playbackState}
	<div
		class="sticky bottom-0 z-10 border-t border-border bg-surface-glass-heavy px-4 py-3 backdrop-blur"
	>
		<div class="mx-auto flex max-w-6xl items-center gap-6 4xl:max-w-400">
			<Info
				{item}
				{imageUrl}
				onNavigate={onNavigateToItem}
				{onShowCover}
				class="min-w-0 shrink-0 basis-56"
			/>

			<div class="flex min-w-0 flex-1 items-center gap-4">
				<Controls
					isPlaying={playbackState.isPlaying}
					onTogglePlayback={handleTogglePlayback}
					onSkip={handleSkip}
					onPreviousEpisode={previousEpisode}
					onNextEpisode={nextEpisode}
					{canSkipPrevious}
					{canSkipNext}
					skipForwardSeconds={playbackSettings.skipForwardSeconds}
					skipBackwardSeconds={playbackSettings.skipBackwardSeconds}
					class="shrink-0"
				/>

				<SeekBar {playbackState} durationSeconds={durationForPlayer()} class="min-w-0 flex-1" />

				<Volume volume={playbackState.volume} class="shrink-0" />
			</div>

			{#if controls}
				<div class="relative ml-1 shrink-0">
					{@render controls()}
				</div>
			{/if}
		</div>
	</div>
{/if}
