import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import type { RouteSelectionState, SidebarSection } from '$lib/state';

export type AppListSection = 'all' | 'unread' | 'media' | 'favorites';
export type ReaderRouteMode = 'feed' | 'reader';

type ListRouteState = {
	itemId: string | null;
	readerPaneMode: ReaderRouteMode;
	search: string;
};

export type AppRoute =
	| { kind: 'home' }
	| { kind: 'settings' }
	| ({ kind: 'section'; section: AppListSection } & ListRouteState)
	| ({ kind: 'feed'; feedId: string } & ListRouteState)
	| ({ kind: 'station'; stationId: string } & ListRouteState)
	| { kind: 'inspect'; feedId: string };

type NavigationOptions = {
	replaceState?: boolean;
	noScroll?: boolean;
	keepFocus?: boolean;
};

type SearchHashSuffix = '' | `?${string}` | `#${string}` | `?${string}#${string}`;

function getListRouteState(url: URL): ListRouteState {
	const itemId = url.searchParams.get('item');
	const readerPaneMode = url.searchParams.get('view') === 'reader' ? 'reader' : 'feed';

	return {
		itemId: itemId && itemId.trim().length > 0 ? itemId : null,
		readerPaneMode,
		search: url.searchParams.get('search') ?? ''
	};
}

function withPreservedParams(target: URL, source?: URL): URL {
	const current = source ?? getCurrentUrl();
	const windowKind = current?.searchParams.get('window');

	if (windowKind) {
		target.searchParams.set('window', windowKind);
	}

	return target;
}

function getCurrentUrl(): URL | null {
	if (typeof window === 'undefined') {
		return null;
	}

	return new URL(window.location.href);
}

function encodeSegment(value: string): string {
	return encodeURIComponent(value);
}

function getRoutePath(route: AppRoute): string {
	switch (route.kind) {
		case 'home':
			return '/';
		case 'settings':
			return '/settings';
		case 'inspect':
			return `/feeds/${encodeSegment(route.feedId)}/inspect`;
		case 'feed':
			return `/feeds/${encodeSegment(route.feedId)}`;
		case 'station':
			return `/stations/${encodeSegment(route.stationId)}`;
		case 'section':
			return `/${route.section}`;
	}
}

function getSearchHashSuffix(url: URL): SearchHashSuffix {
	if (url.search && url.hash) {
		return `?${url.searchParams.toString()}#${url.hash.slice(1)}`;
	}

	if (url.search) {
		return `?${url.searchParams.toString()}`;
	}

	if (url.hash) {
		return `#${url.hash.slice(1)}`;
	}

	return '';
}

function withRootSuffix(
	suffix: SearchHashSuffix
): '/' | `/?${string}` | `/#${string}` | `/?${string}#${string}` {
	return suffix ? `/${suffix}` : '/';
}

function withStaticSuffix<Path extends '/settings' | '/all' | '/unread' | '/media' | '/favorites'>(
	path: Path,
	suffix: SearchHashSuffix
): Path | `${Path}?${string}` | `${Path}#${string}` | `${Path}?${string}#${string}` {
	return suffix ? `${path}${suffix}` : path;
}

function withFeedSuffix(
	feedId: string,
	suffix: SearchHashSuffix
):
	| `/feeds/${string}`
	| `/feeds/${string}?${string}`
	| `/feeds/${string}#${string}`
	| `/feeds/${string}?${string}#${string}` {
	return `/feeds/${encodeSegment(feedId)}${suffix}`;
}

function withFeedInspectSuffix(
	feedId: string,
	suffix: SearchHashSuffix
):
	| `/feeds/${string}/inspect`
	| `/feeds/${string}/inspect?${string}`
	| `/feeds/${string}/inspect#${string}`
	| `/feeds/${string}/inspect?${string}#${string}` {
	return `/feeds/${encodeSegment(feedId)}/inspect${suffix}`;
}

function withStationSuffix(
	stationId: string,
	suffix: SearchHashSuffix
):
	| `/stations/${string}`
	| `/stations/${string}?${string}`
	| `/stations/${string}#${string}`
	| `/stations/${string}?${string}#${string}` {
	return `/stations/${encodeSegment(stationId)}${suffix}`;
}

export function parseAppUrl(url: URL): AppRoute {
	const segments = url.pathname.split('/').filter((segment) => segment.length > 0);

	if (segments.length === 0 || (segments.length === 1 && segments[0] === 'home')) {
		return { kind: 'home' };
	}

	if (segments.length === 1 && segments[0] === 'settings') {
		return { kind: 'settings' };
	}

	if (segments.length === 1 && isAppListSection(segments[0])) {
		return {
			kind: 'section',
			section: segments[0],
			...getListRouteState(url)
		};
	}

	if (segments[0] === 'feeds' && segments[1]) {
		if (segments.length === 3 && segments[2] === 'inspect') {
			return { kind: 'inspect', feedId: decodeURIComponent(segments[1]) };
		}

		if (segments.length === 2) {
			return {
				kind: 'feed',
				feedId: decodeURIComponent(segments[1]),
				...getListRouteState(url)
			};
		}
	}

	if (segments[0] === 'stations' && segments[1] && segments.length === 2) {
		return {
			kind: 'station',
			stationId: decodeURIComponent(segments[1]),
			...getListRouteState(url)
		};
	}

	return { kind: 'home' };
}

export function isAppListSection(value: string): value is AppListSection {
	return value === 'all' || value === 'unread' || value === 'media' || value === 'favorites';
}

export function isListRoute(
	route: AppRoute
): route is Extract<AppRoute, { itemId: string | null }> {
	return route.kind === 'section' || route.kind === 'feed' || route.kind === 'station';
}

export function toRouteSelection(route: AppRoute): RouteSelectionState {
	if (route.kind === 'feed' || route.kind === 'inspect') {
		return {
			selectedFeedId: route.feedId,
			selectedStationId: null,
			selectedSection: null,
			selectedItemId: route.kind === 'feed' ? route.itemId : null,
			searchTerm: route.kind === 'feed' ? route.search : ''
		};
	}

	if (route.kind === 'station') {
		return {
			selectedFeedId: null,
			selectedStationId: route.stationId,
			selectedSection: null,
			selectedItemId: route.itemId,
			searchTerm: route.search
		};
	}

	if (route.kind === 'section') {
		return {
			selectedFeedId: null,
			selectedStationId: null,
			selectedSection: route.section,
			selectedItemId: route.itemId,
			searchTerm: route.search
		};
	}

	return {
		selectedFeedId: null,
		selectedStationId: null,
		selectedSection: route.kind === 'settings' ? 'settings' : 'home',
		selectedItemId: null,
		searchTerm: ''
	};
}

export function toSidebarSection(route: AppRoute): SidebarSection {
	if (route.kind === 'home' || route.kind === 'settings') {
		return route.kind;
	}

	if (route.kind === 'section') {
		return route.section;
	}

	return null;
}

export function buildAppUrl(route: AppRoute, source?: URL): URL {
	const url = new URL(source?.href ?? 'http://jrss.local/');

	url.pathname = getRoutePath(route);
	url.search = '';

	if (isListRoute(route)) {
		if (route.itemId) {
			url.searchParams.set('item', route.itemId);
		}

		if (route.search.trim().length > 0) {
			url.searchParams.set('search', route.search);
		}

		if (route.readerPaneMode === 'reader') {
			url.searchParams.set('view', 'reader');
		}
	}

	return withPreservedParams(url, source);
}

export async function navigateToAppRoute(
	route: AppRoute,
	options: NavigationOptions = {}
): Promise<void> {
	const currentUrl = getCurrentUrl();
	const targetUrl = buildAppUrl(route, currentUrl ?? undefined);
	const suffix = getSearchHashSuffix(targetUrl);
	const gotoOptions = {
		replaceState: options.replaceState,
		noScroll: options.noScroll ?? true,
		keepFocus: options.keepFocus ?? true
	};

	if (route.kind === 'home') {
		await goto(resolve(withRootSuffix(suffix)), gotoOptions);
		return;
	}

	if (route.kind === 'settings') {
		await goto(resolve(withStaticSuffix('/settings', suffix)), gotoOptions);
		return;
	}

	if (route.kind === 'inspect') {
		await goto(resolve(withFeedInspectSuffix(route.feedId, suffix)), gotoOptions);
		return;
	}

	if (route.kind === 'feed') {
		await goto(resolve(withFeedSuffix(route.feedId, suffix)), gotoOptions);
		return;
	}

	if (route.kind === 'station') {
		await goto(resolve(withStationSuffix(route.stationId, suffix)), gotoOptions);
		return;
	}

	if (route.section === 'all') {
		await goto(resolve(withStaticSuffix('/all', suffix)), gotoOptions);
		return;
	}

	if (route.section === 'unread') {
		await goto(resolve(withStaticSuffix('/unread', suffix)), gotoOptions);
		return;
	}

	if (route.section === 'favorites') {
		await goto(resolve(withStaticSuffix('/favorites', suffix)), gotoOptions);
		return;
	}

	await goto(resolve(withStaticSuffix('/media', suffix)), gotoOptions);
}

export async function navigateToHome(options?: NavigationOptions): Promise<void> {
	await navigateToAppRoute({ kind: 'home' }, options);
}

export async function navigateToSettings(options?: NavigationOptions): Promise<void> {
	await navigateToAppRoute({ kind: 'settings' }, options);
}

export async function navigateToSection(
	section: AppListSection,
	options?: NavigationOptions
): Promise<void> {
	await navigateToAppRoute(
		{
			kind: 'section',
			section,
			itemId: null,
			readerPaneMode: 'feed',
			search: ''
		},
		options
	);
}

export async function navigateToFeed(feedId: string, options?: NavigationOptions): Promise<void> {
	await navigateToAppRoute(
		{
			kind: 'feed',
			feedId,
			itemId: null,
			readerPaneMode: 'feed',
			search: ''
		},
		options
	);
}

export async function navigateToStation(
	stationId: string,
	options?: NavigationOptions
): Promise<void> {
	await navigateToAppRoute(
		{
			kind: 'station',
			stationId,
			itemId: null,
			readerPaneMode: 'feed',
			search: ''
		},
		options
	);
}

export async function navigateToFeedInspector(
	feedId: string,
	options?: NavigationOptions
): Promise<void> {
	await navigateToAppRoute({ kind: 'inspect', feedId }, options);
}

async function replaceCurrentRoute(
	mutate: (route: Extract<AppRoute, { itemId: string | null }>) => AppRoute
): Promise<void> {
	const currentUrl = getCurrentUrl();
	if (!currentUrl) {
		return;
	}

	const route = parseAppUrl(currentUrl);
	if (!isListRoute(route)) {
		return;
	}

	await navigateToAppRoute(mutate(route), {
		replaceState: true,
		noScroll: true,
		keepFocus: true
	});
}

export async function replaceCurrentSearchTerm(search: string): Promise<void> {
	await replaceCurrentRoute((route) => ({
		...route,
		search,
		itemId: null,
		readerPaneMode: 'feed'
	}));
}

export async function replaceCurrentSelectedItem(
	itemId: string | null,
	readerPaneMode: ReaderRouteMode = 'feed'
): Promise<void> {
	await replaceCurrentRoute((route) => ({ ...route, itemId, readerPaneMode }));
}

export async function replaceCurrentReaderPaneMode(readerPaneMode: ReaderRouteMode): Promise<void> {
	await replaceCurrentRoute((route) => ({ ...route, readerPaneMode }));
}

export async function navigateToFeedItem(feedId: string, itemId: string): Promise<void> {
	await navigateToAppRoute({
		kind: 'feed',
		feedId,
		itemId,
		readerPaneMode: 'feed',
		search: ''
	});
}

export async function navigateToStationItem(stationId: string, itemId: string): Promise<void> {
	await navigateToAppRoute({
		kind: 'station',
		stationId,
		itemId,
		readerPaneMode: 'feed',
		search: ''
	});
}

export async function navigateToCurrentOrSectionFallback(
	fallback: AppListSection = 'all'
): Promise<void> {
	const currentUrl = getCurrentUrl();
	if (!currentUrl) {
		return;
	}

	const route = parseAppUrl(currentUrl);
	if (isListRoute(route)) {
		await navigateToAppRoute(route);
		return;
	}

	await navigateToSection(fallback);
}
