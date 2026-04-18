<script lang="ts">
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';
	import { tick } from 'svelte';
	import { formatDuration } from '$lib/utils/format';
	import { openAudioContextMenu } from '$lib/utils/tauri-menu';
	import Rewind from '@lucide/svelte/icons/rewind';
	import FastForward from '@lucide/svelte/icons/fast-forward';
	import Play from '@lucide/svelte/icons/play';
	import Pause from '@lucide/svelte/icons/pause';
	import VolumeX from '@lucide/svelte/icons/volume-x';
	import Volume1 from '@lucide/svelte/icons/volume-1';
	import Volume2 from '@lucide/svelte/icons/volume-2';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
		onPlayingChange: (isPlaying: boolean) => void;
		onPositionChange: (positionSeconds: number, durationSeconds: number) => void;
		onPositionPersist: (positionSeconds: number, durationSeconds: number) => Promise<void>;
		onTransitionPersist: (itemId: string, positionSeconds: number) => Promise<void>;
		onEnded: () => void;
		onNavigateToItem: () => void;
		controls?: Snippet;
		toggleSeq?: number;
		seekSeq?: number;
		seekToSeconds?: number;
	};

	const PLAYBACK_PERSIST_INTERVAL_SECONDS = 5;
	const SKIP_SECONDS = 15;

	const TITLE_START_DELAY_MS = 1200;
	const TITLE_END_PAUSE_MS = 900;
	const TITLE_RESET_PAUSE_MS = 3000;
	const TITLE_PIXELS_PER_SECOND = 28;
	const TITLE_LOOP_TICK_MS = 16;

	let {
		item,
		imageUrl,
		playbackState,
		onPlayingChange,
		onPositionChange,
		onPositionPersist,
		onTransitionPersist,
		onEnded,
		onNavigateToItem,
		controls,
		toggleSeq,
		seekSeq,
		seekToSeconds
	}: Props = $props();

	let audioElement: HTMLAudioElement | null = $state(null);
	let activeItemId: string | null = $state(null);
	let lastSyncedSecond = $state(-1);
	let lastPersistedSecond = $state(-1);

	let isTransitioning = $state(false);

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

	let titleViewportEl: HTMLDivElement | null = $state(null);
	let titleTextEl: HTMLSpanElement | null = $state(null);

	let titleIsOverflowing = $state(false);
	let titleOverflowDistance = $state(0);
	let titleOffset = $state(0);
	let titleReducedMotion = $state(false);

	let lastMeasuredTitle = $state<string | null>(null);

	let titleLoopToken = 0;
	let titlePaused = false;
	let titleCurrentAnimationFrame: number | null = null;

	function durationForPlayer(): number {
		if (!item) {
			return 0;
		}

		if (audioElement && Number.isFinite(audioElement.duration)) {
			return Math.floor(audioElement.duration);
		}

		return item.mediaEnclosure.durationSeconds ?? 0;
	}

	function syncPlaybackPosition() {
		if (!audioElement || isTransitioning) {
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
		if (!audioElement || isTransitioning) {
			return;
		}

		const currentSecond = Math.floor(audioElement.currentTime);

		lastSyncedSecond = currentSecond;
		lastPersistedSecond = currentSecond;
		onPositionChange(currentSecond, durationForPlayer());
		void onPositionPersist(currentSecond, durationForPlayer());
	}

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

	function cancelTitleAnimationFrame() {
		if (titleCurrentAnimationFrame !== null) {
			cancelAnimationFrame(titleCurrentAnimationFrame);
			titleCurrentAnimationFrame = null;
		}
	}

	function sleep(ms: number) {
		return new Promise<void>((resolve) => {
			window.setTimeout(resolve, ms);
		});
	}

	async function waitWithPause(ms: number, token: number) {
		let remaining = ms;

		while (remaining > 0) {
			if (token !== titleLoopToken) return false;
			if (!titleIsOverflowing) return false;
			if (titleReducedMotion) return false;

			if (titlePaused) {
				await sleep(TITLE_LOOP_TICK_MS);
				continue;
			}

			const slice = Math.min(TITLE_LOOP_TICK_MS, remaining);
			await sleep(slice);
			remaining -= slice;
		}

		return token === titleLoopToken && titleIsOverflowing && !titleReducedMotion;
	}

	async function animateTitleOffset(to: number, durationMs: number, token: number) {
		cancelTitleAnimationFrame();

		if (durationMs <= 0) {
			titleOffset = to;
			return token === titleLoopToken;
		}

		const from = titleOffset;
		const delta = to - from;

		let animationStart: number | null = null;
		let pausedAt: number | null = null;
		let pausedTotal = 0;

		return await new Promise<boolean>((resolve) => {
			const step = (now: number) => {
				if (token !== titleLoopToken || !titleIsOverflowing || titleReducedMotion) {
					titleCurrentAnimationFrame = null;
					resolve(false);
					return;
				}

				if (animationStart === null) {
					animationStart = now;
				}

				if (titlePaused) {
					if (pausedAt === null) {
						pausedAt = now;
					}

					titleCurrentAnimationFrame = requestAnimationFrame(step);
					return;
				}

				if (pausedAt !== null) {
					pausedTotal += now - pausedAt;
					pausedAt = null;
				}

				const elapsed = now - animationStart - pausedTotal;
				const progress = Math.max(0, Math.min(1, elapsed / durationMs));

				titleOffset = from + delta * progress;

				if (progress >= 1) {
					titleOffset = to;
					titleCurrentAnimationFrame = null;
					resolve(true);
					return;
				}

				titleCurrentAnimationFrame = requestAnimationFrame(step);
			};

			titleCurrentAnimationFrame = requestAnimationFrame(step);
		});
	}

	function stopTitleLoop() {
		titleLoopToken += 1;
		cancelTitleAnimationFrame();
		titleOffset = 0;
	}

	async function measureTitleOverflow() {
		await tick();

		if (!titleViewportEl || !titleTextEl || !item?.title) {
			titleIsOverflowing = false;
			titleOverflowDistance = 0;
			stopTitleLoop();
			return;
		}

		const viewportWidth = titleViewportEl.clientWidth;
		const textWidth = titleTextEl.scrollWidth;
		const overflow = Math.max(0, textWidth - viewportWidth);

		titleOverflowDistance = overflow;
		titleIsOverflowing = overflow > 1;

		if (!titleIsOverflowing || titleReducedMotion) {
			stopTitleLoop();
			return;
		}

		void startTitleLoop();
	}

	async function startTitleLoop() {
		const token = ++titleLoopToken;
		cancelTitleAnimationFrame();
		titleOffset = 0;

		const scrollDurationMs = Math.max(
			3000,
			(titleOverflowDistance / TITLE_PIXELS_PER_SECOND) * 1000
		);

		while (token === titleLoopToken && titleIsOverflowing && !titleReducedMotion) {
			titleOffset = 0;

			{
				const ok = await waitWithPause(TITLE_START_DELAY_MS, token);
				if (!ok) return;
			}

			{
				const ok = await animateTitleOffset(titleOverflowDistance, scrollDurationMs, token);
				if (!ok) return;
			}

			{
				const ok = await waitWithPause(TITLE_END_PAUSE_MS, token);
				if (!ok) return;
			}

			titleOffset = 0;

			{
				const ok = await waitWithPause(TITLE_RESET_PAUSE_MS, token);
				if (!ok) return;
			}
		}
	}

	function pauseTitleMarquee() {
		titlePaused = true;
	}

	function resumeTitleMarquee() {
		titlePaused = false;
	}

	$effect(() => {
		if (typeof window === 'undefined') {
			return;
		}

		const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');

		const apply = () => {
			titleReducedMotion = mediaQuery.matches;
			void measureTitleOverflow();
		};

		apply();
		mediaQuery.addEventListener('change', apply);

		return () => mediaQuery.removeEventListener('change', apply);
	});

	$effect(() => {
		if (!audioElement) {
			return;
		}

		audioElement.volume = volume;
		audioElement.muted = isMuted;
	});

	$effect(() => {
		const nextTitle = item?.title ?? null;

		if (nextTitle === lastMeasuredTitle) {
			return;
		}

		lastMeasuredTitle = nextTitle;
		void measureTitleOverflow();
	});

	$effect(() => {
		const viewport = titleViewportEl;
		const text = titleTextEl;

		if (!viewport) {
			return;
		}

		const resizeObserver = new ResizeObserver(() => {
			void measureTitleOverflow();
		});

		resizeObserver.observe(viewport);

		if (text) {
			resizeObserver.observe(text);
		}

		return () => resizeObserver.disconnect();
	});

	$effect(() => {
		return () => {
			stopTitleLoop();
		};
	});

	let lastToggleSeq = $state(0);

	$effect(() => {
		const seq = toggleSeq ?? 0;
		if (seq !== lastToggleSeq) {
			lastToggleSeq = seq;
			togglePlayback();
		}
	});

	let lastSeekSeq = $state(0);

	$effect(() => {
		const seq = seekSeq ?? 0;
		if (seq !== lastSeekSeq) {
			lastSeekSeq = seq;
			if (audioElement && seekToSeconds !== undefined) {
				audioElement.currentTime = seekToSeconds;
			}
		}
	});

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

	$effect.pre(() => {
		if (!audioElement) {
			return;
		}

		if (activeItemId && item?.id !== activeItemId) {
			isTransitioning = true;

			const departingPosition = Math.floor(audioElement.currentTime);
			audioElement.pause();

			void onTransitionPersist(activeItemId, departingPosition);
		}

		if (!item) {
			activeItemId = null;
			isTransitioning = false;
			return;
		}

		if (item.id !== activeItemId) {
			activeItemId = item.id;
			lastSyncedSecond = Math.floor(item.playbackPositionSeconds);
			lastPersistedSecond = Math.floor(item.playbackPositionSeconds);
			audioElement.currentTime = item.playbackPositionSeconds;
			isTransitioning = false;
		}
	});
</script>

{#if item && playbackState}
	<div
		class="sticky bottom-0 border-t border-border bg-surface-glass-heavy px-4 py-4 backdrop-blur"
	>
		<div class="mx-auto flex max-w-6xl items-center gap-6 4xl:max-w-400">
			<div class="flex min-w-0 shrink-0 basis-48 items-center gap-3">
				{#if imageUrl}
					<button
						class="shrink-0"
						type="button"
						onclick={onNavigateToItem}
						oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
					>
						<img
							src={imageUrl}
							alt=""
							class="size-12 rounded-lg object-cover shadow-sm select-none"
						/>
					</button>
				{/if}

				<div class="min-w-0">
					<p class="text-xs font-medium tracking-[0.18em] text-fg-muted uppercase">Now playing</p>

					<button
						class="mt-1 block w-full text-left text-sm font-semibold text-fg transition-colors select-none hover:text-accent focus-visible:text-accent"
						type="button"
						onclick={onNavigateToItem}
						oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
						onmouseenter={pauseTitleMarquee}
						onmouseleave={resumeTitleMarquee}
						onfocus={pauseTitleMarquee}
						onblur={resumeTitleMarquee}
					>
						<div bind:this={titleViewportEl} class="overflow-hidden">
							<span
								bind:this={titleTextEl}
								class:truncate={!titleIsOverflowing || titleReducedMotion}
								class="block whitespace-nowrap will-change-transform"
								style={`transform: translateX(-${titleOffset}px);`}
							>
								{item.title}
							</span>
						</div>
					</button>
				</div>
			</div>

			<div class="flex min-w-0 flex-1 flex-col gap-2">
				<div class="flex items-center justify-center gap-3">
					<button
						class="flex size-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
						type="button"
						aria-label="Back 15 seconds"
						onclick={() => skip(-SKIP_SECONDS)}
					>
						<Rewind class="size-5" />
					</button>

					<button
						class="btn-primary flex size-9 items-center justify-center rounded-xl text-sm"
						type="button"
						onclick={togglePlayback}
					>
						{#if playbackState.isPlaying}
							<Pause class="size-5" />
						{:else}
							<Play class="size-5" />
						{/if}
					</button>

					<button
						class="flex size-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
						type="button"
						aria-label="Forward 15 seconds"
						onclick={() => skip(SKIP_SECONDS)}
					>
						<FastForward class="size-5" />
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

			<div class="flex shrink-0 items-center gap-2">
				<button
					class="flex h-7 w-7 items-center justify-center rounded-lg text-fg-muted transition-colors hover:text-fg"
					type="button"
					onclick={toggleMute}
					aria-label={isMuted ? 'Unmute' : 'Mute'}
				>
					{#if isMuted || effectiveVolume === 0}
						<VolumeX size={16} />
					{:else if effectiveVolume < 0.5}
						<Volume1 size={16} />
					{:else}
						<Volume2 size={16} />
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

				{#if controls}
					<div class="relative ml-1">
						{@render controls()}
					</div>
				{/if}
			</div>
		</div>

		<audio
			bind:this={audioElement}
			onended={() => onEnded()}
			onloadedmetadata={() => {
				syncPlaybackPosition();

				if (playbackState?.autoPlay && audioElement) {
					void audioElement.play();
				}
			}}
			onpause={() => {
				if (!isTransitioning) {
					onPlayingChange(false);
					persistPlaybackPosition();
				}
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

	.player-range::-moz-range-track {
		height: 4px;
		border-radius: 2px;
		background: var(--track);
		border: none;
	}

	.player-range::-moz-range-progress {
		height: 4px;
		border-radius: 2px;
		background: var(--fill);
	}

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

	@media (prefers-reduced-motion: reduce) {
		[style*='will-change: transform'] {
			transform: translateX(0) !important;
		}
	}
</style>
