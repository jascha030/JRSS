/**
 * Happy-path tests for feedService using @tauri-apps/api/mocks.
 *
 * `mockIPC` sets up `window.__TAURI_INTERNALS__`, which makes `isTauriRuntime()`
 * return true, allowing the full service code path (including mapping logic) to
 * be exercised with controlled IPC responses.
 *
 * Kept in a separate file from feedService.test.ts so that the `__TAURI_INTERNALS__`
 * object left behind by `clearMocks` (it clears `.invoke` but not the object itself)
 * does not affect guard tests that rely on `isTauriRuntime()` returning false.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import {
	listFeeds,
	getItemsByIds,
	markRead,
	savePlayback,
	getItemDetails,
	createStation
} from '$lib/services/feedService';
import type { Feed, RawFeedListItem } from '$lib/types/rss';

afterEach(() => {
	clearMocks();
});

describe('listFeeds — mockIPC', () => {
	it('returns mapped Feed[] from IPC response', async () => {
		const raw: Feed = {
			id: 'feed-1',
			title: 'Test Feed',
			url: 'https://example.com/rss',
			description: 'A test feed',
			kind: 'article',
			createdAt: '2024-01-01T00:00:00Z'
		};
		mockIPC((cmd) => {
			if (cmd === 'list_feeds') return [raw];
		});
		const result = await listFeeds();
		expect(result).toHaveLength(1);
		expect(result[0]).toEqual(raw);
	});

	it('returns [] when IPC returns empty array', async () => {
		mockIPC((cmd) => {
			if (cmd === 'list_feeds') return [];
		});
		expect(await listFeeds()).toEqual([]);
	});
});

describe('getItemsByIds — mockIPC mapping', () => {
	const base: Omit<RawFeedListItem, 'mediaEnclosure'> = {
		id: 'item-1',
		feedId: 'feed-1',
		title: 'Test Article',
		url: 'https://example.com/1',
		summary: 'A summary',
		previewText: 'Preview text',
		readerStatus: 'unfetched',
		publishedAt: '2024-01-01T00:00:00Z',
		read: false,
		playbackPositionSeconds: 0
	};

	it('sets itemType: article when no mediaEnclosure', async () => {
		mockIPC((cmd) => {
			if (cmd === 'get_items_by_ids') return [base];
		});
		const [item] = await getItemsByIds(['item-1']);
		expect(item.itemType).toBe('article');
	});

	it('sets itemType: media when mediaEnclosure is present', async () => {
		const rawMedia: RawFeedListItem = {
			...base,
			id: 'item-2',
			mediaEnclosure: { url: 'https://example.com/audio.mp3', mimeType: 'audio/mpeg' }
		};
		mockIPC((cmd) => {
			if (cmd === 'get_items_by_ids') return [rawMedia];
		});
		const [item] = await getItemsByIds(['item-2']);
		expect(item.itemType).toBe('media');
		if (item.itemType === 'media') {
			expect(item.mediaEnclosure.url).toBe('https://example.com/audio.mp3');
		}
	});
});

describe('getItemDetails — mockIPC', () => {
	it('maps raw full item including FeedItemDetails fields', async () => {
		mockIPC((cmd) => {
			if (cmd === 'get_item_details')
				return {
					id: 'item-1',
					feedId: 'feed-1',
					title: 'Full Article',
					url: 'https://example.com/1',
					summary: 'Summary',
					previewText: 'Preview',
					readerStatus: 'ready',
					publishedAt: '2024-01-01T00:00:00Z',
					read: true,
					playbackPositionSeconds: 0,
					contentHtml: '<p>Content</p>'
				};
		});
		const item = await getItemDetails('item-1');
		expect(item.id).toBe('item-1');
		expect(item.read).toBe(true);
		expect(item.itemType).toBe('article');
		if (item.itemType === 'article') {
			expect(item.contentHtml).toBe('<p>Content</p>');
		}
	});
});

describe('markRead — mockIPC', () => {
	it('sends mark_read command with correct args', async () => {
		const handler = vi.fn().mockReturnValue(undefined);
		mockIPC(handler);
		await markRead('item-42', true);
		expect(handler).toHaveBeenCalledWith('mark_read', { itemId: 'item-42', read: true });
	});

	it('passes read: false correctly', async () => {
		const handler = vi.fn().mockReturnValue(undefined);
		mockIPC(handler);
		await markRead('item-42', false);
		expect(handler).toHaveBeenCalledWith('mark_read', { itemId: 'item-42', read: false });
	});
});

describe('savePlayback — clamping', () => {
	it('floors a positive float before sending', async () => {
		const handler = vi.fn().mockReturnValue(undefined);
		mockIPC(handler);
		await savePlayback('item-1', 42.9);
		expect(handler).toHaveBeenCalledWith('save_playback', {
			itemId: 'item-1',
			positionSeconds: 42
		});
	});

	it('clamps negative value to 0', async () => {
		const handler = vi.fn().mockReturnValue(undefined);
		mockIPC(handler);
		await savePlayback('item-1', -5);
		expect(handler).toHaveBeenCalledWith('save_playback', {
			itemId: 'item-1',
			positionSeconds: 0
		});
	});

	it('preserves exact zero', async () => {
		const handler = vi.fn().mockReturnValue(undefined);
		mockIPC(handler);
		await savePlayback('item-1', 0);
		expect(handler).toHaveBeenCalledWith('save_playback', {
			itemId: 'item-1',
			positionSeconds: 0
		});
	});
});

describe('createStation — mockIPC mapping', () => {
	it('maps raw station fields to Station type', async () => {
		mockIPC((cmd) => {
			if (cmd === 'create_station')
				return {
					id: 'station-abc',
					name: 'My Station',
					episodeFilter: 'all',
					sortOrder: 'newest_first',
					sortOrderPosition: 1000,
					createdAt: '2024-01-01T00:00:00Z',
					feedIds: ['feed-1', 'feed-2']
				};
		});
		const station = await createStation({
			name: 'My Station',
			episodeFilter: 'all',
			sortOrder: 'newest_first',
			feedIds: ['feed-1', 'feed-2']
		});
		expect(station.id).toBe('station-abc');
		expect(station.name).toBe('My Station');
		expect(station.feedIds).toEqual(['feed-1', 'feed-2']);
		expect(station.episodeFilter).toBe('all');
	});
});
