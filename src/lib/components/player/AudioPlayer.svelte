<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';
	import { requestSeekTo, requestSetVolume, requestTogglePlayback } from '$lib/stores/app.svelte';
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

	const SKIP_SECONDS = 15;
	const VOLUME_STEP = 0.1;

	let { item, imageUrl, playbackState, onNavigateToItem, onShowCover, controls }: Props = $props();

	function durationForPlayer(): number {
		if (playbackState && playbackState.durationSeconds > 0) {
			return playbackState.durationSeconds;
		}
		return item?.mediaEnclosure.durationSeconds ?? 0;
	}

	function skip(deltaSeconds: number) {
		const current = playbackState?.positionSeconds ?? 0;
		const dur = durationForPlayer();
		const target = Math.max(0, Math.min(current + deltaSeconds, dur));
		requestSeekTo(target);
	}

	function togglePlayback() {
		requestTogglePlayback();
	}

	function adjustVolume(delta: number) {
		if (!playbackState) return;
		const newVolume = Math.max(0, Math.min(1, playbackState.volume + delta));
		requestSetVolume(newVolume);
	}

	onMount(() => {
		let unlistenPlayPause: UnlistenFn | undefined;
		let unlistenSkipForward: UnlistenFn | undefined;
		let unlistenSkipBackward: UnlistenFn | undefined;
		let unlistenVolumeUp: UnlistenFn | undefined;
		let unlistenVolumeDown: UnlistenFn | undefined;
		let unlistenGoToFeed: UnlistenFn | undefined;

		const setupListeners = async () => {
			unlistenPlayPause = await listen('menu-play-pause', () => {
				if (item) {
					togglePlayback();
				}
			});
			unlistenSkipForward = await listen('menu-skip-forward', () => {
				if (item) {
					skip(SKIP_SECONDS);
				}
			});
			unlistenSkipBackward = await listen('menu-skip-backward', () => {
				if (item) {
					skip(-SKIP_SECONDS);
				}
			});
			unlistenVolumeUp = await listen('menu-volume-up', () => {
				adjustVolume(VOLUME_STEP);
			});
			unlistenVolumeDown = await listen('menu-volume-down', () => {
				adjustVolume(-VOLUME_STEP);
			});
			unlistenGoToFeed = await listen('menu-go-to-feed', () => {
				if (item) {
					onNavigateToItem();
				}
			});
		};

		void setupListeners();

		return () => {
			if (unlistenPlayPause) unlistenPlayPause();
			if (unlistenSkipForward) unlistenSkipForward();
			if (unlistenSkipBackward) unlistenSkipBackward();
			if (unlistenVolumeUp) unlistenVolumeUp();
			if (unlistenVolumeDown) unlistenVolumeDown();
			if (unlistenGoToFeed) unlistenGoToFeed();
		};
	});

	$effect(() => {
		if (!item || !('mediaSession' in navigator)) {
			return;
		}

		const back = () => skip(-SKIP_SECONDS);
		const forward = () => skip(SKIP_SECONDS);

		navigator.mediaSession.setActionHandler('previoustrack', back);
		navigator.mediaSession.setActionHandler('nexttrack', forward);
		navigator.mediaSession.setActionHandler('seekbackward', back);
		navigator.mediaSession.setActionHandler('seekforward', forward);

		return () => {
			navigator.mediaSession.setActionHandler('previoustrack', null);
			navigator.mediaSession.setActionHandler('nexttrack', null);
			navigator.mediaSession.setActionHandler('seekbackward', null);
			navigator.mediaSession.setActionHandler('seekforward', null);
		};
	});
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
					onTogglePlayback={togglePlayback}
					onSkip={skip}
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
