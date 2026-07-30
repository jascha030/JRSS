import { describe, it, expect } from 'vitest';
import { addFeed, listFeeds } from '$lib/services/feed';
import { getItemsByIds, queryItems } from '$lib/services/item';
import { extractCoverPalette } from '$lib/services/palette';
import { loadPlaybackSession, loadPlaybackContext } from '$lib/services/playback/session';

// isTauriRuntime() is false in happy-dom (no window.__TAURI_INTERNALS__),
// so functions guarded by it return early; invokeCommand is never reached.

describe('addFeed — input validation', () => {
	it('throws for an empty string', async () => {
		await expect(addFeed('')).rejects.toThrow(
			'Enter a feed URL, Apple Podcasts URL, or Apple Podcasts ID.'
		);
	});

	it('throws for a whitespace-only string', async () => {
		await expect(addFeed('   ')).rejects.toThrow(
			'Enter a feed URL, Apple Podcasts URL, or Apple Podcasts ID.'
		);
	});
});

describe('getItemsByIds — empty shortcut', () => {
	it('returns [] immediately without calling Tauri', async () => {
		await expect(getItemsByIds([])).resolves.toEqual([]);
	});
});

describe('extractCoverPalette — no-Tauri guards', () => {
	it('returns [] outside Tauri runtime', async () => {
		await expect(extractCoverPalette('https://example.com/art.jpg')).resolves.toEqual([]);
	});

	it('returns [] for a blank URL outside Tauri runtime', async () => {
		await expect(extractCoverPalette('   ')).resolves.toEqual([]);
	});
});

describe('listFeeds — no-Tauri guard', () => {
	it('returns [] outside Tauri runtime', async () => {
		await expect(listFeeds()).resolves.toEqual([]);
	});
});

describe('queryItems — no-Tauri guard', () => {
	it('returns empty page outside Tauri runtime', async () => {
		const result = await queryItems({
			section: 'all',
			offset: 0,
			limit: 20,
			sortOrder: 'newest_first'
		});
		expect(result).toEqual({ items: [], totalCount: 0 });
	});
});

describe('loadPlaybackSession — no-Tauri guard', () => {
	it('returns null outside Tauri runtime', async () => {
		await expect(loadPlaybackSession()).resolves.toBeNull();
	});
});

describe('loadPlaybackContext — no-Tauri guard', () => {
	it('returns null outside Tauri runtime', async () => {
		await expect(loadPlaybackContext()).resolves.toBeNull();
	});
});
