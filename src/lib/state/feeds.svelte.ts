import type { Feed, ItemSortOrder } from '$lib/types/rss';
import {
	addFeed,
	listFeeds,
	refreshFeed,
	removeFeed,
	setFeedSortOrder as persistFeedSortOrder
} from '$lib/services/feedService';
import { invalidateAllQueries } from './items.svelte';
import { selection, selectFeed } from './selection.svelte';

export const feedsState = $state({
	feeds: [] as Feed[],
	syncingFeedIds: [] as string[],
	isCreatingFeed: false
});

export function getFeedById(feedId: string | null): Feed | null {
	if (!feedId) return null;
	return feedsState.feeds.find((f) => f.id === feedId) ?? null;
}

export function addSyncingFeed(feedId: string): void {
	if (!feedsState.syncingFeedIds.includes(feedId)) {
		feedsState.syncingFeedIds.push(feedId);
	}
}

export function removeSyncingFeed(feedId: string): void {
	const index = feedsState.syncingFeedIds.indexOf(feedId);
	if (index >= 0) {
		feedsState.syncingFeedIds.splice(index, 1);
	}
}

export async function loadFeeds(): Promise<void> {
	feedsState.feeds = await listFeeds();

	if (
		selection.selectedFeedId &&
		!feedsState.feeds.some((feed) => feed.id === selection.selectedFeedId)
	) {
		selection.selectedFeedId = null;
		selection.selectedSection = 'all';
	}
}

export async function createFeed(url: string): Promise<Feed> {
	feedsState.isCreatingFeed = true;

	try {
		const createdFeed = await addFeed(url);
		await loadFeeds();
		invalidateAllQueries();
		selectFeed(createdFeed.id);
		return createdFeed;
	} finally {
		feedsState.isCreatingFeed = false;
	}
}

export async function refreshExistingFeed(feedId: string): Promise<Feed> {
	addSyncingFeed(feedId);

	try {
		const refreshedFeed = await refreshFeed(feedId);
		await loadFeeds();
		invalidateAllQueries();

		return refreshedFeed;
	} finally {
		removeSyncingFeed(feedId);
	}
}

export async function deleteExistingFeed(feedId: string): Promise<void> {
	const feed = getFeedById(feedId);
	if (!feed) return;

	const { getCurrentAudioItem, stopPlayback } = await import('./playback.svelte');
	const currentAudioItem = getCurrentAudioItem();
	const { removeFromQueuesByFeedId } = await import('./playback.svelte');

	if (currentAudioItem?.feedId === feedId) {
		await stopPlayback();
	}

	await removeFromQueuesByFeedId(feedId);
	await removeFeed(feedId);

	if (selection.selectedFeedId === feedId) {
		selectFeed(null);
	}

	await loadFeeds();
	invalidateAllQueries();
}

export function setFeedSortOrder(order: ItemSortOrder): void {
	const feedId = selection.selectedFeedId;

	if (!feedId) return;

	const feed = getFeedById(feedId);
	if (!feed) return;

	const currentOrder = feed.sortOrder ?? 'newest_first';
	if (order === currentOrder) {
		return;
	}

	// Update the local feed object immediately for reactive UI
	const feedIndex = feedsState.feeds.findIndex((f) => f.id === feedId);
	if (feedIndex >= 0) {
		feedsState.feeds[feedIndex] = { ...feedsState.feeds[feedIndex], sortOrder: order };
	}

	// Persist to SQLite (fire-and-forget)
	void persistFeedSortOrder(feedId, order).catch((error: unknown) => {
		console.error('Failed to persist feed sort order.', error);
	});

	invalidateAllQueries();
}
