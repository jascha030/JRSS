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
	setSectionSearchTerm
} from './selection.svelte';
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
		expect(selection.selectedSection).toBe('home');
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
		selectSection('favorites');
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
