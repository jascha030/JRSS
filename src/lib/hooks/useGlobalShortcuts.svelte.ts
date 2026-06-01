import { onMount } from 'svelte';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { playbackState, requestTogglePlayback, getCurrentAudioItem } from '$lib/state';
import { playbackSettings } from '$lib/state/settings.svelte';
import {
	appUi,
	toggleSidebar,
	togglePlayerMode,
	openCommandPalette,
	openFeedEditor,
	openStationEditor
} from '$lib/hooks/useAppUi.svelte';
import { popOutMiniPlayer } from '$lib/utils/mini-player';
import { restoreMainWindow } from '$lib/utils/tauri-window';
import { navigateToSettings } from '$lib/utils/navigation/app-router';
import {
	VOLUME_STEP,
	skip,
	adjustVolume,
	nextEpisode,
	previousEpisode
} from '$lib/utils/player-controls';
import { navigateToCurrentAudioItem, cycleSource } from '$lib/utils/navigation/audio-nav';

export function useGlobalShortcuts() {
	onMount(() => {
		const isMiniWindow = window.location.search.includes('window=mini');

		const handleKeyDown = (event: KeyboardEvent) => {
			const target = event.target;
			const isInput =
				target instanceof HTMLInputElement ||
				target instanceof HTMLTextAreaElement ||
				(target instanceof HTMLElement && target.isContentEditable);

			if (event.key === ' ' && !isInput && playbackState.currentPlaybackState) {
				event.preventDefault();
				requestTogglePlayback();
				return;
			}

			if (event.key === 'k' && (event.metaKey || event.ctrlKey) && !isInput) {
				event.preventDefault();
				openCommandPalette();
			}
		};

		document.addEventListener('keydown', handleKeyDown);

		const unlisteners: UnlistenFn[] = [];

		const setup = async () => {
			unlisteners.push(
				await listen('menu-play-pause', () => {
					if (playbackState.currentPlaybackState) {
						requestTogglePlayback();
					}
				})
			);

			unlisteners.push(
				await listen('menu-skip-forward', () => {
					const item = getCurrentAudioItem();
					if (item) {
						skip(
							playbackState.currentPlaybackState,
							item.mediaEnclosure.durationSeconds,
							playbackSettings.skipForwardSeconds
						);
					}
				})
			);

			unlisteners.push(
				await listen('menu-skip-backward', () => {
					const item = getCurrentAudioItem();
					if (item) {
						skip(
							playbackState.currentPlaybackState,
							item.mediaEnclosure.durationSeconds,
							-playbackSettings.skipBackwardSeconds
						);
					}
				})
			);

			unlisteners.push(
				await listen('menu-next-episode', () => {
					if (playbackState.manualQueue.length > 0 || playbackState.autoQueue.length > 0) {
						nextEpisode();
					}
				})
			);

			unlisteners.push(
				await listen('menu-prev-episode', () => {
					if (playbackState.playbackHistory.length > 0) {
						previousEpisode();
					}
				})
			);

			unlisteners.push(
				await listen('menu-volume-up', () => {
					if (playbackState.currentPlaybackState) {
						adjustVolume(playbackState.currentPlaybackState, VOLUME_STEP);
					}
				})
			);

			unlisteners.push(
				await listen('menu-volume-down', () => {
					if (playbackState.currentPlaybackState) {
						adjustVolume(playbackState.currentPlaybackState, -VOLUME_STEP);
					}
				})
			);

			unlisteners.push(
				await listen('menu-go-to-feed', () => {
					navigateToCurrentAudioItem();
				})
			);

			unlisteners.push(
				await listen('menu-settings', () => {
					if (isMiniWindow) {
						void restoreMainWindow();
					} else {
						appUi.playerMode = 'default';
						void navigateToSettings();
					}
				})
			);

			unlisteners.push(
				await listen('menu-toggle-sidebar', () => {
					toggleSidebar();
				})
			);

			unlisteners.push(
				await listen('menu-next-source', () => {
					if (appUi.playerMode === 'cover' || isMiniWindow) return;
					cycleSource(1);
				})
			);

			unlisteners.push(
				await listen('menu-prev-source', () => {
					if (appUi.playerMode === 'cover' || isMiniWindow) return;
					cycleSource(-1);
				})
			);

			unlisteners.push(
				await listen('menu-toggle-cover', () => {
					if (isMiniWindow) return;
					togglePlayerMode();
				})
			);

			unlisteners.push(
				await listen('menu-toggle-mini-player', async () => {
					if (!document.hasFocus()) {
						return;
					}

					if (isMiniWindow) {
						await getCurrentWebviewWindow().close();
						return;
					}

					await popOutMiniPlayer();
				})
			);

			unlisteners.push(
				await listen('menu-add-feed', () => {
					openFeedEditor();
				})
			);

			unlisteners.push(
				await listen('menu-new-station', () => {
					openStationEditor();
				})
			);
		};

		void setup();

		return () => {
			document.removeEventListener('keydown', handleKeyDown);
			for (const unlisten of unlisteners) {
				unlisten();
			}
		};
	});
}
