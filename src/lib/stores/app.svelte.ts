/**
 * App State Facade
 *
 * This module re-exports all domain state modules for ergonomic imports.
 * Each domain owns its own state, selectors, and actions.
 *
 * Import from here for convenience, or directly from domain modules
 * when you need specific dependencies.
 */

// ---------------------------------------------------------------------------
// Selection (navigation, selection state)
// ---------------------------------------------------------------------------
export {
	selection,
	resetSelectionState,
	selectFeed,
	selectStation,
	selectSection,
	selectItem,
	setFeedSearchTerm,
	getSelectedFeed,
	getSelectedStation,
	type SidebarSection
} from '../state/selection.svelte';

// ---------------------------------------------------------------------------
// Query Context (active query composition)
// ---------------------------------------------------------------------------
export {
	getEffectiveSortOrder,
	getActiveQuerySpec,
	getActiveQueryKey,
	getActiveListSection,
	normalizeSearchTerm,
	type ItemsQuerySpec
} from '../state/query-context.svelte';

// ---------------------------------------------------------------------------
// Feeds (feed CRUD, feed list)
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Stations (station CRUD, station list)
// ---------------------------------------------------------------------------
export {
	stationsState,
	resetStationsState,
	loadStations,
	createStation,
	updateExistingStation,
	deleteExistingStation
} from '../state/stations.svelte';

// ---------------------------------------------------------------------------
// Items (item cache, pagination, queries)
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Reader (reader view state)
// ---------------------------------------------------------------------------
export {
	readerState,
	resetReaderState,
	loadReaderView,
	requestOpenInReader,
	getReaderRequestSeq,
	getReaderRequestItemId
} from '../state/reader.svelte';

// ---------------------------------------------------------------------------
// Playback (audio state, queue, playback controls)
// ---------------------------------------------------------------------------
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
	setPlaybackQueue,
	enqueueAudioItem,
	playAudioItemNext,
	moveQueuedItemUp,
	moveQueuedItemDown,
	removeQueuedItem,
	clearQueue,
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

// ---------------------------------------------------------------------------
// App Initialization
// ---------------------------------------------------------------------------
import { resetSelectionState } from '../state/selection.svelte';
import { resetFeedsState, loadFeeds } from '../state/feeds.svelte';
import { resetStationsState, loadStations } from '../state/stations.svelte';
import { resetItemsState, loadInitialItemsPage } from '../state/items.svelte';
import { resetReaderState } from '../state/reader.svelte';
import {
	resetPlaybackState,
	initAudioEventListeners,
	syncAudioSessionFromBackend,
	restorePlaybackContext
} from '../state/playback.svelte';

export async function initializeApp(): Promise<void> {
	// Reset all state slices
	resetSelectionState();
	resetPlaybackState();
	resetFeedsState();
	resetStationsState();
	resetItemsState();
	resetReaderState();

	// Initialize
	await initAudioEventListeners();
	await loadFeeds();
	await loadStations();
	await loadInitialItemsPage();
	await syncAudioSessionFromBackend();
	await restorePlaybackContext();
}
