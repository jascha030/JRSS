import { describe, it, expect, beforeEach } from 'vitest';
import {
	itemsState,
	resetItemsState,
	registerItem,
	registerItems,
	patchItemSummary,
	mergeDetailedItem,
	getItemById,
	getMediaItemById,
	invalidateAllQueries
} from './items.svelte';
import { resetSelectionState, selection, selectItem } from './selection.svelte';
import type { ArticleListItem, MediaListItem, FeedItem } from '$lib/types/item';

function makeArticle(id: string): ArticleListItem {
	return {
		id,
		feedId: 'f1',
		title: 'Article',
		url: '',
		summary: '',
		previewText: '',
		readerStatus: 'unfetched',
		publishedAt: '',
		read: false,
		favorite: false,
		playbackPositionSeconds: 0,
		itemType: 'article'
	};
}

function makeMedia(id: string): MediaListItem {
	return {
		id,
		feedId: 'f1',
		title: 'Media',
		url: '',
		summary: '',
		previewText: '',
		readerStatus: 'unfetched',
		publishedAt: '',
		read: false,
		favorite: false,
		playbackPositionSeconds: 0,
		itemType: 'media',
		mediaEnclosure: { url: 'https://example.com/ep.mp3', mimeType: 'audio/mpeg' }
	};
}

beforeEach(() => {
	resetItemsState();
	resetSelectionState();
});

describe('resetItemsState', () => {
	it('clears all state maps', () => {
		registerItem(makeArticle('a'));
		itemsState.itemIdsByIndexByQueryKey['k'] = { 0: 'a' };
		resetItemsState();
		expect(itemsState.itemSummariesById).toEqual({});
		expect(itemsState.itemDetailsById).toEqual({});
		expect(itemsState.itemIdsByIndexByQueryKey).toEqual({});
		expect(itemsState.totalCountByQueryKey).toEqual({});
	});
});

describe('registerItem', () => {
	it('stores item in itemSummariesById', () => {
		registerItem(makeArticle('a'));
		expect(itemsState.itemSummariesById['a']?.id).toBe('a');
	});
});

describe('registerItems', () => {
	it('stores all items', () => {
		registerItems([makeArticle('a'), makeMedia('b')]);
		expect(Object.keys(itemsState.itemSummariesById)).toHaveLength(2);
	});
});

describe('patchItemSummary', () => {
	it('applies patch to existing item', () => {
		registerItem(makeArticle('a'));
		patchItemSummary('a', { read: true });
		expect(itemsState.itemSummariesById['a']?.read).toBe(true);
	});

	it('preserves other fields when patching', () => {
		registerItem(makeArticle('a'));
		patchItemSummary('a', { playbackPositionSeconds: 42 });
		expect(itemsState.itemSummariesById['a']?.title).toBe('Article');
	});

	it('is a no-op for missing item', () => {
		patchItemSummary('missing', { read: true });
		expect(itemsState.itemSummariesById['missing']).toBeUndefined();
	});
});

describe('mergeDetailedItem', () => {
	it('registers list-level summary and stores details', () => {
		const item: FeedItem = {
			...makeArticle('a'),
			summaryText: 'the summary',
			summaryHtml: '<p>the summary</p>',
			contentText: undefined,
			contentHtml: undefined,
			readerContentHtml: undefined,
			readerContentText: undefined
		};
		mergeDetailedItem(item);
		expect(itemsState.itemSummariesById['a']?.id).toBe('a');
		expect(itemsState.itemDetailsById['a']?.summaryText).toBe('the summary');
	});

	it('detail record does not contain content-heavy detail fields on summary', () => {
		const item: FeedItem = {
			...makeArticle('a'),
			summaryText: 'txt',
			summaryHtml: undefined,
			contentText: undefined,
			contentHtml: undefined,
			readerContentHtml: undefined,
			readerContentText: undefined
		};
		mergeDetailedItem(item);
		// summary item should not expose detail-only fields
		expect('summaryText' in (itemsState.itemSummariesById['a'] ?? {})).toBe(false);
	});
});

describe('getItemById', () => {
	it('returns item when present', () => {
		registerItem(makeArticle('a'));
		expect(getItemById('a')?.id).toBe('a');
	});

	it('returns null when absent', () => {
		expect(getItemById('missing')).toBeNull();
	});
});

describe('getMediaItemById', () => {
	it('returns media item when present', () => {
		registerItem(makeMedia('m'));
		expect(getMediaItemById('m')?.itemType).toBe('media');
	});

	it('returns null for article item', () => {
		registerItem(makeArticle('a'));
		expect(getMediaItemById('a')).toBeNull();
	});

	it('returns null when absent', () => {
		expect(getMediaItemById('missing')).toBeNull();
	});
});

describe('invalidateAllQueries', () => {
	it('clears all query-keyed maps', () => {
		itemsState.itemIdsByIndexByQueryKey['k'] = { 0: 'a' };
		itemsState.totalCountByQueryKey['k'] = 10;
		itemsState.loadedPageOffsetsByQueryKey['k'] = { 0: true };
		itemsState.initialLoadDoneByQueryKey['k'] = true;
		invalidateAllQueries();
		expect(itemsState.itemIdsByIndexByQueryKey).toEqual({});
		expect(itemsState.totalCountByQueryKey).toEqual({});
		expect(itemsState.loadedPageOffsetsByQueryKey).toEqual({});
		expect(itemsState.initialLoadDoneByQueryKey).toEqual({});
	});

	it('clears selectedItemId', () => {
		selectItem('item1');
		invalidateAllQueries();
		expect(selection.selectedItemId).toBeNull();
	});

	it('preserves itemSummariesById (cache not blown away)', () => {
		registerItem(makeArticle('a'));
		invalidateAllQueries();
		expect(itemsState.itemSummariesById['a']?.id).toBe('a');
	});
});
