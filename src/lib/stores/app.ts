import {
	addFeed,
	listFeeds,
	listItems,
	markRead,
	refreshFeed,
	removeFeed,
	savePlayback
} from '$lib/services/feedService';
import type { Feed, FeedItem, PlaybackState } from '$lib/types/rss';
import { derived, get, writable } from 'svelte/store';

export type SidebarSection = 'all' | 'unread' | 'podcasts' | 'saved' | 'settings';

export const selectedFeedId = writable<string | null>(null);
export const selectedItemId = writable<string | null>(null);
export const selectedSection = writable<SidebarSection>('all');
export const feeds = writable<Feed[]>([]);
export const items = writable<FeedItem[]>([]);
export const currentPlaybackState = writable<PlaybackState | null>(null);
export const isCreatingFeed = writable(false);
export const syncingFeedIds = writable<string[]>([]);

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

		if ($selectedSection === 'saved') {
			filteredItems = filteredItems.filter((item) => item.saved);
		}

		if ($selectedSection === 'settings') {
			return [];
		}

		return filteredItems;
	}
);

export const selectedItem = derived(
	[visibleItems, selectedItemId],
	([$visibleItems, $selectedItemId]) => {
		return $visibleItems.find((item) => item.id === $selectedItemId) ?? null;
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

export async function initializeApp(): Promise<void> {
	await refreshData();
	selectedFeedId.set(null);
	selectSection('all');
	selectedItemId.set(get(visibleItems)[0]?.id ?? null);
	currentPlaybackState.set(null);
	isCreatingFeed.set(false);
	syncingFeedIds.set([]);
}

export function selectFeed(feedId: string | null): void {
	selectedItemId.set(null);
	selectedFeedId.set(feedId);
	selectedSection.set('all');
}

export function selectSection(section: SidebarSection): void {
	selectedItemId.set(null);
	selectedSection.set(section);

	if (section !== 'all') {
		selectedFeedId.set(null);
	}
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

export function selectItem(itemId: string): void {
	selectedItemId.set(itemId);
}

export async function markItemRead(itemId: string, read: boolean): Promise<void> {
	await markRead(itemId, read);
	items.update((currentItems) =>
		currentItems.map((item) => (item.id === itemId ? { ...item, read } : item))
	);
}

export function playAudioItem(item: FeedItem): void {
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
