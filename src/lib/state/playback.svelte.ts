import type {
	BackendPlaybackEndedEvent,
	BackendPlaybackState,
	BackendQueueState,
	FeedListItem,
	MediaListItem
} from '$lib/types/rss';
import { isMediaItem } from '$lib/types/rss';
import {
	audioGetState,
	audioPlayWithQueue,
	audioQueueClear,
	audioQueueEnqueue,
	audioQueueGetState,
	audioQueueMoveDown,
	audioQueueMoveUp,
	audioQueuePlayNext,
	audioQueueRemove,
	audioQueueSet,
	audioSeek,
	audioSetVolume,
	audioStop,
	audioToggle,
	loadPlaybackContext,
	savePlaybackContext
} from '$lib/services/feedService';
import { tick } from 'svelte';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { itemsState, patchItemSummary } from './items.svelte';
import { selection } from './selection.svelte';
import { feedsState } from './feeds.svelte';

export const playbackState = $state({
	currentPlaybackState: null as PlaybackState | null,
	isAudioLoading: false,
	manualQueue: [] as string[],
	autoQueue: [] as string[],
	playbackContext: null as { contextType: 'feed' | 'station'; id: string } | null,
	audioItemsById: {} as Record<string, FeedListItem>
});

export type PlaybackState = {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
	volume: number;
};

// ---------------------------------------------------------------------------
// Audio event listeners
// ---------------------------------------------------------------------------

let audioEventUnlisteners: UnlistenFn[] = [];

export async function initAudioEventListeners(): Promise<void> {
	// Clean up any previous listeners
	for (const unlisten of audioEventUnlisteners) {
		unlisten();
	}
	audioEventUnlisteners = [];

	const unlistenState = await listen<BackendPlaybackState>('playback-state-changed', (event) => {
		void ensureAudioItemsLoaded([event.payload.itemId]).then(() => {
			applyBackendPlaybackState(event.payload, true);
		});
	});

	const unlistenEnded = await listen<BackendPlaybackEndedEvent>('playback-ended', (event) => {
		patchItemSummary(event.payload.itemId, { playbackPositionSeconds: 0 });
	});

	const unlistenStopped = await listen('playback-stopped', () => {
		playbackState.currentPlaybackState = null;
	});

	const unlistenQueueChanged = await listen<BackendQueueState>('queue-changed', (event) => {
		const itemIds = [
			...(event.payload.current ? [event.payload.current.itemId] : []),
			...event.payload.manual.map((item) => item.itemId),
			...event.payload.auto.map((item) => item.itemId)
		];

		void ensureAudioItemsLoaded(itemIds).then(() => {
			applyBackendQueueState(event.payload);
		});
	});

	audioEventUnlisteners = [unlistenState, unlistenEnded, unlistenStopped, unlistenQueueChanged];
}

// ---------------------------------------------------------------------------
// Backend-owned playback session sync
// ---------------------------------------------------------------------------

const inFlightAudioItemHydrations: Record<string, Promise<void> | undefined> = {};

async function ensureAudioItemsLoaded(itemIds: string[]): Promise<void> {
	const { loadItemsByIds } = await import('./items.svelte');

	const missingIds = [...new Set(itemIds)].filter(
		(itemId) => !playbackState.audioItemsById[itemId] && !itemsState.itemSummariesById[itemId]
	);

	if (missingIds.length === 0) {
		return;
	}

	const newIds = missingIds.filter((itemId) => !inFlightAudioItemHydrations[itemId]);

	if (newIds.length > 0) {
		const newRequest = (async () => {
			try {
				const items = await loadItemsByIds(newIds);
				for (const item of items) {
					if (isMediaItem(item)) {
						playbackState.audioItemsById[item.id] = item;
						itemsState.itemSummariesById[item.id] = item;
					}
				}
			} catch (error) {
				console.error('Failed to hydrate audio items from backend state.', error);
			} finally {
				for (const itemId of newIds) {
					delete inFlightAudioItemHydrations[itemId];
				}
			}
		})();

		for (const itemId of newIds) {
			inFlightAudioItemHydrations[itemId] = newRequest;
		}
	}

	const pendingRequests = missingIds
		.map((itemId) => inFlightAudioItemHydrations[itemId])
		.filter((request): request is Promise<void> => request !== undefined);

	if (pendingRequests.length > 0) {
		await Promise.all(pendingRequests);
	}
}

function applyBackendQueueState(queueState: BackendQueueState): void {
	playbackState.manualQueue = queueState.manual.map((item) => item.itemId);
	playbackState.autoQueue = queueState.auto.map((item) => item.itemId);

	if (!queueState.current) {
		if (!playbackState.currentPlaybackState?.isPlaying) {
			playbackState.currentPlaybackState = null;
		}
		return;
	}

	if (
		!playbackState.currentPlaybackState ||
		playbackState.currentPlaybackState.itemId !== queueState.current.itemId
	) {
		const currentItem =
			playbackState.audioItemsById[queueState.current.itemId] ??
			itemsState.itemSummariesById[queueState.current.itemId];
		const fallbackPosition =
			currentItem && isMediaItem(currentItem) ? currentItem.playbackPositionSeconds : 0;

		playbackState.currentPlaybackState = {
			itemId: queueState.current.itemId,
			positionSeconds: fallbackPosition,
			durationSeconds: queueState.current.durationSeconds,
			isPlaying: false,
			volume: playbackState.currentPlaybackState?.volume ?? 1
		};
	}
}

function applyBackendPlaybackState(event: BackendPlaybackState, fromEvent: boolean = false): void {
	const positionSeconds = Math.floor(event.positionSeconds);
	const durationSeconds = Math.floor(event.durationSeconds);
	const previous = playbackState.currentPlaybackState;

	// Check if playback state actually changed
	const playbackUnchanged =
		previous &&
		previous.itemId === event.itemId &&
		previous.positionSeconds === positionSeconds &&
		previous.durationSeconds === durationSeconds &&
		previous.isPlaying === event.isPlaying &&
		previous.volume === event.volume;

	if (playbackUnchanged) {
		// Still clear loading if this was an event
		if (fromEvent) {
			playbackState.isAudioLoading = false;
		}
		return;
	}

	const previousItemId = previous?.itemId;
	const wasPlaying = previous?.isPlaying ?? false;

	playbackState.currentPlaybackState = {
		itemId: event.itemId,
		positionSeconds,
		durationSeconds,
		isPlaying: event.isPlaying,
		volume: event.volume
	};

	// Avoid mutating the large item summary maps on every playback tick.
	// We only sync the stored position back into list data when playback is not active.
	if (!event.isPlaying) {
		patchItemSummary(event.itemId, { playbackPositionSeconds: positionSeconds });
	}

	// Only clear loading state when we receive an actual event from the backend
	if (fromEvent) {
		playbackState.isAudioLoading = false;
	}

	// Mark as read when playback starts
	if (event.isPlaying && (!wasPlaying || previousItemId !== event.itemId)) {
		void markItemReadPlayback(event.itemId);
	}
}

async function markItemReadPlayback(itemId: string): Promise<void> {
	const { markItemRead } = await import('./items.svelte');
	try {
		await markItemRead(itemId, true);
	} catch (error) {
		console.error('Failed to mark item as read during playback.', error);
	}
}

export async function syncAudioSessionFromBackend(): Promise<void> {
	const [backendPlaybackState, backendQueueState] = await Promise.all([
		audioGetState(),
		audioQueueGetState()
	]);

	const itemIds = [
		...(backendQueueState.current ? [backendQueueState.current.itemId] : []),
		...backendQueueState.manual.map((item) => item.itemId),
		...backendQueueState.auto.map((item) => item.itemId),
		...(backendPlaybackState ? [backendPlaybackState.itemId] : [])
	];

	await ensureAudioItemsLoaded(itemIds);
	applyBackendQueueState(backendQueueState);

	if (backendPlaybackState) {
		applyBackendPlaybackState(backendPlaybackState);
		return;
	}

	if (!backendQueueState.current) {
		playbackState.currentPlaybackState = null;
	}
}

// ---------------------------------------------------------------------------
// Playback controls
// ---------------------------------------------------------------------------

async function yieldForPlaybackUiPaint(): Promise<void> {
	await tick();
	await new Promise<void>((resolve) => {
		if (typeof window === 'undefined') {
			setTimeout(resolve, 0);
			return;
		}
		window.requestAnimationFrame(() => {
			window.setTimeout(resolve, 0);
		});
	});
}

function itemToQueuedItem(item: MediaListItem): {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
} {
	return {
		itemId: item.id,
		url: item.mediaEnclosure.url,
		title: item.title,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0
	};
}

function queueIdsToQueuedItems(itemIds: string[]): {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
}[] {
	const queueItems: { itemId: string; url: string; title: string; durationSeconds: number }[] = [];

	for (const itemId of itemIds) {
		const queuedItem = playbackState.audioItemsById[itemId] ?? itemsState.itemSummariesById[itemId];
		if (queuedItem && isMediaItem(queuedItem)) {
			queueItems.push(itemToQueuedItem(queuedItem));
		}
	}
	return queueItems;
}

export function playAudioItem(
	item: MediaListItem,
	{
		manualQueueIds = playbackState.manualQueue.filter((itemId) => itemId !== item.id),
		autoQueueIds = playbackState.autoQueue.filter((itemId) => itemId !== item.id),
		startPositionSeconds = item.playbackPositionSeconds,
		context
	}: {
		manualQueueIds?: string[];
		autoQueueIds?: string[];
		startPositionSeconds?: number;
		context?: { contextType: 'feed' | 'station'; id: string } | null;
	} = {}
): void {
	playbackState.audioItemsById[item.id] = item;
	itemsState.itemSummariesById[item.id] = item;
	playbackState.isAudioLoading = true;
	playbackState.currentPlaybackState = {
		itemId: item.id,
		positionSeconds: startPositionSeconds,
		durationSeconds: item.mediaEnclosure.durationSeconds ?? 0,
		isPlaying: false,
		volume: playbackState.currentPlaybackState?.volume ?? 1
	};

	// Set playback context (defaults to feed context if not specified)
	playbackState.playbackContext = context ?? { contextType: 'feed', id: item.feedId };

	// Persist playback context
	void persistPlaybackContext();

	void (async () => {
		try {
			await yieldForPlaybackUiPaint();
			await audioPlayWithQueue(
				itemToQueuedItem(item),
				queueIdsToQueuedItems(manualQueueIds),
				queueIdsToQueuedItems(autoQueueIds),
				startPositionSeconds
			);
		} catch (error: unknown) {
			console.error('Failed to start audio playback.', error);
			playbackState.isAudioLoading = false;
			void syncAudioSessionFromBackend().catch((syncError: unknown) => {
				console.error('Failed to resync audio after playback start failure.', syncError);
			});
		}
	})();
}

export function stopPlayback(): Promise<void> {
	return audioStop().catch((error: unknown) => {
		console.error('Failed to stop audio.', error);
	});
}

export function requestTogglePlayback(): void {
	const currentPlaybackState = playbackState.currentPlaybackState;
	if (!currentPlaybackState) {
		return;
	}

	void audioToggle().catch((error: unknown) => {
		console.error('Failed to toggle playback.', error);
		void syncAudioSessionFromBackend().catch((syncError: unknown) => {
			console.error('Failed to resync audio after toggle failure.', syncError);
		});
	});
}

export function requestSeekTo(positionSeconds: number): void {
	void audioSeek(positionSeconds).catch((error: unknown) => {
		console.error('Failed to seek.', error);
	});
}

let pendingVolumeTimeout: ReturnType<typeof setTimeout> | null = null;

export function requestSetVolume(volume: number): void {
	if (pendingVolumeTimeout !== null) {
		clearTimeout(pendingVolumeTimeout);
	}

	pendingVolumeTimeout = setTimeout(() => {
		pendingVolumeTimeout = null;
		void audioSetVolume(volume).catch((error: unknown) => {
			console.error('Failed to set volume.', error);
			void syncAudioSessionFromBackend().catch((syncError: unknown) => {
				console.error('Failed to resync audio after volume failure.', syncError);
			});
		});
	}, 125);
}

// ---------------------------------------------------------------------------
// Queue operations
// ---------------------------------------------------------------------------

/** Resolve an item ID to a MediaListItem, or null. */
function resolveAudioItem(itemId: string): MediaListItem | null {
	const item = playbackState.audioItemsById[itemId] ?? itemsState.itemSummariesById[itemId];
	return item && isMediaItem(item) ? item : null;
}

export function getManualQueueLength(): number {
	return playbackState.manualQueue.length;
}

export function getUpcomingQueue(): MediaListItem[] {
	const items: MediaListItem[] = [];
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral dedup set, not reactive state
	const seen = new Set<string>();

	for (const itemId of playbackState.manualQueue) {
		const item = resolveAudioItem(itemId);
		if (item && !seen.has(item.id)) {
			seen.add(item.id);
			items.push(item);
		}
	}

	for (const itemId of playbackState.autoQueue) {
		const item = resolveAudioItem(itemId);
		if (item && !seen.has(item.id)) {
			seen.add(item.id);
			items.push(item);
		}
	}

	return items;
}

export function setPlaybackQueue(items: FeedListItem[]): void {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral dedup set, not reactive state
	const seen = new Set<string>();
	const queueItems: { itemId: string; url: string; title: string; durationSeconds: number }[] = [];

	for (const item of items) {
		if (!isMediaItem(item) || seen.has(item.id)) {
			continue;
		}
		seen.add(item.id);
		playbackState.audioItemsById[item.id] = item;
		itemsState.itemSummariesById[item.id] = item;
		queueItems.push(itemToQueuedItem(item));
	}

	void audioQueueSet(queueItems).catch(console.error);
}

export function enqueueAudioItem(item: MediaListItem): void {
	if (playbackState.currentPlaybackState?.itemId === item.id) {
		return;
	}

	if (playbackState.manualQueue.includes(item.id)) {
		return;
	}

	playbackState.audioItemsById[item.id] = item;
	itemsState.itemSummariesById[item.id] = item;

	void audioQueueEnqueue(itemToQueuedItem(item)).catch(console.error);
}

export function playAudioItemNext(item: MediaListItem): void {
	if (playbackState.currentPlaybackState?.itemId === item.id) {
		return;
	}

	playbackState.audioItemsById[item.id] = item;
	itemsState.itemSummariesById[item.id] = item;

	void audioQueuePlayNext(itemToQueuedItem(item)).catch(console.error);
}

export function moveQueuedItemUp(itemId: string): void {
	void audioQueueMoveUp(itemId).catch(console.error);
}

export function moveQueuedItemDown(itemId: string): void {
	void audioQueueMoveDown(itemId).catch(console.error);
}

export function removeQueuedItem(itemId: string): void {
	void audioQueueRemove(itemId).catch(console.error);
}

export function clearQueue(): void {
	void audioQueueClear().catch(console.error);
}

export async function removeFromQueuesByFeedId(feedId: string): Promise<void> {
	const queuedIdsToRemove = [...playbackState.manualQueue, ...playbackState.autoQueue].filter(
		(itemId) => {
			const item = playbackState.audioItemsById[itemId] ?? itemsState.itemSummariesById[itemId];
			return item ? item.feedId === feedId : false;
		}
	);

	for (const itemId of queuedIdsToRemove) {
		await audioQueueRemove(itemId).catch((error: unknown) => {
			console.error('Failed to remove deleted feed item from backend queue.', error);
		});
	}

	playbackState.manualQueue = playbackState.manualQueue.filter((id) => {
		const item = playbackState.audioItemsById[id] ?? itemsState.itemSummariesById[id];
		return item ? item.feedId !== feedId : true;
	});
	playbackState.autoQueue = playbackState.autoQueue.filter((id) => {
		const item = playbackState.audioItemsById[id] ?? itemsState.itemSummariesById[id];
		return item ? item.feedId !== feedId : true;
	});
}

// ---------------------------------------------------------------------------
// Context-aware playback start
// ---------------------------------------------------------------------------

function deriveAutoContinuation(playingItemId: string): string[] {
	const queryKey = getActiveQueryKey();
	if (!queryKey) {
		return [];
	}

	const itemIdsByIndex = itemsState.itemIdsByIndexByQueryKey[queryKey];
	if (!itemIdsByIndex) {
		return [];
	}

	// Build ordered array of item IDs from the sparse index map
	const sortedIndexes = Object.keys(itemIdsByIndex)
		.map(Number)
		.sort((a, b) => a - b);

	// Find the playing item's position
	let playingPosition = -1;
	for (const idx of sortedIndexes) {
		if (itemIdsByIndex[idx] === playingItemId) {
			playingPosition = idx;
			break;
		}
	}

	if (playingPosition < 0) {
		return [];
	}

	// Collect audio items after the playing position
	const manualSet = new Set(playbackState.manualQueue);
	const continuation: string[] = [];

	for (const idx of sortedIndexes) {
		if (idx <= playingPosition) {
			continue;
		}

		const candidateId = itemIdsByIndex[idx];
		if (!candidateId || candidateId === playingItemId) {
			continue;
		}

		// Skip items already in the manual queue
		if (manualSet.has(candidateId)) {
			continue;
		}

		const candidate = itemsState.itemSummariesById[candidateId];
		if (candidate && isMediaItem(candidate)) {
			playbackState.audioItemsById[candidateId] = candidate;
			continuation.push(candidateId);
		}
	}

	return continuation;
}

export function startPlaybackFromContext(item: MediaListItem): void {
	const manualQueueIds = playbackState.manualQueue.filter((itemId) => itemId !== item.id);
	const autoQueueIds = deriveAutoContinuation(item.id);

	// Determine playback context from current selection
	const context: { contextType: 'feed' | 'station'; id: string } | null =
		selection.selectedStationId
			? { contextType: 'station', id: selection.selectedStationId }
			: selection.selectedFeedId
				? { contextType: 'feed', id: selection.selectedFeedId }
				: null;

	playAudioItem(item, {
		manualQueueIds,
		autoQueueIds,
		startPositionSeconds: item.playbackPositionSeconds,
		context
	});
}

export async function playStation(stationId: string): Promise<void> {
	const { queryStationEpisodes } = await import('$lib/services/feedService');
	const page = await queryStationEpisodes(stationId, 0, 500);
	const mediaItems = page.items.filter(isMediaItem);

	if (mediaItems.length === 0) {
		return;
	}

	const firstItem = mediaItems[0];
	const rest = mediaItems.slice(1);

	// Register all items in audioItemsById
	for (const item of mediaItems) {
		playbackState.audioItemsById[item.id] = item;
		itemsState.itemSummariesById[item.id] = item;
	}

	playAudioItem(firstItem, {
		manualQueueIds: rest.map((item) => item.id),
		autoQueueIds: [],
		context: { contextType: 'station', id: stationId }
	});
}

export async function handlePlaybackEnded(): Promise<void> {
	await syncAudioSessionFromBackend();
}

// ---------------------------------------------------------------------------
// Playback context persistence
// ---------------------------------------------------------------------------

export async function restorePlaybackContext(): Promise<void> {
	const context = await loadPlaybackContext();
	if (context) {
		playbackState.playbackContext = context;
	}
}

export async function persistPlaybackContext(): Promise<void> {
	await savePlaybackContext(playbackState.playbackContext);
}

// ---------------------------------------------------------------------------
// Getters
// ---------------------------------------------------------------------------

export function getCurrentAudioItem(): MediaListItem | null {
	const state = playbackState.currentPlaybackState;
	if (!state) {
		return null;
	}

	const currentItem =
		playbackState.audioItemsById[state.itemId] ?? itemsState.itemSummariesById[state.itemId];

	return currentItem && isMediaItem(currentItem) ? currentItem : null;
}

export function getCurrentAudioItemFeed() {
	const item = getCurrentAudioItem();
	if (!item) {
		return null;
	}
	return feedsState.feeds.find((feed) => feed.id === item.feedId) ?? null;
}

export function getPlaybackContext(): { contextType: 'feed' | 'station'; id: string } | null {
	return playbackState.playbackContext;
}

export function isItemCurrentAudio(itemId: string): boolean {
	return playbackState.currentPlaybackState?.itemId === itemId;
}

export function isAudioPlaying(): boolean {
	return playbackState.currentPlaybackState?.isPlaying ?? false;
}

export function isAudioLoading(): boolean {
	return playbackState.isAudioLoading;
}

export function getPlaybackPositionForItem(
	itemId: string,
	fallbackPositionSeconds: number
): number {
	if (playbackState.currentPlaybackState?.itemId === itemId) {
		return playbackState.currentPlaybackState.positionSeconds;
	}
	return fallbackPositionSeconds;
}

import { getActiveQueryKey } from './items.svelte';
