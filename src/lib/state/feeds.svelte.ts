import type { Feed, ItemSortOrder } from '$lib/types/rss';
import {
	addFeed,
	listFeeds,
	refreshFeed,
	removeFeed,
	setFeedSortOrder as persistFeedSortOrder
} from '$lib/services/feedService';
import { invalidateAllQueries, loadInitialItemsPage } from './items.svelte';
import { selection } from './selection.svelte';

export const feedsState = $state({
	feeds: [] as Feed[],
	syncingFeedIds: [] as string[],
	isCreatingFeed: false
});

export function resetFeedsState(): void {
	feedsState.feeds = [];
	feedsState.syncingFeedIds = [];
	feedsState.isCreatingFeed = false;
}

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

		// Select the new feed
		selection.selectedFeedId = createdFeed.id;
		selection.selectedStationId = null;
		selection.selectedSection = null;
		selection.selectedItemId = null;
		selection.feedSearchTerm = '';

		// Invalidate and reload for the new selection
		invalidateAllQueries();
		await loadInitialItemsPage();

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

		// Reload the current view if we're looking at this feed
		await loadInitialItemsPage();

		return refreshedFeed;
	} finally {
		removeSyncingFeed(feedId);
	}
}

export async function deleteExistingFeed(feedId: string): Promise<void> {
	const feed = getFeedById(feedId);
	if (!feed) return;

	// Stop playback and remove from queues if needed
	const { getCurrentAudioItem, stopPlayback, removeFromQueuesByFeedId } =
		await import('./playback.svelte');
	const currentAudioItem = getCurrentAudioItem();

	if (currentAudioItem?.feedId === feedId) {
		await stopPlayback();
	}

	await removeFromQueuesByFeedId(feedId);
	await removeFeed(feedId);

	if (selection.selectedFeedId === feedId) {
		selection.selectedFeedId = null;
		selection.selectedStationId = null;
		selection.selectedSection = 'all';
		selection.selectedItemId = null;
	}

	await loadFeeds();
	invalidateAllQueries();
	await loadInitialItemsPage();
}

export async function setFeedSortOrder(order: ItemSortOrder): Promise<void> {
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

	// Invalidate and reload with new sort order
	invalidateAllQueries();
	await loadInitialItemsPage();
}
