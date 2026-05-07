<script lang="ts">
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';

	import {
		SKIP_SECONDS,
		VOLUME_STEP,
		adjustVolume,
		skip,
		togglePlayback
	} from '$lib/utils/player-controls';
	import { useMenuShortcuts } from '$lib/hooks/useMenuShortcuts.svelte';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import AudioPlayerInfo from './AudioPlayerInfo.svelte';
	import AudioPlayerControls from './AudioPlayerControls.svelte';
	import AudioPlayerVolume from './AudioPlayerVolume.svelte';
	import AudioSeekBar from './AudioSeekBar.svelte';

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
				if (item) handleSkip(SKIP_SECONDS);
			}
		},
		{
			event: 'menu-skip-backward',
			handler: () => {
				if (item) handleSkip(-SKIP_SECONDS);
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

	useMediaSession(() => item, handleSkip);
</script>

{#if item && playbackState}
	<div
		class="sticky bottom-0 z-10 border-t border-border bg-surface-glass-heavy px-4 py-3 backdrop-blur"
	>
		<div class="mx-auto flex max-w-6xl items-center gap-6 4xl:max-w-400">
			<AudioPlayerInfo
				{item}
				{imageUrl}
				onNavigate={onNavigateToItem}
				{onShowCover}
				class="min-w-0 shrink-0 basis-56"
			/>

			<div class="flex min-w-0 flex-1 items-center gap-4">
				<AudioPlayerControls
					durationSeconds={durationForPlayer()}
					isPlaying={playbackState.isPlaying}
					onTogglePlayback={handleTogglePlayback}
					onSkip={handleSkip}
					skipSeconds={SKIP_SECONDS}
					class="shrink-0"
				/>

				<AudioSeekBar
					{playbackState}
					durationSeconds={durationForPlayer()}
					class="min-w-0 flex-1"
				/>

				<AudioPlayerVolume volume={playbackState.volume} class="shrink-0" />
			</div>

			{#if controls}
				<div class="relative ml-1 shrink-0">
					{@render controls()}
				</div>
			{/if}
		</div>
	</div>
{/if}
