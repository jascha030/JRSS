import { browser } from '$app/environment';
import type { FeedKind, MediaEnclosure } from '$lib/types/rss';

export interface ParsedFeedItem {
	externalId: string;
	title: string;
	url: string;
	summary: string;
	publishedAt: string;
	mediaEnclosure?: MediaEnclosure;
}

export interface ParsedFeedSnapshot {
	title: string;
	description: string;
	siteUrl?: string;
	kind: FeedKind;
	items: ParsedFeedItem[];
}

const FEED_ACCEPT_HEADER =
	'application/atom+xml, application/rss+xml, application/xml, text/xml;q=0.9, */*;q=0.8';

// Temporary browser-only workaround for feeds without CORS headers.
// Replace this with Tauri commands or a local backend once the app moves beyond pure browser mode.
const TEMP_CORS_PROXY_BASE_URL = 'https://api.allorigins.win/raw?url=';

function normalizeWhitespace(value: string): string {
	return value.replace(/\s+/g, ' ').trim();
}

function toPlainText(value: string): string {
	const trimmedValue = value.trim();

	if (!trimmedValue) {
		return '';
	}

	const document = new DOMParser().parseFromString(trimmedValue, 'text/html');

	return normalizeWhitespace(document.body.textContent ?? '');
}

function parseXmlDocument(xmlText: string): XMLDocument {
	const document = new DOMParser().parseFromString(xmlText, 'application/xml');
	const parseError = document.querySelector('parsererror');

	if (parseError) {
		throw new Error('The response was not valid RSS or Atom XML.');
	}

	return document;
}

function directChildren(element: Element): Element[] {
	return Array.from(element.children);
}

function directChildrenNamed(element: Element, name: string): Element[] {
	return directChildren(element).filter((child) => child.localName.toLowerCase() === name);
}

function directChild(element: Element, names: string[]): Element | null {
	return (
		directChildren(element).find((child) => names.includes(child.localName.toLowerCase())) ?? null
	);
}

function childText(element: Element, names: string[]): string {
	return normalizeWhitespace(directChild(element, names)?.textContent ?? '');
}

function childPlainText(element: Element, names: string[]): string {
	return toPlainText(directChild(element, names)?.textContent ?? '');
}

function resolveUrl(url: string, baseUrl: string): string {
	return new URL(url, baseUrl).toString();
}

function tryResolveUrl(url: string | null, baseUrl: string): string | null {
	if (!url) {
		return null;
	}

	const trimmedUrl = url.trim();

	if (!trimmedUrl) {
		return null;
	}

	try {
		return resolveUrl(trimmedUrl, baseUrl);
	} catch {
		return null;
	}
}

function parseIsoDate(value: string): string {
	const candidateValue = value.trim();

	if (!candidateValue) {
		return new Date().toISOString();
	}

	const parsedDate = new Date(candidateValue);

	return Number.isNaN(parsedDate.getTime()) ? new Date().toISOString() : parsedDate.toISOString();
}

function parseInteger(value: string | null): number | undefined {
	if (!value) {
		return undefined;
	}

	const parsedNumber = Number.parseInt(value, 10);

	return Number.isFinite(parsedNumber) ? parsedNumber : undefined;
}

function parseDurationSeconds(value: string): number | undefined {
	const trimmedValue = value.trim();

	if (!trimmedValue) {
		return undefined;
	}

	if (/^\d+$/.test(trimmedValue)) {
		const totalSeconds = Number.parseInt(trimmedValue, 10);

		return Number.isFinite(totalSeconds) ? totalSeconds : undefined;
	}

	const parts = trimmedValue.split(':');

	if (parts.length < 2 || parts.length > 3) {
		return undefined;
	}

	const parsedParts = parts.map((part) => Number.parseInt(part, 10));

	if (parsedParts.some((part) => !Number.isFinite(part))) {
		return undefined;
	}

	if (parsedParts.length === 2) {
		return parsedParts[0] * 60 + parsedParts[1];
	}

	return parsedParts[0] * 3600 + parsedParts[1] * 60 + parsedParts[2];
}

function isAudioMimeType(mimeType: string | null): boolean {
	return Boolean(mimeType && mimeType.toLowerCase().startsWith('audio/'));
}

function isLikelyAudioUrl(url: string | null): boolean {
	if (!url) {
		return false;
	}

	return /\.(mp3|m4a|aac|ogg|oga|wav|flac)(\?.*)?$/i.test(url);
}

function guessMimeType(url: string): string {
	const loweredUrl = url.toLowerCase();

	if (loweredUrl.includes('.m4a')) {
		return 'audio/mp4';
	}

	if (loweredUrl.includes('.aac')) {
		return 'audio/aac';
	}

	if (loweredUrl.includes('.ogg') || loweredUrl.includes('.oga')) {
		return 'audio/ogg';
	}

	if (loweredUrl.includes('.wav')) {
		return 'audio/wav';
	}

	if (loweredUrl.includes('.flac')) {
		return 'audio/flac';
	}

	return 'audio/mpeg';
}

function buildEnclosure(
	url: string | null,
	baseUrl: string,
	mimeType: string | null,
	sizeValue: string | null,
	durationValue: string | null
): MediaEnclosure | undefined {
	const resolvedUrl = tryResolveUrl(url, baseUrl);

	if (!resolvedUrl) {
		return undefined;
	}

	if (!isAudioMimeType(mimeType) && !isLikelyAudioUrl(resolvedUrl)) {
		return undefined;
	}

	return {
		url: resolvedUrl,
		mimeType: mimeType?.trim() || guessMimeType(resolvedUrl),
		sizeBytes: parseInteger(sizeValue),
		durationSeconds: durationValue ? parseDurationSeconds(durationValue) : undefined
	};
}

function firstDurationText(element: Element): string | null {
	return (
		directChildren(element)
			.filter((child) => child.localName.toLowerCase() === 'duration')
			.map((child) => normalizeWhitespace(child.textContent ?? ''))
			.find(Boolean) ?? null
	);
}

function parseRssEnclosure(itemElement: Element, feedUrl: string): MediaEnclosure | undefined {
	const durationValue = firstDurationText(itemElement);

	for (const enclosureElement of directChildrenNamed(itemElement, 'enclosure')) {
		const enclosure = buildEnclosure(
			enclosureElement.getAttribute('url'),
			feedUrl,
			enclosureElement.getAttribute('type'),
			enclosureElement.getAttribute('length'),
			durationValue
		);

		if (enclosure) {
			return enclosure;
		}
	}

	for (const mediaElement of directChildrenNamed(itemElement, 'content')) {
		const enclosure = buildEnclosure(
			mediaElement.getAttribute('url') ?? mediaElement.getAttribute('href'),
			feedUrl,
			mediaElement.getAttribute('type') ?? mediaElement.getAttribute('medium'),
			mediaElement.getAttribute('fileSize') ?? mediaElement.getAttribute('length'),
			durationValue ?? mediaElement.getAttribute('duration')
		);

		if (enclosure) {
			return enclosure;
		}
	}

	return undefined;
}

function selectAtomLink(
	element: Element,
	feedUrl: string,
	relation: 'alternate' | 'enclosure'
): string | null {
	const links = directChildrenNamed(element, 'link');

	for (const linkElement of links) {
		const relValue = (linkElement.getAttribute('rel') ?? '').trim();
		const hrefValue = tryResolveUrl(linkElement.getAttribute('href'), feedUrl);

		if (!hrefValue) {
			continue;
		}

		if (relation === 'alternate' && (!relValue || relValue === 'alternate')) {
			return hrefValue;
		}

		if (relation === 'enclosure' && relValue === 'enclosure') {
			return hrefValue;
		}
	}

	return null;
}

function parseAtomEnclosure(entryElement: Element, feedUrl: string): MediaEnclosure | undefined {
	for (const linkElement of directChildrenNamed(entryElement, 'link')) {
		const relValue = (linkElement.getAttribute('rel') ?? '').trim();

		if (relValue !== 'enclosure') {
			continue;
		}

		const enclosure = buildEnclosure(
			linkElement.getAttribute('href'),
			feedUrl,
			linkElement.getAttribute('type'),
			linkElement.getAttribute('length'),
			firstDurationText(entryElement)
		);

		if (enclosure) {
			return enclosure;
		}
	}

	return undefined;
}

function parseRssItem(itemElement: Element, feedUrl: string): ParsedFeedItem {
	const linkUrl = tryResolveUrl(childText(itemElement, ['link']), feedUrl) ?? feedUrl;
	const guidValue = childText(itemElement, ['guid']);
	const titleValue = childText(itemElement, ['title']) || 'Untitled item';
	const publishedAt = parseIsoDate(
		childText(itemElement, ['pubdate', 'date', 'published', 'updated'])
	);
	const summaryValue =
		childPlainText(itemElement, ['description', 'encoded', 'content', 'summary']) ||
		'No summary provided.';
	const enclosure = parseRssEnclosure(itemElement, feedUrl);

	return {
		externalId: guidValue || linkUrl || `${titleValue}-${publishedAt}`,
		title: titleValue,
		url: linkUrl,
		summary: summaryValue,
		publishedAt,
		mediaEnclosure: enclosure
	};
}

function parseAtomItem(entryElement: Element, feedUrl: string): ParsedFeedItem {
	const linkUrl = selectAtomLink(entryElement, feedUrl, 'alternate') ?? feedUrl;
	const idValue = childText(entryElement, ['id']);
	const titleValue = childText(entryElement, ['title']) || 'Untitled item';
	const publishedAt = parseIsoDate(childText(entryElement, ['published', 'updated']));
	const summaryValue =
		childPlainText(entryElement, ['summary', 'content', 'subtitle']) || 'No summary provided.';
	const enclosure = parseAtomEnclosure(entryElement, feedUrl);

	return {
		externalId: idValue || linkUrl || `${titleValue}-${publishedAt}`,
		title: titleValue,
		url: linkUrl,
		summary: summaryValue,
		publishedAt,
		mediaEnclosure: enclosure
	};
}

function parseRssFeed(rootElement: Element, feedUrl: string): ParsedFeedSnapshot {
	const channelElement = directChild(rootElement, ['channel']) ?? rootElement;
	const itemElements =
		channelElement === rootElement
			? directChildrenNamed(rootElement, 'item')
			: directChildrenNamed(channelElement, 'item');
	const items = itemElements.map((itemElement) => parseRssItem(itemElement, feedUrl));
	const siteUrl = tryResolveUrl(childText(channelElement, ['link']), feedUrl) ?? undefined;

	return {
		title: childText(channelElement, ['title']) || new URL(feedUrl).hostname,
		description:
			childPlainText(channelElement, ['description', 'subtitle']) || 'No description provided.',
		siteUrl,
		kind: items.some((item) => item.mediaEnclosure) ? 'podcast' : 'rss',
		items
	};
}

function parseAtomFeed(rootElement: Element, feedUrl: string): ParsedFeedSnapshot {
	const items = directChildrenNamed(rootElement, 'entry').map((entryElement) =>
		parseAtomItem(entryElement, feedUrl)
	);
	const siteUrl = selectAtomLink(rootElement, feedUrl, 'alternate') ?? undefined;

	return {
		title: childText(rootElement, ['title']) || new URL(feedUrl).hostname,
		description: childPlainText(rootElement, ['subtitle']) || 'No description provided.',
		siteUrl,
		kind: items.some((item) => item.mediaEnclosure) ? 'podcast' : 'rss',
		items
	};
}

function parseFeedDocument(xmlText: string, feedUrl: string): ParsedFeedSnapshot {
	const document = parseXmlDocument(xmlText);
	const rootElement = document.documentElement;
	const rootName = rootElement.localName.toLowerCase();

	if (rootName === 'feed') {
		return parseAtomFeed(rootElement, feedUrl);
	}

	if (rootName === 'rss' || rootName === 'rdf') {
		return parseRssFeed(rootElement, feedUrl);
	}

	throw new Error('Unsupported feed format. Only RSS and Atom are supported.');
}

function buildFetchErrorMessage(feedUrl: string): string {
	return `Unable to fetch ${feedUrl}. Browser CORS limits may block this feed. Direct browser fetch was attempted first, then a temporary public proxy fallback.`;
}

async function fetchFeedResponse(url: string): Promise<Response> {
	return fetch(url, {
		headers: {
			accept: FEED_ACCEPT_HEADER
		}
	});
}

async function fetchFeedXml(feedUrl: string): Promise<string> {
	try {
		const response = await fetchFeedResponse(feedUrl);

		if (!response.ok) {
			throw new Error(`Feed request failed with status ${response.status}.`);
		}

		return await response.text();
	} catch {
		const proxyUrl = `${TEMP_CORS_PROXY_BASE_URL}${encodeURIComponent(feedUrl)}`;

		try {
			const response = await fetchFeedResponse(proxyUrl);

			if (!response.ok) {
				throw new Error(`Proxy request failed with status ${response.status}.`);
			}

			return await response.text();
		} catch {
			throw new Error(buildFetchErrorMessage(feedUrl));
		}
	}
}

export async function fetchAndParseFeed(feedUrl: string): Promise<ParsedFeedSnapshot> {
	if (!browser) {
		throw new Error('Feed fetching is only available in the browser for this MVP.');
	}

	const xmlText = await fetchFeedXml(feedUrl);

	return parseFeedDocument(xmlText, feedUrl);
}
