import { invokeCommand, isTauriRuntime } from '$lib/services/tauriClient';
import type { Feed, FeedItem, FeedListItem, ItemPage, ItemPageQuery } from '$lib/types/rss';
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

	return measurePerfAsync(
		'tauri.query_items_page',
		() =>
			invokeCommand<ItemPage<FeedListItem>>('query_items_page', {
				query: {
					feedId: query.feedId ?? null,
					section: query.section,
					offset: query.offset,
					limit: query.limit,
					search: query.search ?? null
				}
			}),
		{
			feedId: query.feedId ?? null,
			section: query.section,
			offset: query.offset,
			limit: query.limit,
			search: query.search ?? null
		}
	);
}

export async function getItemDetails(itemId: string): Promise<FeedItem> {
	return invokeCommand<FeedItem>('get_item_details', { itemId });
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
	return invokeCommand<FeedItem>('load_reader_content', { itemId });
}
