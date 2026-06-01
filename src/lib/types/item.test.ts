import { describe, it, expect } from 'vitest';
import {
	mapRawFeedListItem,
	mapRawFeedItem,
	isMediaItem,
	type RawFeedListItem,
	type RawFeedItem
} from '$lib/types/item';
import { isFeed } from '$lib/types/item';
import type { Feed } from '$lib/types/feed';

const baseRaw: Omit<RawFeedListItem, 'mediaEnclosure'> = {
	id: 'item-1',
	feedId: 'feed-1',
	title: 'Test Item',
	url: 'https://example.com/item',
	summary: 'A summary',
	previewText: 'Preview',
	readerStatus: 'unfetched',
	publishedAt: '2024-01-01T00:00:00Z',
	read: false,
	favorite: false,
	playbackPositionSeconds: 0
};

const enclosure = {
	url: 'https://example.com/audio.mp3',
	mimeType: 'audio/mpeg',
	durationSeconds: 3600
};

describe('mapRawFeedListItem', () => {
	it('produces an ArticleListItem when mediaEnclosure is absent', () => {
		const result = mapRawFeedListItem({ ...baseRaw });
		expect(result.itemType).toBe('article');
		// Type-level check: itemType narrows correctly
		if (result.itemType !== 'article') throw new Error('unreachable');
		expect(result.id).toBe('item-1');
	});

	it('produces a MediaListItem when mediaEnclosure is present', () => {
		const result = mapRawFeedListItem({ ...baseRaw, mediaEnclosure: enclosure });
		expect(result.itemType).toBe('media');
		if (result.itemType !== 'media') throw new Error('unreachable');
		expect(result.mediaEnclosure.url).toBe(enclosure.url);
		expect(result.mediaEnclosure.durationSeconds).toBe(3600);
	});

	it('preserves all base fields on article items', () => {
		const result = mapRawFeedListItem({ ...baseRaw });
		expect(result.feedId).toBe('feed-1');
		expect(result.publishedAt).toBe('2024-01-01T00:00:00Z');
		expect(result.read).toBe(false);
		expect(result.favorite).toBe(false);
	});

	it('preserves all base fields on media items', () => {
		const result = mapRawFeedListItem({ ...baseRaw, mediaEnclosure: enclosure });
		expect(result.feedId).toBe('feed-1');
		expect(result.playbackPositionSeconds).toBe(0);
	});
});

describe('mapRawFeedItem', () => {
	const baseRawFull: RawFeedItem = {
		...baseRaw,
		summaryHtml: '<p>summary</p>',
		contentHtml: '<p>content</p>'
	};

	it('produces an ArticleItem when mediaEnclosure is absent', () => {
		const result = mapRawFeedItem(baseRawFull);
		expect(result.itemType).toBe('article');
	});

	it('produces a MediaItem when mediaEnclosure is present', () => {
		const result = mapRawFeedItem({ ...baseRawFull, mediaEnclosure: enclosure });
		expect(result.itemType).toBe('media');
		if (result.itemType !== 'media') throw new Error('unreachable');
		expect(result.mediaEnclosure.mimeType).toBe('audio/mpeg');
	});

	it('preserves detail fields', () => {
		const result = mapRawFeedItem(baseRawFull);
		expect(result.summaryHtml).toBe('<p>summary</p>');
		expect(result.contentHtml).toBe('<p>content</p>');
	});
});

describe('isMediaItem', () => {
	it('returns false for an article item', () => {
		const item = mapRawFeedListItem({ ...baseRaw });
		expect(isMediaItem(item)).toBe(false);
	});

	it('returns true for a media item', () => {
		const item = mapRawFeedListItem({ ...baseRaw, mediaEnclosure: enclosure });
		expect(isMediaItem(item)).toBe(true);
	});
});

describe('isFeed', () => {
	const feed: Feed = {
		id: 'feed-1',
		title: 'My Feed',
		url: 'https://example.com/feed.xml',
		description: 'A feed',
		kind: 'article',
		createdAt: '2024-01-01T00:00:00Z'
	};

	it('returns true for a Feed object', () => {
		expect(isFeed(feed)).toBe(true);
	});

	it('returns false for a FeedItem', () => {
		const item = mapRawFeedItem({ ...baseRaw });
		expect(isFeed(item)).toBe(false);
	});
});
