export type ItemType = 'article' | 'media';
export type ReaderStatus = 'unfetched' | 'ready' | 'failed';
export type ItemListSection = 'all' | 'unread' | 'media';
export type ItemSortOrder = 'newest_first' | 'oldest_first';

export interface MediaEnclosure {
	url: string;
	mimeType: string;
	sizeBytes?: number;
	durationSeconds?: number;
}

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

export interface ArticleListItem extends FeedListItemBase {
	readonly itemType: 'article';
}

export interface MediaListItem extends FeedListItemBase {
	readonly itemType: 'media';
	mediaEnclosure: MediaEnclosure;
}

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

export type FeedItem = ArticleItem | MediaItem;

export function isMediaItem<T extends FeedListItem>(item: T): item is T & MediaListItem {
	return item.itemType === 'media';
}

export function isFeed(item: import('./feed').Feed | FeedItem): item is import('./feed').Feed {
	return !('feedId' in item);
}

export interface RawFeedListItem extends FeedListItemBase {
	mediaEnclosure?: MediaEnclosure;
}

export type RawFeedItem = RawFeedListItem & FeedItemDetails;

export function mapRawFeedListItem(raw: RawFeedListItem): FeedListItem {
	if (raw.mediaEnclosure) {
		return { ...raw, itemType: 'media', mediaEnclosure: raw.mediaEnclosure };
	}
	return { ...raw, itemType: 'article' };
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

export function mapRawFeedItem(raw: RawFeedItem): FeedItem {
	if (raw.mediaEnclosure) {
		return { ...raw, itemType: 'media', mediaEnclosure: raw.mediaEnclosure };
	}
	return { ...raw, itemType: 'article' };
}
