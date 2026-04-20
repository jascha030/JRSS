import type { Station, Feed } from '$lib/types/rss';

export type SidebarSection = 'all' | 'unread' | 'media' | 'settings' | null;

export const selection = $state({
	selectedFeedId: null as string | null,
	selectedStationId: null as string | null,
	selectedSection: 'all' as SidebarSection,
	selectedItemId: null as string | null,
	feedSearchTerm: ''
});

export function resetSelectionState(): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = null;
	selection.selectedSection = 'all';
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
}

export function selectFeed(feedId: string | null): void {
	selection.selectedFeedId = feedId;
	selection.selectedStationId = null;
	selection.selectedSection = feedId ? null : 'all';
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
}

export function selectStation(stationId: string): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = stationId;
	selection.selectedSection = null;
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
}

export function selectSection(section: SidebarSection): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = null;
	selection.selectedSection = section;
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
}

export function selectItem(itemId: string | null): void {
	selection.selectedItemId = itemId;
}

export function setFeedSearchTerm(term: string): void {
	selection.feedSearchTerm = term;
}

/**
 * Get the currently selected feed from the provided feeds array.
 */
export function getSelectedFeed(feeds: Feed[]): Feed | null {
	return feeds.find((feed) => feed.id === selection.selectedFeedId) ?? null;
}

/**
 * Get the currently selected station from the provided stations array.
 */
export function getSelectedStation(stations: Station[]): Station | null {
	return stations.find((station) => station.id === selection.selectedStationId) ?? null;
}
