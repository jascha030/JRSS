import { describe, it, expect, beforeEach } from 'vitest';
import {
	normalizeSearchTerm,
	getActiveListSection,
	getEffectiveSortOrder,
	getActiveQuerySpec,
	getActiveQueryKey
} from './query-context.svelte';
import {
	resetSelectionState,
	selection,
	selectFeed,
	selectStation,
	selectSection,
	setFeedSearchTerm,
	setSectionSearchTerm,
	setStationSearchTerm
} from './selection.svelte';
import { feedsState, resetFeedsState } from './feeds.svelte';
import { stationsState, resetStationsState } from './stations.svelte';
import type { Feed } from '$lib/types/feed';
import type { Station } from '$lib/types/station';

function makeFeed(id: string, sortOrder?: 'newest_first' | 'oldest_first'): Feed {
	return { id, title: '', url: '', description: '', kind: 'article', createdAt: '', sortOrder };
}

function makeStation(
	id: string,
	sortOrder: 'newest_first' | 'oldest_first' = 'newest_first'
): Station {
	return {
		id,
		name: '',
		episodeFilter: 'all',
		sortOrder,
		sortOrderPosition: 0,
		createdAt: '',
		feedIds: [],
		gradient: 'emerald'
	};
}

beforeEach(() => {
	resetSelectionState();
	resetFeedsState();
	resetStationsState();
});

describe('normalizeSearchTerm', () => {
	it('trims whitespace', () => {
		expect(normalizeSearchTerm('  hello  ')).toBe('hello');
	});

	it('lowercases', () => {
		expect(normalizeSearchTerm('HELLO World')).toBe('hello world');
	});

	it('handles empty string', () => {
		expect(normalizeSearchTerm('')).toBe('');
	});
});

describe('getActiveListSection', () => {
	it('returns null for settings section', () => {
		selectSection('settings');
		expect(getActiveListSection()).toBeNull();
	});

	it('returns "all" when a feed is selected', () => {
		feedsState.feeds = [makeFeed('f1')];
		selectFeed('f1');
		expect(getActiveListSection()).toBe('all');
	});

	it('returns "unread" for unread section', () => {
		selectSection('unread');
		expect(getActiveListSection()).toBe('unread');
	});

	it('returns "media" for media section', () => {
		selectSection('media');
		expect(getActiveListSection()).toBe('media');
	});

	it('returns "favorites" for favorites section', () => {
		selectSection('favorites');
		expect(getActiveListSection()).toBe('favorites');
	});

	it('returns null for default "home" state', () => {
		expect(getActiveListSection()).toBeNull();
	});

	it('returns "all" when no section set (null)', () => {
		selection.selectedSection = null;
		expect(getActiveListSection()).toBe('all');
	});
});

describe('getEffectiveSortOrder', () => {
	it('returns "newest_first" by default', () => {
		expect(getEffectiveSortOrder()).toBe('newest_first');
	});

	it('uses station sortOrder when station selected', () => {
		stationsState.stations = [makeStation('s1', 'oldest_first')];
		selectStation('s1');
		expect(getEffectiveSortOrder()).toBe('oldest_first');
	});

	it('uses feed sortOrder when feed selected', () => {
		feedsState.feeds = [makeFeed('f1', 'oldest_first')];
		selectFeed('f1');
		expect(getEffectiveSortOrder()).toBe('oldest_first');
	});

	it('falls back to "newest_first" when station not found', () => {
		selectStation('missing');
		expect(getEffectiveSortOrder()).toBe('newest_first');
	});

	it('falls back to "newest_first" when feed has no sortOrder', () => {
		feedsState.feeds = [makeFeed('f1')];
		selectFeed('f1');
		expect(getEffectiveSortOrder()).toBe('newest_first');
	});
});

describe('getActiveQuerySpec', () => {
	it('returns null when settings section active', () => {
		selectSection('settings');
		expect(getActiveQuerySpec()).toBeNull();
	});

	it('returns null for default "home" section', () => {
		expect(getActiveQuerySpec()).toBeNull();
	});

	it('returns feed-items spec when "all" section is active', () => {
		selectSection('all');
		const spec = getActiveQuerySpec();
		expect(spec?.kind).toBe('feed-items');
	});

	it('returns station-items spec when station selected', () => {
		stationsState.stations = [makeStation('s1')];
		selectStation('s1');
		const spec = getActiveQuerySpec();
		expect(spec?.kind).toBe('station-items');
		if (spec?.kind === 'station-items') {
			expect(spec.stationId).toBe('s1');
		}
	});

	it('includes normalised search in queryKey from sectionSearchTerm', () => {
		selectSection('all');
		setSectionSearchTerm('  FOO  ');
		const spec = getActiveQuerySpec();
		expect(spec?.queryKey).toContain('search:foo');
	});

	it('includes normalised search in queryKey from feedSearchTerm when feed selected', () => {
		feedsState.feeds = [makeFeed('f1')];
		selectFeed('f1');
		setFeedSearchTerm('Bar');
		const spec = getActiveQuerySpec();
		expect(spec?.queryKey).toContain('search:bar');
	});

	it('includes station search in queryKey from stationSearchTerm', () => {
		stationsState.stations = [makeStation('s1')];
		selectStation('s1');
		setStationSearchTerm('Baz');
		const spec = getActiveQuerySpec();
		expect(spec?.queryKey).toContain('search:baz');
	});

	it('omits search fragment when term is blank', () => {
		selectSection('all');
		const spec = getActiveQuerySpec();
		expect(spec?.queryKey).not.toContain('search:');
	});
});

describe('getActiveQueryKey', () => {
	it('returns null for settings section', () => {
		selectSection('settings');
		expect(getActiveQueryKey()).toBeNull();
	});

	it('returns a non-empty string for a normal section', () => {
		selectSection('all');
		const key = getActiveQueryKey();
		expect(typeof key).toBe('string');
		expect(key!.length).toBeGreaterThan(0);
	});
});
