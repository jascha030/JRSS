export type FeedKind = 'article' | 'media';

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
	sortOrder?: import('./item').ItemSortOrder;
}

export interface PodcastSearchResult {
	name: string;
	artist: string;
	feedUrl: string;
	artworkUrl?: string;
}
