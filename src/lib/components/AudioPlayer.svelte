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

	let isSeeking = $state(false);
	let seekPosition = $state(0);
	let volume = $state(1);
	let isMuted = $state(false);

	let displayPosition = $derived(isSeeking ? seekPosition : (playbackState?.positionSeconds ?? 0));

	let seekPercent = $derived.by(() => {
		const dur = durationForPlayer();
		if (dur <= 0) return 0;
		return (displayPosition / dur) * 100;
	});

	let effectiveVolume = $derived(isMuted ? 0 : volume);
	let volumePercent = $derived(effectiveVolume * 100);

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

	function handleSeekInput(event: Event & { currentTarget: HTMLInputElement }) {
		isSeeking = true;
		seekPosition = Number(event.currentTarget.value);
	}

	function handleSeekChange(event: Event & { currentTarget: HTMLInputElement }) {
		if (!audioElement) {
			return;
		}

		const position = Number(event.currentTarget.value);
		audioElement.currentTime = position;
		isSeeking = false;
	}

	function handleVolumeInput(event: Event & { currentTarget: HTMLInputElement }) {
		volume = Number(event.currentTarget.value);
		if (isMuted && volume > 0) {
			isMuted = false;
		}
	}

	function toggleMute() {
		isMuted = !isMuted;
	}

	$effect(() => {
		if (!audioElement) {
			return;
		}

		audioElement.volume = volume;
		audioElement.muted = isMuted;
	});

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
		<div class="mx-auto flex max-w-6xl items-center gap-6">
			<!-- Left: info -->
			<div class="min-w-0 shrink-0 basis-48">
				<p class="text-xs font-medium tracking-[0.18em] text-fg-muted uppercase">Now playing</p>
				<h3 class="mt-1 truncate text-sm font-semibold text-fg">
					{item.title}
				</h3>
			</div>

			<!-- Middle: controls + seek bar -->
			<div class="flex min-w-0 flex-1 flex-col gap-2">
				<div class="flex items-center justify-center gap-3">
					<button class="btn-primary rounded-xl px-4 py-2" type="button" onclick={togglePlayback}>
						{playbackState.isPlaying ? 'Pause' : 'Play'}
					</button>
					<button
						class="btn-secondary rounded-xl px-4 py-2"
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
				<div class="flex items-center gap-3">
					<span class="w-12 text-right text-xs text-fg-muted tabular-nums">
						{formatDuration(displayPosition)}
					</span>
					<input
						class="player-range flex-1"
						max={durationForPlayer()}
						min={0}
						oninput={handleSeekInput}
						onchange={handleSeekChange}
						step={1}
						style="--progress: {seekPercent}%; --fill: var(--color-accent); --track: var(--color-border)"
						type="range"
						value={displayPosition}
					/>
					<span class="w-12 text-xs text-fg-muted tabular-nums">
						{formatDuration(durationForPlayer())}
					</span>
				</div>
			</div>

			<!-- Right: volume -->
			<div class="flex shrink-0 items-center gap-2">
				<button
					class="flex h-7 w-7 items-center justify-center rounded-lg text-fg-muted transition-colors hover:text-fg"
					type="button"
					onclick={toggleMute}
					aria-label={isMuted ? 'Unmute' : 'Mute'}
				>
					{#if isMuted || effectiveVolume === 0}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M11 5 6 9H2v6h4l5 4zM22 9l-6 6M16 9l6 6" />
						</svg>
					{:else if effectiveVolume < 0.5}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M11 5 6 9H2v6h4l5 4z" />
							<path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
						</svg>
					{:else}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M11 5 6 9H2v6h4l5 4z" />
							<path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
							<path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
						</svg>
					{/if}
				</button>
				<input
					class="player-range w-24"
					max={1}
					min={0}
					oninput={handleVolumeInput}
					step={0.01}
					style="--progress: {volumePercent}%; --fill: var(--color-fg-muted); --track: var(--color-border)"
					type="range"
					value={effectiveVolume}
				/>
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

<style>
	.player-range {
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		cursor: pointer;
		height: 20px;
	}

	.player-range:focus {
		outline: none;
	}

	.player-range:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
		border-radius: 2px;
	}

	/* Webkit track */
	.player-range::-webkit-slider-runnable-track {
		height: 4px;
		border-radius: 2px;
		background: linear-gradient(
			to right,
			var(--fill) 0%,
			var(--fill) var(--progress),
			var(--track) var(--progress),
			var(--track) 100%
		);
	}

	/* Webkit thumb */
	.player-range::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--fill);
		margin-top: -5px;
		transition: transform 0.1s ease;
	}

	.player-range:hover::-webkit-slider-thumb {
		transform: scale(1.2);
	}

	/* Firefox track */
	.player-range::-moz-range-track {
		height: 4px;
		border-radius: 2px;
		background: var(--track);
		border: none;
	}

	/* Firefox progress fill */
	.player-range::-moz-range-progress {
		height: 4px;
		border-radius: 2px;
		background: var(--fill);
	}

	/* Firefox thumb */
	.player-range::-moz-range-thumb {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--fill);
		border: none;
		transition: transform 0.1s ease;
	}

	.player-range:hover::-moz-range-thumb {
		transform: scale(1.2);
	}
</style>
