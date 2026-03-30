<script lang="ts">
	import type { FeedItem, PlaybackState } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';

	type Props = {
		item: FeedItem | null;
		playbackState: PlaybackState | null;
		onPlayingChange: (isPlaying: boolean) => void;
		onStop: () => void;
		onTimeChange: (positionSeconds: number, durationSeconds: number) => Promise<void>;
	};

	let { item, playbackState, onPlayingChange, onStop, onTimeChange }: Props = $props();

	let audioElement: HTMLAudioElement | null = $state(null);
	let activeItemId: string | null = $state(null);
	let lastSyncedSecond = $state(-1);

	function durationForPlayer(): number {
		if (!item) {
			return 0;
		}

		if (audioElement && Number.isFinite(audioElement.duration)) {
			return Math.floor(audioElement.duration);
		}

		return item.mediaEnclosure?.durationSeconds ?? 0;
	}

	function syncPlaybackPosition() {
		if (!audioElement) {
			return;
		}

		const currentSecond = Math.floor(audioElement.currentTime);

		if (currentSecond === lastSyncedSecond) {
			return;
		}

		lastSyncedSecond = currentSecond;
		void onTimeChange(currentSecond, durationForPlayer());
	}

	function togglePlayback() {
		if (!audioElement) {
			return;
		}

		if (audioElement.paused) {
			void audioElement.play();
			return;
		}

		audioElement.pause();
	}

	$effect(() => {
		if (!item || !audioElement) {
			return;
		}

		if (item.id !== activeItemId) {
			activeItemId = item.id;
			lastSyncedSecond = Math.floor(item.playbackPositionSeconds);
			audioElement.currentTime = item.playbackPositionSeconds;
			audioElement.pause();
		}
	});
</script>

{#if item && playbackState && item.mediaEnclosure}
	<div
		class="sticky bottom-0 border-t border-slate-200 bg-white/95 px-4 py-4 backdrop-blur dark:border-slate-800 dark:bg-slate-950/95"
	>
		<div
			class="mx-auto flex max-w-6xl flex-col gap-4 md:flex-row md:items-center md:justify-between"
		>
			<div class="min-w-0">
				<p
					class="text-xs font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
				>
					Now playing
				</p>
				<h3 class="mt-1 truncate text-base font-semibold text-slate-950 dark:text-white">
					{item.title}
				</h3>
				<p class="mt-1 text-sm text-slate-500 dark:text-slate-400">
					{formatDuration(playbackState.positionSeconds)} / {formatDuration(durationForPlayer())}
				</p>
			</div>

			<div class="flex items-center gap-3">
				<button
					class="rounded-xl bg-slate-950 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-slate-800 dark:bg-slate-100 dark:text-slate-950 dark:hover:bg-white"
					type="button"
					onclick={togglePlayback}
				>
					{playbackState.isPlaying ? 'Pause' : 'Play'}
				</button>
				<button
					class="rounded-xl border border-slate-300 px-4 py-2.5 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:text-slate-950 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:text-white"
					type="button"
					onclick={() => {
						audioElement?.pause();
						onStop();
					}}
				>
					Close
				</button>
			</div>
		</div>

		<audio
			bind:this={audioElement}
			onended={() => {
				onPlayingChange(false);
				void onTimeChange(0, durationForPlayer());
			}}
			onloadedmetadata={() => {
				syncPlaybackPosition();
			}}
			onpause={() => {
				onPlayingChange(false);
				syncPlaybackPosition();
			}}
			onplay={() => onPlayingChange(true)}
			ontimeupdate={syncPlaybackPosition}
			preload="metadata"
			src={item.mediaEnclosure.url}
		></audio>
	</div>
{/if}
