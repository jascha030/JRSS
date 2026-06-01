import { feedsState } from '$lib/state/feeds.svelte';
import { selection } from '$lib/state/selection.svelte';
import { getCurrentAudioItem, getPlaybackContext } from '$lib/state/playback.svelte';
import { stationsState } from '$lib/state/stations.svelte';
import { appUi, requestScrollToItem } from '$lib/hooks/useAppUi.svelte';
import {
	navigateToFeed,
	navigateToFeedItem,
	navigateToStation,
	navigateToStationItem
} from './app-router';

export function navigateToCurrentAudioItem(): void {
	const item = getCurrentAudioItem();
	if (!item) {
		return;
	}

	if (appUi.playerMode === 'cover') {
		appUi.playerMode = 'default';
	}

	const context = getPlaybackContext();
	const station =
		context?.contextType === 'station'
			? stationsState.stations.find((s) => s.id === context.id)
			: null;

	if (station && station.feedIds.includes(item.feedId)) {
		void navigateToStationItem(station.id, item.id).then(() => {
			requestScrollToItem(item.id);
		});
		return;
	}

	void navigateToFeedItem(item.feedId, item.id).then(() => {
		requestScrollToItem(item.id);
	});
}

export function cycleSource(direction: 1 | -1): void {
	const sources = [
		...feedsState.feeds.map((feed) => ({ type: 'feed' as const, id: feed.id })),
		...stationsState.stations.map((station) => ({ type: 'station' as const, id: station.id }))
	];

	if (sources.length === 0) {
		return;
	}

	let currentIndex = -1;

	if (selection.selectedFeedId) {
		currentIndex = sources.findIndex(
			(source) => source.type === 'feed' && source.id === selection.selectedFeedId
		);
	} else if (selection.selectedStationId) {
		currentIndex = sources.findIndex(
			(source) => source.type === 'station' && source.id === selection.selectedStationId
		);
	}

	if (currentIndex === -1) {
		currentIndex = direction === 1 ? 0 : sources.length - 1;
	} else {
		currentIndex = (currentIndex + direction + sources.length) % sources.length;
	}

	const next = sources[currentIndex];

	if (next.type === 'feed') {
		void navigateToFeed(next.id);
		return;
	}

	void navigateToStation(next.id);
}
