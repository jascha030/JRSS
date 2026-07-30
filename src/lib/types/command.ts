export type CommandCategory = 'Navigation' | 'Actions' | 'Playback' | 'View' | 'Feeds' | 'Stations';

export interface CommandPaletteItem {
	id: string;
	title: string;
	icon: string;
	category: CommandCategory;
	keywords: string[];
	badge?: string;
	action: () => void;
}
