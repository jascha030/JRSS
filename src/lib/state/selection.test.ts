import { describe, it, expect, beforeEach } from 'vitest';
import {
	selection,
	resetSelectionState,
	selectFeed,
	selectStation,
	selectSection,
	selectItem,
	setFeedSearchTerm,
	setStationSearchTerm,
	setSectionSearchTerm,
	getSelectedFeed,
	getSelectedStation
} from './selection.svelte';
import type { Feed } from '$lib/types/feed';
import type { Station } from '$lib/types/station';

function makeFeed(id: string): Feed {
	return { id, title: 'Feed', url: '', description: '', kind: 'article', createdAt: '' };
}

function makeStation(id: string): Station {
	return {
		id,
		name: 'Station',
		episodeFilter: 'all',
		sortOrder: 'newest_first',
		sortOrderPosition: 0,
		createdAt: '',
		feedIds: [],
		gradient: 'emerald'
	};
}

beforeEach(() => {
	resetSelectionState();
});

describe('resetSelectionState', () => {
	it('restores defaults', () => {
		selection.selectedFeedId = 'x';
		selection.feedSearchTerm = 'foo';
		resetSelectionState();
		expect(selection.selectedFeedId).toBeNull();
		expect(selection.feedSearchTerm).toBe('');
		expect(selection.selectedSection).toBe('all');
	});
});

describe('selectFeed', () => {
	it('sets feedId and clears station / item', () => {
		selectFeed('f1');
		expect(selection.selectedFeedId).toBe('f1');
		expect(selection.selectedStationId).toBeNull();
		expect(selection.selectedItemId).toBeNull();
	});

	it('sets selectedSection to null when feedId given', () => {
		selectFeed('f1');
		expect(selection.selectedSection).toBeNull();
	});

	it('sets selectedSection to "all" when feedId is null', () => {
		selectFeed(null);
		expect(selection.selectedSection).toBe('all');
	});

	it('clears feedSearchTerm and sectionSearchTerm', () => {
		selection.feedSearchTerm = 'foo';
		selection.sectionSearchTerm = 'bar';
		selectFeed('f1');
		expect(selection.feedSearchTerm).toBe('');
		expect(selection.sectionSearchTerm).toBe('');
	});
});

describe('selectStation', () => {
	it('sets stationId and clears feed / section / item', () => {
		selectStation('s1');
		expect(selection.selectedStationId).toBe('s1');
		expect(selection.selectedFeedId).toBeNull();
		expect(selection.selectedItemId).toBeNull();
		expect(selection.selectedSection).toBeNull();
	});

	it('clears all search terms', () => {
		selection.feedSearchTerm = 'a';
		selection.stationSearchTerm = 'b';
		selection.sectionSearchTerm = 'c';
		selectStation('s1');
		expect(selection.feedSearchTerm).toBe('');
		expect(selection.stationSearchTerm).toBe('');
		expect(selection.sectionSearchTerm).toBe('');
	});
});

describe('selectSection', () => {
	it('sets section and clears feed / station / item', () => {
		selectSection('unread');
		expect(selection.selectedSection).toBe('unread');
		expect(selection.selectedFeedId).toBeNull();
		expect(selection.selectedStationId).toBeNull();
		expect(selection.selectedItemId).toBeNull();
	});

	it('clears feedSearchTerm and sectionSearchTerm', () => {
		selection.feedSearchTerm = 'x';
		selection.sectionSearchTerm = 'y';
		selectSection('media');
		expect(selection.feedSearchTerm).toBe('');
		expect(selection.sectionSearchTerm).toBe('');
	});
});

describe('selectItem', () => {
	it('sets itemId', () => {
		selectItem('item1');
		expect(selection.selectedItemId).toBe('item1');
	});

	it('accepts null', () => {
		selectItem('item1');
		selectItem(null);
		expect(selection.selectedItemId).toBeNull();
	});
});

describe('search term setters', () => {
	it('setFeedSearchTerm', () => {
		setFeedSearchTerm('hello');
		expect(selection.feedSearchTerm).toBe('hello');
	});

	it('setStationSearchTerm', () => {
		setStationSearchTerm('world');
		expect(selection.stationSearchTerm).toBe('world');
	});

	it('setSectionSearchTerm', () => {
		setSectionSearchTerm('test');
		expect(selection.sectionSearchTerm).toBe('test');
	});
});

describe('getSelectedFeed', () => {
	it('returns the matching feed', () => {
		const feeds = [makeFeed('a'), makeFeed('b')];
		selection.selectedFeedId = 'b';
		expect(getSelectedFeed(feeds)?.id).toBe('b');
	});

	it('returns null when no selection', () => {
		expect(getSelectedFeed([makeFeed('a')])).toBeNull();
	});

	it('returns null when selection not in list', () => {
		selection.selectedFeedId = 'z';
		expect(getSelectedFeed([makeFeed('a')])).toBeNull();
	});
});

describe('getSelectedStation', () => {
	it('returns the matching station', () => {
		const stations = [makeStation('s1'), makeStation('s2')];
		selection.selectedStationId = 's1';
		expect(getSelectedStation(stations)?.id).toBe('s1');
	});

	it('returns null when no selection', () => {
		expect(getSelectedStation([makeStation('s1')])).toBeNull();
	});
});
