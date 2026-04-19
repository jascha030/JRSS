export type FeedKind = 'article' | 'media';
export type ItemType = 'article' | 'media';
export type ReaderStatus = 'unfetched' | 'ready' | 'failed';
export type ItemListSection = 'all' | 'unread' | 'media';
export type ItemSortOrder = 'newest_first' | 'oldest_first';

export interface Feed {
	id: string;
	title: string;
	url: string;
	description: string;
	kind: FeedKind;
	siteUrl?: string;
	imageUrl?: string;
	createdAt: string;
	lastFetchedAt?: string;
	sortOrder?: ItemSortOrder;
}

export interface MediaEnclosure {
	url: string;
	mimeType: string;
	sizeBytes?: number;
	durationSeconds?: number;
}

// ---------------------------------------------------------------------------
// Feed item base — shared fields across all item types
// ---------------------------------------------------------------------------

interface FeedListItemBase {
	id: string;
	feedId: string;
	title: string;
	url: string;
	summary: string;
	previewText: string;
	readerStatus: ReaderStatus;
	readerTitle?: string;
	readerByline?: string;
	readerExcerpt?: string;
	readerFetchedAt?: string;
	publishedAt: string;
	read: boolean;
	playbackPositionSeconds: number;
}

// ---------------------------------------------------------------------------
// Discriminated item unions — itemType narrows the shape
// ---------------------------------------------------------------------------

export interface ArticleListItem extends FeedListItemBase {
	readonly itemType: 'article';
}

export interface MediaListItem extends FeedListItemBase {
	readonly itemType: 'media';
	/** Always present on media items. */
	mediaEnclosure: MediaEnclosure;
}

/** Discriminated union of list-level items. Narrow via `item.itemType`. */
export type FeedListItem = ArticleListItem | MediaListItem;

export interface FeedItemDetails {
	id: string;
	summaryText?: string;
	summaryHtml?: string;
	contentText?: string;
	contentHtml?: string;
	readerContentHtml?: string;
	readerContentText?: string;
}

export type ArticleItem = ArticleListItem & FeedItemDetails;
export type MediaItem = MediaListItem & FeedItemDetails;

/** Discriminated union of full items. Narrow via `item.itemType`. */
export type FeedItem = ArticleItem | MediaItem;

/** Type guard: narrows a media item from the discriminated union. */
export function isMediaItem<T extends FeedListItem>(item: T): item is T & MediaListItem {
	return item.itemType === 'media';
}

/** Type guard: narrows a `Feed | FeedItem` union to `Feed`. */
export function isFeed(item: Feed | FeedItem): item is Feed {
	return !('feedId' in item);
}

// ---------------------------------------------------------------------------
// Raw IPC shapes — flat types matching the Rust serialization format.
// Mapped into discriminated unions at the service boundary.
// ---------------------------------------------------------------------------

export interface RawFeedListItem extends FeedListItemBase {
	mediaEnclosure?: MediaEnclosure;
}

export type RawFeedItem = RawFeedListItem & FeedItemDetails;

/** Map a flat Rust-serialized item into the discriminated union. */
export function mapRawFeedListItem(raw: RawFeedListItem): FeedListItem {
	if (raw.mediaEnclosure) {
		return { ...raw, itemType: 'media', mediaEnclosure: raw.mediaEnclosure };
	}
	return { ...raw, itemType: 'article' };
}

/** Map a flat Rust-serialized full item into the discriminated union. */
export function mapRawFeedItem(raw: RawFeedItem): FeedItem {
	if (raw.mediaEnclosure) {
		return { ...raw, itemType: 'media', mediaEnclosure: raw.mediaEnclosure };
	}
	return { ...raw, itemType: 'article' };
}

// ---------------------------------------------------------------------------
// Other shared types
// ---------------------------------------------------------------------------

export interface PlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
	volume: number;
}

export interface ItemPageQuery {
	feedId?: string;
	section: ItemListSection;
	offset: number;
	limit: number;
	search?: string;
	sortOrder?: ItemSortOrder;
}

export interface ItemPage<T> {
	items: T[];
	totalCount: number;
}

/**
 * Persisted playback session — matches the Rust PlaybackSessionRecord shape.
 * Stored as JSON in SQLite; restored on app launch.
 */
export interface PlaybackSession {
	currentItemId?: string;
	positionSeconds: number;
	durationSeconds: number;
	manualQueue: string[];
	autoQueue: string[];
}

// ---------------------------------------------------------------------------
// Backend audio engine — event payloads from Rust
// ---------------------------------------------------------------------------

/** Mirrors Rust `PlaybackStateEvent` emitted by the audio thread. */
export interface BackendPlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
	volume: number;
}

/** Mirrors Rust `PlaybackEndedEvent`. */
export interface BackendPlaybackEndedEvent {
	itemId: string;
}

export interface BackendQueuedItem {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
}

export interface BackendQueueState {
	manual: BackendQueuedItem[];
	auto: BackendQueuedItem[];
	current: BackendQueuedItem | null;
}

// ---------------------------------------------------------------------------
// Stations — podcast playlist grouping
// ---------------------------------------------------------------------------

export type StationEpisodeFilter = 'all' | 'unplayed';

export interface Station {
	id: string;
	name: string;
	episodeFilter: StationEpisodeFilter;
	sortOrder: ItemSortOrder;
	sortOrderPosition: number;
	createdAt: string;
	feedIds: string[];
}

export interface CreateStationInput {
	name: string;
	episodeFilter: StationEpisodeFilter;
	sortOrder: ItemSortOrder;
	feedIds: string[];
}

export interface UpdateStationInput {
	id: string;
	name?: string;
	episodeFilter?: StationEpisodeFilter;
	sortOrder?: ItemSortOrder;
	feedIds?: string[];
}
