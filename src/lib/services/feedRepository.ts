import { browser } from '$app/environment';
import type { Feed, FeedItem, MediaEnclosure } from '$lib/types/rss';

export interface FeedDatabase {
	feeds: Feed[];
	items: FeedItem[];
}

const STORAGE_KEY = 'jrss.feed-db.v1';
const LEGACY_MOCK_FEED_IDS = new Set(['feed-briefing', 'feed-podcast']);
const emptyDatabase: FeedDatabase = {
	feeds: [],
	items: []
};

let memoryStore = cloneDatabase(emptyDatabase);

function cloneDatabase(database: FeedDatabase): FeedDatabase {
	return structuredClone(database);
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null;
}

function isString(value: unknown): value is string {
	return typeof value === 'string';
}

function isBoolean(value: unknown): value is boolean {
	return typeof value === 'boolean';
}

function isNumber(value: unknown): value is number {
	return typeof value === 'number' && Number.isFinite(value);
}

function isMediaEnclosure(value: unknown): value is MediaEnclosure {
	if (!isRecord(value)) {
		return false;
	}

	const sizeBytes = value['sizeBytes'];
	const durationSeconds = value['durationSeconds'];

	return (
		isString(value['url']) &&
		isString(value['mimeType']) &&
		(sizeBytes === undefined || isNumber(sizeBytes)) &&
		(durationSeconds === undefined || isNumber(durationSeconds))
	);
}

function isFeed(value: unknown): value is Feed {
	if (!isRecord(value)) {
		return false;
	}

	const kind = value['kind'];
	const siteUrl = value['siteUrl'];
	const lastFetchedAt = value['lastFetchedAt'];

	return (
		isString(value['id']) &&
		isString(value['title']) &&
		isString(value['url']) &&
		isString(value['description']) &&
		(kind === 'rss' || kind === 'podcast') &&
		(siteUrl === undefined || isString(siteUrl)) &&
		isString(value['createdAt']) &&
		(lastFetchedAt === undefined || isString(lastFetchedAt))
	);
}

function isFeedItem(value: unknown): value is FeedItem {
	if (!isRecord(value)) {
		return false;
	}

	const mediaEnclosure = value['mediaEnclosure'];

	return (
		isString(value['id']) &&
		isString(value['feedId']) &&
		isString(value['title']) &&
		isString(value['url']) &&
		isString(value['summary']) &&
		isString(value['publishedAt']) &&
		isBoolean(value['read']) &&
		isBoolean(value['saved']) &&
		isNumber(value['playbackPositionSeconds']) &&
		(mediaEnclosure === undefined || isMediaEnclosure(mediaEnclosure))
	);
}

function parseDatabase(value: unknown): FeedDatabase | null {
	if (!isRecord(value)) {
		return null;
	}

	const feeds = value['feeds'];
	const items = value['items'];

	if (!Array.isArray(feeds) || !Array.isArray(items)) {
		return null;
	}

	if (!feeds.every(isFeed) || !items.every(isFeedItem)) {
		return null;
	}

	return { feeds, items };
}

function stripLegacyMockData(database: FeedDatabase): FeedDatabase {
	if (database.feeds.length === 0) {
		return cloneDatabase(database);
	}

	const realFeeds = database.feeds.filter((feed) => !LEGACY_MOCK_FEED_IDS.has(feed.id));

	if (realFeeds.length === 0) {
		return cloneDatabase(emptyDatabase);
	}

	if (realFeeds.length === database.feeds.length) {
		return cloneDatabase(database);
	}

	const realFeedIds = new Set(realFeeds.map((feed) => feed.id));

	return {
		feeds: realFeeds,
		items: database.items.filter((item) => realFeedIds.has(item.feedId))
	};
}

export function loadFeedDatabase(): FeedDatabase {
	if (!browser) {
		return cloneDatabase(memoryStore);
	}

	const rawValue = localStorage.getItem(STORAGE_KEY);

	if (!rawValue) {
		saveFeedDatabase(emptyDatabase);
		return cloneDatabase(emptyDatabase);
	}

	try {
		const parsedValue: unknown = JSON.parse(rawValue);
		const parsedDatabase = parseDatabase(parsedValue);

		if (parsedDatabase) {
			const sanitizedDatabase = stripLegacyMockData(parsedDatabase);
			memoryStore = cloneDatabase(sanitizedDatabase);
			return cloneDatabase(sanitizedDatabase);
		}
	} catch {
		// Ignore malformed local storage and fall back to an empty database.
	}

	saveFeedDatabase(emptyDatabase);
	return cloneDatabase(emptyDatabase);
}

export function saveFeedDatabase(database: FeedDatabase): void {
	memoryStore = cloneDatabase(database);

	if (browser) {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(database));
	}
}

export function sortFeeds(feeds: Feed[]): Feed[] {
	return [...feeds].sort((left, right) => left.title.localeCompare(right.title));
}

export function sortItems(items: FeedItem[]): FeedItem[] {
	return [...items].sort(
		(left, right) => new Date(right.publishedAt).getTime() - new Date(left.publishedAt).getTime()
	);
}
