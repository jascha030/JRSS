import { invokeCommand, isTauriRuntime } from '$lib/services/tauriClient';
import type {
	AppSettings,
	BackendQueueState,
	BackendPlaybackState,
	CreateStationInput,
	Feed,
	FeedItem,
	FeedListItem,
	ItemPage,
	ItemPageQuery,
	ItemSortOrder,
	PlaybackSession,
	RawFeedItem,
	RawFeedListItem,
	Station,
	UpdateStationInput
} from '$lib/types/rss';
import {
	DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
	mapRawFeedItem,
	mapRawFeedListItem
} from '$lib/types/rss';
import { measurePerfAsync } from '$lib/utils/perfDebug';

function normalizeFeedInput(url: string): string {
	return url.trim();
}

export async function listFeeds(): Promise<Feed[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	return invokeCommand<Feed[]>('list_feeds');
}

export async function addFeed(url: string): Promise<Feed> {
	const normalizedInput = normalizeFeedInput(url);

	if (!normalizedInput) {
		throw new Error('Enter a feed URL, Apple Podcasts URL, or Apple Podcasts ID.');
	}

	return invokeCommand<Feed>('add_feed', { url: normalizedInput });
}

export async function refreshFeed(id: string): Promise<Feed> {
	return invokeCommand<Feed>('refresh_feed', { id });
}

export async function removeFeed(id: string): Promise<void> {
	await invokeCommand('remove_feed', { id });
}

export async function queryItemsPage(query: ItemPageQuery): Promise<ItemPage<FeedListItem>> {
	if (!isTauriRuntime()) {
		return {
			items: [],
			totalCount: 0
		};
	}

	const raw = await measurePerfAsync(
		'tauri.query_items_page',
		() =>
			invokeCommand<ItemPage<RawFeedListItem>>('query_items_page', {
				query: {
					feedId: query.feedId ?? null,
					section: query.section,
					offset: query.offset,
					limit: query.limit,
					search: query.search ?? null,
					sortOrder: query.sortOrder ?? 'newest_first'
				}
			}),
		{
			feedId: query.feedId ?? null,
			section: query.section,
			offset: query.offset,
			limit: query.limit,
			search: query.search ?? null,
			sortOrder: query.sortOrder ?? 'newest_first'
		}
	);

	return { items: raw.items.map(mapRawFeedListItem), totalCount: raw.totalCount };
}

export async function getItemDetails(itemId: string): Promise<FeedItem> {
	const raw = await invokeCommand<RawFeedItem>('get_item_details', { itemId });
	return mapRawFeedItem(raw);
}

export async function markRead(itemId: string, read: boolean): Promise<void> {
	await invokeCommand('mark_read', { itemId, read });
}

export async function savePlayback(itemId: string, positionSeconds: number): Promise<void> {
	await invokeCommand('save_playback', {
		itemId,
		positionSeconds: Math.max(0, Math.floor(positionSeconds))
	});
}

export async function loadReaderContent(itemId: string): Promise<FeedItem> {
	const raw = await invokeCommand<RawFeedItem>('load_reader_content', { itemId });
	return mapRawFeedItem(raw);
}

export async function savePlaybackSession(session: PlaybackSession): Promise<void> {
	await invokeCommand('save_playback_session', { session });
}

export async function loadPlaybackSession(): Promise<PlaybackSession | null> {
	if (!isTauriRuntime()) {
		return null;
	}

	return invokeCommand<PlaybackSession | null>('load_playback_session');
}

export async function clearPlaybackSession(): Promise<void> {
	await invokeCommand('clear_playback_session');
}

export async function loadAppSettings(): Promise<AppSettings> {
	if (!isTauriRuntime()) {
		return {
			maxAudioCacheSizeBytes: DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES
		};
	}

	return invokeCommand<AppSettings>('load_app_settings');
}

export async function saveAppSettings(settings: AppSettings): Promise<AppSettings> {
	if (!isTauriRuntime()) {
		return settings;
	}

	return invokeCommand<AppSettings>('save_app_settings', { settings });
}

export async function savePlaybackContext(
	context: { contextType: 'feed' | 'station'; id: string } | null
): Promise<void> {
	await invokeCommand('save_playback_context', { context });
}

export async function loadPlaybackContext(): Promise<{
	contextType: 'feed' | 'station';
	id: string;
} | null> {
	if (!isTauriRuntime()) {
		return null;
	}
	return invokeCommand<{ contextType: 'feed' | 'station'; id: string } | null>(
		'load_playback_context'
	);
}

export async function getItemsByIds(itemIds: string[]): Promise<FeedListItem[]> {
	if (itemIds.length === 0) {
		return [];
	}

	const raw = await invokeCommand<RawFeedListItem[]>('get_items_by_ids', { itemIds });
	return raw.map(mapRawFeedListItem);
}

export async function setFeedSortOrder(
	feedId: string,
	sortOrder: ItemSortOrder | null
): Promise<void> {
	await invokeCommand('set_feed_sort_order', {
		feedId,
		sortOrder
	});
}

// ---------------------------------------------------------------------------
// Stations
// ---------------------------------------------------------------------------

/** Raw IPC shape — station fields are flat, feedIds is a sibling. */
interface RawStationWithFeeds {
	id: string;
	name: string;
	episodeFilter: string;
	sortOrder: string;
	sortOrderPosition: number;
	createdAt: string;
	feedIds: string[];
}

function mapRawStation(raw: RawStationWithFeeds): Station {
	return {
		id: raw.id,
		name: raw.name,
		episodeFilter: raw.episodeFilter as Station['episodeFilter'],
		sortOrder: raw.sortOrder as Station['sortOrder'],
		sortOrderPosition: raw.sortOrderPosition,
		createdAt: raw.createdAt,
		feedIds: raw.feedIds
	};
}

export async function listStations(): Promise<Station[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	const raw = await invokeCommand<RawStationWithFeeds[]>('list_stations');
	return raw.map(mapRawStation);
}

export async function createStation(input: CreateStationInput): Promise<Station> {
	const raw = await invokeCommand<RawStationWithFeeds>('create_station', { input });
	return mapRawStation(raw);
}

export async function updateStation(input: UpdateStationInput): Promise<Station> {
	const raw = await invokeCommand<RawStationWithFeeds>('update_station', { input });
	return mapRawStation(raw);
}

export async function deleteStation(id: string): Promise<void> {
	await invokeCommand('delete_station', { id });
}

export async function queryStationEpisodes(
	stationId: string,
	offset: number,
	limit: number
): Promise<ItemPage<FeedListItem>> {
	const raw = await invokeCommand<ItemPage<RawFeedListItem>>('query_station_episodes', {
		stationId,
		offset,
		limit
	});
	return { items: raw.items.map(mapRawFeedListItem), totalCount: raw.totalCount };
}

// ---------------------------------------------------------------------------
// Audio playback — backend-owned via rodio
// ---------------------------------------------------------------------------

export async function audioPlay(
	itemId: string,
	url: string,
	startPositionSeconds: number,
	durationHintSeconds: number
): Promise<void> {
	await invokeCommand('audio_play', {
		itemId,
		url,
		startPositionSeconds,
		durationHintSeconds
	});
}

export async function audioPause(): Promise<void> {
	await invokeCommand('audio_pause');
}

export async function audioResume(): Promise<void> {
	await invokeCommand('audio_resume');
}

export async function audioToggle(): Promise<void> {
	await invokeCommand('audio_toggle');
}

export async function audioStop(): Promise<void> {
	await invokeCommand('audio_stop');
}

export async function audioSeek(positionSeconds: number): Promise<void> {
	await invokeCommand('audio_seek', { positionSeconds });
}

export async function audioSetVolume(volume: number): Promise<void> {
	await invokeCommand('audio_set_volume', { volume });
}

export async function audioSetSpeed(speed: number): Promise<void> {
	await invokeCommand('audio_set_speed', { speed });
}

export async function audioGetState(): Promise<BackendPlaybackState | null> {
	return invokeCommand<BackendPlaybackState | null>('audio_get_state');
}

// ---------------------------------------------------------------------------
// Queue management — backend-owned for headless playback
// ---------------------------------------------------------------------------

export interface QueuedItem {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
}

export async function audioPlayWithQueue(
	item: QueuedItem,
	manualQueue: QueuedItem[],
	autoQueue: QueuedItem[],
	startPositionSeconds: number
): Promise<void> {
	await invokeCommand('audio_play_with_queue', {
		item,
		manualQueue,
		autoQueue,
		startPositionSeconds
	});
}

export async function audioQueueEnqueue(item: QueuedItem): Promise<void> {
	await invokeCommand('audio_queue_enqueue', { item });
}

export async function audioQueuePlayNext(item: QueuedItem): Promise<void> {
	await invokeCommand('audio_queue_play_next', { item });
}

export async function audioQueueRemove(itemId: string): Promise<void> {
	await invokeCommand('audio_queue_remove', { itemId });
}

export async function audioQueueMoveUp(itemId: string): Promise<void> {
	await invokeCommand('audio_queue_move_up', { itemId });
}

export async function audioQueueMoveDown(itemId: string): Promise<void> {
	await invokeCommand('audio_queue_move_down', { itemId });
}

export async function audioQueueClear(): Promise<void> {
	await invokeCommand('audio_queue_clear');
}

export async function audioQueueGetState(): Promise<BackendQueueState> {
	return invokeCommand<BackendQueueState>('audio_queue_get_state');
}

export async function audioQueueSet(items: QueuedItem[]): Promise<void> {
	await invokeCommand('audio_queue_set', { items });
}

// ---------------------------------------------------------------------------
// Cover art palette extraction
// ---------------------------------------------------------------------------

export async function extractCoverPalette(imageUrl: string): Promise<string[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	if (!imageUrl.trim()) {
		return [];
	}

	return invokeCommand<string[]>('extract_cover_palette', { imageUrl });
}
