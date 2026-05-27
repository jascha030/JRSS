import type { Feed } from '$lib/types/feed';
import type { Station } from '$lib/types/station';

export type SidebarSection = 'home' | 'all' | 'unread' | 'media' | 'settings' | null;

export type RouteSelectionState = {
	selectedFeedId: string | null;
	selectedStationId: string | null;
	selectedSection: SidebarSection;
	selectedItemId: string | null;
	searchTerm: string;
};

export const selection = $state({
	selectedFeedId: null as string | null,
	selectedStationId: null as string | null,
	selectedSection: 'home' as SidebarSection,
	selectedItemId: null as string | null,
	feedSearchTerm: '',
	stationSearchTerm: '',
	sectionSearchTerm: ''
});

export function resetSelectionState(): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = null;
	selection.selectedSection = 'home';
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
	selection.stationSearchTerm = '';
	selection.sectionSearchTerm = '';
}

export function applyRouteSelection(state: RouteSelectionState): void {
	selection.selectedFeedId = state.selectedFeedId;
	selection.selectedStationId = state.selectedStationId;
	selection.selectedSection = state.selectedSection;
	selection.selectedItemId = state.selectedItemId;
	selection.feedSearchTerm = state.selectedFeedId ? state.searchTerm : '';
	selection.stationSearchTerm = state.selectedStationId ? state.searchTerm : '';
	selection.sectionSearchTerm =
		state.selectedFeedId === null &&
		state.selectedStationId === null &&
		(state.selectedSection === 'all' ||
			state.selectedSection === 'unread' ||
			state.selectedSection === 'media')
			? state.searchTerm
			: '';
}

export function selectFeed(feedId: string | null): void {
	const isReselecting = selection.selectedFeedId === feedId;

	selection.selectedFeedId = feedId;
	selection.selectedStationId = null;
	selection.selectedSection = feedId ? null : 'all';
	selection.feedSearchTerm = '';
	selection.sectionSearchTerm = '';

	if (!isReselecting) {
		selection.selectedItemId = null;
	}
}

export function selectStation(stationId: string): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = stationId;
	selection.selectedSection = null;
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
	selection.stationSearchTerm = '';
	selection.sectionSearchTerm = '';
}

export function selectSection(section: SidebarSection): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = null;
	selection.selectedSection = section;
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
	selection.sectionSearchTerm = '';
}

export function selectItem(itemId: string | null): void {
	selection.selectedItemId = itemId;
}

export function setFeedSearchTerm(term: string): void {
	selection.feedSearchTerm = term;
}

export function setStationSearchTerm(term: string): void {
	selection.stationSearchTerm = term;
}

export function setSectionSearchTerm(term: string): void {
	selection.sectionSearchTerm = term;
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
