import {
	addFeed,
	getItemDetails,
	listFeeds,
	listItems,
	loadReaderContent,
	markRead,
	refreshFeed,
	removeFeed,
	savePlayback
} from '$lib/services/feedService';
import type { Feed, FeedItem, FeedItemDetails, FeedListItem, PlaybackState } from '$lib/types/rss';
import { derived, get, writable } from 'svelte/store';

export type SidebarSection = 'all' | 'unread' | 'podcasts' | 'settings' | null;

export const selectedFeedId = writable<string | null>(null);
export const selectedItemId = writable<string | null>(null);
export const selectedSection = writable<SidebarSection>('all');
export const feeds = writable<Feed[]>([]);
export const items = writable<FeedListItem[]>([]);
export const currentPlaybackState = writable<PlaybackState | null>(null);
export const isCreatingFeed = writable(false);
export const syncingFeedIds = writable<string[]>([]);
export const readerLoadingItemIds = writable<string[]>([]);
const itemDetailsById = writable<Record<string, FeedItemDetails>>({});

export const selectedFeed = derived([feeds, selectedFeedId], ([$feeds, $selectedFeedId]) => {
	return $feeds.find((feed) => feed.id === $selectedFeedId) ?? null;
});

export const visibleItems = derived(
	[items, selectedFeedId, selectedSection],
	([$items, $selectedFeedId, $selectedSection]) => {
		let filteredItems = $selectedFeedId
			? $items.filter((item) => item.feedId === $selectedFeedId)
			: $items;

		if ($selectedSection === 'unread') {
			filteredItems = filteredItems.filter((item) => !item.read);
		}

		if ($selectedSection === 'podcasts') {
			filteredItems = filteredItems.filter((item) => Boolean(item.mediaEnclosure));
		}

		if ($selectedSection === 'settings') {
			return [];
		}

		return filteredItems;
	}
);

const selectedListItem = derived(
	[visibleItems, selectedItemId],
	([$visibleItems, $selectedItemId]) => {
		return $visibleItems.find((item) => item.id === $selectedItemId) ?? null;
	}
);

export const selectedItem = derived(
	[selectedListItem, itemDetailsById],
	([$selectedListItem, $itemDetailsById]): FeedItem | null => {
		if (!$selectedListItem) {
			return null;
		}

		const itemDetails = $itemDetailsById[$selectedListItem.id];

		return itemDetails ? { ...$selectedListItem, ...itemDetails } : $selectedListItem;
	}
);

export const selectedItemFeed = derived([feeds, selectedItem], ([$feeds, $selectedItem]) => {
	if (!$selectedItem) {
		return null;
	}

	return $feeds.find((feed) => feed.id === $selectedItem.feedId) ?? null;
});

export const currentAudioItem = derived(
	[items, currentPlaybackState],
	([$items, $playbackState]) => {
		if (!$playbackState) {
			return null;
		}

		const matchingItem = $items.find((item) => item.id === $playbackState.itemId);

		return matchingItem?.mediaEnclosure ? matchingItem : null;
	}
);

function storeItemDetails(item: FeedItem): void {
	itemDetailsById.update((currentDetailsById) => ({
		...currentDetailsById,
		[item.id]: {
			id: item.id,
			summaryText: item.summaryText,
			summaryHtml: item.summaryHtml,
			contentText: item.contentText,
			contentHtml: item.contentHtml,
			readerContentHtml: item.readerContentHtml,
			readerContentText: item.readerContentText
		}
	}));
}

function updateListedItem(nextItem: FeedItem): void {
	items.update((currentItems) =>
		currentItems.map((item) =>
			item.id === nextItem.id
				? {
						...item,
						title: nextItem.title,
						url: nextItem.url,
						summary: nextItem.summary,
						readerStatus: nextItem.readerStatus,
						readerTitle: nextItem.readerTitle,
						readerByline: nextItem.readerByline,
						readerExcerpt: nextItem.readerExcerpt,
						readerFetchedAt: nextItem.readerFetchedAt,
						publishedAt: nextItem.publishedAt,
						read: nextItem.read,
						playbackPositionSeconds: nextItem.playbackPositionSeconds,
						mediaEnclosure: nextItem.mediaEnclosure
					}
				: item
		)
	);
}

function cacheDetailedItem(item: FeedItem): void {
	updateListedItem(item);
	storeItemDetails(item);
}

visibleItems.subscribe(($visibleItems) => {
	const currentSelectedItemId = get(selectedItemId);

	if ($visibleItems.length === 0) {
		if (currentSelectedItemId !== null) {
			selectedItemId.set(null);
		}

		return;
	}

	if (!currentSelectedItemId) {
		selectedItemId.set($visibleItems[0].id);
		return;
	}

	if (!$visibleItems.some((item) => item.id === currentSelectedItemId)) {
		selectedItemId.set($visibleItems[0].id);
	}
});

async function refreshData(): Promise<void> {
	const [nextFeeds, nextItems] = await Promise.all([listFeeds(), listItems()]);
	const playbackState = get(currentPlaybackState);

	feeds.set(nextFeeds);
	items.set(nextItems);

	if (!playbackState) {
		return;
	}

	const matchingItem = nextItems.find((item) => item.id === playbackState.itemId);

	if (!matchingItem?.mediaEnclosure) {
		currentPlaybackState.set(null);
		return;
	}

	currentPlaybackState.set({
		itemId: matchingItem.id,
		positionSeconds: matchingItem.playbackPositionSeconds,
		durationSeconds: matchingItem.mediaEnclosure.durationSeconds ?? playbackState.durationSeconds,
		isPlaying: playbackState.isPlaying
	});
}

function addSyncingFeedId(feedId: string): void {
	syncingFeedIds.update((currentIds) =>
		currentIds.includes(feedId) ? currentIds : [...currentIds, feedId]
	);
}

function removeSyncingFeedId(feedId: string): void {
	syncingFeedIds.update((currentIds) => currentIds.filter((currentId) => currentId !== feedId));
}

function addReaderLoadingItemId(itemId: string): void {
	readerLoadingItemIds.update((currentIds) =>
		currentIds.includes(itemId) ? currentIds : [...currentIds, itemId]
	);
}

function removeReaderLoadingItemId(itemId: string): void {
	readerLoadingItemIds.update((currentIds) =>
		currentIds.filter((currentId) => currentId !== itemId)
	);
}

export async function initializeApp(): Promise<void> {
	await refreshData();
	itemDetailsById.set({});
	selectedFeedId.set(null);
	selectSection('all');
	selectedItemId.set(get(visibleItems)[0]?.id ?? null);
	currentPlaybackState.set(null);
	isCreatingFeed.set(false);
	syncingFeedIds.set([]);
	readerLoadingItemIds.set([]);
}

export function selectFeed(feedId: string | null): void {
	selectedItemId.set(null);
	selectedFeedId.set(feedId);
	selectedSection.set(feedId ? null : 'all');
}

export function selectSection(section: SidebarSection): void {
	selectedItemId.set(null);
	selectedFeedId.set(null);
	selectedSection.set(section);
}

export async function createFeed(url: string): Promise<Feed> {
	isCreatingFeed.set(true);

	try {
		const createdFeed = await addFeed(url);
		await refreshData();
		selectedItemId.set(null);
		selectedFeedId.set(createdFeed.id);
		selectedSection.set('all');

		return createdFeed;
	} finally {
		isCreatingFeed.set(false);
	}
}

export async function refreshExistingFeed(feedId: string): Promise<Feed> {
	addSyncingFeedId(feedId);

	try {
		const refreshedFeed = await refreshFeed(feedId);
		await refreshData();

		return refreshedFeed;
	} finally {
		removeSyncingFeedId(feedId);
	}
}

export async function deleteFeed(feedId: string): Promise<void> {
	const playbackState = get(currentPlaybackState);
	const activeAudioItem = get(items).find((item) => item.id === playbackState?.itemId);

	await removeFeed(feedId);
	await refreshData();

	if (get(selectedFeedId) === feedId) {
		selectedFeedId.set(null);
	}

	if (activeAudioItem?.feedId === feedId) {
		currentPlaybackState.set(null);
	}

	removeSyncingFeedId(feedId);
}

export async function loadItemDetails(itemId: string): Promise<FeedItem> {
	const currentItem = get(items).find((item) => item.id === itemId);
	const currentItemDetails = get(itemDetailsById)[itemId];

	if (currentItem && currentItemDetails) {
		return { ...currentItem, ...currentItemDetails };
	}

	const detailedItem = await getItemDetails(itemId);
	cacheDetailedItem(detailedItem);

	return detailedItem;
}

export function selectItem(itemId: string): void {
	selectedItemId.set(itemId);
}

export async function markItemRead(itemId: string, read: boolean): Promise<void> {
	await markRead(itemId, read);
	items.update((currentItems) =>
		currentItems.map((item) => (item.id === itemId ? { ...item, read } : item))
	);
}

export async function loadReaderView(itemId: string): Promise<FeedItem> {
	addReaderLoadingItemId(itemId);

	try {
		const updatedItem = await loadReaderContent(itemId);
		cacheDetailedItem(updatedItem);

		return updatedItem;
	} finally {
		removeReaderLoadingItemId(itemId);
	}
}

export function playAudioItem(item: FeedListItem): void {
	if (!item.mediaEnclosure) {
		return;
	}

	currentPlaybackState.set({
		itemId: item.id,
		positionSeconds: item.playbackPositionSeconds,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0,
		isPlaying: false
	});
}

export function stopPlayback(): void {
	currentPlaybackState.set(null);
}

export function setPlaybackPlaying(isPlaying: boolean): void {
	currentPlaybackState.update((playbackState) => {
		if (!playbackState) {
			return null;
		}

		return { ...playbackState, isPlaying };
	});
}

export async function updatePlaybackPosition(
	positionSeconds: number,
	durationSeconds: number
): Promise<void> {
	const playbackState = get(currentPlaybackState);

	if (!playbackState) {
		return;
	}

	await savePlayback(playbackState.itemId, positionSeconds);

	items.update((currentItems) =>
		currentItems.map((item) =>
			item.id === playbackState.itemId
				? { ...item, playbackPositionSeconds: positionSeconds }
				: item
		)
	);

	currentPlaybackState.update((currentValue) => {
		if (!currentValue) {
			return null;
		}

		return {
			...currentValue,
			positionSeconds,
			durationSeconds
		};
	});
}
