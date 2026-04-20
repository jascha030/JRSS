import type { ItemPageQuery, ItemSortOrder } from '$lib/types/rss';
import { feedsState } from './feeds.svelte';
import { selection, getActiveListSection, normalizeSearchTerm } from './selection.svelte';
import { stationsState } from './stations.svelte';

const PAGE_SIZE = 100;
export const DEFAULT_SORT_ORDER: ItemSortOrder = 'newest_first';

export type ItemsQuerySpec =
	| {
			kind: 'feed-items';
			queryKey: string;
			query: ItemPageQuery;
	  }
	| {
			kind: 'station-items';
			queryKey: string;
			stationId: string;
			sortOrder: ItemSortOrder;
	  };

function buildQueryKey(
	feedId: string | null,
	section: 'all' | 'unread' | 'media',
	search: string,
	sortOrder: ItemSortOrder
): string {
	const normalizedSearch = normalizeSearchTerm(search);
	const base = `${section}::${feedId ?? 'all-feeds'}::${sortOrder}`;

	return normalizedSearch ? `${base}::search:${normalizedSearch}` : base;
}

export function getEffectiveSortOrder(): ItemSortOrder {
	if (selection.selectedStationId) {
		const station = stationsState.stations.find((s) => s.id === selection.selectedStationId);
		return station?.sortOrder ?? DEFAULT_SORT_ORDER;
	}

	if (selection.selectedFeedId) {
		const feed = feedsState.feeds.find((f) => f.id === selection.selectedFeedId);
		return feed?.sortOrder ?? DEFAULT_SORT_ORDER;
	}

	return DEFAULT_SORT_ORDER;
}

export function getActiveQuerySpec(): ItemsQuerySpec | null {
	if (selection.selectedStationId) {
		const station = stationsState.stations.find((s) => s.id === selection.selectedStationId);
		const sortOrder = station?.sortOrder ?? DEFAULT_SORT_ORDER;

		return {
			kind: 'station-items',
			queryKey: `station::${selection.selectedStationId}::${sortOrder}`,
			stationId: selection.selectedStationId,
			sortOrder
		};
	}

	const section = getActiveListSection();

	if (!section) {
		return null;
	}

	const search = selection.selectedFeedId ? normalizeSearchTerm(selection.feedSearchTerm) : '';
	const sortOrder = getEffectiveSortOrder();

	return {
		kind: 'feed-items',
		queryKey: buildQueryKey(selection.selectedFeedId, section, search, sortOrder),
		query: {
			feedId: selection.selectedFeedId ?? undefined,
			section,
			offset: 0,
			limit: PAGE_SIZE,
			search: search || undefined,
			sortOrder
		}
	};
}

export function getActiveQueryKey(): string | null {
	return getActiveQuerySpec()?.queryKey ?? null;
}
