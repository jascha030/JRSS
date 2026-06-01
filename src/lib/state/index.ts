/**
 * App State Facade
 *
 * This module re-exports all domain state modules for ergonomic imports.
 * Each domain owns its own state, selectors, and actions.
 *
 * Import from here for convenience, or directly from domain modules
 * when you need specific dependencies.
 */

export { appState, resetAppState, markAppInitialized } from './app.svelte';

export {
	selection,
	resetSelectionState,
	applyRouteSelection,
	selectFeed,
	selectStation,
	selectSection,
	selectItem,
	setFeedSearchTerm,
	setStationSearchTerm,
	setSectionSearchTerm,
	getSelectedFeed,
	getSelectedStation,
	type SidebarSection,
	type RouteSelectionState
} from './selection.svelte';

export {
	getEffectiveSortOrder,
	getActiveQuerySpec,
	getActiveQueryKey,
	getActiveListSection,
	normalizeSearchTerm,
	type ItemsQuerySpec
} from './query-context.svelte';

export {
	feedsState,
	resetFeedsState,
	loadFeeds,
	createFeed,
	refreshExistingFeed,
	deleteExistingFeed,
	deleteExistingFeed as deleteFeed,
	getFeedById,
	addSyncingFeed,
	removeSyncingFeed,
	setFeedSortOrder
} from './feeds.svelte';

export {
	stationsState,
	resetStationsState,
	loadStations,
	createStation,
	updateExistingStation,
	deleteExistingStation
} from './stations.svelte';

export {
	itemsState,
	resetItemsState,
	invalidateAllQueries,
	loadInitialItemsPage,
	ensureVisibleRangeLoaded,
	ensureItemLoaded,
	loadItemDetails,
	markItemFavorite,
	markItemRead,
	markItemsFavorite,
	markItemsRead,
	loadItemsByIds,
	getSelectedItem,
	getActiveTotalCount,
	getActiveItemIdsByIndex,
	getActiveLoadedPageOffsets,
	getIsActiveInitialLoading,
	registerItem,
	registerItems,
	patchItemSummary,
	getItemById,
	getMediaItemById
} from './items.svelte';

export {
	readerState,
	resetReaderState,
	loadReaderView,
	requestOpenInReader,
	getReaderRequestSeq,
	getReaderRequestItemId
} from './reader.svelte';

export {
	inspectorState,
	resetInspectorState,
	openInspector,
	closeInspector
} from './inspector.svelte';

export {
	themeState,
	applyColorScheme,
	applyAccentColor,
	applyThemeCss,
	initTheme,
	resetThemeState
} from './theme.svelte';

export {
	playbackState,
	resetPlaybackState,
	initAudioEventListeners,
	syncAudioSessionFromBackend,
	playAudioItem,
	stopPlayback,
	requestTogglePlayback,
	requestSeekTo,
	requestSetVolume,
	requestNextEpisode,
	requestPreviousEpisode,
	getManualQueueLength,
	getUpcomingQueue,
	getPlaybackHistory,
	setPlaybackQueue,
	enqueueAudioItem,
	playAudioItemNext,
	moveQueuedItemUp,
	moveQueuedItemDown,
	removeQueuedItem,
	clearQueue,
	clearPlaybackHistory,
	startPlaybackFromContext,
	playStation,
	handlePlaybackEnded,
	restorePlaybackContext,
	persistPlaybackContext,
	getCurrentAudioItem,
	getCurrentAudioItemFeed,
	getPlaybackContext,
	isItemCurrentAudio,
	isAudioPlaying,
	isAudioLoading,
	getPlaybackPositionForItem,
	getCoverTheme,
	type CoverTheme
} from './playback.svelte';

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { isTauriRuntime } from '../services/tauri';
import { loadAppSettings, loadTheme } from '../services/settings';
import { markAppInitialized, resetAppState } from './app.svelte';
import { resetSelectionState } from './selection.svelte';
import { resetFeedsState, loadFeeds } from './feeds.svelte';
import { resetStationsState, loadStations } from './stations.svelte';
import { resetItemsState, invalidateAllQueries, loadInitialItemsPage } from './items.svelte';
import { resetInspectorState } from './inspector.svelte';
import { resetReaderState } from './reader.svelte';
import {
	resetPlaybackState,
	initAudioEventListeners,
	syncAudioSessionFromBackend,
	restorePlaybackContext
} from './playback.svelte';
import { initTheme, resetThemeState } from './theme.svelte';
import { playbackSettings } from './settings.svelte';
import { DEFAULT_SKIP_FORWARD_SECONDS, DEFAULT_SKIP_BACKWARD_SECONDS } from '$lib/types/settings';

export async function initializeApp(): Promise<void> {
	resetAppState();
	resetSelectionState();
	resetPlaybackState();
	resetFeedsState();
	resetStationsState();
	resetItemsState();
	resetReaderState();
	resetInspectorState();
	resetThemeState();

	// Apply persisted theme before first render to prevent FOUC.
	try {
		const settings = await loadAppSettings();
		let themeCss: string | undefined;

		if (settings.themeName) {
			try {
				themeCss = await loadTheme(settings.themeName);
			} catch {
				// Ignore: fallback to built-in theme.
			}
		}

		initTheme(settings, themeCss);

		playbackSettings.setSkipForwardSeconds(
			settings.skipForwardSeconds ?? DEFAULT_SKIP_FORWARD_SECONDS
		);

		playbackSettings.setSkipBackwardSeconds(
			settings.skipBackwardSeconds ?? DEFAULT_SKIP_BACKWARD_SECONDS
		);
	} catch {
		initTheme({ colorScheme: 'system', accentColor: null, themeName: null });
		playbackSettings.setSkipForwardSeconds(DEFAULT_SKIP_FORWARD_SECONDS);
		playbackSettings.setSkipBackwardSeconds(DEFAULT_SKIP_BACKWARD_SECONDS);
	}

	await initAudioEventListeners();
	await loadFeeds();
	await loadStations();
	await loadInitialItemsPage();
	await syncAudioSessionFromBackend();
	await restorePlaybackContext();

	if (isTauriRuntime()) {
		if (_unlistenAutoRefresh) {
			_unlistenAutoRefresh();
		}
		_unlistenAutoRefresh = await listen('auto-refresh-complete', () => {
			void loadFeeds()
				.then(() => invalidateAllQueries())
				.then(() => loadInitialItemsPage());
		});
	}

	markAppInitialized();
}

let _unlistenAutoRefresh: UnlistenFn | undefined;
