<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { MediaListItem } from '$lib/types/item';
	import type { PlaybackState } from '$lib/types/playback';
	import type { CoverTheme } from '$lib/state/playback.svelte';
	import { requestTogglePlayback } from '$lib/state';
	import { playbackSettings } from '$lib/state/settings.svelte';
	import SeekBar from './player/SeekBar.svelte';
	import Controls from './player/Controls.svelte';
	import Info from './player/Info.svelte';
	import VerticalVolume from './player/VerticalVolume.svelte';

	type Props = {
		item: MediaListItem;
		playbackState: PlaybackState;
		displayImageUrl?: string;
		overlayImageUrl?: string;
		coverTheme: CoverTheme;
		cardHeight: number;
		controlsHeight: number;
		effectiveControlsHeight: number;
		controlsBlurFeather: number;
		controlsBlurFeather85: number;
		controlsBlurFeather70: number;
		controlsBlurFeather55: number;
		controlsBlurFeather40: number;
		controlsBlurFeather25: number;
		controlsBlurFeather12: number;
		canSkipPrevious: boolean;
		canSkipNext: boolean;
		onArtworkError: (event: Event) => void;
		onSkip: (deltaSeconds: number) => void;
		onPreviousEpisode: () => void;
		onNextEpisode: () => void;
	};

	let {
		item,
		playbackState,
		displayImageUrl,
		overlayImageUrl,
		coverTheme,
		cardHeight = $bindable(),
		controlsHeight = $bindable(),
		effectiveControlsHeight,
		controlsBlurFeather,
		controlsBlurFeather85,
		controlsBlurFeather70,
		controlsBlurFeather55,
		controlsBlurFeather40,
		controlsBlurFeather25,
		controlsBlurFeather12,
		canSkipPrevious,
		canSkipNext,
		onArtworkError,
		onSkip,
		onPreviousEpisode,
		onNextEpisode
	}: Props = $props();
</script>

<div
	bind:clientHeight={cardHeight}
	class="group/container cover-card relative inset-0 isolate aspect-square w-full overflow-hidden rounded-lg bg-surface-elevated shadow-lg"
	style:--cover-image={overlayImageUrl ? `url(${JSON.stringify(overlayImageUrl)})` : 'none'}
	style:--controls-height={`${effectiveControlsHeight}px`}
	style:--controls-blur-feather={`${controlsBlurFeather}px`}
	style:--controls-blur-feather-85={`${controlsBlurFeather85}px`}
	style:--controls-blur-feather-70={`${controlsBlurFeather70}px`}
	style:--controls-blur-feather-55={`${controlsBlurFeather55}px`}
	style:--controls-blur-feather-40={`${controlsBlurFeather40}px`}
	style:--controls-blur-feather-25={`${controlsBlurFeather25}px`}
	style:--controls-blur-feather-12={`${controlsBlurFeather12}px`}
>
	{#if displayImageUrl}
		<img
			src={displayImageUrl}
			alt=""
			onerror={onArtworkError}
			class="relative z-0 h-full w-full object-cover"
			draggable="false"
			data-tauri-drag-region
		/>
	{:else}
		<div
			class="relative z-0 flex h-full w-full items-center justify-center bg-surface-elevated"
			data-tauri-drag-region
		>
			<Icon icon="lucide:disc-3" class="size-16 text-fg-muted" />
		</div>
	{/if}

	{#if overlayImageUrl}
		<div
			class="controls-image-blur pointer-events-none absolute inset-0 z-1 opacity-0 transition-opacity duration-200 group-hover/container:opacity-100"
			aria-hidden="true"
		></div>
	{:else}
		<div
			class="pointer-events-none absolute inset-0 z-1 bg-linear-to-t from-black/70 via-black/35 to-transparent opacity-0 transition-opacity duration-200 group-hover/container:opacity-100"
			aria-hidden="true"
		></div>
	{/if}

	<div
		bind:clientHeight={controlsHeight}
		class="cover-theme absolute right-0 bottom-0 left-0 z-10 flex flex-col p-4 opacity-0 transition-opacity duration-200 group-hover/container:opacity-100 xs:px-8"
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
		<div class="flex flex-row gap-2">
			<Info
				{item}
				imageUrl={displayImageUrl}
				showCover={false}
				class="mb-4 w-full justify-center"
			/>

			<div class="min-w-0">
				<VerticalVolume volume={playbackState.volume} />
			</div>
		</div>

		<div class="w-full">
			<SeekBar
				{playbackState}
				durationSeconds={playbackState.durationSeconds || item.mediaEnclosure.durationSeconds || 0}
			/>
		</div>

		<div class="grid grid-cols-3">
			<div class="col-start-2 flex items-center justify-center gap-4">
				<Controls
					isPlaying={playbackState.isPlaying}
					skipForwardSeconds={playbackSettings.skipForwardSeconds}
					skipBackwardSeconds={playbackSettings.skipBackwardSeconds}
					onTogglePlayback={requestTogglePlayback}
					{onSkip}
					{onPreviousEpisode}
					{onNextEpisode}
					{canSkipPrevious}
					{canSkipNext}
				/>
			</div>
		</div>
	</div>
</div>

<style>
	.cover-card {
		--controls-height: 150px;
		--controls-blur-feather: 160px;
		--controls-blur-feather-85: 136px;
		--controls-blur-feather-70: 112px;
		--controls-blur-feather-55: 88px;
		--controls-blur-feather-40: 64px;
		--controls-blur-feather-25: 40px;
		--controls-blur-feather-12: 19px;
	}

	.controls-image-blur {
		overflow: hidden;
		contain: paint;
	}

	.controls-image-blur::before {
		content: '';
		position: absolute;
		inset: -44px;

		background-image: var(--cover-image);
		background-position: center;
		background-size: cover;
		background-repeat: no-repeat;

		filter: blur(24px) saturate(1.08) brightness(0.64);
		transform: scale(1.055) translateZ(0);
		transform-origin: center;

		-webkit-mask-image: linear-gradient(
			to bottom,
			rgb(0 0 0 / 0) calc(100% - var(--controls-height) - var(--controls-blur-feather)),
			rgb(0 0 0 / 0.015) calc(100% - var(--controls-height) - var(--controls-blur-feather-85)),
			rgb(0 0 0 / 0.04) calc(100% - var(--controls-height) - var(--controls-blur-feather-70)),
			rgb(0 0 0 / 0.09) calc(100% - var(--controls-height) - var(--controls-blur-feather-55)),
			rgb(0 0 0 / 0.19) calc(100% - var(--controls-height) - var(--controls-blur-feather-40)),
			rgb(0 0 0 / 0.38) calc(100% - var(--controls-height) - var(--controls-blur-feather-25)),
			rgb(0 0 0 / 0.68) calc(100% - var(--controls-height) - var(--controls-blur-feather-12)),
			rgb(0 0 0 / 1) calc(100% - var(--controls-height)),
			rgb(0 0 0 / 1) 100%
		);

		mask-image: linear-gradient(
			to bottom,
			rgb(0 0 0 / 0) calc(100% - var(--controls-height) - var(--controls-blur-feather)),
			rgb(0 0 0 / 0.015) calc(100% - var(--controls-height) - var(--controls-blur-feather-85)),
			rgb(0 0 0 / 0.04) calc(100% - var(--controls-height) - var(--controls-blur-feather-70)),
			rgb(0 0 0 / 0.09) calc(100% - var(--controls-height) - var(--controls-blur-feather-55)),
			rgb(0 0 0 / 0.19) calc(100% - var(--controls-height) - var(--controls-blur-feather-40)),
			rgb(0 0 0 / 0.38) calc(100% - var(--controls-height) - var(--controls-blur-feather-25)),
			rgb(0 0 0 / 0.68) calc(100% - var(--controls-height) - var(--controls-blur-feather-12)),
			rgb(0 0 0 / 1) calc(100% - var(--controls-height)),
			rgb(0 0 0 / 1) 100%
		);
	}

	.controls-image-blur::after {
		content: '';
		position: absolute;
		inset: 0;

		background: linear-gradient(
			to bottom,
			rgb(0 0 0 / 0) calc(100% - var(--controls-height) - var(--controls-blur-feather)),
			rgb(0 0 0 / 0.012) calc(100% - var(--controls-height) - var(--controls-blur-feather-85)),
			rgb(0 0 0 / 0.035) calc(100% - var(--controls-height) - var(--controls-blur-feather-70)),
			rgb(0 0 0 / 0.075) calc(100% - var(--controls-height) - var(--controls-blur-feather-55)),
			rgb(0 0 0 / 0.14) calc(100% - var(--controls-height) - var(--controls-blur-feather-40)),
			rgb(0 0 0 / 0.26) calc(100% - var(--controls-height) - var(--controls-blur-feather-25)),
			rgb(0 0 0 / 0.44) calc(100% - var(--controls-height) - var(--controls-blur-feather-12)),
			rgb(0 0 0 / 0.62) calc(100% - var(--controls-height)),
			rgb(0 0 0 / 0.68) 100%
		);
	}

	.cover-theme {
		text-shadow: 0 1px 2px rgb(0 0 0 / 0.55);
	}
</style>
