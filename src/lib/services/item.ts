import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';
import type {
	FeedItem,
	FeedListItem,
	ItemPage,
	ItemPageQuery,
	ItemSortOrder,
	RawFeedItem,
	RawFeedListItem
} from '$lib/types/item';
import { mapRawFeedItem, mapRawFeedListItem } from '$lib/types/item';
import { measurePerfAsync } from '$lib/utils/perfDebug';

export async function queryItemsPage(query: ItemPageQuery): Promise<ItemPage<FeedListItem>> {
	if (!isTauriRuntime()) {
		return {
			items: [],
			totalCount: 0
		};
	}

	const raw = await measurePerfAsync(
		'tauri.query_items_page',
		() =>
			invokeCommand<ItemPage<RawFeedListItem>>('query_items_page', {
				query: {
					feedId: query.feedId ?? null,
					section: query.section,
					offset: query.offset,
					limit: query.limit,
					search: query.search ?? null,
					sortOrder: query.sortOrder ?? 'newest_first'
				}
			}),
		{
			feedId: query.feedId ?? null,
			section: query.section,
			offset: query.offset,
			limit: query.limit,
			search: query.search ?? null,
			sortOrder: query.sortOrder ?? 'newest_first'
		}
	);

	return { items: raw.items.map(mapRawFeedListItem), totalCount: raw.totalCount };
}

export async function getItemDetails(itemId: string): Promise<FeedItem> {
	const raw = await invokeCommand<RawFeedItem>('get_item_details', { itemId });
	return mapRawFeedItem(raw);
}

export async function markRead(itemId: string, read: boolean): Promise<void> {
	await invokeCommand('mark_read', { itemId, read });
}

export async function markReadBatch(itemIds: string[], read: boolean): Promise<void> {
	await invokeCommand('mark_read_batch', { itemIds, read });
}

export async function markFavorite(itemId: string, favorite: boolean): Promise<void> {
	await invokeCommand('mark_favorite', { itemId, favorite });
}

export async function markFavoriteBatch(itemIds: string[], favorite: boolean): Promise<void> {
	await invokeCommand('mark_favorite_batch', { itemIds, favorite });
}

export async function savePlayback(itemId: string, positionSeconds: number): Promise<void> {
	await invokeCommand('save_playback', {
		itemId,
		positionSeconds: Math.max(0, Math.floor(positionSeconds))
	});
}

export async function loadReaderContent(itemId: string): Promise<FeedItem> {
	const raw = await invokeCommand<RawFeedItem>('load_reader_content', { itemId });
	return mapRawFeedItem(raw);
}

export async function getItemsByIds(itemIds: string[]): Promise<FeedListItem[]> {
	if (itemIds.length === 0) {
		return [];
	}

	const raw = await invokeCommand<RawFeedListItem[]>('get_items_by_ids', { itemIds });
	return raw.map(mapRawFeedListItem);
}

export interface ItemsQuery {
	feedId?: string;
	stationId?: string;
	section: 'all' | 'unread' | 'media' | 'favorites';
	offset: number;
	limit: number;
	search?: string;
	sortOrder: ItemSortOrder;
}

export async function queryItems(query: ItemsQuery): Promise<ItemPage<FeedListItem>> {
	if (!isTauriRuntime()) {
		return {
			items: [],
			totalCount: 0
		};
	}

	const raw = await measurePerfAsync(
		'tauri.query_items',
		() =>
			invokeCommand<ItemPage<RawFeedListItem>>('query_items', {
				query: {
					feedId: query.feedId ?? null,
					stationId: query.stationId ?? null,
					section: query.section,
					offset: query.offset,
					limit: query.limit,
					search: query.search ?? null,
					sortOrder: query.sortOrder ?? 'newest_first'
				}
			}),
		{
			feedId: query.feedId ?? null,
			stationId: query.stationId ?? null,
			section: query.section,
			offset: query.offset
		}
	);

	return {
		items: raw.items.map(mapRawFeedListItem),
		totalCount: raw.totalCount
	};
}
