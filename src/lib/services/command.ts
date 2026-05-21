import {
	selectSection,
	selectFeed,
	selectStation,
	closeInspector,
	playbackState,
	requestTogglePlayback,
	requestNextEpisode,
	requestPreviousEpisode,
	requestSeekTo
} from '$lib/state';
import type { Feed } from '$lib/types/feed';
import type { Station } from '$lib/types/station';
import type { CommandCategory, CommandPaletteItem } from '$lib/types/command';

type BuildPaletteItemsOptions = {
	feeds: Feed[];
	stations: Station[];
	isPlaying: boolean;
	term: string;
	onClose: () => void;
	onToggleCover: () => void;
	onToggleMiniPlayer: () => void;
	onToggleSidebar: () => void;
	onAddFeed: () => void;
	onAddStation: () => void;
};

const categoryOrder: Record<CommandCategory, number> = {
	Navigation: 0,
	Actions: 1,
	Playback: 2,
	View: 3,
	Feeds: 4,
	Stations: 5
};

function normalizeTerm(value: string): string {
	return value.trim().toLowerCase();
}

function withClose(action: () => void, onClose: () => void): () => void {
	return () => {
		action();
		onClose();
	};
}

function matchesItem(term: string, item: CommandPaletteItem): boolean {
	if (!term) return true;

	return (
		item.title.toLowerCase().includes(term) ||
		item.keywords.some((keyword) => keyword.toLowerCase().includes(term))
	);
}

function scoreTextMatch(text: string, term: string): number {
	const lower = text.toLowerCase();

	if (lower.startsWith(term)) return 0;
	if (lower.includes(term)) return 1;

	return -1;
}

function sortItems(items: CommandPaletteItem[]): CommandPaletteItem[] {
	return [...items].sort((a, b) => {
		const categoryDelta = categoryOrder[a.category] - categoryOrder[b.category];
		if (categoryDelta !== 0) return categoryDelta;
		return a.title.localeCompare(b.title);
	});
}

function buildBaseItems(options: Omit<BuildPaletteItemsOptions, 'feeds' | 'stations' | 'term'>) {
	const {
		isPlaying,
		onClose,
		onToggleCover,
		onToggleMiniPlayer,
		onToggleSidebar,
		onAddFeed,
		onAddStation
	} = options;

	const items: CommandPaletteItem[] = [
		{
			id: 'nav-home',
			title: 'Go to Home',
			icon: 'heroicons:home',
			category: 'Navigation',
			keywords: ['home', 'dashboard'],
			action: withClose(() => {
				closeInspector();
				selectSection('home');
			}, onClose)
		},
		{
			id: 'nav-all',
			title: 'Go to All Feeds',
			icon: 'heroicons:squares-2x2',
			category: 'Navigation',
			keywords: ['all', 'feeds', 'everything'],
			action: withClose(() => {
				closeInspector();
				selectSection('all');
			}, onClose)
		},
		{
			id: 'nav-unread',
			title: 'Go to Unread',
			icon: 'heroicons:inbox',
			category: 'Navigation',
			keywords: ['unread', 'inbox', 'new'],
			action: withClose(() => {
				closeInspector();
				selectSection('unread');
			}, onClose)
		},
		{
			id: 'nav-media',
			title: 'Go to Media',
			icon: 'heroicons:microphone',
			category: 'Navigation',
			keywords: ['media', 'podcasts', 'audio'],
			action: withClose(() => {
				closeInspector();
				selectSection('media');
			}, onClose)
		},
		{
			id: 'nav-settings',
			title: 'Go to Settings',
			icon: 'heroicons:cog-6-tooth',
			category: 'Navigation',
			keywords: ['settings', 'preferences', 'config'],
			action: withClose(() => {
				closeInspector();
				selectSection('settings');
			}, onClose)
		},
		{
			id: 'add-feed',
			title: 'Add Feed',
			icon: 'lucide:plus',
			category: 'Actions',
			keywords: ['add', 'feed', 'subscribe', 'url'],
			action: withClose(() => {
				onAddFeed();
			}, onClose)
		},
		{
			id: 'add-station',
			title: 'Add Station',
			icon: 'lucide:radio',
			category: 'Actions',
			keywords: ['add', 'station', 'playlist', 'create'],
			action: withClose(() => {
				onAddStation();
			}, onClose)
		}
	];

	if (isPlaying) {
		items.push(
			{
				id: 'play-pause',
				title: 'Pause',
				icon: 'lucide:pause',
				category: 'Playback',
				keywords: ['play', 'pause', 'toggle'],
				action: withClose(() => {
					requestTogglePlayback();
				}, onClose)
			},
			{
				id: 'next-episode',
				title: 'Next Episode',
				icon: 'lucide:skip-forward',
				category: 'Playback',
				keywords: ['next', 'skip', 'forward', 'episode'],
				action: withClose(() => {
					requestNextEpisode();
				}, onClose)
			},
			{
				id: 'prev-episode',
				title: 'Previous Episode',
				icon: 'lucide:skip-back',
				category: 'Playback',
				keywords: ['previous', 'prev', 'back', 'episode'],
				action: withClose(() => {
					requestPreviousEpisode();
				}, onClose)
			},
			{
				id: 'skip-forward',
				title: 'Skip Forward 15s',
				icon: 'lucide:forward',
				category: 'Playback',
				keywords: ['skip', 'forward', 'jump', 'seek', 'ahead'],
				action: withClose(() => {
					const pos = playbackState.currentPlaybackState?.positionSeconds ?? 0;
					requestSeekTo(pos + 15);
				}, onClose)
			},
			{
				id: 'skip-back',
				title: 'Skip Back 15s',
				icon: 'lucide:rewind',
				category: 'Playback',
				keywords: ['skip', 'back', 'rewind', 'jump', 'seek'],
				action: withClose(() => {
					const pos = playbackState.currentPlaybackState?.positionSeconds ?? 0;
					requestSeekTo(Math.max(0, pos - 15));
				}, onClose)
			}
		);
	}

	items.push(
		{
			id: 'toggle-cover',
			title: 'Toggle Cover View',
			icon: 'heroicons:arrows-pointing-out',
			category: 'View',
			keywords: ['cover', 'fullscreen', 'artwork'],
			action: withClose(() => {
				onToggleCover();
			}, onClose)
		},
		{
			id: 'toggle-mini',
			title: 'Toggle Mini Player',
			icon: 'heroicons:window',
			category: 'View',
			keywords: ['mini', 'player', 'popout', 'pip'],
			action: withClose(() => {
				onToggleMiniPlayer();
			}, onClose)
		},
		{
			id: 'toggle-sidebar',
			title: 'Toggle Sidebar',
			icon: 'lucide:panel-left',
			category: 'View',
			keywords: ['sidebar', 'toggle', 'collapse', 'expand'],
			action: withClose(() => {
				onToggleSidebar();
			}, onClose)
		}
	);

	return items;
}

function buildFeedItems(feeds: Feed[], term: string, onClose: () => void): CommandPaletteItem[] {
	if (!term) return [];

	return feeds
		.map((feed) => ({
			feed,
			score: scoreTextMatch(feed.title, term)
		}))
		.filter((entry) => entry.score >= 0)
		.sort((a, b) => {
			if (a.score !== b.score) return a.score - b.score;
			return a.feed.title.localeCompare(b.feed.title);
		})
		.slice(0, 5)
		.map(({ feed }) => ({
			id: `feed:${feed.id}`,
			title: feed.title,
			icon: 'lucide:rss',
			category: 'Feeds' as const,
			keywords: [feed.title],
			badge: 'Feed',
			action: withClose(() => {
				closeInspector();
				selectFeed(feed.id);
			}, onClose)
		}));
}

function buildStationItems(
	stations: Station[],
	term: string,
	onClose: () => void
): CommandPaletteItem[] {
	if (!term) return [];

	return stations
		.map((station) => ({
			station,
			score: scoreTextMatch(station.name, term)
		}))
		.filter((entry) => entry.score >= 0)
		.sort((a, b) => {
			if (a.score !== b.score) return a.score - b.score;
			return a.station.name.localeCompare(b.station.name);
		})
		.slice(0, 5)
		.map(({ station }) => ({
			id: `station:${station.id}`,
			title: station.name,
			icon: 'lucide:radio',
			category: 'Stations' as const,
			keywords: [station.name],
			badge: 'Station',
			action: withClose(() => {
				closeInspector();
				selectStation(station.id);
			}, onClose)
		}));
}

export function getCommandPaletteItems(options: BuildPaletteItemsOptions): CommandPaletteItem[] {
	const term = normalizeTerm(options.term);
	const baseItems = buildBaseItems(options);
	const matchedBaseItems = baseItems.filter((item) => matchesItem(term, item));
	const feedItems = buildFeedItems(options.feeds, term, options.onClose);
	const stationItems = buildStationItems(options.stations, term, options.onClose);

	return sortItems([...matchedBaseItems, ...feedItems, ...stationItems]);
}

export function getNextHighlightedIndex(current: number, total: number, step: 1 | -1): number {
	if (total === 0) return -1;
	return (current + step + total) % total;
}

export function clampHighlightedIndex(current: number, total: number): number {
	if (total === 0) return -1;
	if (current < 0) return 0;
	if (current >= total) return total - 1;
	return current;
}

export function shouldShowCommandCategory(items: CommandPaletteItem[], index: number): boolean {
	if (index === 0) return true;
	return items[index - 1]?.category !== items[index]?.category;
}
