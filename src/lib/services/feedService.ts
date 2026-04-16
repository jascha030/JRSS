import { invokeCommand, isTauriRuntime } from '$lib/services/tauriClient';
import type {
	Feed,
	FeedItem,
	FeedListItem,
	ItemPage,
	ItemPageQuery,
	ItemSortOrder,
	PlaybackSession,
	RawFeedItem,
	RawFeedListItem
} from '$lib/types/rss';
import { mapRawFeedItem, mapRawFeedListItem } from '$lib/types/rss';
import { measurePerfAsync } from '$lib/utils/perfDebug';

function normalizeFeedInput(url: string): string {
	return url.trim();
}

export async function listFeeds(): Promise<Feed[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	return invokeCommand<Feed[]>('list_feeds');
}

export async function addFeed(url: string): Promise<Feed> {
	const normalizedInput = normalizeFeedInput(url);

	if (!normalizedInput) {
		throw new Error('Enter a feed URL, Apple Podcasts URL, or Apple Podcasts ID.');
	}

	return invokeCommand<Feed>('add_feed', { url: normalizedInput });
}

export async function refreshFeed(id: string): Promise<Feed> {
	return invokeCommand<Feed>('refresh_feed', { id });
}

export async function removeFeed(id: string): Promise<void> {
	await invokeCommand('remove_feed', { id });
}

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

export async function savePlaybackSession(session: PlaybackSession): Promise<void> {
	await invokeCommand('save_playback_session', { session });
}

export async function loadPlaybackSession(): Promise<PlaybackSession | null> {
	if (!isTauriRuntime()) {
		return null;
	}

	return invokeCommand<PlaybackSession | null>('load_playback_session');
}

export async function clearPlaybackSession(): Promise<void> {
	await invokeCommand('clear_playback_session');
}

export async function getItemsByIds(itemIds: string[]): Promise<FeedListItem[]> {
	if (itemIds.length === 0) {
		return [];
	}

	const raw = await invokeCommand<RawFeedListItem[]>('get_items_by_ids', { itemIds });
	return raw.map(mapRawFeedListItem);
}

export async function setFeedSortOrder(
	feedId: string,
	sortOrder: ItemSortOrder | null
): Promise<void> {
	await invokeCommand('set_feed_sort_order', {
		feedId,
		sortOrder
	});
}
