import {
	fetchAndParseFeed,
	type ParsedFeedItem,
	type ParsedFeedSnapshot
} from '$lib/services/feedParser';
import {
	loadFeedDatabase,
	saveFeedDatabase,
	sortFeeds,
	sortItems,
	type FeedDatabase
} from '$lib/services/feedRepository';
import type { Feed, FeedItem } from '$lib/types/rss';

function normalizeFeedUrl(url: string): string {
	return new URL(url).toString();
}

function hashString(value: string): string {
	let hash = 2166136261;

	for (let index = 0; index < value.length; index += 1) {
		hash ^= value.charCodeAt(index);
		hash = Math.imul(hash, 16777619);
	}

	return (hash >>> 0).toString(16).padStart(8, '0');
}

function buildFeedId(feedUrl: string): string {
	return `feed-${hashString(feedUrl)}`;
}

function buildItemId(feedId: string, externalId: string): string {
	return `item-${feedId}-${hashString(externalId)}`;
}

function findFeedByUrl(database: FeedDatabase, feedUrl: string): Feed | undefined {
	return database.feeds.find((feed) => {
		try {
			return normalizeFeedUrl(feed.url) === feedUrl;
		} catch {
			return feed.url === feedUrl;
		}
	});
}

function mergeFeedItems(
	feedId: string,
	existingItems: FeedItem[],
	parsedItems: ParsedFeedItem[]
): FeedItem[] {
	const mergedItems = new Map(existingItems.map((item) => [item.id, item]));
	const dedupedParsedItems = new Map(parsedItems.map((item) => [item.externalId, item]));

	for (const parsedItem of dedupedParsedItems.values()) {
		const itemId = buildItemId(feedId, parsedItem.externalId);
		const existingItem = mergedItems.get(itemId);

		mergedItems.set(itemId, {
			id: itemId,
			feedId,
			title: parsedItem.title,
			url: parsedItem.url,
			summary: parsedItem.summary,
			publishedAt: parsedItem.publishedAt,
			read: existingItem?.read ?? false,
			saved: existingItem?.saved ?? false,
			playbackPositionSeconds: existingItem?.playbackPositionSeconds ?? 0,
			mediaEnclosure: parsedItem.mediaEnclosure ?? existingItem?.mediaEnclosure
		});
	}

	return Array.from(mergedItems.values());
}

function buildFeedRecord(
	feedUrl: string,
	parsedFeed: ParsedFeedSnapshot,
	existingFeed?: Feed
): Feed {
	const fetchedAt = new Date().toISOString();

	return {
		id: existingFeed?.id ?? buildFeedId(feedUrl),
		title: parsedFeed.title || existingFeed?.title || new URL(feedUrl).hostname,
		url: feedUrl,
		description: parsedFeed.description || existingFeed?.description || 'No description provided.',
		kind: parsedFeed.kind,
		siteUrl: parsedFeed.siteUrl ?? existingFeed?.siteUrl ?? new URL(feedUrl).origin,
		createdAt: existingFeed?.createdAt ?? fetchedAt,
		lastFetchedAt: fetchedAt
	};
}

async function ingestFeed(feedUrl: string, existingFeed?: Feed): Promise<Feed> {
	const database = loadFeedDatabase();
	const parsedFeed = await fetchAndParseFeed(feedUrl);
	const nextFeed = buildFeedRecord(feedUrl, parsedFeed, existingFeed);
	const otherFeeds = database.feeds.filter((feed) => feed.id !== nextFeed.id);
	const otherItems = database.items.filter((item) => item.feedId !== nextFeed.id);
	const existingFeedItems = database.items.filter((item) => item.feedId === nextFeed.id);
	const mergedFeedItems = mergeFeedItems(nextFeed.id, existingFeedItems, parsedFeed.items);

	saveFeedDatabase({
		feeds: [...otherFeeds, nextFeed],
		items: [...otherItems, ...mergedFeedItems]
	});

	return nextFeed;
}

export async function listFeeds(): Promise<Feed[]> {
	return sortFeeds(loadFeedDatabase().feeds);
}

export async function addFeed(url: string): Promise<Feed> {
	let normalizedUrl: string;

	try {
		normalizedUrl = normalizeFeedUrl(url);
	} catch {
		throw new Error('Enter a valid feed URL.');
	}

	const database = loadFeedDatabase();
	const existingFeed = findFeedByUrl(database, normalizedUrl);

	return ingestFeed(normalizedUrl, existingFeed);
}

export async function refreshFeed(id: string): Promise<Feed> {
	const database = loadFeedDatabase();
	const existingFeed = database.feeds.find((feed) => feed.id === id);

	if (!existingFeed) {
		throw new Error('Feed not found.');
	}

	return ingestFeed(existingFeed.url, existingFeed);
}

export async function removeFeed(id: string): Promise<void> {
	const database = loadFeedDatabase();

	saveFeedDatabase({
		feeds: database.feeds.filter((feed) => feed.id !== id),
		items: database.items.filter((item) => item.feedId !== id)
	});
}

export async function listItems(feedId?: string): Promise<FeedItem[]> {
	const database = loadFeedDatabase();
	const matchingItems = feedId
		? database.items.filter((item) => item.feedId === feedId)
		: database.items;

	return sortItems(matchingItems);
}

export async function markRead(itemId: string, read: boolean): Promise<void> {
	const database = loadFeedDatabase();

	database.items = database.items.map((item) => (item.id === itemId ? { ...item, read } : item));
	saveFeedDatabase(database);
}

export async function savePlayback(itemId: string, positionSeconds: number): Promise<void> {
	const database = loadFeedDatabase();

	database.items = database.items.map((item) =>
		item.id === itemId
			? { ...item, playbackPositionSeconds: Math.max(0, Math.floor(positionSeconds)) }
			: item
	);

	saveFeedDatabase(database);
}
