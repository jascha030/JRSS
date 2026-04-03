import { invokeCommand, isTauriRuntime } from '$lib/services/tauriClient';
import type { Feed, FeedItem } from '$lib/types/rss';

function normalizeFeedUrl(url: string): string {
	return new URL(url).toString();
}

export async function listFeeds(): Promise<Feed[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	return invokeCommand<Feed[]>('list_feeds');
}

export async function addFeed(url: string): Promise<Feed> {
	let normalizedUrl: string;

	try {
		normalizedUrl = normalizeFeedUrl(url);
	} catch {
		throw new Error('Enter a valid feed URL.');
	}

	return invokeCommand<Feed>('add_feed', { url: normalizedUrl });
}

export async function refreshFeed(id: string): Promise<Feed> {
	return invokeCommand<Feed>('refresh_feed', { id });
}

export async function removeFeed(id: string): Promise<void> {
	await invokeCommand('remove_feed', { id });
}

export async function listItems(feedId?: string): Promise<FeedItem[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	return invokeCommand<FeedItem[]>('list_items', { feedId: feedId ?? null });
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
