<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import Icon from '@iconify/svelte';
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import { requestSeekTo, requestTogglePlayback, requestSetVolume } from '$lib/stores/app.svelte';
	import { restoreMainWindow } from '$lib/utils/tauri-window';
	import AudioSeekBar from '../player/AudioSeekBar.svelte';
	import AudioPlayerControls from '../player/AudioPlayerControls.svelte';

	const SKIP_SECONDS = 15;
	const VOLUME_STEP = 0.1;

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		feedTitle?: string;
		playbackState: PlaybackState | null;
	};

	let { item, imageUrl, feedTitle, playbackState }: Props = $props();

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
		let unlistenSkipForward: UnlistenFn | undefined;
		let unlistenSkipBackward: UnlistenFn | undefined;
		let unlistenVolumeUp: UnlistenFn | undefined;
		let unlistenVolumeDown: UnlistenFn | undefined;
		let unlistenSettings: UnlistenFn | undefined;

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

<div class="flex h-full w-full flex-col overflow-hidden bg-surface-shell">
	<!-- Main content with square album art -->
	<div class="flex flex-1 flex-col items-center justify-center pb-4">
		{#if item && playbackState}
			<!-- Square album art -->
			<div
				class="group relative inset-0 mb-4 aspect-square w-full overflow-hidden rounded-lg shadow-lg"
			>
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
					class="absolute right-0 bottom-0 left-0 flex flex-col p-4 opacity-0 transition-opacity group-hover:opacity-100"
				>
					<!-- Track info -->
					<div class="mb-4 w-full text-center">
						<h2 class="line-clamp-1 text-lg font-semibold text-fg">{item.title}</h2>
						<p class="line-clamp-1 text-sm text-fg-muted">{feedTitle}</p>
					</div>

					<!-- Seek bar -->
					<div class="mb-4 w-full">
						<AudioSeekBar
							{playbackState}
							durationSeconds={playbackState.durationSeconds ||
								item.mediaEnclosure.durationSeconds ||
								0}
						/>
					</div>

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
			</div>
		{:else}
			<div class="flex flex-1 flex-col items-center justify-center text-fg-muted">
				<Icon icon="lucide:disc-3" class="mb-4 size-16" />
				<p class="text-sm">Nothing playing</p>
			</div>
		{/if}
	</div>
</div>
