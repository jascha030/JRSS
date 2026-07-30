import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';
import type { Feed, PodcastSearchResult } from '$lib/types/feed';
import type { ItemSortOrder } from '$lib/types/item';

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

export async function setFeedSortOrder(
	feedId: string,
	sortOrder: ItemSortOrder | null
): Promise<void> {
	await invokeCommand('set_feed_sort_order', {
		feedId,
		sortOrder
	});
}

export async function fetchFeedRawXml(feedId: string): Promise<string> {
	if (!isTauriRuntime()) {
		throw new Error('Feed inspector requires the Tauri runtime.');
	}
	return invokeCommand<string>('fetch_feed_raw', { feedId });
}

export async function getFeedsUnreadCounts(): Promise<Record<string, number>> {
	if (!isTauriRuntime()) {
		return {};
	}
	return invokeCommand<Record<string, number>>('get_feeds_unread_counts');
}

export async function searchPodcasts(term: string): Promise<PodcastSearchResult[]> {
	return invokeCommand<PodcastSearchResult[]>('search_podcasts', { term });
}

export async function exportFeed(feedId: string): Promise<number> {
	return invokeCommand<number>('export_feed', { feedId });
}
