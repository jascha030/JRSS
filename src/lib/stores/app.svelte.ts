import {
	addFeed,
	getItemDetails,
	listFeeds,
	loadReaderContent,
	markRead,
	queryItemsPage,
	refreshFeed,
	removeFeed,
	savePlayback
} from '$lib/services/feedService';
import type {
	Feed,
	FeedItem,
	FeedItemDetails,
	FeedListItem,
	ItemListSection,
	ItemPageQuery,
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
	feeds: Feed[];
	currentPlaybackState: PlaybackState | null;
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
	feeds: [],
	currentPlaybackState: null,
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

function buildQueryKey(feedId: string | null, section: ItemListSection): QueryKey {
	return `${section}::${feedId ?? 'all-feeds'}`;
}

function getActiveQuerySpec(): { queryKey: QueryKey; query: ItemPageQuery } | null {
	const section = getActiveListSection();

	if (!section) {
		return null;
	}

	return {
		queryKey: buildQueryKey(app.selectedFeedId, section),
		query: {
			feedId: app.selectedFeedId ?? undefined,
			section,
			offset: 0,
			limit: PAGE_SIZE
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
		app.selectedItemId = null;
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

export async function initializeApp(): Promise<void> {
	app.selectedFeedId = null;
	app.selectedItemId = null;
	app.selectedSection = 'all';
	app.currentPlaybackState = null;
	app.isCreatingFeed = false;
	app.syncingFeedIds = [];
	app.readerLoadingItemIds = [];
	app.itemSummariesById = {};
	app.itemDetailsById = {};
	app.audioItemsById = {};
	invalidateAllQueries();
	await loadFeeds();
	await loadInitialItemsPage();
}

export function selectFeed(feedId: string | null): void {
	app.selectedItemId = null;
	app.selectedFeedId = feedId;
	app.selectedSection = feedId ? null : 'all';
	loadInitialItemsPageInBackground();
}

export function selectSection(section: SidebarSection): void {
	app.selectedItemId = null;
	app.selectedFeedId = null;
	app.selectedSection = section;
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

export function playAudioItem(item: FeedListItem): void {
	if (!item.mediaEnclosure) {
		return;
	}

	app.audioItemsById[item.id] = item;
	app.currentPlaybackState = {
		itemId: item.id,
		positionSeconds: item.playbackPositionSeconds,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0,
		isPlaying: false
	};
}

export function stopPlayback(): void {
	app.currentPlaybackState = null;
}

export function setPlaybackPlaying(isPlaying: boolean): void {
	if (!app.currentPlaybackState) {
		return;
	}

	app.currentPlaybackState = {
		...app.currentPlaybackState,
		isPlaying
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
