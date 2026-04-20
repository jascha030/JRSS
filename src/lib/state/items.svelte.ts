import type { FeedItem, FeedItemDetails, FeedListItem, MediaListItem } from '$lib/types/rss';
import { isMediaItem } from '$lib/types/rss';
import {
	getItemDetails,
	getItemsByIds,
	markRead,
	queryItemsPage,
	queryStationEpisodes
} from '$lib/services/feedService';
import { measurePerfAsync } from '$lib/utils/perfDebug';
import { selection } from './selection.svelte';
import {
	getActiveQuerySpec,
	getActiveQueryKey,
	getActiveListSection,
	normalizeSearchTerm,
	type ItemsQuerySpec
} from './query-context.svelte';

const PAGE_SIZE = 100;
const PAGE_PREFETCH = 1;

type QueryKey = string;
type ItemIdsByIndex = Record<number, string>;
type PageOffsets = Record<number, boolean>;

export const itemsState = $state({
	itemSummariesById: {} as Record<string, FeedListItem>,
	itemDetailsById: {} as Record<string, FeedItemDetails>,
	itemIdsByIndexByQueryKey: {} as Record<QueryKey, ItemIdsByIndex>,
	totalCountByQueryKey: {} as Record<QueryKey, number>,
	loadedPageOffsetsByQueryKey: {} as Record<QueryKey, PageOffsets>,
	loadingPageOffsetsByQueryKey: {} as Record<QueryKey, PageOffsets>,
	initialLoadDoneByQueryKey: {} as Record<QueryKey, boolean>
});

export function resetItemsState(): void {
	itemsState.itemSummariesById = {};
	itemsState.itemDetailsById = {};
	itemsState.itemIdsByIndexByQueryKey = {};
	itemsState.totalCountByQueryKey = {};
	itemsState.loadedPageOffsetsByQueryKey = {};
	itemsState.loadingPageOffsetsByQueryKey = {};
	itemsState.initialLoadDoneByQueryKey = {};
}

export function invalidateAllQueries(): void {
	itemsState.itemIdsByIndexByQueryKey = {};
	itemsState.loadedPageOffsetsByQueryKey = {};
	itemsState.loadingPageOffsetsByQueryKey = {};
	itemsState.totalCountByQueryKey = {};
	itemsState.initialLoadDoneByQueryKey = {};
	selection.selectedItemId = null;
}

// Re-export for convenience
export { getActiveQuerySpec, getActiveQueryKey, type ItemsQuerySpec };

export function getActiveTotalCount(): number {
	const queryKey = getActiveQueryKey();
	if (!queryKey) {
		return 0;
	}
	return itemsState.totalCountByQueryKey[queryKey] ?? 0;
}

export function getActiveItemIdsByIndex(): ItemIdsByIndex {
	const queryKey = getActiveQueryKey();
	if (!queryKey) {
		return {};
	}
	return itemsState.itemIdsByIndexByQueryKey[queryKey] ?? {};
}

export function getActiveLoadedPageOffsets(): PageOffsets {
	const queryKey = getActiveQueryKey();
	if (!queryKey) {
		return {};
	}
	return itemsState.loadedPageOffsetsByQueryKey[queryKey] ?? {};
}

export function getIsActiveInitialLoading(): boolean {
	const queryKey = getActiveQueryKey();
	if (!queryKey) {
		return false;
	}
	return (
		!itemsState.initialLoadDoneByQueryKey[queryKey] &&
		Object.keys(itemsState.loadingPageOffsetsByQueryKey[queryKey] ?? {}).length > 0
	);
}

function ensureQueryState(queryKey: QueryKey): void {
	if (!itemsState.itemIdsByIndexByQueryKey[queryKey]) {
		itemsState.itemIdsByIndexByQueryKey[queryKey] = {};
	}
	if (!itemsState.loadedPageOffsetsByQueryKey[queryKey]) {
		itemsState.loadedPageOffsetsByQueryKey[queryKey] = {};
	}
	if (!itemsState.loadingPageOffsetsByQueryKey[queryKey]) {
		itemsState.loadingPageOffsetsByQueryKey[queryKey] = {};
	}
	if (itemsState.totalCountByQueryKey[queryKey] === undefined) {
		itemsState.totalCountByQueryKey[queryKey] = 0;
	}
	if (itemsState.initialLoadDoneByQueryKey[queryKey] === undefined) {
		itemsState.initialLoadDoneByQueryKey[queryKey] = false;
	}
}

function resetQueryState(queryKey: QueryKey): void {
	itemsState.itemIdsByIndexByQueryKey[queryKey] = {};
	itemsState.loadedPageOffsetsByQueryKey[queryKey] = {};
	itemsState.loadingPageOffsetsByQueryKey[queryKey] = {};
	itemsState.totalCountByQueryKey[queryKey] = 0;
	itemsState.initialLoadDoneByQueryKey[queryKey] = false;
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

export function storeItemDetails(item: FeedItem): void {
	itemsState.itemDetailsById[item.id] = {
		id: item.id,
		summaryText: item.summaryText,
		summaryHtml: item.summaryHtml,
		contentText: item.contentText,
		contentHtml: item.contentHtml,
		readerContentHtml: item.readerContentHtml,
		readerContentText: item.readerContentText
	};
}

// ---------------------------------------------------------------------------
// Shared item mutation helpers
// These ensure both itemsState and playback audioItemsById stay in sync
// ---------------------------------------------------------------------------

/**
 * Register an item in the shared cache.
 * Call this when loading items that may be used by playback.
 */
export function registerItem(item: FeedListItem): void {
	itemsState.itemSummariesById[item.id] = item;
}

/**
 * Register multiple items in the shared cache.
 */
export function registerItems(items: FeedListItem[]): void {
	for (const item of items) {
		itemsState.itemSummariesById[item.id] = item;
	}
}

/**
 * Patch shared base fields on an item.
 * Updates itemsState. Playback module should call this via import.
 */
export function patchItemSummary(
	itemId: string,
	patch: Partial<Pick<FeedListItem, 'read' | 'playbackPositionSeconds'>>
): void {
	const existingItem = itemsState.itemSummariesById[itemId];
	if (existingItem) {
		itemsState.itemSummariesById[itemId] = { ...existingItem, ...patch };
	}
}

export function mergeDetailedItem(item: FeedItem): void {
	itemsState.itemSummariesById[item.id] = toFeedListItem(item);
	storeItemDetails(item);
}

function mergeItemsPage(
	queryKey: QueryKey,
	offset: number,
	items: FeedListItem[],
	totalCount: number
): void {
	ensureQueryState(queryKey);
	const itemIdsByIndex = itemsState.itemIdsByIndexByQueryKey[queryKey];

	for (const [index, item] of items.entries()) {
		itemsState.itemSummariesById[item.id] = item;
		itemIdsByIndex[offset + index] = item.id;
	}

	itemsState.totalCountByQueryKey[queryKey] = totalCount;
	itemsState.loadedPageOffsetsByQueryKey[queryKey][offset] = true;
	itemsState.initialLoadDoneByQueryKey[queryKey] = true;
}

function getFirstLoadedItemId(queryKey: QueryKey): string | null {
	const itemIdsByIndex = itemsState.itemIdsByIndexByQueryKey[queryKey];
	if (!itemIdsByIndex) {
		return null;
	}

	const sortedIndexes = Object.keys(itemIdsByIndex)
		.map(Number)
		.sort((a, b) => a - b);

	for (const index of sortedIndexes) {
		const itemId = itemIdsByIndex[index];
		if (itemId) {
			return itemId;
		}
	}
	return null;
}

function ensureSelectionAfterPageLoad(queryKey: QueryKey): void {
	const totalCount = itemsState.totalCountByQueryKey[queryKey] ?? 0;
	if (totalCount === 0) {
		if (!normalizeSearchTerm(selection.feedSearchTerm)) {
			selection.selectedItemId = null;
		}
		return;
	}

	if (selection.selectedItemId && itemsState.itemSummariesById[selection.selectedItemId]) {
		return;
	}

	selection.selectedItemId = getFirstLoadedItemId(queryKey);
}

async function loadPage(spec: ItemsQuerySpec, offset: number): Promise<void> {
	const safeOffset = Math.max(0, Math.floor(offset / PAGE_SIZE) * PAGE_SIZE);
	ensureQueryState(spec.queryKey);

	if (itemsState.loadedPageOffsetsByQueryKey[spec.queryKey][safeOffset]) {
		return;
	}

	if (itemsState.loadingPageOffsetsByQueryKey[spec.queryKey][safeOffset]) {
		return;
	}

	itemsState.loadingPageOffsetsByQueryKey[spec.queryKey][safeOffset] = true;

	try {
		const page = await measurePerfAsync(
			'items.loadPage',
			async () => {
				if (spec.kind === 'station-items') {
					return await queryStationEpisodes(spec.stationId, safeOffset, PAGE_SIZE);
				} else {
					return await queryItemsPage({
						...spec.query,
						offset: safeOffset,
						limit: PAGE_SIZE
					});
				}
			},
			{ queryKey: spec.queryKey, offset: safeOffset }
		);

		mergeItemsPage(spec.queryKey, safeOffset, page.items, page.totalCount);

		if (getActiveQueryKey() === spec.queryKey) {
			ensureSelectionAfterPageLoad(spec.queryKey);
		}
	} finally {
		delete itemsState.loadingPageOffsetsByQueryKey[spec.queryKey][safeOffset];
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

export async function loadInitialItemsPage(): Promise<void> {
	const querySpec = getActiveQuerySpec();

	if (!querySpec) {
		selection.selectedItemId = null;
		return;
	}

	resetQueryState(querySpec.queryKey);
	await loadPage(querySpec, 0);
}

export async function ensureVisibleRangeLoaded(
	startIndex: number,
	endIndex: number
): Promise<void> {
	const querySpec = getActiveQuerySpec();
	if (!querySpec) {
		return;
	}

	const totalCount = itemsState.totalCountByQueryKey[querySpec.queryKey];
	const candidateOffsets = getPageOffsetsForRange(startIndex, endIndex);

	for (const offset of candidateOffsets) {
		if (totalCount !== undefined && totalCount > 0 && offset >= totalCount) {
			continue;
		}
		await loadPage(querySpec, offset);
	}
}

export async function ensureItemLoaded(itemId: string): Promise<void> {
	const querySpec = getActiveQuerySpec();
	if (!querySpec) {
		return;
	}

	const queryKey = querySpec.queryKey;
	const itemIdsByIndex = itemsState.itemIdsByIndexByQueryKey[queryKey];

	// Check if item is already loaded
	if (itemIdsByIndex && Object.values(itemIdsByIndex).includes(itemId)) {
		return;
	}

	const totalCount = itemsState.totalCountByQueryKey[queryKey] ?? 0;
	if (totalCount === 0) {
		return;
	}

	// Load pages until we find the item or exhaust the list
	let offset = 0;
	const maxOffset = Math.min(totalCount, 10000);

	while (offset < maxOffset) {
		await loadPage(querySpec, offset);
		const updatedItemIdsByIndex = itemsState.itemIdsByIndexByQueryKey[queryKey];
		if (updatedItemIdsByIndex && Object.values(updatedItemIdsByIndex).includes(itemId)) {
			return;
		}
		offset += PAGE_SIZE;
	}
}

export async function loadItemDetails(itemId: string): Promise<FeedItem> {
	const currentItem = itemsState.itemSummariesById[itemId];
	const currentItemDetails = itemsState.itemDetailsById[itemId];

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

export async function markItemRead(itemId: string, read: boolean): Promise<void> {
	const previousItem = itemsState.itemSummariesById[itemId];
	patchItemSummary(itemId, { read });

	try {
		await markRead(itemId, read);
		if (getActiveListSection() === 'unread') {
			await loadInitialItemsPage();
		}
	} catch (error) {
		if (previousItem) {
			itemsState.itemSummariesById[itemId] = previousItem;
		}
		throw error;
	}
}

export async function loadItemsByIds(itemIds: string[]): Promise<FeedListItem[]> {
	const items = await getItemsByIds(itemIds);
	for (const item of items) {
		itemsState.itemSummariesById[item.id] = item;
	}
	return items;
}

export function getSelectedItem(): FeedItem | null {
	if (!selection.selectedItemId) {
		return null;
	}

	const listItem = itemsState.itemSummariesById[selection.selectedItemId];
	if (!listItem) {
		return null;
	}

	const details = itemsState.itemDetailsById[selection.selectedItemId];
	return details ? { ...listItem, ...details } : listItem;
}

/**
 * Resolve an item by ID, checking itemSummariesById.
 * Used by playback to look up queue items.
 */
export function getItemById(itemId: string): FeedListItem | null {
	return itemsState.itemSummariesById[itemId] ?? null;
}

/**
 * Resolve a media item by ID.
 */
export function getMediaItemById(itemId: string): MediaListItem | null {
	const item = itemsState.itemSummariesById[itemId];
	return item && isMediaItem(item) ? item : null;
}
