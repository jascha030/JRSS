<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { listen, type UnlistenFn as EventUnlistenFn } from '@tauri-apps/api/event';
	import Icon from '@iconify/svelte';
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import { requestTogglePlayback } from '$lib/stores/app.svelte';
	import { restoreMainWindow } from '$lib/utils/tauri-window';
	import { SKIP_SECONDS, VOLUME_STEP, adjustVolume, skip } from '$lib/utils/player-controls';
	import { useMediaSession } from '$lib/hooks/useMediaSession.svelte';
	import AudioSeekBar from './player/AudioSeekBar.svelte';
	import AudioPlayerControls from './player/AudioPlayerControls.svelte';
	import AudioPlayerVolume from './player/AudioPlayerVolume.svelte';
	import AudioPlayerInfo from './player/AudioPlayerInfo.svelte';
	import { getCoverTheme } from '$lib/state/playback.svelte';
	import CoverThemeStyles from './player/CoverThemeStyles.svelte';

	let coverTheme = $derived(getCoverTheme());

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
	};

	let { item, imageUrl, playbackState }: Props = $props();

	function handleSkip(deltaSeconds: number) {
		skip(playbackState, item?.mediaEnclosure.durationSeconds, deltaSeconds);
	}

	function handleAdjustVolume(delta: number) {
		adjustVolume(playbackState, delta);
	}

	onMount(() => {
		const miniWindow = getCurrentWebviewWindow();
		const unlisten = miniWindow.onCloseRequested(async (event) => {
			event.preventDefault();
			await restoreMainWindow();
			await miniWindow.destroy();
		});

		let unlistenSkipForward: EventUnlistenFn | undefined;
		let unlistenSkipBackward: EventUnlistenFn | undefined;
		let unlistenVolumeUp: EventUnlistenFn | undefined;
		let unlistenVolumeDown: EventUnlistenFn | undefined;
		let unlistenSettings: EventUnlistenFn | undefined;

		const setupListeners = async () => {
			unlistenSkipForward = await listen('menu-skip-forward', () => {
				if (item) handleSkip(SKIP_SECONDS);
			});
			unlistenSkipBackward = await listen('menu-skip-backward', () => {
				if (item) handleSkip(-SKIP_SECONDS);
			});
			unlistenVolumeUp = await listen('menu-volume-up', () => {
				handleAdjustVolume(VOLUME_STEP);
			});
			unlistenVolumeDown = await listen('menu-volume-down', () => {
				handleAdjustVolume(-VOLUME_STEP);
			});
			unlistenSettings = await listen('menu-settings', async () => {
				await restoreMainWindow();
				await miniWindow.destroy();
			});
		};

		void setupListeners();

		const handleKeyDown = (e: KeyboardEvent) => {
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

	useMediaSession(() => item, handleSkip);
</script>

<CoverThemeStyles />

<div class="flex aspect-square h-full w-full flex-col overflow-hidden bg-surface-shell">
	<div class="flex aspect-square h-full w-full flex-1 flex-col items-center justify-center">
		{#if item && playbackState}
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
					class="cover-theme absolute right-0 bottom-0 left-0 flex flex-col bg-black/30 p-4 opacity-0 backdrop-blur-xs transition-opacity group-hover:opacity-100"
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
					<AudioPlayerInfo {item} {imageUrl} showCover={false} class="mb-4 w-full justify-center" />

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
							<AudioPlayerControls
								durationSeconds={playbackState.durationSeconds ||
									item.mediaEnclosure.durationSeconds ||
									0}
								isPlaying={playbackState.isPlaying}
								skipSeconds={15}
								onTogglePlayback={requestTogglePlayback}
								onSkip={handleSkip}
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
