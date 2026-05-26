import { isMediaItem } from '$lib/types/item';
import { feedsState } from './feeds.svelte';
import { inspectorState } from './inspector.svelte';
import { getSelectedItem, itemsState } from './items.svelte';
import { getCurrentAudioItem, getCurrentAudioItemFeed } from './playback.svelte';
import { readerState } from './reader.svelte';
import { selection } from './selection.svelte';
import { stationsState } from './stations.svelte';

export function getSelectedFeed() {
	return feedsState.feeds.find((f) => f.id === selection.selectedFeedId) ?? null;
}

export function getSelectedStation() {
	return stationsState.stations.find((s) => s.id === selection.selectedStationId) ?? null;
}

export function getSelectedItemOrNull() {
	return getSelectedItem();
}

export function getCurrentAudioItemOrNull() {
	return getCurrentAudioItem();
}

export function getCurrentAudioItemFeedOrNull() {
	return getCurrentAudioItemFeed();
}

export function getIsSelectedFeedRefreshing() {
	const feed = getSelectedFeed();
	return feed ? feedsState.syncingFeedIds.includes(feed.id) : false;
}

export function getIsInspectorActive() {
	return inspectorState.activeFeedId !== null;
}

export function getIsSelectedItemReaderLoading() {
	const item = getSelectedItem();
	return item ? readerState.readerLoadingItemIds.includes(item.id) : false;
}

export function getHasSelectedItemReaderContent() {
	const item = getSelectedItem();
	return item?.readerStatus === 'ready';
}

export function getCanUseReaderMode() {
	const item = getSelectedItem();
	return item ? !isMediaItem(item) : false;
}

export function getIsReaderPaneActive() {
	return getHasSelectedItemReaderContent() && getCanUseReaderMode();
}

export function getActiveTotalCount() {
	const queryKey = selection.selectedStationId
		? `station::${selection.selectedStationId}::${stationsState.stations.find((s) => s.id === selection.selectedStationId)?.sortOrder ?? 'newest_first'}`
		: selection.selectedFeedId
			? `all::${selection.selectedFeedId}::newest_first`
			: selection.selectedSection &&
				  selection.selectedSection !== 'home' &&
				  selection.selectedSection !== 'settings'
				? `${selection.selectedSection}::all-feeds::newest_first`
				: null;

	if (!queryKey) return 0;
	return itemsState.totalCountByQueryKey[queryKey] ?? 0;
}
