import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';
import type { FeedListItem, ItemPage } from '$lib/types/item';
import { mapRawFeedListItem, type RawFeedListItem } from '$lib/types/item';
import type { CreateStationInput, Station, UpdateStationInput } from '$lib/types/station';

interface RawStationWithFeeds {
	id: string;
	name: string;
	episodeFilter: string;
	sortOrder: string;
	sortOrderPosition: number;
	createdAt: string;
	feedIds: string[];
	gradient: string;
}

function mapRawStation(raw: RawStationWithFeeds): Station {
	return {
		id: raw.id,
		name: raw.name,
		episodeFilter: raw.episodeFilter as Station['episodeFilter'],
		sortOrder: raw.sortOrder as Station['sortOrder'],
		sortOrderPosition: raw.sortOrderPosition,
		createdAt: raw.createdAt,
		feedIds: raw.feedIds,
		gradient: raw.gradient as Station['gradient']
	};
}

export async function listStations(): Promise<Station[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	const raw = await invokeCommand<RawStationWithFeeds[]>('list_stations');
	return raw.map(mapRawStation);
}

export async function createStation(input: CreateStationInput): Promise<Station> {
	const raw = await invokeCommand<RawStationWithFeeds>('create_station', { input });
	return mapRawStation(raw);
}

export async function updateStation(input: UpdateStationInput): Promise<Station> {
	const raw = await invokeCommand<RawStationWithFeeds>('update_station', { input });
	return mapRawStation(raw);
}

export async function deleteStation(id: string): Promise<void> {
	await invokeCommand('delete_station', { id });
}

export async function queryStationEpisodes(
	stationId: string,
	offset: number,
	limit: number,
	search?: string
): Promise<ItemPage<FeedListItem>> {
	const raw = await invokeCommand<ItemPage<RawFeedListItem>>('query_station_episodes', {
		stationId,
		offset,
		limit,
		search
	});
	return { items: raw.items.map(mapRawFeedListItem), totalCount: raw.totalCount };
}
