import {
	addFeed,
	clearPlaybackSession,
	getItemDetails,
	listFeeds,
	loadPlaybackSession,
	loadReaderContent,
	markRead,
	queryItemsPage,
	refreshFeed,
	removeFeed,
	savePlayback,
	savePlaybackSession
} from '$lib/services/feedService';
import type {
	Feed,
	FeedItem,
	FeedItemDetails,
	FeedListItem,
	ItemListSection,
	ItemPageQuery,
	PlaybackSession,
	PlaybackState
} from '$lib/types/rss';
import { logPerf, measurePerfAsync } from '$lib/utils/perfDebug';

export type SidebarSection = 'all' | 'unread' | 'podcasts' | 'settings' | null;

const PAGE_SIZE = 100;
const PAGE_PREFETCH = 1;

type QueryKey = string;
type ItemIdsByIndex = Record<number, string>;
type PageOffsets = Record<number, boolean>;

interface AppState {
	selectedFeedId: string | null;
	selectedItemId: string | null;
	selectedSection: SidebarSection;
	feedSearchTerm: string;
	feeds: Feed[];
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
}

const initialState: AppState = {
	selectedFeedId: null,
	selectedItemId: null,
	selectedSection: 'all',
	feedSearchTerm: '',
	feeds: [],
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
	initialLoadDoneByQueryKey: {}
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

function buildQueryKey(feedId: string | null, section: ItemListSection, search: string): QueryKey {
	const normalizedSearch = normalizeSearchTerm(search);
	const base = `${section}::${feedId ?? 'all-feeds'}`;

	return normalizedSearch ? `${base}::search:${normalizedSearch}` : base;
}

function getActiveQuerySpec(): { queryKey: QueryKey; query: ItemPageQuery } | null {
	const section = getActiveListSection();

	if (!section) {
		return null;
	}

	const search = app.selectedFeedId ? normalizeSearchTerm(app.feedSearchTerm) : '';

	return {
		queryKey: buildQueryKey(app.selectedFeedId, section, search),
		query: {
			feedId: app.selectedFeedId ?? undefined,
			section,
			offset: 0,
			limit: PAGE_SIZE,
			search: search || undefined
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
	return {
		id: item.id,
		feedId: item.feedId,
		title: item.title,
		url: item.url,
		summary: item.summary,
		previewText: item.previewText,
		readerStatus: item.readerStatus,
		readerTitle: item.readerTitle,
		readerByline: item.readerByline,
		readerExcerpt: item.readerExcerpt,
		readerFetchedAt: item.readerFetchedAt,
		publishedAt: item.publishedAt,
		read: item.read,
		playbackPositionSeconds: item.playbackPositionSeconds,
		mediaEnclosure: item.mediaEnclosure
	};
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

function patchItemSummary(itemId: string, patch: Partial<FeedListItem>): void {
	const existingItem = app.itemSummariesById[itemId];

	if (existingItem) {
		app.itemSummariesById[itemId] = {
			...existingItem,
			...patch
		};
	}

	const existingAudioItem = app.audioItemsById[itemId];

	if (existingAudioItem) {
		app.audioItemsById[itemId] = {
			...existingAudioItem,
			...patch
		};
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
		const page = await measurePerfAsync(
			'app.loadPage',
			() =>
				queryItemsPage({
					...baseQuery,
					offset: safeOffset,
					limit: PAGE_SIZE
				}),
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

export function getCurrentAudioItem(): FeedListItem | null {
	const playbackState = app.currentPlaybackState;

	if (!playbackState) {
		return null;
	}

	const currentItem =
		app.audioItemsById[playbackState.itemId] ?? app.itemSummariesById[playbackState.itemId];

	return currentItem?.mediaEnclosure ? currentItem : null;
}

export async function loadFeeds(): Promise<void> {
	app.feeds = await listFeeds();

	if (app.selectedFeedId && !app.feeds.some((feed) => feed.id === app.selectedFeedId)) {
		app.selectedFeedId = null;
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
 * Must be called after feeds and items are loaded so that item
 * summaries are available for resolution.
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
		return;
	}

	// Resolve the current item
	const currentItem =
		app.itemSummariesById[session.currentItemId] ?? app.audioItemsById[session.currentItemId];

	if (!currentItem?.mediaEnclosure) {
		// Current item gone — try to restore just the queues
		app.manualQueue = filterResolvableAudioIds(session.manualQueue);
		app.autoQueue = filterResolvableAudioIds(session.autoQueue);
		return;
	}

	// Hydrate current item into audioItemsById
	app.audioItemsById[currentItem.id] = currentItem;

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
	app.manualQueue = filterResolvableAudioIds(session.manualQueue);
	app.autoQueue = filterResolvableAudioIds(session.autoQueue);

	// Remove current item from queues if it somehow ended up there
	app.manualQueue = app.manualQueue.filter((id) => id !== currentItem.id);
	app.autoQueue = app.autoQueue.filter((id) => id !== currentItem.id);
}

/**
 * Filter a list of item IDs to only those that resolve to audio items
 * currently available in the store. Registers resolved items in audioItemsById.
 */
function filterResolvableAudioIds(ids: string[]): string[] {
	const result: string[] = [];

	for (const id of ids) {
		const item = app.itemSummariesById[id] ?? app.audioItemsById[id];

		if (item?.mediaEnclosure) {
			app.audioItemsById[id] = item;
			result.push(id);
		}
	}

	return result;
}

export async function initializeApp(): Promise<void> {
	app.selectedFeedId = null;
	app.selectedItemId = null;
	app.selectedSection = 'all';
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
	await loadFeeds();
	await loadInitialItemsPage();
	await restoreSession();
}

export function selectFeed(feedId: string | null): void {
	app.selectedItemId = null;
	app.selectedFeedId = feedId;
	app.selectedSection = feedId ? null : 'all';
	app.feedSearchTerm = '';
	loadInitialItemsPageInBackground();
}

export function selectSection(section: SidebarSection): void {
	app.selectedItemId = null;
	app.selectedFeedId = null;
	app.selectedSection = section;
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

export function selectItem(itemId: string): void {
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

export function playAudioItem(item: FeedListItem, { autoPlay = true } = {}): void {
	if (!item.mediaEnclosure) {
		return;
	}

	app.audioItemsById[item.id] = item;
	app.currentPlaybackState = {
		itemId: item.id,
		positionSeconds: item.playbackPositionSeconds,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0,
		isPlaying: false,
		autoPlay
	};
}

export function stopPlayback(): void {
	app.currentPlaybackState = null;
	persistSession();
}

export function setPlaybackPlaying(isPlaying: boolean): void {
	if (!app.currentPlaybackState) {
		return;
	}

	app.currentPlaybackState = {
		...app.currentPlaybackState,
		isPlaying,
		autoPlay: false
	};
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

/** Resolve an item ID to a FeedListItem with a mediaEnclosure, or null. */
function resolveAudioItem(itemId: string): FeedListItem | null {
	const item = app.audioItemsById[itemId] ?? app.itemSummariesById[itemId];
	return item?.mediaEnclosure ? item : null;
}

/**
 * Shift the first resolvable audio item off a queue array, mutating it in place.
 * Returns the resolved item, or null if the queue is exhausted.
 */
function shiftNextAudioItem(queue: string[]): FeedListItem | null {
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
export function getUpcomingQueue(): FeedListItem[] {
	const items: FeedListItem[] = [];
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
	const seen = new Set<string>();
	const ids: string[] = [];

	for (const item of items) {
		if (!item.mediaEnclosure || seen.has(item.id)) {
			continue;
		}

		seen.add(item.id);
		app.audioItemsById[item.id] = item;
		ids.push(item.id);
	}

	app.manualQueue = ids;
	app.autoQueue = [];
}

/** Append an item to the end of the manual queue. No-op if already queued or currently playing. */
export function enqueueAudioItem(item: FeedListItem): void {
	if (!item.mediaEnclosure) {
		return;
	}

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
	persistSession();
}

/** Insert an item as the next item to play (index 0 of the manual queue). Deduplicates. */
export function playAudioItemNext(item: FeedListItem): void {
	if (!item.mediaEnclosure) {
		return;
	}

	if (app.currentPlaybackState?.itemId === item.id) {
		return;
	}

	app.audioItemsById[item.id] = item;
	const filteredManual = app.manualQueue.filter((id) => id !== item.id);
	app.autoQueue = app.autoQueue.filter((id) => id !== item.id);
	app.manualQueue = [item.id, ...filteredManual];
	persistSession();
}

/** Move a queued item one position earlier within its segment. */
export function moveQueuedItemUp(itemId: string): void {
	const manualIndex = app.manualQueue.indexOf(itemId);

	if (manualIndex > 0) {
		const next = [...app.manualQueue];
		[next[manualIndex - 1], next[manualIndex]] = [next[manualIndex], next[manualIndex - 1]];
		app.manualQueue = next;
		return;
	}

	// If it's the first auto item and there's a manual queue, promote to last manual position
	const autoIndex = app.autoQueue.indexOf(itemId);

	if (autoIndex > 0) {
		const next = [...app.autoQueue];
		[next[autoIndex - 1], next[autoIndex]] = [next[autoIndex], next[autoIndex - 1]];
		app.autoQueue = next;
	} else if (autoIndex === 0) {
		// Promote from auto to manual (move to end of manual queue)
		app.autoQueue = app.autoQueue.filter((id) => id !== itemId);
		app.manualQueue = [...app.manualQueue, itemId];
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
		return;
	}

	if (manualIndex === app.manualQueue.length - 1) {
		// Demote from manual to auto (move to start of auto queue)
		app.manualQueue = app.manualQueue.filter((id) => id !== itemId);
		app.autoQueue = [itemId, ...app.autoQueue];
		return;
	}

	const autoIndex = app.autoQueue.indexOf(itemId);

	if (autoIndex >= 0 && autoIndex < app.autoQueue.length - 1) {
		const next = [...app.autoQueue];
		[next[autoIndex], next[autoIndex + 1]] = [next[autoIndex + 1], next[autoIndex]];
		app.autoQueue = next;
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
	persistSession();
}

/** Clear both queue segments without affecting the currently playing item. */
export function clearQueue(): void {
	app.manualQueue = [];
	app.autoQueue = [];
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

		if (candidate?.mediaEnclosure) {
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
export function startPlaybackFromContext(item: FeedListItem): void {
	if (!item.mediaEnclosure) {
		return;
	}

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
