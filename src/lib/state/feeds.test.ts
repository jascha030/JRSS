import { describe, it, expect, beforeEach } from 'vitest';
import {
	feedsState,
	resetFeedsState,
	getFeedById,
	addSyncingFeed,
	removeSyncingFeed
} from './feeds.svelte';
import type { Feed } from '$lib/types/rss';

function makeFeed(id: string): Feed {
	return { id, title: 'Feed', url: '', description: '', kind: 'article', createdAt: '' };
}

beforeEach(() => {
	resetFeedsState();
});

describe('resetFeedsState', () => {
	it('clears all state', () => {
		feedsState.feeds = [makeFeed('a')];
		feedsState.syncingFeedIds = ['a'];
		feedsState.isCreatingFeed = true;
		resetFeedsState();
		expect(feedsState.feeds).toHaveLength(0);
		expect(feedsState.syncingFeedIds).toHaveLength(0);
		expect(feedsState.isCreatingFeed).toBe(false);
	});
});

describe('getFeedById', () => {
	it('returns the feed when present', () => {
		feedsState.feeds = [makeFeed('a'), makeFeed('b')];
		expect(getFeedById('b')?.id).toBe('b');
	});

	it('returns null when absent', () => {
		feedsState.feeds = [makeFeed('a')];
		expect(getFeedById('z')).toBeNull();
	});

	it('returns null for null input', () => {
		feedsState.feeds = [makeFeed('a')];
		expect(getFeedById(null)).toBeNull();
	});
});

describe('addSyncingFeed', () => {
	it('adds a feedId', () => {
		addSyncingFeed('f1');
		expect(feedsState.syncingFeedIds).toContain('f1');
	});

	it('is idempotent — does not add duplicates', () => {
		addSyncingFeed('f1');
		addSyncingFeed('f1');
		expect(feedsState.syncingFeedIds.filter((id) => id === 'f1')).toHaveLength(1);
	});

	it('can track multiple feeds simultaneously', () => {
		addSyncingFeed('f1');
		addSyncingFeed('f2');
		expect(feedsState.syncingFeedIds).toContain('f1');
		expect(feedsState.syncingFeedIds).toContain('f2');
	});
});

describe('removeSyncingFeed', () => {
	it('removes an existing feedId', () => {
		addSyncingFeed('f1');
		removeSyncingFeed('f1');
		expect(feedsState.syncingFeedIds).not.toContain('f1');
	});

	it('is a no-op when feedId not present', () => {
		removeSyncingFeed('missing');
		expect(feedsState.syncingFeedIds).toHaveLength(0);
	});

	it('only removes the targeted feedId', () => {
		addSyncingFeed('f1');
		addSyncingFeed('f2');
		removeSyncingFeed('f1');
		expect(feedsState.syncingFeedIds).not.toContain('f1');
		expect(feedsState.syncingFeedIds).toContain('f2');
	});
});
