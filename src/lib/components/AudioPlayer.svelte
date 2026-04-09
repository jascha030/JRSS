<script lang="ts">
	import type { FeedListItem, PlaybackState } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';

	type Props = {
		item: FeedListItem | null;
		playbackState: PlaybackState | null;
		onPlayingChange: (isPlaying: boolean) => void;
		onPositionChange: (positionSeconds: number, durationSeconds: number) => void;
		onPositionPersist: (positionSeconds: number, durationSeconds: number) => Promise<void>;
		onStop: () => void;
	};

	const PLAYBACK_PERSIST_INTERVAL_SECONDS = 5;

	let { item, playbackState, onPlayingChange, onPositionChange, onPositionPersist, onStop }: Props =
		$props();

	let audioElement: HTMLAudioElement | null = $state(null);
	let activeItemId: string | null = $state(null);
	let lastSyncedSecond = $state(-1);
	let lastPersistedSecond = $state(-1);

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
		onPositionChange(currentSecond, durationForPlayer());

		if (
			lastPersistedSecond < 0 ||
			Math.abs(currentSecond - lastPersistedSecond) >= PLAYBACK_PERSIST_INTERVAL_SECONDS
		) {
			lastPersistedSecond = currentSecond;
			void onPositionPersist(currentSecond, durationForPlayer());
		}
	}

	function persistPlaybackPosition() {
		if (!audioElement) {
			return;
		}

		const currentSecond = Math.floor(audioElement.currentTime);

		lastSyncedSecond = currentSecond;
		lastPersistedSecond = currentSecond;
		onPositionChange(currentSecond, durationForPlayer());
		void onPositionPersist(currentSecond, durationForPlayer());
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
		if (!audioElement) {
			return;
		}

		if (activeItemId && item?.id !== activeItemId) {
			persistPlaybackPosition();
		}

		if (!item) {
			return;
		}

		if (item.id !== activeItemId) {
			activeItemId = item.id;
			lastSyncedSecond = Math.floor(item.playbackPositionSeconds);
			lastPersistedSecond = Math.floor(item.playbackPositionSeconds);
			audioElement.currentTime = item.playbackPositionSeconds;
			audioElement.pause();
		}
	});
</script>

{#if item && playbackState && item.mediaEnclosure}
	<div
		class="sticky bottom-0 border-t border-border bg-surface-glass-heavy px-4 py-4 backdrop-blur"
	>
		<div
			class="mx-auto flex max-w-6xl flex-col gap-4 md:flex-row md:items-center md:justify-between"
		>
			<div class="min-w-0">
				<p class="text-xs font-medium tracking-[0.18em] text-fg-muted uppercase">Now playing</p>
				<h3 class="mt-1 truncate text-base font-semibold text-fg">
					{item.title}
				</h3>
				<p class="mt-1 text-sm text-fg-muted">
					{formatDuration(playbackState.positionSeconds)} / {formatDuration(durationForPlayer())}
				</p>
			</div>

			<div class="flex items-center gap-3">
				<button class="btn-primary rounded-xl px-4 py-2.5" type="button" onclick={togglePlayback}>
					{playbackState.isPlaying ? 'Pause' : 'Play'}
				</button>
				<button
					class="btn-secondary rounded-xl px-4 py-2.5"
					type="button"
					onclick={() => {
						audioElement?.pause();
						persistPlaybackPosition();
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
				onPositionChange(0, durationForPlayer());
				void onPositionPersist(0, durationForPlayer());
			}}
			onloadedmetadata={() => {
				syncPlaybackPosition();
			}}
			onpause={() => {
				onPlayingChange(false);
				persistPlaybackPosition();
			}}
			onplay={() => onPlayingChange(true)}
			ontimeupdate={syncPlaybackPosition}
			preload="metadata"
			src={item.mediaEnclosure.url}
		></audio>
	</div>
{/if}
