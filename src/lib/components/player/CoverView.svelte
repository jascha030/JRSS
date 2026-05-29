<script lang="ts">
	import type { MediaListItem } from '$lib/types/item';
	import { getFeedById } from '$lib/state';
	import { getCoverTheme } from '$lib/state/playback.svelte';
	import { playbackSettings } from '$lib/state/settings.svelte';
	import { playbackState as globalPlaybackState } from '$lib/state/playback.svelte';
	import { usePlayerControls } from '$lib/hooks/usePlayerControls.svelte';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import { appUi } from '$lib/hooks/useAppUi.svelte';
	import { navigateToCurrentAudioItem } from '$lib/navigation/audio-nav';
	import { popOutMiniPlayer } from '$lib/utils/mini-player';
	import Icon from '@iconify/svelte';
	import Controls from './Controls.svelte';
	import Info from './Info.svelte';
	import SeekBar from './SeekBar.svelte';
	import QueueList from './QueueList.svelte';
	import CoverThemeStyles from './CoverThemeStyles.svelte';
	import VerticalVolume from './VerticalVolume.svelte';
	import { useArtwork } from '$lib/hooks/useArtwork.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		class?: string;
	};

	let { item, imageUrl, class: className = '' }: Props = $props();

	const player = usePlayerControls(() => item);
	const playbackState = $derived(globalPlaybackState.currentPlaybackState);
	const coverTheme = $derived(getCoverTheme());

	const artwork = useArtwork(
		() => imageUrl,
		() => (item ? getFeedById(item.feedId)?.imageUrl : undefined)
	);

	useMediaSession(() => item, player.handleSkip, player.previousEpisode, player.nextEpisode);

	let artworkFrameWidth = $state(0);
	const artworkSize = $derived(artworkFrameWidth ? `${artworkFrameWidth}px` : '100%');
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

		<button
			type="button"
			class="cover-view-close absolute top-18 left-18 z-30 flex size-12 items-center justify-center rounded-full transition-colors"
			aria-label="Close cover view"
			onclick={() => (appUi.playerMode = 'default')}
		>
			<Icon icon="lucide:x" class="size-6" />
		</button>

		<button
			type="button"
			class="cover-view-popout absolute top-18 left-28 z-30 flex size-12 items-center justify-center rounded-full transition-colors"
			aria-label="Open mini player"
			onclick={() => void popOutMiniPlayer()}
		>
			<Icon icon="lucide:picture-in-picture-2" class="size-6" />
		</button>

		<div
			class="relative z-10 grid h-full min-h-140 grid-cols-[minmax(0,1.6fr)_minmax(20rem,0.9fr)] items-stretch justify-center gap-16 px-8 py-16"
		>
			<div class="flex h-full min-h-100 min-w-0 flex-col justify-center gap-16 overflow-hidden">
				<div
					class="mx-auto flex w-full max-w-6xl flex-col gap-4 px-4"
					style:--artwork-size={artworkSize}
				>
					{#snippet artworkImage(src: string)}
						<img
							{src}
							alt=""
							onerror={artwork.handleError}
							class="cover-view-artwork aspect-square w-auto max-w-full rounded-4xl object-contain shadow-sm select-none"
						/>
					{/snippet}

					{#snippet artworkPlaceholder()}
						<div
							class="cover-view-artwork grid aspect-square max-w-full place-items-center rounded-lg text-(--cover-fg-subtle)"
							style:background-color={coverTheme.panelBg}
						>
							<Icon icon="lucide:disc-3" class="size-16" />
						</div>
					{/snippet}

					<div class="mx-auto flex min-h-0 w-full items-center justify-center p-4">
						<div
							class="cover-view-artwork-frame flex min-h-0 items-center justify-center"
							bind:offsetWidth={artworkFrameWidth}
						>
							{#await artwork.artworkChoice}
								{#if artwork.activeEpisodeUrl}
									{@render artworkImage(artwork.activeEpisodeUrl)}
								{:else if artwork.activeFallbackUrl}
									{@render artworkImage(artwork.activeFallbackUrl)}
								{:else}
									{@render artworkPlaceholder()}
								{/if}
							{:then selectedImageUrl}
								{#if selectedImageUrl}
									{@render artworkImage(selectedImageUrl)}
								{:else}
									{@render artworkPlaceholder()}
								{/if}
							{:catch}
								{#if artwork.activeEpisodeUrl}
									{@render artworkImage(artwork.activeEpisodeUrl)}
								{:else if artwork.activeFallbackUrl}
									{@render artworkImage(artwork.activeFallbackUrl)}
								{:else}
									{@render artworkPlaceholder()}
								{/if}
							{/await}
						</div>
					</div>

					<div
						class="controls-row grid min-w-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-4 4xl:max-w-400"
					>
						<div class="min-w-0">
							<Info {item} showCover={false} onNavigate={navigateToCurrentAudioItem} />
						</div>
					</div>

					<div class="controls-row flex min-w-0 flex-row gap-4">
						<SeekBar {playbackState} durationSeconds={player.durationForPlayer()} class="mt-1" />
						<VerticalVolume volume={playbackState.volume} />
					</div>

					<div class="controls-row mx:auto">
						<Controls
							isPlaying={playbackState.isPlaying}
							skipForwardSeconds={playbackSettings.skipForwardSeconds}
							skipBackwardSeconds={playbackSettings.skipBackwardSeconds}
							onTogglePlayback={player.handleTogglePlayback}
							onSkip={player.handleSkip}
							onPreviousEpisode={player.previousEpisode}
							onNextEpisode={player.nextEpisode}
							canSkipPrevious={player.canSkipPrevious}
							canSkipNext={player.canSkipNext}
						/>
					</div>
				</div>
			</div>

			<div class="cover-view-side-panel flex h-full min-h-100 min-w-0 flex-col rounded-2xl">
				<div class="flex h-16 shrink-0 items-center justify-between border-b border-white/10 px-4">
					<div>
						<h2 class="text-sm font-semibold text-white">Playing next</h2>
						<p class="text-xs text-white/60">
							{globalPlaybackState.playbackHistory.length +
								globalPlaybackState.manualQueue.length +
								globalPlaybackState.autoQueue.length}
							{globalPlaybackState.playbackHistory.length +
								globalPlaybackState.manualQueue.length +
								globalPlaybackState.autoQueue.length ===
							1
								? 'episode'
								: 'episodes'}
						</p>
					</div>

					{#if globalPlaybackState.manualQueue.length > 0}
						<button
							type="button"
							class="rounded-lg px-3 py-1.5 text-xs font-medium text-white/60 transition-colors hover:bg-white/10 hover:text-white"
							onclick={async () => {
								const { clearQueue } = await import('$lib/state');
								clearQueue();
							}}
						>
							Clear
						</button>
					{/if}
				</div>

				<div class="flex-1 overflow-y-auto">
					<QueueList appearance="inverse" rowPaddingClass="px-4" separatorPaddingClass="px-4" />
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
		width: 100%;
		height: 100%;
	}

	.cover-view-artwork-frame {
		width: min(100%, min(60vh, calc(100dvh - 22rem)));
		aspect-ratio: 1;
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
