export type FeedKind = 'rss' | 'podcast';

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

export interface FeedItem {
	id: string;
	feedId: string;
	title: string;
	url: string;
	summary: string;
	contentText?: string;
	contentHtml?: string;
	publishedAt: string;
	read: boolean;
	saved: boolean;
	playbackPositionSeconds: number;
	mediaEnclosure?: MediaEnclosure;
}

export interface PlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
}
