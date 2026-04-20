import type { Station, CreateStationInput, UpdateStationInput } from '$lib/types/rss';
import {
	createStation as createStationService,
	deleteStation as deleteStationService,
	listStations,
	updateStation as updateStationService
} from '$lib/services/feedService';
import { invalidateAllQueries, loadInitialItemsPage } from './items.svelte';
import { selection } from './selection.svelte';

export const stationsState = $state({
	stations: [] as Station[]
});

export function resetStationsState(): void {
	stationsState.stations = [];
}

export async function loadStations(): Promise<void> {
	stationsState.stations = await listStations();

	if (
		selection.selectedStationId &&
		!stationsState.stations.some((station) => station.id === selection.selectedStationId)
	) {
		selection.selectedStationId = null;
		selection.selectedSection = 'all';
	}
}

export async function createStation(input: CreateStationInput): Promise<Station> {
	const station = await createStationService(input);
	await loadStations();
	return station;
}

export async function updateExistingStation(input: UpdateStationInput): Promise<Station> {
	const station = await updateStationService(input);
	await loadStations();

	// If the updated station is currently selected, reload its episodes
	if (selection.selectedStationId === input.id) {
		invalidateAllQueries();
		await loadInitialItemsPage();
	}

	return station;
}

export async function deleteExistingStation(stationId: string): Promise<void> {
	await deleteStationService(stationId);

	if (selection.selectedStationId === stationId) {
		selection.selectedStationId = null;
		selection.selectedSection = 'all';
	}

	await loadStations();
	invalidateAllQueries();
	await loadInitialItemsPage();
}
