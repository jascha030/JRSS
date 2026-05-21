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

export interface StaticPaletteCommandDefinition {
	id: string;
	title: string;
	icon: string;
	category: Exclude<CommandCategory, 'Feeds' | 'Stations'>;
	keywords: string[];
}
