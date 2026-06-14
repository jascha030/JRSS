import { describe, it, expect, beforeEach } from 'vitest';
import {
	playbackState,
	resetPlaybackState,
	getCurrentAudioItem,
	isItemCurrentAudio,
	isAudioPlaying,
	isAudioLoading,
	getManualQueueLength,
	getPlaybackHistory,
	getUpcomingQueue,
	getCoverTheme,
	getPlaybackPositionForItem,
	getPlaybackContext
} from './playback.svelte';

const FALLBACK_BG1 = '#0f172a';
const FALLBACK_FG = '#ffffff';

beforeEach(() => {
	resetPlaybackState();
});

describe('resetPlaybackState', () => {
	it('nulls currentPlaybackState', () => {
		playbackState.currentPlaybackState = {
			itemId: 'i1',
			positionSeconds: 30,
			durationSeconds: 300,
			fileDurationSeconds: null,
			isPlaying: true,
			isBuffering: false,
			isFullyDownloaded: false,
			volume: 1
		};
		resetPlaybackState();
		expect(playbackState.currentPlaybackState).toBeNull();
	});

	it('clears queues and history', () => {
		playbackState.manualQueue = ['a'];
		playbackState.autoQueue = ['b'];
		playbackState.playbackHistory = ['c'];
		resetPlaybackState();
		expect(playbackState.manualQueue).toHaveLength(0);
		expect(playbackState.autoQueue).toHaveLength(0);
		expect(playbackState.playbackHistory).toHaveLength(0);
	});

	it('restores fallback cover theme', () => {
		resetPlaybackState();
		expect(playbackState.coverTheme.bg1).toBe(FALLBACK_BG1);
		expect(playbackState.coverTheme.fg).toBe(FALLBACK_FG);
	});

	it('clears audioItemsById', () => {
		resetPlaybackState();
		expect(playbackState.audioItemsById).toEqual({});
	});

	it('nulls playbackContext', () => {
		playbackState.playbackContext = { contextType: 'feed', id: 'f1' };
		resetPlaybackState();
		expect(playbackState.playbackContext).toBeNull();
	});
});

describe('getters with no active playback', () => {
	it('getCurrentAudioItem returns null', () => {
		expect(getCurrentAudioItem()).toBeNull();
	});

	it('isAudioPlaying returns false', () => {
		expect(isAudioPlaying()).toBe(false);
	});

	it('isAudioLoading returns false', () => {
		expect(isAudioLoading()).toBe(false);
	});

	it('getManualQueueLength returns 0', () => {
		expect(getManualQueueLength()).toBe(0);
	});

	it('getPlaybackHistory returns []', () => {
		expect(getPlaybackHistory()).toEqual([]);
	});

	it('getUpcomingQueue returns []', () => {
		expect(getUpcomingQueue()).toEqual([]);
	});

	it('getPlaybackContext returns null', () => {
		expect(getPlaybackContext()).toBeNull();
	});
});

describe('isItemCurrentAudio', () => {
	it('returns false when no playback state', () => {
		expect(isItemCurrentAudio('i1')).toBe(false);
	});

	it('returns false when a different item is playing', () => {
		playbackState.currentPlaybackState = {
			itemId: 'other',
			positionSeconds: 0,
			durationSeconds: 100,
			fileDurationSeconds: null,
			isPlaying: true,
			isBuffering: false,
			isFullyDownloaded: false,
			volume: 1
		};
		expect(isItemCurrentAudio('i1')).toBe(false);
	});

	it('returns true when item is current', () => {
		playbackState.currentPlaybackState = {
			itemId: 'i1',
			positionSeconds: 0,
			durationSeconds: 100,
			fileDurationSeconds: null,
			isPlaying: true,
			isBuffering: false,
			isFullyDownloaded: false,
			volume: 1
		};
		expect(isItemCurrentAudio('i1')).toBe(true);
	});
});

describe('getPlaybackPositionForItem', () => {
	it('returns fallback when no playback state', () => {
		expect(getPlaybackPositionForItem('i1', 42)).toBe(42);
	});

	it('returns fallback when a different item is playing', () => {
		playbackState.currentPlaybackState = {
			itemId: 'other',
			positionSeconds: 99,
			durationSeconds: 100,
			fileDurationSeconds: null,
			isPlaying: true,
			isBuffering: false,
			isFullyDownloaded: false,
			volume: 1
		};
		expect(getPlaybackPositionForItem('i1', 42)).toBe(42);
	});

	it('returns live position when item is current', () => {
		playbackState.currentPlaybackState = {
			itemId: 'i1',
			positionSeconds: 55,
			durationSeconds: 100,
			fileDurationSeconds: null,
			isPlaying: true,
			isBuffering: false,
			isFullyDownloaded: false,
			volume: 1
		};
		expect(getPlaybackPositionForItem('i1', 42)).toBe(55);
	});
});

describe('getCoverTheme', () => {
	it('returns the fallback theme by default', () => {
		const theme = getCoverTheme();
		expect(theme.bg1).toBe(FALLBACK_BG1);
		expect(theme.fg).toBe(FALLBACK_FG);
	});
});
