/**
 * Tests for the Tauri event listeners registered by `initAudioEventListeners`.
 *
 * Uses `mockIPC` with `shouldMockEvents: true` so that `listen` and `emit`
 * from `@tauri-apps/api/event` work end-to-end through the mock IPC bridge.
 *
 * `initAudioEventListeners` is called in `beforeEach` after `mockIPC` so that
 * the event handlers are registered against the mock bridge (not the real IPC).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import {
	playbackState,
	resetPlaybackState,
	initAudioEventListeners
} from '$lib/state/playback.svelte';
import { itemsState, resetItemsState, registerItem } from '$lib/state/items.svelte';
import { mapRawFeedListItem } from '$lib/types/item';
import type { RawFeedListItem } from '$lib/types/item';
import type { BackendPlaybackState, BackendQueueState } from '$lib/types/playback';

/** A minimal media item fixture for use in playback tests. */
function makeMediaItem(id = 'item-1', positionSeconds = 0): RawFeedListItem {
	return {
		id,
		feedId: 'feed-1',
		title: 'Episode 1',
		url: 'https://example.com/ep1',
		summary: '',
		previewText: '',
		readerStatus: 'unfetched',
		publishedAt: '2024-01-01T00:00:00Z',
		read: false,
		playbackPositionSeconds: positionSeconds,
		mediaEnclosure: { url: 'https://cdn.example.com/ep1.mp3', mimeType: 'audio/mpeg' }
	};
}

/** A minimal BackendPlaybackState fixture. */
function makeBackendState(overrides: Partial<BackendPlaybackState> = {}): BackendPlaybackState {
	return {
		itemId: 'item-1',
		positionSeconds: 30,
		durationSeconds: 120,
		isPlaying: true,
		volume: 1,
		...overrides
	};
}

beforeEach(async () => {
	resetPlaybackState();
	resetItemsState();
	// Set up mock IPC bridge with event support enabled, then register listeners.
	// `get_items_by_ids` and `extract_cover_palette` may be called by async
	// handlers; return safe defaults so they don't error.
	mockIPC(
		(cmd) => {
			if (cmd === 'get_items_by_ids') return [];
			if (cmd === 'extract_cover_palette') return [];
		},
		{ shouldMockEvents: true }
	);
	await initAudioEventListeners();
});

afterEach(() => {
	clearMocks();
});

describe('playback-stopped event', () => {
	it('sets currentPlaybackState to null', async () => {
		playbackState.currentPlaybackState = {
			itemId: 'item-1',
			positionSeconds: 30,
			durationSeconds: 120,
			isPlaying: true,
			volume: 1
		};

		await emit('playback-stopped');

		expect(playbackState.currentPlaybackState).toBeNull();
	});

	it('is a no-op when currentPlaybackState is already null', async () => {
		expect(playbackState.currentPlaybackState).toBeNull();
		await emit('playback-stopped');
		expect(playbackState.currentPlaybackState).toBeNull();
	});
});

describe('playback-ended event', () => {
	it('resets playbackPositionSeconds to 0 in item cache', async () => {
		// Pre-register an item with a non-zero playback position.
		registerItem(mapRawFeedListItem(makeMediaItem('item-1', 60)));

		await emit('playback-ended', { itemId: 'item-1' });

		expect(itemsState.itemSummariesById['item-1']?.playbackPositionSeconds).toBe(0);
	});
});

describe('playback-state-changed event', () => {
	it('updates currentPlaybackState from backend payload', async () => {
		// Pre-register so ensureAudioItemsLoaded short-circuits (no IPC fetch).
		registerItem(mapRawFeedListItem(makeMediaItem('item-1')));

		const backendState = makeBackendState({ positionSeconds: 45.7, durationSeconds: 120.2 });

		await emit('playback-state-changed', backendState);

		// The handler is async (ensureAudioItemsLoaded → .then → applyBackendPlaybackState),
		// so poll until the state mutation lands.
		await vi.waitFor(() => {
			expect(playbackState.currentPlaybackState?.itemId).toBe('item-1');
		});

		// applyBackendPlaybackState floors position and duration.
		expect(playbackState.currentPlaybackState?.positionSeconds).toBe(45);
		expect(playbackState.currentPlaybackState?.durationSeconds).toBe(120);
		expect(playbackState.currentPlaybackState?.isPlaying).toBe(true);
	});

	it('syncs position back to item cache when playback stops', async () => {
		registerItem(mapRawFeedListItem(makeMediaItem('item-1', 0)));

		await emit(
			'playback-state-changed',
			makeBackendState({ isPlaying: false, positionSeconds: 38 })
		);

		await vi.waitFor(() => {
			expect(itemsState.itemSummariesById['item-1']?.playbackPositionSeconds).toBe(38);
		});
	});
});

describe('queue-changed event', () => {
	it('updates queue arrays from backend payload', async () => {
		const queueState: BackendQueueState = {
			history: [{ itemId: 'item-hist', url: '', title: 'History', durationSeconds: 0 }],
			manual: [{ itemId: 'item-manual', url: '', title: 'Manual', durationSeconds: 0 }],
			auto: [],
			current: null
		};

		await emit('queue-changed', queueState);

		await vi.waitFor(() => {
			expect(playbackState.playbackHistory).toContain('item-hist');
		});
		expect(playbackState.manualQueue).toContain('item-manual');
		expect(playbackState.autoQueue).toEqual([]);
	});

	it('clears currentPlaybackState when current is null and nothing is playing', async () => {
		playbackState.currentPlaybackState = {
			itemId: 'item-1',
			positionSeconds: 10,
			durationSeconds: 120,
			isPlaying: false,
			volume: 1
		};

		const emptyQueue: BackendQueueState = {
			history: [],
			manual: [],
			auto: [],
			current: null
		};

		await emit('queue-changed', emptyQueue);

		await vi.waitFor(() => {
			expect(playbackState.currentPlaybackState).toBeNull();
		});
	});
});
