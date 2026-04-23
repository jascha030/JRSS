/* eslint-disable svelte/prefer-svelte-reactivity */
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
	savePlaybackContext,
	queryStationEpisodes,
	extractCoverPalette
} from '$lib/services/feedService';
import { tick } from 'svelte';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
	itemsState,
	patchItemSummary,
	loadItemsByIds,
	markItemRead,
	registerItem,
	getActiveQueryKey
} from './items.svelte';
import { selection } from './selection.svelte';
import { feedsState } from './feeds.svelte';

export type PlaybackState = {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
	volume: number;
};

export type CoverTheme = {
	bg1: string;
	bg2: string;
	bg3: string;
	glow1: string;
	glow2: string;
	glow3: string;
	fg: string;
	fgMuted: string;
	fgSubtle: string;
	accent: string;
	accentContrast: string;
	panelBg: string;
	panelBorder: string;
	buttonBg: string;
	buttonBgHover: string;
};

const FALLBACK_COVER_THEME: CoverTheme = {
	bg1: '#0f172a',
	bg2: '#1e293b',
	bg3: '#334155',
	glow1: 'rgba(59, 130, 246, 0.28)',
	glow2: 'rgba(168, 85, 247, 0.22)',
	glow3: 'rgba(236, 72, 153, 0.20)',
	fg: '#ffffff',
	fgMuted: 'rgba(255, 255, 255, 0.78)',
	fgSubtle: 'rgba(255, 255, 255, 0.56)',
	accent: '#ffffff',
	accentContrast: '#0f172a',
	panelBg: 'rgba(255, 255, 255, 0.08)',
	panelBorder: 'rgba(255, 255, 255, 0.16)',
	buttonBg: 'rgba(255, 255, 255, 0.10)',
	buttonBgHover: 'rgba(255, 255, 255, 0.16)'
};

export const playbackState = $state({
	currentPlaybackState: null as PlaybackState | null,
	isAudioLoading: false,
	manualQueue: [] as string[],
	autoQueue: [] as string[],
	playbackContext: null as { contextType: 'feed' | 'station'; id: string } | null,
	/**
	 * Cache of items known to playback.
	 * These are also registered in itemsState.itemSummariesById for consistency.
	 */
	audioItemsById: {} as Record<string, FeedListItem>,
	/**
	 * Pre-calculated cover theme for the current track.
	 * Extracted when a new track loads so CoverView can display instantly.
	 */
	coverTheme: FALLBACK_COVER_THEME as CoverTheme
});

export function resetPlaybackState(): void {
	playbackState.currentPlaybackState = null;
	playbackState.isAudioLoading = false;
	playbackState.manualQueue = [];
	playbackState.autoQueue = [];
	playbackState.playbackContext = null;
	playbackState.audioItemsById = {};
	playbackState.coverTheme = FALLBACK_COVER_THEME;
}

// ---------------------------------------------------------------------------
// Cover theme extraction
// ---------------------------------------------------------------------------

type Rgb = {
	r: number;
	g: number;
	b: number;
};

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value));
}

function hexToRgb(hex: string): Rgb {
	const normalized = hex.replace('#', '').trim();
	const full =
		normalized.length === 3
			? normalized
					.split('')
					.map((part) => part + part)
					.join('')
			: normalized;

	const int = Number.parseInt(full, 16);

	return {
		r: (int >> 16) & 255,
		g: (int >> 8) & 255,
		b: int & 255
	};
}

function rgbToHex({ r, g, b }: Rgb): string {
	return `#${[r, g, b]
		.map((value) => clamp(Math.round(value), 0, 255).toString(16).padStart(2, '0'))
		.join('')}`;
}

function rgba({ r, g, b }: Rgb, alpha: number): string {
	return `rgba(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}, ${alpha})`;
}

function mix(a: Rgb, b: Rgb, amount: number): Rgb {
	const t = clamp(amount, 0, 1);
	return {
		r: a.r + (b.r - a.r) * t,
		g: a.g + (b.g - a.g) * t,
		b: a.b + (b.b - a.b) * t
	};
}

function average(colors: Rgb[]): Rgb {
	if (colors.length === 0) return { r: 255, g: 255, b: 255 };

	const total = colors.reduce(
		(acc, color) => {
			acc.r += color.r;
			acc.g += color.g;
			acc.b += color.b;
			return acc;
		},
		{ r: 0, g: 0, b: 0 }
	);

	return {
		r: total.r / colors.length,
		g: total.g / colors.length,
		b: total.b / colors.length
	};
}

function channelToLinear(value: number): number {
	const s = value / 255;
	return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

function luminance(color: Rgb): number {
	return (
		0.2126 * channelToLinear(color.r) +
		0.7152 * channelToLinear(color.g) +
		0.0722 * channelToLinear(color.b)
	);
}

function contrastRatio(a: Rgb, b: Rgb): number {
	const l1 = luminance(a);
	const l2 = luminance(b);
	const lighter = Math.max(l1, l2);
	const darker = Math.min(l1, l2);
	return (lighter + 0.05) / (darker + 0.05);
}

function readableTextFor(color: Rgb): Rgb {
	const white = hexToRgb('#ffffff');
	const slate = hexToRgb('#0f172a');

	return contrastRatio(white, color) >= contrastRatio(slate, color) ? white : slate;
}

function buildThemeFromPalette(hexes: string[]): CoverTheme {
	const source = hexes.map(hexToRgb).slice(0, 3);

	while (source.length < 3) {
		source.push(source[source.length - 1] ?? hexToRgb('#334155'));
	}

	const slate950 = hexToRgb('#020617');
	const slate900 = hexToRgb('#0f172a');
	const slate800 = hexToRgb('#1e293b');
	const white = hexToRgb('#ffffff');

	const bg1 = mix(source[0], slate950, 0.52);
	const bg2 = mix(source[1], slate900, 0.45);
	const bg3 = mix(source[2], slate800, 0.36);

	const averageBg = average([bg1, bg2, bg3]);
	const fgBase = readableTextFor(averageBg);
	const lightForeground = contrastRatio(white, averageBg) >= 4.5 || fgBase.r > 127;

	const fg = lightForeground ? white : hexToRgb('#0f172a');
	const accentBase = mix(source[0], white, lightForeground ? 0.08 : 0);
	const accentContrast = readableTextFor(accentBase);

	return {
		bg1: rgbToHex(bg1),
		bg2: rgbToHex(bg2),
		bg3: rgbToHex(bg3),
		glow1: rgba(source[0], 0.3),
		glow2: rgba(source[1], 0.24),
		glow3: rgba(source[2], 0.2),
		fg: rgbToHex(fg),
		fgMuted: rgba(fg, lightForeground ? 0.78 : 0.76),
		fgSubtle: rgba(fg, lightForeground ? 0.56 : 0.52),
		accent: rgbToHex(accentBase),
		accentContrast: rgbToHex(accentContrast),
		panelBg: lightForeground ? 'rgba(255, 255, 255, 0.08)' : 'rgba(15, 23, 42, 0.08)',
		panelBorder: lightForeground ? 'rgba(255, 255, 255, 0.16)' : 'rgba(15, 23, 42, 0.12)',
		buttonBg: lightForeground ? 'rgba(255, 255, 255, 0.10)' : 'rgba(15, 23, 42, 0.08)',
		buttonBgHover: lightForeground ? 'rgba(255, 255, 255, 0.16)' : 'rgba(15, 23, 42, 0.14)'
	};
}

/**
 * Pre-calculate cover theme from an image URL.
 * Call this when a new track loads so the theme is ready when CoverView opens.
 */
export async function precalculateCoverTheme(imageUrl: string | undefined): Promise<void> {
	if (!imageUrl) {
		playbackState.coverTheme = FALLBACK_COVER_THEME;
		return;
	}

	try {
		const palette = await extractCoverPalette(imageUrl);
		playbackState.coverTheme =
			palette.length > 0 ? buildThemeFromPalette(palette) : FALLBACK_COVER_THEME;
	} catch (error) {
		console.error('Failed to extract cover palette:', error);
		playbackState.coverTheme = FALLBACK_COVER_THEME;
	}
}

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
		patchAudioItem(event.payload.itemId, { playbackPositionSeconds: 0 });
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
// Item cache helpers - keep both caches in sync
// ---------------------------------------------------------------------------

/**
 * Register an item in both playback and items caches.
 */
function registerAudioItem(item: FeedListItem): void {
	playbackState.audioItemsById[item.id] = item;
	registerItem(item);
}

/**
 * Patch an item in the playback cache (in addition to items cache).
 */
function patchAudioItem(
	itemId: string,
	patch: Partial<Pick<FeedListItem, 'read' | 'playbackPositionSeconds'>>
): void {
	const existingItem = playbackState.audioItemsById[itemId];
	if (existingItem) {
		playbackState.audioItemsById[itemId] = { ...existingItem, ...patch };
	}
}

/**
 * Resolve an item from either cache.
 */
function resolveItem(itemId: string): FeedListItem | null {
	return playbackState.audioItemsById[itemId] ?? itemsState.itemSummariesById[itemId] ?? null;
}

/**
 * Resolve a media item from either cache.
 */
function resolveAudioItem(itemId: string): MediaListItem | null {
	const item = resolveItem(itemId);
	return item && isMediaItem(item) ? item : null;
}

// ---------------------------------------------------------------------------
// Backend-owned playback session sync
// ---------------------------------------------------------------------------

const inFlightAudioItemHydrations: Record<string, Promise<void> | undefined> = {};

async function ensureAudioItemsLoaded(itemIds: string[]): Promise<void> {
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
						registerAudioItem(item);
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
		const currentItem = resolveItem(queueState.current.itemId);
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
		if (fromEvent) {
			playbackState.isAudioLoading = false;
		}
		return;
	}

	const previousItemId = previous?.itemId;
	const wasPlaying = previous?.isPlaying ?? false;

	// If item changed, pre-calculate the cover theme
	if (previousItemId !== event.itemId) {
		const item = resolveItem(event.itemId);
		if (item) {
			const feed = feedsState.feeds.find((f) => f.id === item.feedId);
			void precalculateCoverTheme(feed?.imageUrl);
		}
	}

	playbackState.currentPlaybackState = {
		itemId: event.itemId,
		positionSeconds,
		durationSeconds,
		isPlaying: event.isPlaying,
		volume: event.volume
	};

	// Sync position back to caches when playback stops
	if (!event.isPlaying) {
		patchItemSummary(event.itemId, { playbackPositionSeconds: positionSeconds });
		patchAudioItem(event.itemId, { playbackPositionSeconds: positionSeconds });
	}

	if (fromEvent) {
		playbackState.isAudioLoading = false;
	}

	// Mark as read when playback starts
	if (event.isPlaying && (!wasPlaying || previousItemId !== event.itemId)) {
		void markItemRead(event.itemId, true).catch((error) => {
			console.error('Failed to mark item as read during playback.', error);
		});
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
	}

	if (!backendQueueState.current) {
		playbackState.currentPlaybackState = null;
		return;
	}

	// Precalculate cover theme for restored session (handles app restart case)
	const currentItem = resolveItem(backendQueueState.current.itemId);
	if (currentItem) {
		const feed = feedsState.feeds.find((f) => f.id === currentItem.feedId);
		void precalculateCoverTheme(feed?.imageUrl);
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
		const queuedItem = resolveAudioItem(itemId);
		if (queuedItem) {
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
	registerAudioItem(item);
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

	// Pre-calculate cover theme for the new track
	const feed = feedsState.feeds.find((f) => f.id === item.feedId);
	void precalculateCoverTheme(feed?.imageUrl);

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
    console.log(playbackState);
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

export function getManualQueueLength(): number {
	return playbackState.manualQueue.length;
}

export function getUpcomingQueue(): MediaListItem[] {
	const items: MediaListItem[] = [];
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
	const seen = new Set<string>();
	const queueItems: { itemId: string; url: string; title: string; durationSeconds: number }[] = [];

	for (const item of items) {
		if (!isMediaItem(item) || seen.has(item.id)) {
			continue;
		}
		seen.add(item.id);
		registerAudioItem(item);
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

	registerAudioItem(item);

	void audioQueueEnqueue(itemToQueuedItem(item)).catch(console.error);
}

export function playAudioItemNext(item: MediaListItem): void {
	if (playbackState.currentPlaybackState?.itemId === item.id) {
		return;
	}

	registerAudioItem(item);

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
			const item = resolveItem(itemId);
			return item ? item.feedId === feedId : false;
		}
	);

	for (const itemId of queuedIdsToRemove) {
		await audioQueueRemove(itemId).catch((error: unknown) => {
			console.error('Failed to remove deleted feed item from backend queue.', error);
		});
	}

	playbackState.manualQueue = playbackState.manualQueue.filter((id) => {
		const item = resolveItem(id);
		return item ? item.feedId !== feedId : true;
	});
	playbackState.autoQueue = playbackState.autoQueue.filter((id) => {
		const item = resolveItem(id);
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
			registerAudioItem(candidate);
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
	const page = await queryStationEpisodes(stationId, 0, 500);
	const mediaItems = page.items.filter(isMediaItem);

	if (mediaItems.length === 0) {
		return;
	}

	const firstItem = mediaItems[0];
	const rest = mediaItems.slice(1);

	// Register all items
	for (const item of mediaItems) {
		registerAudioItem(item);
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

	return resolveAudioItem(state.itemId);
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

export function getCoverTheme(): CoverTheme {
	return playbackState.coverTheme;
}
