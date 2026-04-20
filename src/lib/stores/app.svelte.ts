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
	selectFeed,
	selectStation,
	selectSection,
	selectItem,
	setFeedSearchTerm,
	getEffectiveSortOrder,
	getSelectedFeed,
	getSelectedStation,
	getActiveListSection,
	normalizeSearchTerm,
	type SidebarSection
} from '../state/selection.svelte';

// ---------------------------------------------------------------------------
// Feeds (feed CRUD, feed list)
// ---------------------------------------------------------------------------
export {
	feedsState,
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
	invalidateAllQueries,
	loadInitialItemsPage,
	ensureVisibleRangeLoaded,
	ensureItemLoaded,
	loadItemDetails,
	markItemRead,
	loadItemsByIds,
	getSelectedItem,
	getActiveQueryKey,
	getActiveTotalCount,
	getActiveItemIdsByIndex,
	getActiveLoadedPageOffsets,
	getIsActiveInitialLoading,
	setItemsDependencies,
	type ItemsQuerySpec
} from '../state/items.svelte';

// ---------------------------------------------------------------------------
// Reader (reader view state)
// ---------------------------------------------------------------------------
export {
	readerState,
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
	getPlaybackPositionForItem
} from '../state/playback.svelte';

// ---------------------------------------------------------------------------
// App Initialization
// ---------------------------------------------------------------------------
import { selection } from '../state/selection.svelte';
import { feedsState, loadFeeds } from '../state/feeds.svelte';
import { stationsState, loadStations } from '../state/stations.svelte';
import {
	itemsState,
	invalidateAllQueries,
	setItemsDependencies,
	loadInitialItemsPage
} from '../state/items.svelte';
import { readerState } from '../state/reader.svelte';
import {
	playbackState,
	initAudioEventListeners,
	syncAudioSessionFromBackend,
	restorePlaybackContext
} from '../state/playback.svelte';

export async function initializeApp(): Promise<void> {
	// Reset selection state
	selection.selectedFeedId = null;
	selection.selectedItemId = null;
	selection.selectedSection = 'all';
	selection.selectedStationId = null;
	selection.feedSearchTerm = '';

	// Reset playback state
	playbackState.currentPlaybackState = null;
	playbackState.isAudioLoading = false;
	playbackState.manualQueue = [];
	playbackState.autoQueue = [];
	playbackState.playbackContext = null;

	// Reset feed state
	feedsState.isCreatingFeed = false;
	feedsState.syncingFeedIds = [];

	// Reset reader state
	readerState.readerLoadingItemIds = [];
	readerState.readerRequestSeq = 0;
	readerState.readerRequestItemId = null;

	// Reset item state
	itemsState.itemSummariesById = {};
	itemsState.itemDetailsById = {};
	invalidateAllQueries();

	// Initialize
	await initAudioEventListeners();
	await loadFeeds();
	await loadStations();
	await loadInitialItemsPage();
	await syncAudioSessionFromBackend();
	await restorePlaybackContext();
}

// ---------------------------------------------------------------------------
// Legacy state export (for gradual migration)
// This replicates the old `export const app = $state(...)` structure
// to minimize breaking changes during migration.
// ---------------------------------------------------------------------------

/**
 * Legacy composite state object.
 * @deprecated Import specific state modules instead for better tree-shaking and clarity.
 */
export const app = {
	// Selection
	get selectedFeedId() {
		return selection.selectedFeedId;
	},
	get selectedItemId() {
		return selection.selectedItemId;
	},
	get selectedSection() {
		return selection.selectedSection;
	},
	get selectedStationId() {
		return selection.selectedStationId;
	},
	get feedSearchTerm() {
		return selection.feedSearchTerm;
	},

	// Feeds
	get feeds() {
		return feedsState.feeds;
	},
	get syncingFeedIds() {
		return feedsState.syncingFeedIds;
	},
	get isCreatingFeed() {
		return feedsState.isCreatingFeed;
	},

	// Stations
	get stations() {
		return stationsState.stations;
	},

	// Items
	get itemSummariesById() {
		return itemsState.itemSummariesById;
	},
	get itemDetailsById() {
		return itemsState.itemDetailsById;
	},
	get itemIdsByIndexByQueryKey() {
		return itemsState.itemIdsByIndexByQueryKey;
	},
	get totalCountByQueryKey() {
		return itemsState.totalCountByQueryKey;
	},
	get loadedPageOffsetsByQueryKey() {
		return itemsState.loadedPageOffsetsByQueryKey;
	},
	get loadingPageOffsetsByQueryKey() {
		return itemsState.loadingPageOffsetsByQueryKey;
	},
	get initialLoadDoneByQueryKey() {
		return itemsState.initialLoadDoneByQueryKey;
	},

	// Playback
	get currentPlaybackState() {
		return playbackState.currentPlaybackState;
	},
	get isAudioLoading() {
		return playbackState.isAudioLoading;
	},
	get manualQueue() {
		return playbackState.manualQueue;
	},
	get autoQueue() {
		return playbackState.autoQueue;
	},
	get playbackContext() {
		return playbackState.playbackContext;
	},
	get audioItemsById() {
		return playbackState.audioItemsById;
	},

	// Reader
	get readerLoadingItemIds() {
		return readerState.readerLoadingItemIds;
	},
	get readerRequestSeq() {
		return readerState.readerRequestSeq;
	},
	get readerRequestItemId() {
		return readerState.readerRequestItemId;
	}
};
