import type { Feed } from '$lib/types/feed';
import type { ItemSortOrder } from '$lib/types/item';
import {
	addFeed,
	exportFeed as invokeExportFeed,
	listFeeds,
	refreshFeed,
	removeFeed,
	setFeedSortOrder as persistFeedSortOrder
} from '$lib/services/feed';
import { getCachedImageUrl } from '$lib/services/imageCache';
import { invalidateAllQueries, loadInitialItemsPage } from './items.svelte';
import { selection } from './selection.svelte';
import { getCurrentAudioItem, stopPlayback, removeFromQueuesByFeedId } from './playback.svelte';
import { log } from '$lib/services/log';

export const feedsState = $state({
	feeds: [] as Feed[],
	syncingFeedIds: [] as string[],
	isCreatingFeed: false,
	exportingFeedIds: [] as string[]
});

export const feedImageUrls = $state<Record<string, string>>({});
export const feedImageUrlsLarge = $state<Record<string, string>>({});

export function resetFeedsState(): void {
	feedsState.feeds = [];
	feedsState.syncingFeedIds = [];
	feedsState.isCreatingFeed = false;
	feedsState.exportingFeedIds = [];
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

	for (const feed of feedsState.feeds) {
		if (feed.imageUrl) {
			if (!feedImageUrls[feed.id]) {
				getCachedImageUrl(feed.imageUrl, 256).then((url) => {
					if (url) feedImageUrls[feed.id] = url;
				});
			}
			if (!feedImageUrlsLarge[feed.id]) {
				getCachedImageUrl(feed.imageUrl).then((url) => {
					if (url) feedImageUrlsLarge[feed.id] = url;
				});
			}
		}
	}

	if (
		selection.selectedFeedId &&
		!feedsState.feeds.some((feed) => feed.id === selection.selectedFeedId)
	) {
		selection.selectedFeedId = null;
	}
}

export async function createFeed(url: string): Promise<Feed> {
	feedsState.isCreatingFeed = true;

	try {
		const createdFeed = await addFeed(url);
		await loadFeeds();
		invalidateAllQueries();

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

		await loadInitialItemsPage();

		return refreshedFeed;
	} finally {
		removeSyncingFeed(feedId);
	}
}

export async function deleteExistingFeed(feedId: string): Promise<void> {
	const feed = getFeedById(feedId);
	if (!feed) return;

	const currentAudioItem = getCurrentAudioItem();

	if (currentAudioItem?.feedId === feedId) {
		await stopPlayback();
	}

	await removeFromQueuesByFeedId(feedId);
	await removeFeed(feedId);

	if (selection.selectedFeedId === feedId) {
		selection.selectedFeedId = null;
		selection.selectedStationId = null;
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

	const feedIndex = feedsState.feeds.findIndex((f) => f.id === feedId);
	if (feedIndex >= 0) {
		feedsState.feeds[feedIndex] = { ...feedsState.feeds[feedIndex], sortOrder: order };
	}

	// Persist to SQLite (fire-and-forget)
	void persistFeedSortOrder(feedId, order).catch((error: unknown) => {
		log.error(`Failed to persist feed sort order: ${error}`);
	});

	invalidateAllQueries();
	await loadInitialItemsPage();
}

export function isExportingFeed(feedId: string): boolean {
	return feedsState.exportingFeedIds.includes(feedId);
}

export function addExportingFeed(feedId: string): void {
	if (!feedsState.exportingFeedIds.includes(feedId)) {
		feedsState.exportingFeedIds.push(feedId);
	}
}

export function removeExportingFeed(feedId: string): void {
	const index = feedsState.exportingFeedIds.indexOf(feedId);
	if (index >= 0) {
		feedsState.exportingFeedIds.splice(index, 1);
	}
}

export async function exportExistingFeed(feedId: string): Promise<number> {
	addExportingFeed(feedId);

	try {
		const exportedCount = await invokeExportFeed(feedId);
		return exportedCount;
	} finally {
		removeExportingFeed(feedId);
	}
}
