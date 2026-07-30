import type { ItemSortOrder } from './item';

export type StationEpisodeFilter = 'all' | 'unplayed';

export type StationGradient =
	'emerald' | 'violet' | 'rose' | 'amber' | 'cyan' | 'fuchsia' | 'slate' | 'orange';

export interface Station {
	id: string;
	name: string;
	episodeFilter: StationEpisodeFilter;
	sortOrder: ItemSortOrder;
	sortOrderPosition: number;
	createdAt: string;
	feedIds: string[];
	gradient: StationGradient;
}

export interface CreateStationInput {
	name: string;
	episodeFilter: StationEpisodeFilter;
	sortOrder: ItemSortOrder;
	feedIds: string[];
	gradient: StationGradient;
}

export interface UpdateStationInput {
	id: string;
	name?: string;
	episodeFilter?: StationEpisodeFilter;
	sortOrder?: ItemSortOrder;
	feedIds?: string[];
	gradient?: StationGradient;
}
