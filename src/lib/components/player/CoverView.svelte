<script lang="ts">
	import type { Feed } from '$lib/types/feed';
	import type { MediaListItem } from '$lib/types/item';
	import type { PlaybackState } from '$lib/types/playback';
	import { requestTogglePlayback } from '$lib/state';
	import { getCoverTheme } from '$lib/state/playback.svelte';
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
	import Icon from '@iconify/svelte';
	import Controls from './Controls.svelte';
	import Info from './Info.svelte';
	import SeekBar from './SeekBar.svelte';
	import QueueList from './QueueList.svelte';
	import CoverThemeStyles from './CoverThemeStyles.svelte';
	import VerticalVolume from './VerticalVolume.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
		onNavigateToItem?: () => void;
		onClose?: () => void;
		onPopOut?: () => void;
		class?: string;
		historyItems?: MediaListItem[];
		queueItems?: MediaListItem[];
		feeds?: Feed[];
		onRemoveQueueItem?: (itemId: string) => void;
		onMoveQueueItemUp?: (itemId: string) => void;
		onMoveQueueItemDown?: (itemId: string) => void;
		onClearQueue?: () => void;
	};

	let {
		item,
		imageUrl,
		playbackState,
		onNavigateToItem,
		onClose,
		onPopOut,
		class: className = '',
		historyItems = [],
		queueItems = [],
		feeds = [],
		onRemoveQueueItem,
		onMoveQueueItemUp,
		onMoveQueueItemDown,
		onClearQueue
	}: Props = $props();

	let coverTheme = $derived(getCoverTheme());

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
				if (item && onNavigateToItem) onNavigateToItem();
			}
		}
	]);

	useMediaSession(() => item, handleSkip, previousEpisode, nextEpisode);

	let artworkElement: HTMLImageElement | HTMLDivElement | null = $state(null);
	let artworkSize = $state('auto');

	$effect(() => {
		if (!artworkElement) return;

		const updateSize = () => {
			const width = (artworkElement as HTMLElement)?.offsetWidth;
			if (width) {
				artworkSize = `${width}px`;
			}
		};

		updateSize();

		const observer = new ResizeObserver(updateSize);
		observer.observe(artworkElement);

		return () => observer.disconnect();
	});
</script>

<CoverThemeStyles />

<div
	data-tauri-drag-region
	class="fixed top-0 right-0 left-0 z-9999 h-12 cursor-default select-none"
	aria-hidden="true"
></div>

{#if item && playbackState}
	<div
		class={`cover-theme fixed inset-0 min-h-150 overflow-hidden px-12 ${className}`}
		style:--cover-bg-1={coverTheme.bg1}
		style:--cover-bg-2={coverTheme.bg2}
		style:--cover-bg-3={coverTheme.bg3}
		style:--cover-glow-1={coverTheme.glow1}
		style:--cover-glow-2={coverTheme.glow2}
		style:--cover-glow-3={coverTheme.glow3}
		style:--cover-fg={coverTheme.fg}
		style:--cover-fg-muted={coverTheme.fgMuted}
		style:--cover-fg-subtle={coverTheme.fgSubtle}
		style:--cover-accent={coverTheme.accent}
		style:--cover-accent-contrast={coverTheme.accentContrast}
		style:--cover-panel-bg={coverTheme.panelBg}
		style:--cover-panel-border={coverTheme.panelBorder}
		style:--cover-button-bg={coverTheme.buttonBg}
		style:--cover-button-bg-hover={coverTheme.buttonBgHover}
		style:--color-fg-muted={coverTheme.fgMuted}
		style:--cover-seek-fill={coverTheme.accent}
	>
		<div class="pointer-events-none absolute inset-0">
			<div class="cover-view-backdrop absolute inset-0"></div>
			<div class="cover-view-glow cover-view-glow-1"></div>
			<div class="cover-view-glow cover-view-glow-2"></div>
			<div class="cover-view-glow cover-view-glow-3"></div>
			<div class="cover-view-scrim absolute inset-0"></div>
		</div>

		{#if onClose}
			<button
				type="button"
				class="cover-view-close absolute top-18 left-18 z-30 flex size-12 items-center justify-center rounded-full transition-colors"
				aria-label="Close cover view"
				onclick={onClose}
			>
				<Icon icon="lucide:x" class="size-6" />
			</button>
		{/if}

		{#if onPopOut}
			<button
				type="button"
				class="cover-view-popout absolute top-18 left-28 z-30 flex size-12 items-center justify-center rounded-full transition-colors"
				aria-label="Open mini player"
				onclick={onPopOut}
			>
				<Icon icon="lucide:picture-in-picture-2" class="size-6" />
			</button>
		{/if}

		<div
			class="relative z-10 grid h-full min-h-140 grid-cols-[minmax(0,1.6fr)_minmax(20rem,0.9fr)] items-stretch justify-center gap-16 px-8 py-16"
		>
			<div class="flex h-full min-h-100 min-w-0 flex-col justify-center gap-16 overflow-hidden">
				<div
					class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-4"
					style:--artwork-size={artworkSize}
				>
					<div class="mx-auto flex min-h-0 w-full items-center justify-center p-4">
						{#if imageUrl}
							<img
								bind:this={artworkElement}
								src={imageUrl}
								alt=""
								class="cover-view-artwork aspect-square w-auto max-w-full rounded-4xl object-contain shadow-sm select-none"
							/>
						{:else}
							<div
								bind:this={artworkElement}
								class="cover-view-artwork grid aspect-square max-w-full place-items-center rounded-lg text-(--cover-fg-subtle)"
								style:background-color={coverTheme.panelBg}
							>
								<Icon icon="lucide:disc-3" class="size-16" />
							</div>
						{/if}
					</div>

					<div
						class="controls-row grid min-w-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-4 4xl:max-w-400"
					>
						<div class="min-w-0">
							<Info {item} showCover={false} onNavigate={onNavigateToItem} />
						</div>
					</div>

					<div class="controls-row flex min-w-0 flex-row gap-4">
						<SeekBar {playbackState} durationSeconds={durationForPlayer()} class="mt-1" />
						<VerticalVolume volume={playbackState.volume} />
					</div>

					<div class="controls-row mx:auto">
						<Controls
							durationSeconds={playbackState.durationSeconds ||
								item.mediaEnclosure.durationSeconds ||
								0}
							isPlaying={playbackState.isPlaying}
							skipForwardSeconds={playbackSettings.skipForwardSeconds}
							skipBackwardSeconds={playbackSettings.skipBackwardSeconds}
							onTogglePlayback={requestTogglePlayback}
							onSkip={handleSkip}
							onPreviousEpisode={previousEpisode}
							onNextEpisode={nextEpisode}
							{canSkipPrevious}
							{canSkipNext}
						/>
					</div>

					<!-- <div class="controls-row grid grid-cols-2 xs:grid-cols-3"> -->
					<!-- <div class="flex gap-4 xs:col-start-2 xs:items-center xs:justify-center"></div> -->
					<!-- <div class="flex min-w-0 items-center justify-end gap-2 self-end"></div> -->
					<!-- </div> -->
				</div>
			</div>

			<div class="cover-view-side-panel flex h-full min-h-100 min-w-0 flex-col rounded-2xl">
				<div class="flex h-16 shrink-0 items-center justify-between border-b border-white/10 px-4">
					<div>
						<h2 class="text-sm font-semibold text-white">Playing next</h2>
						<p class="text-xs text-white/60">
							{historyItems.length + queueItems.length}
							{historyItems.length + queueItems.length === 1 ? 'episode' : 'episodes'}
						</p>
					</div>

					{#if queueItems.length > 0 && onClearQueue}
						<button
							type="button"
							class="rounded-lg px-3 py-1.5 text-xs font-medium text-white/60 transition-colors hover:bg-white/10 hover:text-white"
							onclick={onClearQueue}
						>
							Clear
						</button>
					{/if}
				</div>

				<div class="flex-1 overflow-y-auto">
					<QueueList
						{historyItems}
						{queueItems}
						{feeds}
						appearance="inverse"
						rowPaddingClass="px-4"
						separatorPaddingClass="px-4"
						onRemoveItem={onRemoveQueueItem}
						onMoveItemUp={onMoveQueueItemUp}
						onMoveItemDown={onMoveQueueItemDown}
					/>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.cover-theme {
		color: var(--cover-fg);
	}

	.cover-view-backdrop {
		background: linear-gradient(135deg, var(--cover-bg-1), var(--cover-bg-2), var(--cover-bg-3));
	}

	.cover-view-scrim {
		background: linear-gradient(180deg, rgba(2, 6, 23, 0.16) 0%, rgba(2, 6, 23, 0.3) 100%);
		backdrop-filter: blur(80px) saturate(135%);
	}

	.cover-view-glow {
		position: absolute;
		border-radius: 9999px;
		filter: blur(120px);
		opacity: 1;
		pointer-events: none;
	}

	.cover-view-glow-1 {
		top: 6%;
		left: 6%;
		width: 26rem;
		height: 26rem;
		background: var(--cover-glow-1);
	}

	.cover-view-glow-2 {
		top: 18%;
		right: 8%;
		width: 28rem;
		height: 28rem;
		background: var(--cover-glow-2);
	}

	.cover-view-glow-3 {
		bottom: 8%;
		left: 24%;
		width: 32rem;
		height: 32rem;
		background: var(--cover-glow-3);
	}

	.cover-view-close {
		color: var(--cover-fg-muted);
		background: var(--cover-button-bg);
		border: 1px solid var(--cover-panel-border);
		backdrop-filter: blur(18px);
	}

	.cover-view-close:hover {
		color: var(--cover-fg);
		background: var(--cover-button-bg-hover);
	}

	.cover-view-popout {
		color: var(--cover-fg-muted);
		background: var(--cover-button-bg);
		border: 1px solid var(--cover-panel-border);
		backdrop-filter: blur(18px);
	}

	.cover-view-popout:hover {
		color: var(--cover-fg);
		background: var(--cover-button-bg-hover);
	}

	.cover-view-side-panel {
		background: rgba(0, 0, 0, 0.18);
		border: 1px solid rgba(255, 255, 255, 0.12);
		backdrop-filter: blur(22px);
	}

	.cover-view-artwork {
		max-height: min(60vh, calc(100dvh - 22rem));
	}

	.cover-view-side-panel > :global(div:first-child) {
		border-color: rgba(255, 255, 255, 0.14);
	}

	.controls-row {
		width: var(--artwork-size);
		margin-left: auto;
		margin-right: auto;
	}

	@media (max-width: 400px) {
		.controls-row {
			width: 100%;
		}
	}
</style>
