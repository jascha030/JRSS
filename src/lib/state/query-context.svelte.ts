/**
 * Query Context
 *
 * Owns the composition of current view state into query specifications.
 * This is the single source of truth for "what data should we be showing?"
 *
 * Composes from: selection, feedsState, stationsState
 * Consumed by: items, playback, page
 */

import type { ItemPageQuery, ItemSortOrder } from '$lib/types/rss';
import { selection } from './selection.svelte';
import { feedsState } from './feeds.svelte';
import { stationsState } from './stations.svelte';

const DEFAULT_SORT_ORDER: ItemSortOrder = 'newest_first';
const PAGE_SIZE = 100;

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

/**
 * Compute the effective sort order for the active view.
 */
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

export function normalizeSearchTerm(term: string): string {
	return term.trim().toLowerCase();
}

export function getActiveListSection(): 'all' | 'unread' | 'media' | null {
	if (selection.selectedSection === 'settings') {
		return null;
	}

	if (selection.selectedFeedId) {
		return 'all';
	}

	if (!selection.selectedSection) {
		return 'all';
	}

	return selection.selectedSection;
}

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
