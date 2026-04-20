import type { Feed, Station } from '$lib/types/rss';

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

function triggerItemsReload(): void {
	void import('./items.svelte')
		.then(({ loadInitialItemsPageInBackground }) => {
			loadInitialItemsPageInBackground();
		})
		.catch((error: unknown) => {
			console.error('Failed to trigger items reload.', error);
		});
}

export function selectFeed(feedId: string | null): void {
	selection.selectedFeedId = feedId;
	selection.selectedStationId = null;
	selection.selectedSection = feedId ? null : 'all';
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
	triggerItemsReload();
}

export function selectStation(stationId: string): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = stationId;
	selection.selectedSection = null;
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
	triggerItemsReload();
}

export function selectSection(section: SidebarSection): void {
	selection.selectedFeedId = null;
	selection.selectedStationId = null;
	selection.selectedSection = section;
	selection.selectedItemId = null;
	selection.feedSearchTerm = '';
	triggerItemsReload();
}

export function setSelectedItem(itemId: string | null): void {
	selection.selectedItemId = itemId;
}

export function normalizeSearchTerm(term: string): string {
	return term.trim().toLowerCase();
}

export function setFeedSearchTerm(term: string): void {
	if (!selection.selectedFeedId) {
		return;
	}

	const previousNormalized = normalizeSearchTerm(selection.feedSearchTerm);
	const nextNormalized = normalizeSearchTerm(term);

	selection.feedSearchTerm = term;

	if (previousNormalized !== nextNormalized) {
		triggerItemsReload();
	}
}

export function getSelectedFeed(
	feeds: Feed[],
	selectedFeedId: string | null = selection.selectedFeedId
): Feed | null {
	return feeds.find((feed) => feed.id === selectedFeedId) ?? null;
}

export function getSelectedStation(
	stations: Station[],
	selectedStationId: string | null = selection.selectedStationId
): Station | null {
	return stations.find((station) => station.id === selectedStationId) ?? null;
}

export function getActiveListSection(
	selectedSection: SidebarSection = selection.selectedSection,
	selectedFeedId: string | null = selection.selectedFeedId
): 'all' | 'unread' | 'media' | null {
	if (selectedSection === 'settings') {
		return null;
	}

	if (selectedFeedId) {
		return 'all';
	}

	if (!selectedSection) {
		return 'all';
	}

	return selectedSection;
}
