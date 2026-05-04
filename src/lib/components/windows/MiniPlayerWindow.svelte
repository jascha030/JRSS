<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { listen, type UnlistenFn as EventUnlistenFn } from '@tauri-apps/api/event';
	import Icon from '@iconify/svelte';
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import { requestSeekTo, requestTogglePlayback, requestSetVolume } from '$lib/stores/app.svelte';
	import { restoreMainWindow } from '$lib/utils/tauri-window';
	import AudioSeekBar from '../player/AudioSeekBar.svelte';
	import AudioPlayerControls from '../player/AudioPlayerControls.svelte';
	import AudioPlayerVolume from '../player/AudioPlayerVolume.svelte';
	import AudioPlayerInfo from '../player/AudioPlayerInfo.svelte';
	import { getCoverTheme } from '$lib/state/playback.svelte';

	let coverTheme = $derived(getCoverTheme());

	const SKIP_SECONDS = 15;
	const VOLUME_STEP = 0.1;

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
	};

	let { item, imageUrl, playbackState }: Props = $props();

	function skip(deltaSeconds: number) {
		const current = playbackState?.positionSeconds ?? 0;
		const duration = playbackState?.durationSeconds ?? 0;
		const target = Math.max(0, Math.min(current + deltaSeconds, duration));
		requestSeekTo(target);
	}

	function adjustVolume(delta: number) {
		if (!playbackState) return;
		const newVolume = Math.max(0, Math.min(1, playbackState.volume + delta));
		requestSetVolume(newVolume);
	}

	onMount(() => {
		const miniWindow = getCurrentWebviewWindow();
		const unlisten = miniWindow.onCloseRequested(async (event) => {
			event.preventDefault();
			await restoreMainWindow();
			await miniWindow.destroy();
		});

		// Listen for modifier-based shortcuts from global shortcuts (skip/volume/settings)
		let unlistenSkipForward: EventUnlistenFn | undefined;
		let unlistenSkipBackward: EventUnlistenFn | undefined;
		let unlistenVolumeUp: EventUnlistenFn | undefined;
		let unlistenVolumeDown: EventUnlistenFn | undefined;
		let unlistenSettings: EventUnlistenFn | undefined;

		const setupListeners = async () => {
			unlistenSkipForward = await listen('menu-skip-forward', () => {
				if (item) skip(SKIP_SECONDS);
			});
			unlistenSkipBackward = await listen('menu-skip-backward', () => {
				if (item) skip(-SKIP_SECONDS);
			});
			unlistenVolumeUp = await listen('menu-volume-up', () => {
				adjustVolume(VOLUME_STEP);
			});
			unlistenVolumeDown = await listen('menu-volume-down', () => {
				adjustVolume(-VOLUME_STEP);
			});
			unlistenSettings = await listen('menu-settings', async () => {
				// Close mini-player - main window will handle the rest
				await restoreMainWindow();
				await miniWindow.destroy();
			});
		};

		void setupListeners();

		// Handle Space key directly (no modifier, can't be global shortcut)
		const handleKeyDown = (e: KeyboardEvent) => {
			// Don't trigger if user is typing in an input
			if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
				return;
			}

			if (e.key === ' ') {
				e.preventDefault();
				if (item) requestTogglePlayback();
			}
		};

		document.addEventListener('keydown', handleKeyDown);

		return () => {
			unlisten.then((fn) => fn());
			document.removeEventListener('keydown', handleKeyDown);
			if (unlistenSkipForward) unlistenSkipForward();
			if (unlistenSkipBackward) unlistenSkipBackward();
			if (unlistenVolumeUp) unlistenVolumeUp();
			if (unlistenVolumeDown) unlistenVolumeDown();
			if (unlistenSettings) unlistenSettings();
		};
	});
</script>

<div class="flex aspect-square h-full w-full flex-col overflow-hidden bg-surface-shell">
	<!-- Main content with square album art -->
	<div class="flex aspect-square h-full w-full flex-1 flex-col items-center justify-center">
		{#if item && playbackState}
			<!-- Square album art -->
			<div class="group relative inset-0 aspect-square w-full overflow-hidden rounded-lg shadow-lg">
				{#if imageUrl}
					<img
						src={imageUrl}
						alt=""
						class="h-full w-full object-cover"
						draggable="false"
						data-tauri-drag-region
					/>
				{:else}
					<div
						class="bg-surface-elevated flex h-full w-full items-center justify-center"
						data-tauri-drag-region
					>
						<Icon icon="lucide:disc-3" class="size-16 text-fg-muted" />
					</div>
				{/if}

				<div
					class="mini-player-theme absolute right-0 bottom-0 left-0 flex flex-col bg-black/30 p-4 opacity-0 backdrop-blur-xs transition-opacity group-hover:opacity-100"
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
					<!-- Track info -->
					<AudioPlayerInfo {item} {imageUrl} showCover={false} class="mb-4 w-full justify-center" />

					<!-- Seek bar -->
					<div class="mb-4 w-full">
						<AudioSeekBar
							{playbackState}
							durationSeconds={playbackState.durationSeconds ||
								item.mediaEnclosure.durationSeconds ||
								0}
						/>
					</div>

					<div class="grid grid-cols-2 xs:grid-cols-3">
						<div class="flex gap-4 xs:col-start-2 xs:items-center xs:justify-center">
							<!-- Controls -->
							<AudioPlayerControls
								durationSeconds={playbackState.durationSeconds ||
									item.mediaEnclosure.durationSeconds ||
									0}
								isPlaying={playbackState.isPlaying}
								skipSeconds={15}
								onTogglePlayback={requestTogglePlayback}
								onSkip={skip}
							/>
						</div>

						<div class="flex min-w-0 items-center justify-end gap-2 self-end">
							<AudioPlayerVolume volume={playbackState.volume} />
						</div>
					</div>
				</div>
			</div>
		{:else}
			<div class="flex flex-1 flex-col items-center justify-center text-fg-muted">
				<Icon icon="lucide:disc-3" class="mb-4 size-16" />
				<p class="text-sm">Nothing playing</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.mini-player-theme {
		color: var(--cover-fg);
	}

	.mini-player-theme :global(.text-fg) {
		color: var(--cover-fg) !important;
	}

	.mini-player-theme :global(.text-fg-muted) {
		color: var(--cover-fg-muted) !important;
	}

	.mini-player-theme :global(.text-fg-subtle) {
		color: var(--cover-fg-subtle) !important;
	}

	.mini-player-theme :global(.text-accent),
	.mini-player-theme :global(.hover\:text-accent:hover),
	.mini-player-theme :global(.focus-visible\:text-accent:focus-visible) {
		color: var(--cover-accent) !important;
	}

	.mini-player-theme :global(.preset-icon-subtle) {
		color: var(--cover-fg) !important;
		background: var(--cover-button-bg) !important;
		border-color: var(--cover-panel-border) !important;
		backdrop-filter: blur(18px);
	}

	.mini-player-theme :global(.preset-icon-subtle:hover) {
		background: var(--cover-button-bg-hover) !important;
	}

	.mini-player-theme :global(.preset-filled-accent) {
		color: var(--cover-accent-contrast) !important;
		background: var(--cover-accent) !important;
		border-color: transparent !important;
	}

	.mini-player-theme :global(.preset-filled-accent:hover) {
		filter: brightness(1.04);
	}

	.mini-player-theme :global(.btn-icon) {
		box-shadow: none;
	}

	.mini-player-theme :global(.player-range) {
		--fill: var(--cover-seek-fill) !important;
	}

	.mini-player-theme :global(.player-range:focus-visible) {
		outline-color: var(--cover-seek-fill) !important;
	}
</style>
