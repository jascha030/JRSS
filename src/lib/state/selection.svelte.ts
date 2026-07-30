export type SidebarSection = 'home' | 'all' | 'unread' | 'media' | 'favorites' | 'settings' | null;

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

function switchSelectionContext(
	feedId: string | null,
	stationId: string | null,
	section: SidebarSection,
	options: { preserveItemId?: boolean } = {}
): void {
	selection.selectedFeedId = feedId;
	selection.selectedStationId = stationId;
	selection.selectedSection = section;
	selection.feedSearchTerm = '';
	selection.stationSearchTerm = '';
	selection.sectionSearchTerm = '';

	if (!options.preserveItemId) {
		selection.selectedItemId = null;
	}
}

export function resetSelectionState(): void {
	switchSelectionContext(null, null, 'home');
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
			state.selectedSection === 'media' ||
			state.selectedSection === 'favorites')
			? state.searchTerm
			: '';
}

export function selectFeed(feedId: string | null): void {
	const isReselecting = selection.selectedFeedId === feedId;
	switchSelectionContext(feedId, null, feedId ? null : 'all', {
		preserveItemId: isReselecting
	});
}

export function selectStation(stationId: string): void {
	switchSelectionContext(null, stationId, null);
}

export function selectSection(section: SidebarSection): void {
	switchSelectionContext(null, null, section);
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
