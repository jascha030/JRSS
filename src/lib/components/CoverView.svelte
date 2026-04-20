<script lang="ts">
	/**
	 * Cover view component - a full-screen player view showing album art prominently
	 * with the same audio controls as the compact AudioPlayer.
	 */
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';
	import { requestSeekTo, requestTogglePlayback } from '$lib/stores/app.svelte';
	import AudioPlayerControls from './player/AudioPlayerControls.svelte';
	import AudioPlayerInfo from './player/AudioPlayerInfo.svelte';
	import AudioSeekBar from './player/AudioSeekBar.svelte';
	import AudioPlayerVolume from './player/AudioPlayerVolume.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
		onNavigateToItem?: () => void;
		controls?: Snippet;
		class?: string;
	};

	const SKIP_SECONDS = 15;

	let {
		item,
		imageUrl,
		playbackState,
		onNavigateToItem,
		controls,
		class: className = ''
	}: Props = $props();

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

	$effect(() => {
		if (!item) return;

		function handleKeydown(event: KeyboardEvent) {
			if (event.code !== 'Space') return;

			const tag = event.target instanceof Element ? event.target.tagName : '';
			if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || tag === 'BUTTON') return;
			if (event.target instanceof HTMLElement && event.target.isContentEditable) return;

			event.preventDefault();
			togglePlayback();
		}

		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
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
	<div class="fixed inset-0 bg-black">
		<div class="grid h-full grid-cols-4 items-center justify-center gap-8 p-8">
			<div class="col-span-3">
				<div class="flex h-full flex-col items-center justify-center gap-8 p-8 {className}">
					<AudioPlayerInfo
						{item}
						{imageUrl}
						onNavigate={onNavigateToItem}
						imageSize="xl"
						showLabel={false}
						class="flex-col items-center text-center"
					/>

					<div class="mx-auto flex w-full max-w-6xl items-center gap-6 4xl:max-w-400">
						<AudioPlayerControls
							durationSeconds={durationForPlayer()}
							isPlaying={playbackState.isPlaying}
							onTogglePlayback={togglePlayback}
							onSkip={skip}
							skipSeconds={SKIP_SECONDS}
						/>
						<AudioSeekBar {playbackState} durationSeconds={durationForPlayer()} class="mt-1" />
						<AudioPlayerVolume volume={playbackState.volume} />

						{#if controls}
							<div class="relative">
								{@render controls()}
							</div>
						{/if}
					</div>
				</div>
			</div>
		</div>
	</div>
{/if}
