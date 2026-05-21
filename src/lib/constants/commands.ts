import type { CommandCategory, StaticPaletteCommandDefinition } from '$lib/types/command';

export const STATIC_PALETTE_COMMANDS = {
	navigation: [
		{
			id: 'nav-home',
			title: 'Go to Home',
			icon: 'heroicons:home',
			category: 'Navigation',
			keywords: ['home', 'dashboard']
		},
		{
			id: 'nav-all',
			title: 'Go to All Feeds',
			icon: 'heroicons:squares-2x2',
			category: 'Navigation',
			keywords: ['all', 'feeds', 'everything']
		},
		{
			id: 'nav-unread',
			title: 'Go to Unread',
			icon: 'heroicons:inbox',
			category: 'Navigation',
			keywords: ['unread', 'inbox', 'new']
		},
		{
			id: 'nav-media',
			title: 'Go to Media',
			icon: 'heroicons:microphone',
			category: 'Navigation',
			keywords: ['media', 'podcasts', 'audio']
		},
		{
			id: 'nav-settings',
			title: 'Go to Settings',
			icon: 'heroicons:cog-6-tooth',
			category: 'Navigation',
			keywords: ['settings', 'preferences', 'config']
		}
	] satisfies StaticPaletteCommandDefinition[],

	actions: [
		{
			id: 'add-feed',
			title: 'Add Feed',
			icon: 'lucide:plus',
			category: 'Actions',
			keywords: ['add', 'feed', 'subscribe', 'url']
		},
		{
			id: 'add-station',
			title: 'Add Station',
			icon: 'lucide:radio',
			category: 'Actions',
			keywords: ['add', 'station', 'playlist', 'create']
		}
	] satisfies StaticPaletteCommandDefinition[],

	view: [
		{
			id: 'toggle-cover',
			title: 'Toggle Cover View',
			icon: 'heroicons:arrows-pointing-out',
			category: 'View',
			keywords: ['cover', 'fullscreen', 'artwork']
		},
		{
			id: 'toggle-mini',
			title: 'Toggle Mini Player',
			icon: 'heroicons:window',
			category: 'View',
			keywords: ['mini', 'player', 'popout', 'pip']
		},
		{
			id: 'toggle-sidebar',
			title: 'Toggle Sidebar',
			icon: 'lucide:panel-left',
			category: 'View',
			keywords: ['sidebar', 'toggle', 'collapse', 'expand']
		}
	] satisfies StaticPaletteCommandDefinition[]
} as const;

export const PLAYBACK_COMMAND_DEFINITIONS = [
	{
		id: 'play-pause',
		title: 'Pause',
		icon: 'lucide:pause',
		category: 'Playback',
		keywords: ['play', 'pause', 'toggle']
	},
	{
		id: 'next-episode',
		title: 'Next Episode',
		icon: 'lucide:skip-forward',
		category: 'Playback',
		keywords: ['next', 'skip', 'forward', 'episode']
	},
	{
		id: 'prev-episode',
		title: 'Previous Episode',
		icon: 'lucide:skip-back',
		category: 'Playback',
		keywords: ['previous', 'prev', 'back', 'episode']
	},
	{
		id: 'skip-forward',
		title: 'Skip Forward 15s',
		icon: 'lucide:forward',
		category: 'Playback',
		keywords: ['skip', 'forward', 'jump', 'seek', 'ahead']
	},
	{
		id: 'skip-back',
		title: 'Skip Back 15s',
		icon: 'lucide:rewind',
		category: 'Playback',
		keywords: ['skip', 'back', 'rewind', 'jump', 'seek']
	}
] satisfies StaticPaletteCommandDefinition[];

export const PALETTE_CATEGORY_ORDER: Record<CommandCategory, number> = {
	Navigation: 0,
	Actions: 1,
	Playback: 2,
	View: 3,
	Feeds: 4,
	Stations: 5
};
