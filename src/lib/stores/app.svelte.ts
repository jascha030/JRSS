import {
	addFeed,
	audioGetState,
	audioPlayWithQueue,
	audioQueueClear,
	audioQueueEnqueue,
	audioQueueGetState,
	audioQueueMoveDown,
	audioQueueMoveUp,
	audioQueuePlayNext,
	audioQueueRemove,
	audioQueueSet,
	audioSeek,
	audioStop,
	audioToggle,
	clearPlaybackSession,
	createStation as createStationService,
	deleteStation as deleteStationService,
	getItemDetails,
	getItemsByIds,
	listFeeds,
	listStations,
	loadPlaybackSession,
	loadReaderContent,
	markRead,
	queryItemsPage,
	queryStationEpisodes,
	refreshFeed,
	removeFeed,
	savePlayback,
	savePlaybackSession,
	setFeedSortOrder as persistFeedSortOrder,
	updateStation as updateStationService
} from '$lib/services/feedService';
import type {
	BackendPlaybackEndedEvent,
	BackendPlaybackState,
	CreateStationInput,
	Feed,
	FeedItem,
	FeedItemDetails,
	FeedListItem,
	ItemListSection,
	ItemPageQuery,
	ItemSortOrder,
	MediaListItem,
	PlaybackSession,
	PlaybackState,
	Station,
	UpdateStationInput
} from '$lib/types/rss';
import { isMediaItem } from '$lib/types/rss';
import { logPerf, measurePerfAsync } from '$lib/utils/perfDebug';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type SidebarSection = 'all' | 'unread' | 'media' | 'settings' | null;

const PAGE_SIZE = 100;
const PAGE_PREFETCH = 1;

type QueryKey = string;
type ItemIdsByIndex = Record<number, string>;
type PageOffsets = Record<number, boolean>;

interface AppState {
	selectedFeedId: string | null;
	selectedItemId: string | null;
	selectedSection: SidebarSection;
	selectedStationId: string | null;
	feedSearchTerm: string;
	feeds: Feed[];
	stations: Station[];
	currentPlaybackState: PlaybackState | null;
	/**
	 * Explicit user-initiated queue entries (Play next / Add to queue).
	 * These always play before auto-continuation entries.
	 */
	manualQueue: string[];
	/**
	 * Auto-generated continuation from the playback context.
	 * Rebuilt whenever the user starts a new episode via normal playback.
	 * Plays after manualQueue is exhausted.
	 */
	autoQueue: string[];
	isCreatingFeed: boolean;
	syncingFeedIds: string[];
	readerLoadingItemIds: string[];
	itemSummariesById: Record<string, FeedListItem>;
	itemDetailsById: Record<string, FeedItemDetails>;
	audioItemsById: Record<string, FeedListItem>;
	itemIdsByIndexByQueryKey: Record<QueryKey, ItemIdsByIndex>;
	totalCountByQueryKey: Record<QueryKey, number>;
	loadedPageOffsetsByQueryKey: Record<QueryKey, PageOffsets>;
	loadingPageOffsetsByQueryKey: Record<QueryKey, PageOffsets>;
	initialLoadDoneByQueryKey: Record<QueryKey, boolean>;
	/**
	 * When bumped, signals the page to open the item at `readerRequestItemId`
	 * in reader view. Consumed by the page-level $effect.
	 */
	readerRequestSeq: number;
	readerRequestItemId: string | null;
}

const DEFAULT_SORT_ORDER: ItemSortOrder = 'newest_first';

/**
 * Compute the effective sort order for the active view.
 * - Specific feed selected → feed's persisted sortOrder, or default
 * - Section view → default (newest_first)
 */
export function getEffectiveSortOrder(): ItemSortOrder {
	if (app.selectedStationId) {
		const station = app.stations.find((s) => s.id === app.selectedStationId);
		return station?.sortOrder ?? DEFAULT_SORT_ORDER;
	}

	if (app.selectedFeedId) {
		const feed = app.feeds.find((f) => f.id === app.selectedFeedId);
		return feed?.sortOrder ?? DEFAULT_SORT_ORDER;
	}

	return DEFAULT_SORT_ORDER;
}

const initialState: AppState = {
	selectedFeedId: null,
	selectedItemId: null,
	selectedSection: 'all',
	selectedStationId: null,
	feedSearchTerm: '',
	feeds: [],
	stations: [],
	currentPlaybackState: null,
	manualQueue: [],
	autoQueue: [],
	isCreatingFeed: false,
	syncingFeedIds: [],
	readerLoadingItemIds: [],
	itemSummariesById: {},
	itemDetailsById: {},
	audioItemsById: {},
	itemIdsByIndexByQueryKey: {},
	totalCountByQueryKey: {},
	loadedPageOffsetsByQueryKey: {},
	loadingPageOffsetsByQueryKey: {},
	initialLoadDoneByQueryKey: {},
	readerRequestSeq: 0,
	readerRequestItemId: null
};

const EMPTY_ITEM_IDS_BY_INDEX: ItemIdsByIndex = {};
const EMPTY_PAGE_OFFSETS: PageOffsets = {};

export const app = $state(initialState);

function addUnique(values: string[], value: string): void {
	if (!values.includes(value)) {
		values.push(value);
	}
}

function removeValue(values: string[], value: string): void {
	const index = values.indexOf(value);

	if (index >= 0) {
		values.splice(index, 1);
	}
}

function toItemListSection(section: SidebarSection): ItemListSection | null {
	if (!section || section === 'settings') {
		return null;
	}

	return section;
}

function getActiveListSection(): ItemListSection | null {
	if (app.selectedSection === 'settings') {
		return null;
	}

	if (app.selectedFeedId) {
		return 'all';
	}

	return toItemListSection(app.selectedSection) ?? 'all';
}

function normalizeSearchTerm(term: string): string {
	return term.trim().toLowerCase();
}

function buildQueryKey(
	feedId: string | null,
	section: ItemListSection,
	search: string,
	sortOrder: ItemSortOrder
): QueryKey {
	const normalizedSearch = normalizeSearchTerm(search);
	const base = `${section}::${feedId ?? 'all-feeds'}::${sortOrder}`;

	return normalizedSearch ? `${base}::search:${normalizedSearch}` : base;
}

function getActiveQuerySpec(): { queryKey: QueryKey; query: ItemPageQuery } | null {
	if (app.selectedStationId) {
		const station = app.stations.find((s) => s.id === app.selectedStationId);
		const sortOrder = station?.sortOrder ?? DEFAULT_SORT_ORDER;

		return {
			queryKey: `station::${app.selectedStationId}::${sortOrder}`,
			query: {
				// feedId carries the station ID in station mode — not used by queryItemsPage
				feedId: app.selectedStationId,
				section: 'media',
				offset: 0,
				limit: PAGE_SIZE,
				sortOrder
			}
		};
	}

	const section = getActiveListSection();

	if (!section) {
		return null;
	}

	const search = app.selectedFeedId ? normalizeSearchTerm(app.feedSearchTerm) : '';
	const sortOrder = getEffectiveSortOrder();

	return {
		queryKey: buildQueryKey(app.selectedFeedId, section, search, sortOrder),
		query: {
			feedId: app.selectedFeedId ?? undefined,
			section,
			offset: 0,
			limit: PAGE_SIZE,
			search: search || undefined,
			sortOrder
		}
	};
}

function ensureQueryState(queryKey: QueryKey): void {
	if (!app.itemIdsByIndexByQueryKey[queryKey]) {
		app.itemIdsByIndexByQueryKey[queryKey] = {};
	}

	if (!app.loadedPageOffsetsByQueryKey[queryKey]) {
		app.loadedPageOffsetsByQueryKey[queryKey] = {};
	}

	if (!app.loadingPageOffsetsByQueryKey[queryKey]) {
		app.loadingPageOffsetsByQueryKey[queryKey] = {};
	}

	if (app.totalCountByQueryKey[queryKey] === undefined) {
		app.totalCountByQueryKey[queryKey] = 0;
	}

	if (app.initialLoadDoneByQueryKey[queryKey] === undefined) {
		app.initialLoadDoneByQueryKey[queryKey] = false;
	}
}

function resetQueryState(queryKey: QueryKey): void {
	app.itemIdsByIndexByQueryKey[queryKey] = {};
	app.loadedPageOffsetsByQueryKey[queryKey] = {};
	app.loadingPageOffsetsByQueryKey[queryKey] = {};
	app.totalCountByQueryKey[queryKey] = 0;
	app.initialLoadDoneByQueryKey[queryKey] = false;
}

function invalidateAllQueries(): void {
	app.itemIdsByIndexByQueryKey = {};
	app.loadedPageOffsetsByQueryKey = {};
	app.loadingPageOffsetsByQueryKey = {};
	app.totalCountByQueryKey = {};
	app.initialLoadDoneByQueryKey = {};
	app.selectedItemId = null;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function toFeedListItem(item: FeedItem): FeedListItem {
	const {
		summaryText: _1,
		summaryHtml: _2,
		contentText: _3,
		contentHtml: _4,
		readerContentHtml: _5,
		readerContentText: _6,
		...listItem
	} = item;
	return listItem;
}

function storeItemDetails(item: FeedItem): void {
	app.itemDetailsById[item.id] = {
		id: item.id,
		summaryText: item.summaryText,
		summaryHtml: item.summaryHtml,
		contentText: item.contentText,
		contentHtml: item.contentHtml,
		readerContentHtml: item.readerContentHtml,
		readerContentText: item.readerContentText
	};
}

/** Patch shared base fields on an item. Does not alter the discriminant or mediaEnclosure. */
function patchItemSummary(
	itemId: string,
	patch: Partial<Pick<FeedListItem, 'read' | 'playbackPositionSeconds'>>
): void {
	const existingItem = app.itemSummariesById[itemId];

	if (existingItem) {
		app.itemSummariesById[itemId] = { ...existingItem, ...patch };
	}

	const existingAudioItem = app.audioItemsById[itemId];

	if (existingAudioItem) {
		app.audioItemsById[itemId] = { ...existingAudioItem, ...patch };
	}
}

function mergeDetailedItem(item: FeedItem): void {
	app.itemSummariesById[item.id] = toFeedListItem(item);
	storeItemDetails(item);

	if (app.audioItemsById[item.id]) {
		app.audioItemsById[item.id] = toFeedListItem(item);
	}
}

function mergeItemsPage(
	queryKey: QueryKey,
	offset: number,
	items: FeedListItem[],
	totalCount: number
): void {
	ensureQueryState(queryKey);
	const itemIdsByIndex = app.itemIdsByIndexByQueryKey[queryKey];

	for (const [index, item] of items.entries()) {
		app.itemSummariesById[item.id] = item;
		itemIdsByIndex[offset + index] = item.id;

		if (app.audioItemsById[item.id]) {
			app.audioItemsById[item.id] = item;
		}
	}

	app.totalCountByQueryKey[queryKey] = totalCount;
	app.loadedPageOffsetsByQueryKey[queryKey][offset] = true;
	app.initialLoadDoneByQueryKey[queryKey] = true;
}

function getFirstLoadedItemId(queryKey: QueryKey): string | null {
	const itemIdsByIndex = app.itemIdsByIndexByQueryKey[queryKey];

	if (!itemIdsByIndex) {
		return null;
	}

	const sortedIndexes = Object.keys(itemIdsByIndex)
		.map(Number)
		.sort((left, right) => left - right);

	for (const index of sortedIndexes) {
		const itemId = itemIdsByIndex[index];

		if (itemId) {
			return itemId;
		}
	}

	return null;
}

function ensureSelectionAfterPageLoad(queryKey: QueryKey): void {
	const totalCount = app.totalCountByQueryKey[queryKey] ?? 0;

	if (totalCount === 0) {
		if (!normalizeSearchTerm(app.feedSearchTerm)) {
			app.selectedItemId = null;
		}

		return;
	}

	if (app.selectedItemId && app.itemSummariesById[app.selectedItemId]) {
		return;
	}

	app.selectedItemId = getFirstLoadedItemId(queryKey);
}

async function loadPage(
	queryKey: QueryKey,
	baseQuery: ItemPageQuery,
	offset: number
): Promise<void> {
	const safeOffset = Math.max(0, Math.floor(offset / PAGE_SIZE) * PAGE_SIZE);

	ensureQueryState(queryKey);

	if (app.loadedPageOffsetsByQueryKey[queryKey][safeOffset]) {
		return;
	}

	if (app.loadingPageOffsetsByQueryKey[queryKey][safeOffset]) {
		return;
	}

	app.loadingPageOffsetsByQueryKey[queryKey][safeOffset] = true;

	try {
		const isStationQuery = app.selectedStationId !== null;
		const page = await measurePerfAsync(
			'app.loadPage',
			() => {
				if (isStationQuery && app.selectedStationId) {
					return queryStationEpisodes(app.selectedStationId, safeOffset, PAGE_SIZE);
				}

				return queryItemsPage({
					...baseQuery,
					offset: safeOffset,
					limit: PAGE_SIZE
				});
			},
			{
				queryKey,
				offset: safeOffset
			}
		);

		mergeItemsPage(queryKey, safeOffset, page.items, page.totalCount);

		if (getActiveQueryKey() === queryKey) {
			ensureSelectionAfterPageLoad(queryKey);
		}
	} finally {
		delete app.loadingPageOffsetsByQueryKey[queryKey][safeOffset];
	}
}

function getPageOffsetsForRange(startIndex: number, endIndex: number): number[] {
	const startPageOffset = Math.max(0, Math.floor(startIndex / PAGE_SIZE) * PAGE_SIZE);
	const endPageOffset = Math.max(0, Math.floor(endIndex / PAGE_SIZE) * PAGE_SIZE);
	const offsets: number[] = [];

	for (
		let offset = startPageOffset - PAGE_SIZE * PAGE_PREFETCH;
		offset <= endPageOffset + PAGE_SIZE * PAGE_PREFETCH;
		offset += PAGE_SIZE
	) {
		if (offset >= 0) {
			offsets.push(offset);
		}
	}

	return offsets;
}

function loadInitialItemsPageInBackground(): void {
	void loadInitialItemsPage().catch((error: unknown) => {
		console.error('Failed to load initial item page.', error);
	});
}

export function getActiveQueryKey(): string | null {
	return getActiveQuerySpec()?.queryKey ?? null;
}

export function getSelectedFeed(): Feed | null {
	return app.feeds.find((feed) => feed.id === app.selectedFeedId) ?? null;
}

export function getActiveTotalCount(): number {
	const queryKey = getActiveQueryKey();

	if (!queryKey) {
		return 0;
	}

	return app.totalCountByQueryKey[queryKey] ?? 0;
}

export function getActiveItemIdsByIndex(): ItemIdsByIndex {
	const queryKey = getActiveQueryKey();

	if (!queryKey) {
		return EMPTY_ITEM_IDS_BY_INDEX;
	}

	return app.itemIdsByIndexByQueryKey[queryKey] ?? EMPTY_ITEM_IDS_BY_INDEX;
}

export function getActiveLoadedPageOffsets(): PageOffsets {
	const queryKey = getActiveQueryKey();

	if (!queryKey) {
		return EMPTY_PAGE_OFFSETS;
	}

	return app.loadedPageOffsetsByQueryKey[queryKey] ?? EMPTY_PAGE_OFFSETS;
}

export function getIsActiveInitialLoading(): boolean {
	const queryKey = getActiveQueryKey();

	if (!queryKey) {
		return false;
	}

	return (
		!app.initialLoadDoneByQueryKey[queryKey] &&
		Object.keys(app.loadingPageOffsetsByQueryKey[queryKey] ?? {}).length > 0
	);
}

export function getSelectedItem(): FeedItem | null {
	if (!app.selectedItemId) {
		return null;
	}

	const listItem = app.itemSummariesById[app.selectedItemId];

	if (!listItem) {
		return null;
	}

	const details = app.itemDetailsById[app.selectedItemId];

	return details
		? {
				...listItem,
				...details
			}
		: listItem;
}

export function getSelectedItemFeed(): Feed | null {
	const item = getSelectedItem();

	if (!item) {
		return null;
	}

	return app.feeds.find((feed) => feed.id === item.feedId) ?? null;
}

/** Whether the given item is the currently loaded audio item. */
export function isItemCurrentAudio(itemId: string): boolean {
	return app.currentPlaybackState?.itemId === itemId;
}

/** Whether the current audio item is actively playing (not paused). */
export function isAudioPlaying(): boolean {
	return app.currentPlaybackState?.isPlaying ?? false;
}

export function getCurrentAudioItem(): MediaListItem | null {
	const playbackState = app.currentPlaybackState;

	if (!playbackState) {
		return null;
	}

	const currentItem =
		app.audioItemsById[playbackState.itemId] ?? app.itemSummariesById[playbackState.itemId];

	return currentItem && isMediaItem(currentItem) ? currentItem : null;
}

export function getCurrentAudioItemFeed(): Feed | null {
	const item = getCurrentAudioItem();

	if (!item) {
		return null;
	}

	return app.feeds.find((feed) => feed.id === item.feedId) ?? null;
}

export async function loadFeeds(): Promise<void> {
	app.feeds = await listFeeds();

	if (app.selectedFeedId && !app.feeds.some((feed) => feed.id === app.selectedFeedId)) {
		app.selectedFeedId = null;
		app.selectedSection = 'all';
	}
}

export async function loadStations(): Promise<void> {
	app.stations = await listStations();

	if (
		app.selectedStationId &&
		!app.stations.some((station) => station.id === app.selectedStationId)
	) {
		app.selectedStationId = null;
		app.selectedSection = 'all';
	}
}

export async function loadInitialItemsPage(): Promise<void> {
	const querySpec = getActiveQuerySpec();

	if (!querySpec) {
		app.selectedItemId = null;
		return;
	}

	resetQueryState(querySpec.queryKey);
	await loadPage(querySpec.queryKey, querySpec.query, 0);
}

export async function ensureVisibleRangeLoaded(
	startIndex: number,
	endIndex: number
): Promise<void> {
	const querySpec = getActiveQuerySpec();

	if (!querySpec) {
		return;
	}

	logPerf('app.ensureVisibleRangeLoaded', {
		queryKey: querySpec.queryKey,
		startIndex,
		endIndex
	});

	const totalCount = app.totalCountByQueryKey[querySpec.queryKey];
	const candidateOffsets = getPageOffsetsForRange(startIndex, endIndex);

	for (const offset of candidateOffsets) {
		if (totalCount !== undefined && totalCount > 0 && offset >= totalCount) {
			continue;
		}

		await loadPage(querySpec.queryKey, querySpec.query, offset);
	}
}

// ---------------------------------------------------------------------------
// Playback session persistence
// ---------------------------------------------------------------------------

/**
 * Snapshot the current playback + queue state to SQLite.
 * Fire-and-forget — callers do not await this.
 */
function persistSession(): void {
	const playbackState = app.currentPlaybackState;

	if (!playbackState) {
		void clearPlaybackSession().catch((error: unknown) => {
			console.error('Failed to clear playback session.', error);
		});
		return;
	}

	const session: PlaybackSession = {
		currentItemId: playbackState.itemId,
		positionSeconds: playbackState.positionSeconds,
		durationSeconds: playbackState.durationSeconds,
		manualQueue: [...app.manualQueue],
		autoQueue: [...app.autoQueue]
	};

	void savePlaybackSession(session).catch((error: unknown) => {
		console.error('Failed to persist playback session.', error);
	});
}

/**
 * Restore a persisted playback session during app initialization.
 *
 * Fetches the current item and all queued items **directly from SQLite by ID**,
 * so restore does not depend on what the active list page happens to contain.
 *
 * The restored playback starts **paused** (autoPlay = false).
 * Gracefully skips items that no longer exist locally.
 */
async function restoreSession(): Promise<void> {
	let session: PlaybackSession | null;

	try {
		session = await loadPlaybackSession();
	} catch (error) {
		console.error('Failed to load playback session.', error);
		return;
	}

	if (!session) {
		console.debug('[session-restore] No saved playback session found.');
		return;
	}

	// Collect every unique ID we need to resolve from the backend
	const allIds = new Set<string>([
		session.currentItemId,
		...session.manualQueue,
		...session.autoQueue
	]);
	const allIdsArray = [...allIds];

	let fetchedItems: FeedListItem[];

	try {
		fetchedItems = await getItemsByIds(allIdsArray);
	} catch (error) {
		console.error('Failed to fetch items for session restore.', error);
		return;
	}

	// Index fetched items for O(1) lookup
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral lookup map, not reactive state
	const fetchedById = new Map<string, FeedListItem>();

	for (const item of fetchedItems) {
		fetchedById.set(item.id, item);
	}

	// Resolve current item
	const currentItem = fetchedById.get(session.currentItemId);

	if (!currentItem || !isMediaItem(currentItem)) {
		// Current item gone — try to restore just the queues
		app.manualQueue = filterResolvableAudioIds(session.manualQueue, fetchedById);
		app.autoQueue = filterResolvableAudioIds(session.autoQueue, fetchedById);
		const queueCount = app.manualQueue.length + app.autoQueue.length;
		console.debug(`[session-restore] Current item missing. Restored ${queueCount} queued items.`);
		return;
	}

	// Hydrate current item into audioItemsById and itemSummariesById
	app.audioItemsById[currentItem.id] = currentItem;
	app.itemSummariesById[currentItem.id] = currentItem;

	// Use the persisted position if available, fall back to the item's saved position
	const restoredPosition =
		session.positionSeconds > 0 ? session.positionSeconds : currentItem.playbackPositionSeconds;

	app.currentPlaybackState = {
		itemId: currentItem.id,
		positionSeconds: restoredPosition,
		durationSeconds:
			session.durationSeconds > 0
				? session.durationSeconds
				: (currentItem.mediaEnclosure.durationSeconds ?? 0),
		isPlaying: false,
		autoPlay: false
	};

	// Restore queues, filtering out items that no longer exist
	app.manualQueue = filterResolvableAudioIds(session.manualQueue, fetchedById);
	app.autoQueue = filterResolvableAudioIds(session.autoQueue, fetchedById);

	// Remove current item from queues if it somehow ended up there
	app.manualQueue = app.manualQueue.filter((id) => id !== currentItem.id);
	app.autoQueue = app.autoQueue.filter((id) => id !== currentItem.id);

	console.debug(
		`[session-restore] Restored: "${currentItem.title}" at ${Math.floor(restoredPosition)}s, ` +
			`manual=${app.manualQueue.length}, auto=${app.autoQueue.length}`
	);
}

/**
 * Filter a list of item IDs to only those that resolve to audio items
 * from the pre-fetched map. Registers resolved items in audioItemsById.
 */
function filterResolvableAudioIds(ids: string[], fetchedById: Map<string, FeedListItem>): string[] {
	const result: string[] = [];

	for (const id of ids) {
		const item = fetchedById.get(id);

		if (item && isMediaItem(item)) {
			app.audioItemsById[id] = item;
			app.itemSummariesById[id] = item;
			result.push(id);
		}
	}

	return result;
}

// ---------------------------------------------------------------------------
// Backend audio event listeners
// ---------------------------------------------------------------------------

let audioEventUnlisteners: UnlistenFn[] = [];

async function initAudioEventListeners(): Promise<void> {
	// Clean up any previous listeners
	for (const unlisten of audioEventUnlisteners) {
		unlisten();
	}
	audioEventUnlisteners = [];

	const unlistenState = await listen<BackendPlaybackState>('playback-state-changed', (event) => {
		applyBackendPlaybackState(event.payload);
	});

	const unlistenEnded = await listen<BackendPlaybackEndedEvent>('playback-ended', () => {
		// Backend now handles queue auto-advance — just update local state from backend
		void syncQueueFromBackend();
	});

	const unlistenStopped = await listen('playback-stopped', () => {
		app.currentPlaybackState = null;
		persistSession();
	});

	const unlistenQueueChanged = await listen<{
		current: { itemId: string; url: string; title: string; durationSeconds: number } | null;
		manualCount: number;
		autoCount: number;
	}>('queue-changed', async () => {
		// Queue changed in backend — fetch updated state and sync to frontend
		try {
			const backendQueue = await audioQueueGetState();
			// We store item IDs for display, but actual queue logic is in backend
			// For now, just don't clear - we'll rebuild from playback context
		} catch (e) {
			console.error('Failed to sync queue from backend', e);
		}
	});

	audioEventUnlisteners = [unlistenState, unlistenEnded, unlistenStopped, unlistenQueueChanged];
}

export async function initializeApp(): Promise<void> {
	app.selectedFeedId = null;
	app.selectedItemId = null;
	app.selectedSection = 'all';
	app.selectedStationId = null;
	app.feedSearchTerm = '';
	app.currentPlaybackState = null;
	app.manualQueue = [];
	app.autoQueue = [];
	app.isCreatingFeed = false;
	app.syncingFeedIds = [];
	app.readerLoadingItemIds = [];
	app.itemSummariesById = {};
	app.itemDetailsById = {};
	app.audioItemsById = {};
	invalidateAllQueries();
	await initAudioEventListeners();
	await loadFeeds();
	await loadStations();
	await loadInitialItemsPage();
	await restoreSession();
}

export function selectFeed(feedId: string | null): void {
	app.selectedItemId = null;
	app.selectedFeedId = feedId;
	app.selectedSection = feedId ? null : 'all';
	app.selectedStationId = null;
	app.feedSearchTerm = '';
	loadInitialItemsPageInBackground();
}

export function selectSection(section: SidebarSection): void {
	app.selectedItemId = null;
	app.selectedFeedId = null;
	app.selectedSection = section;
	app.selectedStationId = null;
	app.feedSearchTerm = '';
	loadInitialItemsPageInBackground();
}

export function setFeedSearchTerm(term: string): void {
	if (!app.selectedFeedId) {
		return;
	}

	const normalized = normalizeSearchTerm(term);
	const previousNormalized = normalizeSearchTerm(app.feedSearchTerm);

	if (normalized === previousNormalized) {
		app.feedSearchTerm = term;
		return;
	}

	app.feedSearchTerm = term;
	loadInitialItemsPageInBackground();
}

export function setItemSortOrder(order: ItemSortOrder): void {
	const currentOrder = getEffectiveSortOrder();

	if (order === currentOrder) {
		return;
	}

	const feedId = app.selectedFeedId;

	if (feedId) {
		// Update the local feed object immediately for reactive UI
		const feedIndex = app.feeds.findIndex((f) => f.id === feedId);

		if (feedIndex >= 0) {
			app.feeds[feedIndex] = { ...app.feeds[feedIndex], sortOrder: order };
		}

		// Persist to SQLite (fire-and-forget)
		void persistFeedSortOrder(feedId, order).catch((error: unknown) => {
			console.error('Failed to persist feed sort order.', error);
		});
	}

	loadInitialItemsPageInBackground();
}

export async function createFeed(url: string): Promise<Feed> {
	app.isCreatingFeed = true;

	try {
		const createdFeed = await addFeed(url);
		await loadFeeds();
		invalidateAllQueries();
		app.selectedFeedId = createdFeed.id;
		app.selectedSection = null;
		await loadInitialItemsPage();

		return createdFeed;
	} finally {
		app.isCreatingFeed = false;
	}
}

export async function refreshExistingFeed(feedId: string): Promise<Feed> {
	addUnique(app.syncingFeedIds, feedId);

	try {
		const refreshedFeed = await refreshFeed(feedId);
		await loadFeeds();
		invalidateAllQueries();
		await loadInitialItemsPage();

		return refreshedFeed;
	} finally {
		removeValue(app.syncingFeedIds, feedId);
	}
}

export async function deleteFeed(feedId: string): Promise<void> {
	const currentAudioItem = getCurrentAudioItem();

	await removeFeed(feedId);

	if (app.selectedFeedId === feedId) {
		app.selectedFeedId = null;
		app.selectedSection = 'all';
	}

	if (currentAudioItem?.feedId === feedId) {
		app.currentPlaybackState = null;
		delete app.audioItemsById[currentAudioItem.id];
	}

	const isFromDeletedFeed = (itemId: string): boolean => {
		const item = app.audioItemsById[itemId] ?? app.itemSummariesById[itemId];
		return item ? item.feedId === feedId : true;
	};

	app.manualQueue = app.manualQueue.filter((id) => !isFromDeletedFeed(id));
	app.autoQueue = app.autoQueue.filter((id) => !isFromDeletedFeed(id));
	persistSession();

	await loadFeeds();
	invalidateAllQueries();
	await loadInitialItemsPage();
	removeValue(app.syncingFeedIds, feedId);
}

export async function loadItemDetails(itemId: string): Promise<FeedItem> {
	const currentItem = app.itemSummariesById[itemId];
	const currentItemDetails = app.itemDetailsById[itemId];

	if (currentItem && currentItemDetails) {
		return {
			...currentItem,
			...currentItemDetails
		};
	}

	const detailedItem = await getItemDetails(itemId);
	mergeDetailedItem(detailedItem);

	return detailedItem;
}

export async function selectItem(itemId: string): Promise<void> {
	const item = app.itemSummariesById[itemId];

	if (item && !isMediaItem(item)) {
		await markItemRead(itemId, true);
	}

	app.selectedItemId = itemId;
}

export async function markItemRead(itemId: string, read: boolean): Promise<void> {
	const previousItem = app.itemSummariesById[itemId];

	patchItemSummary(itemId, { read });

	try {
		await markRead(itemId, read);

		if (getActiveListSection() === 'unread') {
			await loadInitialItemsPage();
		}
	} catch (error) {
		if (previousItem) {
			app.itemSummariesById[itemId] = previousItem;
		}

		throw error;
	}
}

export async function loadReaderView(itemId: string): Promise<FeedItem> {
	addUnique(app.readerLoadingItemIds, itemId);

	try {
		const updatedItem = await loadReaderContent(itemId);
		mergeDetailedItem(updatedItem);

		return updatedItem;
	} finally {
		removeValue(app.readerLoadingItemIds, itemId);
	}
}

/**
 * Signal the page to select an item and open it in reader view.
 * The page-level $effect watches `readerRequestSeq` and handles the async flow.
 */
export function requestOpenInReader(itemId: string): void {
	selectItem(itemId);
	app.readerRequestItemId = itemId;
	app.readerRequestSeq += 1;
}

/**
 * Read the current reader-request sequence number (for the page to watch).
 */
export function getReaderRequestSeq(): number {
	return app.readerRequestSeq;
}

/**
 * Read the item ID from the last reader-open request.
 */
export function getReaderRequestItemId(): string | null {
	return app.readerRequestItemId;
}

function itemToQueuedItem(item: MediaListItem): {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
} {
	return {
		itemId: item.id,
		url: item.mediaEnclosure.url,
		title: item.title,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0
	};
}

export function playAudioItem(item: MediaListItem, { autoPlay = true } = {}): void {
	app.audioItemsById[item.id] = item;
	app.currentPlaybackState = {
		itemId: item.id,
		positionSeconds: item.playbackPositionSeconds,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0,
		isPlaying: false,
		autoPlay
	};

	if (autoPlay) {
		// Build the queue: items in manual + auto queues (excluding current item)
		const queueItems: { itemId: string; url: string; title: string; durationSeconds: number }[] =
			[];
		const currentItemId = item.id;

		for (const id of app.manualQueue) {
			if (id !== currentItemId) {
				const qItem = app.audioItemsById[id] ?? app.itemSummariesById[id];
				if (qItem && isMediaItem(qItem)) {
					queueItems.push(itemToQueuedItem(qItem));
				}
			}
		}
		for (const id of app.autoQueue) {
			if (id !== currentItemId) {
				const qItem = app.audioItemsById[id] ?? app.itemSummariesById[id];
				if (qItem && isMediaItem(qItem)) {
					queueItems.push(itemToQueuedItem(qItem));
				}
			}
		}

		void audioPlayWithQueue(itemToQueuedItem(item), queueItems, item.playbackPositionSeconds).catch(
			(error: unknown) => {
				console.error('Failed to start audio playback.', error);
			}
		);
	}
}

export function stopPlayback(): void {
	app.currentPlaybackState = null;
	persistSession();
	void audioStop().catch((error: unknown) => {
		console.error('Failed to stop audio.', error);
	});
}

/**
 * Toggle play/pause on the current audio item via the backend.
 * Has no effect if nothing is loaded.
 */
export function requestTogglePlayback(): void {
	console.log('[requestTogglePlayback] called, currentPlaybackState:', app.currentPlaybackState);
	if (!app.currentPlaybackState) {
		console.log('[requestTogglePlayback] early return - no currentPlaybackState');
		return;
	}
	// Also fetch actual backend state to see if there's a playing item
	audioGetState().then((backendState: BackendPlaybackState | null) => {
		console.log('[requestTogglePlayback] backend state:', backendState);
	});
	void audioToggle().catch((error: unknown) => {
		console.error('Failed to toggle playback.', error);
	});
}

/**
 * Signal the backend to seek the current item to `positionSeconds`.
 */
export function requestSeekTo(positionSeconds: number): void {
	if (!app.currentPlaybackState) {
		return;
	}
	void audioSeek(positionSeconds).catch((error: unknown) => {
		console.error('Failed to seek.', error);
	});
}

export async function setPlaybackPlaying(isPlaying: boolean): Promise<void> {
	if (!app.currentPlaybackState) {
		return;
	}

	const itemId = app.currentPlaybackState.itemId;

	if (isPlaying && itemId) {
		await markItemRead(itemId, true);
	}

	app.currentPlaybackState = {
		...app.currentPlaybackState,
		isPlaying,
		autoPlay: false
	};
}

/**
 * Apply a backend playback state event to the store.
 * Called by the Tauri event listener — not by UI components.
 */
function applyBackendPlaybackState(event: BackendPlaybackState): void {
	if (!app.currentPlaybackState) {
		return;
	}

	// Only apply if it matches the item we think is playing
	if (event.itemId !== app.currentPlaybackState.itemId) {
		return;
	}

	const positionSeconds = Math.floor(event.positionSeconds);
	const durationSeconds = Math.floor(event.durationSeconds);

	patchItemSummary(event.itemId, { playbackPositionSeconds: positionSeconds });

	app.currentPlaybackState = {
		...app.currentPlaybackState,
		positionSeconds,
		durationSeconds,
		isPlaying: event.isPlaying,
		autoPlay: false
	};
}

async function syncQueueFromBackend(): Promise<void> {
	const backendQueue = await audioQueueGetState();
	// Update frontend display state from backend
	// Note: The actual queue operations go through backend commands
	// This just syncs the display state for UI purposes
	const currentItem = backendQueue.current;
	if (currentItem && app.currentPlaybackState) {
		// Check if the current playing item changed (auto-advance happened)
		if (currentItem.itemId !== app.currentPlaybackState.itemId) {
			// Auto-advance happened — update current state
			const item =
				app.audioItemsById[currentItem.itemId] ?? app.itemSummariesById[currentItem.itemId];
			if (item && isMediaItem(item)) {
				app.currentPlaybackState = {
					itemId: currentItem.itemId,
					positionSeconds: 0,
					durationSeconds: currentItem.durationSeconds,
					isPlaying: true,
					autoPlay: false
				};
				app.audioItemsById[currentItem.itemId] = item;
				// Mark as read when playback auto-advances
				void markItemRead(currentItem.itemId, true);
			}
		}
	}
	persistSession();
}

export function updatePlaybackPosition(positionSeconds: number, durationSeconds: number): void {
	if (!app.currentPlaybackState) {
		return;
	}

	const itemId = app.currentPlaybackState.itemId;
	patchItemSummary(itemId, { playbackPositionSeconds: positionSeconds });
	app.currentPlaybackState = {
		...app.currentPlaybackState,
		positionSeconds,
		durationSeconds
	};
}

export async function persistPlaybackPosition(
	positionSeconds: number,
	durationSeconds: number
): Promise<void> {
	if (!app.currentPlaybackState) {
		return;
	}

	const itemId = app.currentPlaybackState.itemId;
	updatePlaybackPosition(positionSeconds, durationSeconds);
	await savePlayback(itemId, positionSeconds);
}

/**
 * Persist a specific item's playback position by explicit ID.
 * Used during item transitions in the AudioPlayer so the departing
 * item's position is saved against the correct item, even though
 * currentPlaybackState already points to the new item.
 */
export async function persistPlaybackForItem(
	itemId: string,
	positionSeconds: number
): Promise<void> {
	patchItemSummary(itemId, { playbackPositionSeconds: positionSeconds });
	await savePlayback(itemId, positionSeconds);
}

// ---------------------------------------------------------------------------
// Playback queue — dual-segment: manualQueue (user-explicit) + autoQueue (context continuation)
// ---------------------------------------------------------------------------

/** Resolve an item ID to a MediaListItem, or null. */
function resolveAudioItem(itemId: string): MediaListItem | null {
	const item = app.audioItemsById[itemId] ?? app.itemSummariesById[itemId];
	return item && isMediaItem(item) ? item : null;
}

/**
 * Shift the first resolvable audio item off a queue array, mutating it in place.
 * Returns the resolved item, or null if the queue is exhausted.
 */
function shiftNextAudioItem(queue: string[]): MediaListItem | null {
	while (queue.length > 0) {
		const nextId = queue[0];
		queue.splice(0, 1);

		const nextItem = resolveAudioItem(nextId);

		if (nextItem) {
			return nextItem;
		}
	}

	return null;
}

/**
 * Count of manual queue entries. Exposed so the QueueDrawer can show a
 * divider between manual and auto-continuation entries.
 */
export function getManualQueueLength(): number {
	return app.manualQueue.length;
}

/**
 * Items waiting in the queue, resolved to their FeedListItem summaries.
 * Manual entries come first, then auto-continuation entries.
 * Missing or non-audio items are excluded.
 */
export function getUpcomingQueue(): MediaListItem[] {
	const items: MediaListItem[] = [];
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral dedup set, not reactive state
	const seen = new Set<string>();

	for (const itemId of app.manualQueue) {
		const item = resolveAudioItem(itemId);

		if (item && !seen.has(item.id)) {
			seen.add(item.id);
			items.push(item);
		}
	}

	for (const itemId of app.autoQueue) {
		const item = resolveAudioItem(itemId);

		if (item && !seen.has(item.id)) {
			seen.add(item.id);
			items.push(item);
		}
	}

	return items;
}

/**
 * Replace the queue wholesale.
 * Filters out items without mediaEnclosure and deduplicates.
 * Clears both manual and auto queues.
 */
export function setPlaybackQueue(items: FeedListItem[]): void {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral dedup set, not reactive state
	const seen = new Set<string>();
	const ids: string[] = [];

	for (const item of items) {
		if (!isMediaItem(item) || seen.has(item.id)) {
			continue;
		}

		seen.add(item.id);
		app.audioItemsById[item.id] = item;
		ids.push(item.id);
	}

	app.manualQueue = ids;
	app.autoQueue = [];

	// Sync to backend - only include media items
	const queueItems = items.filter(isMediaItem).map((item) => ({
		itemId: item.id,
		url: item.mediaEnclosure.url,
		title: item.title,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0
	}));
	void audioQueueSet(queueItems).catch(console.error);
}

/** Append an item to the end of the manual queue. No-op if already queued or currently playing. */
export function enqueueAudioItem(item: MediaListItem): void {
	if (app.currentPlaybackState?.itemId === item.id) {
		return;
	}

	if (app.manualQueue.includes(item.id)) {
		return;
	}

	app.audioItemsById[item.id] = item;

	// Also remove from autoQueue to avoid duplicates in the combined view
	app.autoQueue = app.autoQueue.filter((id) => id !== item.id);
	app.manualQueue = [...app.manualQueue, item.id];

	// Sync to backend
	void audioQueueEnqueue(itemToQueuedItem(item)).catch(console.error);
	persistSession();
}

/** Insert an item as the next item to play (index 0 of the manual queue). Deduplicates. */
export function playAudioItemNext(item: MediaListItem): void {
	if (app.currentPlaybackState?.itemId === item.id) {
		return;
	}

	app.audioItemsById[item.id] = item;
	const filteredManual = app.manualQueue.filter((id) => id !== item.id);
	app.autoQueue = app.autoQueue.filter((id) => id !== item.id);
	app.manualQueue = [item.id, ...filteredManual];

	// Sync to backend
	void audioQueuePlayNext(itemToQueuedItem(item)).catch(console.error);
	persistSession();
}

/** Move a queued item one position earlier within its segment. */
export function moveQueuedItemUp(itemId: string): void {
	const manualIndex = app.manualQueue.indexOf(itemId);

	if (manualIndex > 0) {
		const next = [...app.manualQueue];
		[next[manualIndex - 1], next[manualIndex]] = [next[manualIndex], next[manualIndex - 1]];
		app.manualQueue = next;
		void audioQueueMoveUp(itemId).catch(console.error);
		return;
	}

	// If it's the first auto item and there's a manual queue, promote to last manual position
	const autoIndex = app.autoQueue.indexOf(itemId);

	if (autoIndex > 0) {
		const next = [...app.autoQueue];
		[next[autoIndex - 1], next[autoIndex]] = [next[autoIndex], next[autoIndex - 1]];
		app.autoQueue = next;
		void audioQueueMoveUp(itemId).catch(console.error);
	} else if (autoIndex === 0) {
		// Promote from auto to manual (move to end of manual queue)
		app.autoQueue = app.autoQueue.filter((id) => id !== itemId);
		app.manualQueue = [...app.manualQueue, itemId];
		void audioQueueMoveUp(itemId).catch(console.error);
	}
	persistSession();
}

/** Move a queued item one position later within its segment. */
export function moveQueuedItemDown(itemId: string): void {
	const manualIndex = app.manualQueue.indexOf(itemId);

	if (manualIndex >= 0 && manualIndex < app.manualQueue.length - 1) {
		const next = [...app.manualQueue];
		[next[manualIndex], next[manualIndex + 1]] = [next[manualIndex + 1], next[manualIndex]];
		app.manualQueue = next;
		void audioQueueMoveDown(itemId).catch(console.error);
		return;
	}

	if (manualIndex === app.manualQueue.length - 1) {
		// Demote from manual to auto (move to start of auto queue)
		app.manualQueue = app.manualQueue.filter((id) => id !== itemId);
		app.autoQueue = [itemId, ...app.autoQueue];
		void audioQueueMoveDown(itemId).catch(console.error);
		return;
	}

	const autoIndex = app.autoQueue.indexOf(itemId);

	if (autoIndex >= 0 && autoIndex < app.autoQueue.length - 1) {
		const next = [...app.autoQueue];
		[next[autoIndex], next[autoIndex + 1]] = [next[autoIndex + 1], next[autoIndex]];
		app.autoQueue = next;
		void audioQueueMoveDown(itemId).catch(console.error);
	}
	persistSession();
}

/** Remove a specific item from whichever queue segment contains it. */
export function removeQueuedItem(itemId: string): void {
	if (app.manualQueue.includes(itemId)) {
		app.manualQueue = app.manualQueue.filter((id) => id !== itemId);
	} else {
		app.autoQueue = app.autoQueue.filter((id) => id !== itemId);
	}
	void audioQueueRemove(itemId).catch(console.error);
	persistSession();
}

/** Clear both queue segments without affecting the currently playing item. */
export function clearQueue(): void {
	app.manualQueue = [];
	app.autoQueue = [];
	void audioQueueClear().catch(console.error);
	persistSession();
}

// ---------------------------------------------------------------------------
// Context-aware playback start
// ---------------------------------------------------------------------------

/**
 * Derive the ordered list of audio item IDs that follow the given item
 * within the currently active query context.
 *
 * Reads from the store's `itemIdsByIndex` for the active query key,
 * which is the source of truth for the current view's ordering —
 * independent of DOM state.
 *
 * Returns only items that have a mediaEnclosure, excluding the playing item.
 */
function deriveAutoContinuation(playingItemId: string): string[] {
	const queryKey = getActiveQueryKey();

	if (!queryKey) {
		return [];
	}

	const itemIdsByIndex = app.itemIdsByIndexByQueryKey[queryKey];

	if (!itemIdsByIndex) {
		return [];
	}

	const totalCount = app.totalCountByQueryKey[queryKey] ?? 0;

	// Build ordered array of item IDs from the sparse index map
	const sortedIndexes = Object.keys(itemIdsByIndex)
		.map(Number)
		.sort((a, b) => a - b);

	// Find the playing item's position
	let playingPosition = -1;

	for (const idx of sortedIndexes) {
		if (itemIdsByIndex[idx] === playingItemId) {
			playingPosition = idx;
			break;
		}
	}

	if (playingPosition < 0) {
		// Playing item not found in current context — no continuation
		return [];
	}

	// Collect audio items after the playing position
	const manualSet = new Set(app.manualQueue);
	const continuation: string[] = [];

	for (const idx of sortedIndexes) {
		if (idx <= playingPosition) {
			continue;
		}

		const candidateId = itemIdsByIndex[idx];

		if (!candidateId || candidateId === playingItemId) {
			continue;
		}

		// Skip items already in the manual queue
		if (manualSet.has(candidateId)) {
			continue;
		}

		const candidate = app.itemSummariesById[candidateId];

		if (candidate && isMediaItem(candidate)) {
			app.audioItemsById[candidateId] = candidate;
			continuation.push(candidateId);
		}
	}

	// If there are un-loaded pages after our current position, we can't
	// include them — this is the graceful degradation. The continuation
	// covers what we have loaded.
	if (continuation.length === 0 && playingPosition < totalCount - 1) {
		// Items exist beyond our position but aren't loaded yet.
		// Future enhancement: load additional pages here.
	}

	return continuation;
}

/**
 * High-level action: start playback of an item in the context of the
 * current ordered view. This is the normal "press play" path.
 *
 * 1. Starts the item immediately
 * 2. Rebuilds the autoQueue from the active query context
 * 3. Preserves manual queue entries untouched
 */
export function startPlaybackFromContext(item: MediaListItem): void {
	console.log('[startPlaybackFromContext] called for:', item.id, item.title);
	// Remove from either queue if present (it's now playing, not queued)
	app.manualQueue = app.manualQueue.filter((id) => id !== item.id);
	app.autoQueue = app.autoQueue.filter((id) => id !== item.id);

	// Start playback
	playAudioItem(item, { autoPlay: true });

	// Rebuild auto-continuation from context
	app.autoQueue = deriveAutoContinuation(item.id);

	persistSession();
}

/**
 * Called when the `<audio>` element fires `ended`.
 * Persists position 0 for the finished item, then advances to the
 * next valid queue entry or stops cleanly.
 *
 * Drains manualQueue first, then autoQueue.
 *
 * Critically: we do NOT null out currentPlaybackState before setting
 * the next item. A null gap would tear down and recreate the <audio>
 * element in the {#if} block, losing the DOM node and causing
 * unreliable autoplay.
 */
export async function handlePlaybackEnded(): Promise<void> {
	const finishedState = app.currentPlaybackState;

	if (!finishedState) {
		return;
	}

	// Persist finished item at position 0
	const finishedItemId = finishedState.itemId;
	patchItemSummary(finishedItemId, { playbackPositionSeconds: 0 });

	// Try manual queue first, then auto queue
	const nextItem = shiftNextAudioItem(app.manualQueue) ?? shiftNextAudioItem(app.autoQueue);

	if (nextItem) {
		// Directly swap — no null gap
		playAudioItem(nextItem, { autoPlay: true });
		await savePlayback(finishedItemId, 0);
		persistSession();
		return;
	}

	// Queue exhausted — stop cleanly
	app.currentPlaybackState = null;
	await savePlayback(finishedItemId, 0);
	persistSession();
}

// ---------------------------------------------------------------------------
// Stations — podcast playlist grouping
// ---------------------------------------------------------------------------

export function getSelectedStation(): Station | null {
	return app.stations.find((station) => station.id === app.selectedStationId) ?? null;
}

export function selectStation(stationId: string): void {
	app.selectedItemId = null;
	app.selectedFeedId = null;
	app.selectedSection = null;
	app.selectedStationId = stationId;
	app.feedSearchTerm = '';
	loadInitialItemsPageInBackground();
}

export async function createStation(input: CreateStationInput): Promise<Station> {
	const station = await createStationService(input);
	await loadStations();
	return station;
}

export async function updateExistingStation(input: UpdateStationInput): Promise<Station> {
	const station = await updateStationService(input);
	await loadStations();

	// If the updated station is currently selected, reload its episodes
	if (app.selectedStationId === input.id) {
		invalidateAllQueries();
		await loadInitialItemsPage();
	}

	return station;
}

export async function deleteExistingStation(stationId: string): Promise<void> {
	await deleteStationService(stationId);

	if (app.selectedStationId === stationId) {
		app.selectedStationId = null;
		app.selectedSection = 'all';
	}

	await loadStations();
	invalidateAllQueries();
	await loadInitialItemsPage();
}

/**
 * Load all station episodes and replace the playback queue.
 * Starts playing the first episode immediately.
 */
export async function playStation(stationId: string): Promise<void> {
	const page = await queryStationEpisodes(stationId, 0, 500);
	const mediaItems = page.items.filter(isMediaItem);

	if (mediaItems.length === 0) {
		return;
	}

	const firstItem = mediaItems[0];
	const rest = mediaItems.slice(1);

	// Register all items in audioItemsById
	for (const item of mediaItems) {
		app.audioItemsById[item.id] = item;
		app.itemSummariesById[item.id] = item;
	}

	// Start first item
	playAudioItem(firstItem, { autoPlay: true });

	// Set the rest as the queue
	app.manualQueue = rest.map((item) => item.id);
	app.autoQueue = [];

	persistSession();
}
