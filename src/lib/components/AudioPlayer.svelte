<script lang="ts">
	import type { FeedListItem, PlaybackState } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';

	type Props = {
		item: FeedListItem | null;
		playbackState: PlaybackState | null;
		onPlayingChange: (isPlaying: boolean) => void;
		onPositionChange: (positionSeconds: number, durationSeconds: number) => void;
		onPositionPersist: (positionSeconds: number, durationSeconds: number) => Promise<void>;
	};

	const PLAYBACK_PERSIST_INTERVAL_SECONDS = 5;

	let { item, playbackState, onPlayingChange, onPositionChange, onPositionPersist }: Props =
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

	const SKIP_SECONDS = 15;

	function skip(deltaSeconds: number) {
		if (!audioElement) {
			return;
		}

		audioElement.currentTime = Math.max(
			0,
			Math.min(audioElement.currentTime + deltaSeconds, durationForPlayer())
		);
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

	/* $effect.pre so we can persist position while audioElement is still in the DOM,
	   before the {#if} block tears it down when item becomes null. */
	$effect.pre(() => {
		if (!audioElement) {
			return;
		}

		if (activeItemId && item?.id !== activeItemId) {
			audioElement.pause();
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
					<button
						class="flex h-9 w-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
						type="button"
						aria-label="Back 15 seconds"
						onclick={() => skip(-SKIP_SECONDS)}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 20 20"
							fill="currentColor"
							class="size-5"
						>
							<path
								fill-rule="evenodd"
								d="M7.793 2.232a.75.75 0 0 1-.025 1.06L3.622 7.25h10.003a5.375 5.375 0 0 1 0 10.75H10.75a.75.75 0 0 1 0-1.5h2.875a3.875 3.875 0 0 0 0-7.75H3.622l4.146 3.957a.75.75 0 0 1-1.036 1.085l-5.5-5.25a.75.75 0 0 1 0-1.085l5.5-5.25a.75.75 0 0 1 1.06.025Z"
								clip-rule="evenodd"
							/>
						</svg>
					</button>
					<button class="btn-primary rounded-xl px-4 py-2" type="button" onclick={togglePlayback}>
						{#if playbackState.isPlaying}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 20 20"
								fill="currentColor"
								class="size-5"
							>
								<path
									d="M5.75 3a.75.75 0 0 0-.75.75v12.5c0 .414.336.75.75.75h1.5a.75.75 0 0 0 .75-.75V3.75A.75.75 0 0 0 7.25 3h-1.5ZM12.75 3a.75.75 0 0 0-.75.75v12.5c0 .414.336.75.75.75h1.5a.75.75 0 0 0 .75-.75V3.75a.75.75 0 0 0-.75-.75h-1.5Z"
								/>
							</svg>
						{:else}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 20 20"
								fill="currentColor"
								class="size-5"
							>
								<path
									d="M6.3 2.84A1.5 1.5 0 0 0 4 4.11v11.78a1.5 1.5 0 0 0 2.3 1.27l9.344-5.891a1.5 1.5 0 0 0 0-2.538L6.3 2.841Z"
								/>
							</svg>
						{/if}
					</button>
					<button
						class="flex h-9 w-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
						type="button"
						aria-label="Forward 15 seconds"
						onclick={() => skip(SKIP_SECONDS)}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 20 20"
							fill="currentColor"
							class="size-5"
						>
							<path
								fill-rule="evenodd"
								d="M12.207 2.232a.75.75 0 0 0 .025 1.06l4.146 3.958H6.375a5.375 5.375 0 0 0 0 10.75H9.25a.75.75 0 0 0 0-1.5H6.375a3.875 3.875 0 0 1 0-7.75h10.003l-4.146 3.957a.75.75 0 0 0 1.036 1.085l5.5-5.25a.75.75 0 0 0 0-1.085l-5.5-5.25a.75.75 0 0 0-1.06.025Z"
								clip-rule="evenodd"
							/>
						</svg>
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
