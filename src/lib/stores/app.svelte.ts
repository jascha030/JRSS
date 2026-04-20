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
	audioSetVolume,
	audioStop,
	audioToggle,
	createStation as createStationService,
	deleteStation as deleteStationService,
	getItemDetails,
	getItemsByIds,
	listFeeds,
	listStations,
	loadPlaybackContext,
	loadReaderContent,
	markRead,
	queryItemsPage,
	queryStationEpisodes,
	refreshFeed,
	removeFeed,
	savePlaybackContext,
	setFeedSortOrder as persistFeedSortOrder,
	updateStation as updateStationService
} from '$lib/services/feedService';
import type {
	BackendQueueState,
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
	PlaybackState,
	Station,
	UpdateStationInput
} from '$lib/types/rss';
import { isMediaItem } from '$lib/types/rss';
import { logPerf, measurePerfAsync } from '$lib/utils/perfDebug';
import { tick } from 'svelte';
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
	 * Whether audio is currently loading/buffering.
	 * Set to true when starting playback, false when playback begins or fails.
	 */
	isAudioLoading: boolean;
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
	/**
	 * Tracks the origin context of the current playback session.
	 * Used to navigate back to the correct view (feed or station) when clicking
	 * on the now playing item in the audio player.
	 */
	playbackContext: { contextType: 'feed' | 'station'; id: string } | null;
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
	isAudioLoading: false,
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
	readerRequestItemId: null,
	playbackContext: null
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

function toFeedListItem(item: FeedItem): FeedListItem {
	const {
		summaryText,
		summaryHtml,
		contentText,
		contentHtml,
		readerContentHtml,
		readerContentText,
		...listItem
	} = item;
	void summaryText;
	void summaryHtml;
	void contentText;
	void contentHtml;
	void readerContentHtml;
	void readerContentText;
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
		const nextRead = patch.read ?? existingItem.read;
		const nextPlaybackPosition =
			patch.playbackPositionSeconds ?? existingItem.playbackPositionSeconds;

		if (
			nextRead !== existingItem.read ||
			nextPlaybackPosition !== existingItem.playbackPositionSeconds
		) {
			app.itemSummariesById[itemId] = { ...existingItem, ...patch };
		}
	}

	const existingAudioItem = app.audioItemsById[itemId];

	if (existingAudioItem) {
		const nextRead = patch.read ?? existingAudioItem.read;
		const nextPlaybackPosition =
			patch.playbackPositionSeconds ?? existingAudioItem.playbackPositionSeconds;

		if (
			nextRead !== existingAudioItem.read ||
			nextPlaybackPosition !== existingAudioItem.playbackPositionSeconds
		) {
			app.audioItemsById[itemId] = { ...existingAudioItem, ...patch };
		}
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

/** Whether audio is currently loading/buffering. */
export function isAudioLoading(): boolean {
	return app.isAudioLoading;
}

export function getPlaybackPositionForItem(
	itemId: string,
	fallbackPositionSeconds: number
): number {
	if (app.currentPlaybackState?.itemId === itemId) {
		return app.currentPlaybackState.positionSeconds;
	}

	return fallbackPositionSeconds;
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

export function getPlaybackContext(): { contextType: 'feed' | 'station'; id: string } | null {
	return app.playbackContext;
}

/**
 * Ensure a specific item is loaded by finding and loading its page.
 * Used when navigating to an item that may not be in the currently loaded pages.
 */
export async function ensureItemLoaded(itemId: string): Promise<void> {
	const querySpec = getActiveQuerySpec();

	if (!querySpec) {
		return;
	}

	const queryKey = querySpec.queryKey;
	const itemIdsByIndex = app.itemIdsByIndexByQueryKey[queryKey];

	// Check if item is already loaded
	if (itemIdsByIndex && Object.values(itemIdsByIndex).includes(itemId)) {
		return;
	}

	// Get total count to know how far to search
	const totalCount = app.totalCountByQueryKey[queryKey] ?? 0;
	if (totalCount === 0) {
		return;
	}

	// Load pages until we find the item or exhaust the list
	// We'll search in chunks of PAGE_SIZE
	let offset = 0;
	const maxOffset = Math.min(totalCount, 10000); // Safety limit

	while (offset < maxOffset) {
		await loadPage(queryKey, querySpec.query, offset);

		// Check if item is now loaded after this page
		const updatedItemIdsByIndex = app.itemIdsByIndexByQueryKey[queryKey];
		if (updatedItemIdsByIndex && Object.values(updatedItemIdsByIndex).includes(itemId)) {
			return;
		}

		offset += PAGE_SIZE;
	}
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
// Backend-owned playback session sync
// ---------------------------------------------------------------------------

const inFlightAudioItemHydrations: Record<string, Promise<void> | undefined> = {};

async function ensureAudioItemsLoaded(itemIds: string[]): Promise<void> {
	const missingIds = [...new Set(itemIds)].filter(
		(itemId) => !app.audioItemsById[itemId] && !app.itemSummariesById[itemId]
	);

	if (missingIds.length === 0) {
		return;
	}

	const newIds = missingIds.filter((itemId) => !inFlightAudioItemHydrations[itemId]);

	if (newIds.length > 0) {
		const newRequest = (async () => {
			try {
				const items = await getItemsByIds(newIds);

				for (const item of items) {
					if (isMediaItem(item)) {
						app.audioItemsById[item.id] = item;
						app.itemSummariesById[item.id] = item;
					}
				}
			} catch (error) {
				console.error('Failed to hydrate audio items from backend state.', error);
			} finally {
				for (const itemId of newIds) {
					delete inFlightAudioItemHydrations[itemId];
				}
			}
		})();

		for (const itemId of newIds) {
			inFlightAudioItemHydrations[itemId] = newRequest;
		}
	}

	const pendingRequests = missingIds
		.map((itemId) => inFlightAudioItemHydrations[itemId])
		.filter((request): request is Promise<void> => request !== undefined);

	if (pendingRequests.length > 0) {
		await Promise.all(pendingRequests);
	}
}

function applyBackendQueueState(queueState: BackendQueueState): void {
	app.manualQueue = queueState.manual.map((item) => item.itemId);
	app.autoQueue = queueState.auto.map((item) => item.itemId);

	if (!queueState.current) {
		if (!app.currentPlaybackState?.isPlaying) {
			app.currentPlaybackState = null;
		}
		return;
	}

	if (!app.currentPlaybackState || app.currentPlaybackState.itemId !== queueState.current.itemId) {
		const currentItem =
			app.audioItemsById[queueState.current.itemId] ??
			app.itemSummariesById[queueState.current.itemId];
		const fallbackPosition =
			currentItem && isMediaItem(currentItem) ? currentItem.playbackPositionSeconds : 0;

		app.currentPlaybackState = {
			itemId: queueState.current.itemId,
			positionSeconds: fallbackPosition,
			durationSeconds: queueState.current.durationSeconds,
			isPlaying: false,
			volume: app.currentPlaybackState?.volume ?? 1
		};
	}
}

async function syncAudioSessionFromBackend(): Promise<void> {
	const [backendPlaybackState, backendQueueState] = await Promise.all([
		audioGetState(),
		audioQueueGetState()
	]);

	const itemIds = [
		...(backendQueueState.current ? [backendQueueState.current.itemId] : []),
		...backendQueueState.manual.map((item) => item.itemId),
		...backendQueueState.auto.map((item) => item.itemId),
		...(backendPlaybackState ? [backendPlaybackState.itemId] : [])
	];

	await ensureAudioItemsLoaded(itemIds);
	applyBackendQueueState(backendQueueState);

	if (backendPlaybackState) {
		applyBackendPlaybackState(backendPlaybackState);
		return;
	}

	if (!backendQueueState.current) {
		app.currentPlaybackState = null;
	}
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
		void ensureAudioItemsLoaded([event.payload.itemId]).then(() => {
			applyBackendPlaybackState(event.payload, true);
		});
	});

	const unlistenEnded = await listen<BackendPlaybackEndedEvent>('playback-ended', (event) => {
		patchItemSummary(event.payload.itemId, { playbackPositionSeconds: 0 });
	});

	const unlistenStopped = await listen('playback-stopped', () => {
		app.currentPlaybackState = null;
	});

	const unlistenQueueChanged = await listen<BackendQueueState>('queue-changed', (event) => {
		const itemIds = [
			...(event.payload.current ? [event.payload.current.itemId] : []),
			...event.payload.manual.map((item) => item.itemId),
			...event.payload.auto.map((item) => item.itemId)
		];

		void ensureAudioItemsLoaded(itemIds).then(() => {
			applyBackendQueueState(event.payload);
		});
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
	await syncAudioSessionFromBackend();
	await restorePlaybackContext();
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
	const queuedIdsToRemove = [...app.manualQueue, ...app.autoQueue].filter((itemId) => {
		const item = app.audioItemsById[itemId] ?? app.itemSummariesById[itemId];
		return item ? item.feedId === feedId : false;
	});

	if (currentAudioItem?.feedId === feedId) {
		await audioStop().catch((error: unknown) => {
			console.error('Failed to stop deleted feed playback.', error);
		});
	}

	for (const itemId of queuedIdsToRemove) {
		await audioQueueRemove(itemId).catch((error: unknown) => {
			console.error('Failed to remove deleted feed item from backend queue.', error);
		});
	}

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

	await loadFeeds();
	invalidateAllQueries();
	await loadInitialItemsPage();
	await syncAudioSessionFromBackend();
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

function queueIdsToQueuedItems(itemIds: string[]): {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
}[] {
	const queueItems: { itemId: string; url: string; title: string; durationSeconds: number }[] = [];

	for (const itemId of itemIds) {
		const queuedItem = app.audioItemsById[itemId] ?? app.itemSummariesById[itemId];

		if (queuedItem && isMediaItem(queuedItem)) {
			queueItems.push(itemToQueuedItem(queuedItem));
		}
	}

	return queueItems;
}

async function yieldForPlaybackUiPaint(): Promise<void> {
	await tick();

	await new Promise<void>((resolve) => {
		if (typeof window === 'undefined') {
			setTimeout(resolve, 0);
			return;
		}

		window.requestAnimationFrame(() => {
			window.setTimeout(resolve, 0);
		});
	});
}

export function playAudioItem(
	item: MediaListItem,
	{
		manualQueueIds = app.manualQueue.filter((itemId) => itemId !== item.id),
		autoQueueIds = app.autoQueue.filter((itemId) => itemId !== item.id),
		startPositionSeconds = item.playbackPositionSeconds,
		context
	}: {
		manualQueueIds?: string[];
		autoQueueIds?: string[];
		startPositionSeconds?: number;
		context?: { contextType: 'feed' | 'station'; id: string } | null;
	} = {}
): void {
	app.audioItemsById[item.id] = item;
	app.itemSummariesById[item.id] = item;
	app.isAudioLoading = true;
	app.currentPlaybackState = {
		itemId: item.id,
		positionSeconds: startPositionSeconds,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0,
		isPlaying: false,
		volume: app.currentPlaybackState?.volume ?? 1
	};

	// Set playback context (defaults to feed context if not specified)
	app.playbackContext = context ?? { contextType: 'feed', id: item.feedId };

	// Persist playback context
	void persistPlaybackContext();

	void (async () => {
		try {
			await yieldForPlaybackUiPaint();
			await audioPlayWithQueue(
				itemToQueuedItem(item),
				queueIdsToQueuedItems(manualQueueIds),
				queueIdsToQueuedItems(autoQueueIds),
				startPositionSeconds
			);
		} catch (error: unknown) {
			console.error('Failed to start audio playback.', error);
			app.isAudioLoading = false;
			void syncAudioSessionFromBackend().catch((syncError: unknown) => {
				console.error('Failed to resync audio after playback start failure.', syncError);
			});
		}
	})();
	// Note: Loading state is cleared by applyBackendPlaybackState when it receives
	// a playback-state-changed event from the backend (with fromEvent=true)
}

export function stopPlayback(): void {
	void audioStop().catch((error: unknown) => {
		console.error('Failed to stop audio.', error);
	});
}

/**
 * Toggle play/pause on the current audio item via the backend.
 * Has no effect if nothing is loaded.
 */
export function requestTogglePlayback(): void {
	const currentPlaybackState = app.currentPlaybackState;

	if (!currentPlaybackState) {
		return;
	}

	void audioToggle().catch((error: unknown) => {
		console.error('Failed to toggle playback.', error);
		void syncAudioSessionFromBackend().catch((syncError: unknown) => {
			console.error('Failed to resync audio after toggle failure.', syncError);
		});
	});
}

/**
 * Signal the backend to seek the current item to `positionSeconds`.
 */
export function requestSeekTo(positionSeconds: number): void {
	void audioSeek(positionSeconds).catch((error: unknown) => {
		console.error('Failed to seek.', error);
	});
}

let pendingVolumeTimeout: ReturnType<typeof setTimeout> | null = null;

export function requestSetVolume(volume: number): void {
	if (pendingVolumeTimeout !== null) {
		clearTimeout(pendingVolumeTimeout);
	}

	pendingVolumeTimeout = setTimeout(() => {
		pendingVolumeTimeout = null;
		void audioSetVolume(volume).catch((error: unknown) => {
			console.error('Failed to set volume.', error);
			void syncAudioSessionFromBackend().catch((syncError: unknown) => {
				console.error('Failed to resync audio after volume failure.', syncError);
			});
		});
	}, 125);
}

/**
 * Apply a backend playback state event to the store.
 * Called by the Tauri event listener — not by UI components.
 */
function applyBackendPlaybackState(event: BackendPlaybackState, fromEvent: boolean = false): void {
	const positionSeconds = Math.floor(event.positionSeconds);
	const durationSeconds = Math.floor(event.durationSeconds);
	const previous = app.currentPlaybackState;

	// Check if playback state actually changed
	const playbackUnchanged =
		previous &&
		previous.itemId === event.itemId &&
		previous.positionSeconds === positionSeconds &&
		previous.durationSeconds === durationSeconds &&
		previous.isPlaying === event.isPlaying &&
		previous.volume === event.volume;

	if (playbackUnchanged) {
		// Still clear loading if this was an event
		if (fromEvent) {
			app.isAudioLoading = false;
		}
		return;
	}

	const previousItemId = previous?.itemId;
	const wasPlaying = previous?.isPlaying ?? false;

	app.currentPlaybackState = {
		itemId: event.itemId,
		positionSeconds,
		durationSeconds,
		isPlaying: event.isPlaying,
		volume: event.volume
	};

	// Avoid mutating the large item summary maps on every playback tick.
	// We only sync the stored position back into list data when playback is not active.
	if (!event.isPlaying) {
		patchItemSummary(event.itemId, { playbackPositionSeconds: positionSeconds });
	}

	// Only clear loading state when we receive an actual event from the backend,
	// not when syncing state via polling (which happens before audio is ready)
	if (fromEvent) {
		app.isAudioLoading = false;
	}

	if (event.isPlaying && (!wasPlaying || previousItemId !== event.itemId)) {
		void markItemRead(event.itemId, true);
	}
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
	const queueItems: { itemId: string; url: string; title: string; durationSeconds: number }[] = [];

	for (const item of items) {
		if (!isMediaItem(item) || seen.has(item.id)) {
			continue;
		}

		seen.add(item.id);
		app.audioItemsById[item.id] = item;
		app.itemSummariesById[item.id] = item;
		queueItems.push(itemToQueuedItem(item));
	}

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
	app.itemSummariesById[item.id] = item;

	void audioQueueEnqueue(itemToQueuedItem(item)).catch(console.error);
}

/** Insert an item as the next item to play (index 0 of the manual queue). Deduplicates. */
export function playAudioItemNext(item: MediaListItem): void {
	if (app.currentPlaybackState?.itemId === item.id) {
		return;
	}

	app.audioItemsById[item.id] = item;
	app.itemSummariesById[item.id] = item;

	void audioQueuePlayNext(itemToQueuedItem(item)).catch(console.error);
}

/** Move a queued item one position earlier within its segment. */
export function moveQueuedItemUp(itemId: string): void {
	void audioQueueMoveUp(itemId).catch(console.error);
}

/** Move a queued item one position later within its segment. */
export function moveQueuedItemDown(itemId: string): void {
	void audioQueueMoveDown(itemId).catch(console.error);
}

/** Remove a specific item from whichever queue segment contains it. */
export function removeQueuedItem(itemId: string): void {
	void audioQueueRemove(itemId).catch(console.error);
}

/** Clear both queue segments without affecting the currently playing item. */
export function clearQueue(): void {
	void audioQueueClear().catch(console.error);
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
	const manualQueueIds = app.manualQueue.filter((itemId) => itemId !== item.id);
	const autoQueueIds = deriveAutoContinuation(item.id);

	// Determine playback context from current selection
	const context: { contextType: 'feed' | 'station'; id: string } | null = app.selectedStationId
		? { contextType: 'station', id: app.selectedStationId }
		: app.selectedFeedId
			? { contextType: 'feed', id: app.selectedFeedId }
			: null;

	playAudioItem(item, {
		manualQueueIds,
		autoQueueIds,
		startPositionSeconds: item.playbackPositionSeconds,
		context
	});
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
	await syncAudioSessionFromBackend();
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

	playAudioItem(firstItem, {
		manualQueueIds: rest.map((item) => item.id),
		autoQueueIds: [],
		context: { contextType: 'station', id: stationId }
	});
}

// ---------------------------------------------------------------------------
// Playback context persistence
// ---------------------------------------------------------------------------

/**
 * Restore playback context from persisted storage.
 * Called during app initialization.
 */
export async function restorePlaybackContext(): Promise<void> {
	const context = await loadPlaybackContext();
	if (context) {
		app.playbackContext = context;
	}
}

/**
 * Persist playback context to storage.
 */
export async function persistPlaybackContext(): Promise<void> {
	await savePlaybackContext(app.playbackContext);
}
