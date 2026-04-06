export type FeedKind = 'rss' | 'podcast';
export type ReaderStatus = 'unfetched' | 'ready' | 'failed';

export interface Feed {
	id: string;
	title: string;
	url: string;
	description: string;
	kind: FeedKind;
	siteUrl?: string;
	createdAt: string;
	lastFetchedAt?: string;
}

export interface MediaEnclosure {
	url: string;
	mimeType: string;
	sizeBytes?: number;
	durationSeconds?: number;
}

export interface FeedListItem {
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
	mediaEnclosure?: MediaEnclosure;
}

export interface FeedItemDetails {
	id: string;
	summaryText?: string;
	summaryHtml?: string;
	contentText?: string;
	contentHtml?: string;
	readerContentHtml?: string;
	readerContentText?: string;
}

export type FeedItem = FeedListItem & FeedItemDetails;

export interface PlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
}
