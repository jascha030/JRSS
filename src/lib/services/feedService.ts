import { browser } from '$app/environment';
import type { Feed, FeedItem, MediaEnclosure } from '$lib/types/rss';

interface FeedDatabase {
	feeds: Feed[];
	items: FeedItem[];
}

const STORAGE_KEY = 'jrss.feed-db.v1';

const seedData: FeedDatabase = {
	feeds: [
		{
			id: 'feed-briefing',
			title: 'Morning Briefing',
			url: 'https://example.com/briefing.xml',
			description: 'A compact daily feed for headlines and links worth saving.',
			kind: 'rss',
			siteUrl: 'https://example.com',
			createdAt: '2026-03-28T08:00:00.000Z'
		},
		{
			id: 'feed-podcast',
			title: 'Local Wave FM',
			url: 'https://example.com/local-wave.xml',
			description: 'Mock podcast episodes with enclosure metadata and resume support.',
			kind: 'podcast',
			siteUrl: 'https://example.com/podcast',
			createdAt: '2026-03-28T09:00:00.000Z'
		}
	],
	items: [
		{
			id: 'item-briefing-1',
			feedId: 'feed-briefing',
			title: 'Designing a local-first reading queue',
			url: 'https://example.com/articles/local-first-reading-queue',
			summary: 'Notes on keeping unread state, saved items, and sync boundaries entirely local.',
			publishedAt: '2026-03-30T08:15:00.000Z',
			read: false,
			saved: true,
			playbackPositionSeconds: 0
		},
		{
			id: 'item-briefing-2',
			feedId: 'feed-briefing',
			title: 'Static frontends still scale for feed apps',
			url: 'https://example.com/articles/static-frontends-for-feeds',
			summary:
				'A browser-first architecture can stay clean if parsing, storage, and playback are kept behind services.',
			publishedAt: '2026-03-29T15:45:00.000Z',
			read: true,
			saved: false,
			playbackPositionSeconds: 0
		},
		{
			id: 'item-podcast-1',
			feedId: 'feed-podcast',
			title: 'Episode 12: Offline queues and why they matter',
			url: 'https://example.com/podcast/offline-queues',
			summary: 'A short mock episode covering playback persistence and service boundaries.',
			publishedAt: '2026-03-30T06:00:00.000Z',
			read: false,
			saved: false,
			playbackPositionSeconds: 0,
			mediaEnclosure: {
				url: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3',
				mimeType: 'audio/mpeg',
				durationSeconds: 372,
				sizeBytes: 8945229
			}
		},
		{
			id: 'item-podcast-2',
			feedId: 'feed-podcast',
			title: 'Episode 11: Shipping a simple reader before the parser exists',
			url: 'https://example.com/podcast/simple-reader-foundation',
			summary: 'Mock metadata for resuming playback from the last saved second.',
			publishedAt: '2026-03-27T18:30:00.000Z',
			read: true,
			saved: true,
			playbackPositionSeconds: 143,
			mediaEnclosure: {
				url: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-2.mp3',
				mimeType: 'audio/mpeg',
				durationSeconds: 415,
				sizeBytes: 10240000
			}
		}
	]
};

let memoryStore = cloneDatabase(seedData);

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

	return (
		isString(value['id']) &&
		isString(value['title']) &&
		isString(value['url']) &&
		isString(value['description']) &&
		(kind === 'rss' || kind === 'podcast') &&
		(siteUrl === undefined || isString(siteUrl)) &&
		isString(value['createdAt'])
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

function readDatabase(): FeedDatabase {
	if (!browser) {
		return cloneDatabase(memoryStore);
	}

	const rawValue = localStorage.getItem(STORAGE_KEY);

	if (!rawValue) {
		writeDatabase(seedData);
		return cloneDatabase(seedData);
	}

	try {
		const parsedValue: unknown = JSON.parse(rawValue);
		const parsedDatabase = parseDatabase(parsedValue);

		if (parsedDatabase) {
			memoryStore = cloneDatabase(parsedDatabase);
			return cloneDatabase(parsedDatabase);
		}
	} catch {
		// Ignore malformed local storage and fall back to seed data.
	}

	writeDatabase(seedData);
	return cloneDatabase(seedData);
}

function writeDatabase(database: FeedDatabase): void {
	memoryStore = cloneDatabase(database);

	if (browser) {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(database));
	}
}

function sortFeeds(feeds: Feed[]): Feed[] {
	return [...feeds].sort((left, right) => left.title.localeCompare(right.title));
}

function sortItems(items: FeedItem[]): FeedItem[] {
	return [...items].sort(
		(left, right) => new Date(right.publishedAt).getTime() - new Date(left.publishedAt).getTime()
	);
}

export async function listFeeds(): Promise<Feed[]> {
	return sortFeeds(readDatabase().feeds);
}

export async function addFeed(url: string): Promise<Feed> {
	let parsedUrl: URL;

	try {
		parsedUrl = new URL(url);
	} catch {
		throw new Error('Enter a valid feed URL.');
	}

	const database = readDatabase();
	const normalizedUrl = parsedUrl.toString();
	const nextFeed: Feed = {
		id: crypto.randomUUID(),
		title: parsedUrl.hostname.replace(/^www\./, ''),
		url: normalizedUrl,
		description: 'Feed added locally. Hook up parsing or Tauri commands later.',
		kind: normalizedUrl.toLowerCase().includes('podcast') ? 'podcast' : 'rss',
		siteUrl: parsedUrl.origin,
		createdAt: new Date().toISOString()
	};

	database.feeds = [...database.feeds, nextFeed];
	writeDatabase(database);

	return nextFeed;
}

export async function removeFeed(id: string): Promise<void> {
	const database = readDatabase();

	database.feeds = database.feeds.filter((feed) => feed.id !== id);
	database.items = database.items.filter((item) => item.feedId !== id);

	writeDatabase(database);
}

export async function listItems(feedId?: string): Promise<FeedItem[]> {
	const database = readDatabase();
	const matchingItems = feedId
		? database.items.filter((item) => item.feedId === feedId)
		: database.items;

	return sortItems(matchingItems);
}

export async function markRead(itemId: string, read: boolean): Promise<void> {
	const database = readDatabase();

	database.items = database.items.map((item) => (item.id === itemId ? { ...item, read } : item));
	writeDatabase(database);
}

export async function savePlayback(itemId: string, positionSeconds: number): Promise<void> {
	const database = readDatabase();

	database.items = database.items.map((item) =>
		item.id === itemId
			? { ...item, playbackPositionSeconds: Math.max(0, Math.floor(positionSeconds)) }
			: item
	);

	writeDatabase(database);
}
