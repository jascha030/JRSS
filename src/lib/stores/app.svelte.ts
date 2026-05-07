/**
 * App State Facade
 *
 * This module re-exports all domain state modules for ergonomic imports.
 * Each domain owns its own state, selectors, and actions.
 *
 * Import from here for convenience, or directly from domain modules
 * when you need specific dependencies.
 */

export {
	selection,
	resetSelectionState,
	selectFeed,
	selectStation,
	selectSection,
	selectItem,
	setFeedSearchTerm,
	setStationSearchTerm,
	setSectionSearchTerm,
	getSelectedFeed,
	getSelectedStation,
	type SidebarSection
} from '../state/selection.svelte';

export {
	getEffectiveSortOrder,
	getActiveQuerySpec,
	getActiveQueryKey,
	getActiveListSection,
	normalizeSearchTerm,
	type ItemsQuerySpec
} from '../state/query-context.svelte';

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
} from '../state/feeds.svelte';

export {
	stationsState,
	resetStationsState,
	loadStations,
	createStation,
	updateExistingStation,
	deleteExistingStation
} from '../state/stations.svelte';

export {
	itemsState,
	resetItemsState,
	invalidateAllQueries,
	loadInitialItemsPage,
	ensureVisibleRangeLoaded,
	ensureItemLoaded,
	loadItemDetails,
	markItemRead,
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
} from '../state/items.svelte';

export {
	readerState,
	resetReaderState,
	loadReaderView,
	requestOpenInReader,
	getReaderRequestSeq,
	getReaderRequestItemId
} from '../state/reader.svelte';

export {
	themeState,
	applyColorScheme,
	applyAccentColor,
	initTheme,
	resetThemeState
} from '../state/theme.svelte';

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
} from '../state/playback.svelte';

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { isTauriRuntime } from '../services/tauriClient';
import { loadAppSettings } from '../services/feedService';
import { resetSelectionState } from '../state/selection.svelte';
import { resetFeedsState, loadFeeds } from '../state/feeds.svelte';
import { resetStationsState, loadStations } from '../state/stations.svelte';
import { resetItemsState, invalidateAllQueries, loadInitialItemsPage } from '../state/items.svelte';
import { resetReaderState } from '../state/reader.svelte';
import {
	resetPlaybackState,
	initAudioEventListeners,
	syncAudioSessionFromBackend,
	restorePlaybackContext
} from '../state/playback.svelte';
import { initTheme, resetThemeState } from '../state/theme.svelte';

export async function initializeApp(): Promise<void> {
	resetSelectionState();
	resetPlaybackState();
	resetFeedsState();
	resetStationsState();
	resetItemsState();
	resetReaderState();
	resetThemeState();

	// Apply persisted theme before first render to prevent FOUC.
	try {
		const settings = await loadAppSettings();
		initTheme(settings);
	} catch {
		initTheme({ colorScheme: 'system', accentColor: null });
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
}

let _unlistenAutoRefresh: UnlistenFn | undefined;
