import { toast } from 'svelte-sonner';
import {
	getActiveQueryKey,
	loadInitialItemsPage,
	loadItemDetails,
	selection,
	readerState,
	stationsState,
	createFeed,
	updateExistingStation,
	createStation
} from '$lib/state';
import {
	appUi,
	closeFeedEditor,
	closeStationEditor,
	switchToReaderView
} from '$lib/hooks/useAppUi.svelte';
import { navigateToFeed, navigateToStation } from '$lib/utils/navigation/app-router';
import type { CreateStationInput } from '$lib/types/station';
import { log } from '$lib/services/log';

export function useAppOrchestrator() {
	let lastQueryKey = $state<string | null>(null);
	let lastConsumedReaderSeq = 0;

	const activeQueryKey = $derived(getActiveQueryKey());
	const selectedItemId = $derived(selection.selectedItemId);
	const readerRequestSeq = $derived(readerState.readerRequestSeq);

	$effect(() => {
		const queryKey = activeQueryKey;

		if (!queryKey) {
			lastQueryKey = null;
			return;
		}

		if (queryKey === lastQueryKey) {
			return;
		}

		lastQueryKey = queryKey;
		void loadInitialItemsPage().catch((error: unknown) => {
			log.error(`Failed to load items: ${error}`);
		});
	});

	$effect(() => {
		if (!selectedItemId) {
			return;
		}

		void loadItemDetails(selectedItemId).catch((error: unknown) => {
			toast.error(error instanceof Error ? error.message : 'Unable to load article details.');
		});
	});

	$effect(() => {
		if (readerRequestSeq <= lastConsumedReaderSeq) {
			return;
		}

		lastConsumedReaderSeq = readerRequestSeq;
		const itemId = readerState.readerRequestItemId;

		if (itemId) {
			void switchToReaderView(itemId);
		}
	});
}

export async function addFeedFromUrl(url: string): Promise<void> {
	try {
		const createdFeed = await createFeed(url);
		closeFeedEditor();
		await navigateToFeed(createdFeed.id);
		toast.success('Feed loaded and saved locally.');
	} catch (error: unknown) {
		toast.error(error instanceof Error ? error.message : 'Unable to add that feed.');
	}
}

export async function saveStation(input: CreateStationInput): Promise<void> {
	const stationEditorId = appUi.dialog.kind === 'station-editor' ? appUi.dialog.stationId : null;
	const editingStation =
		stationEditorId !== null
			? (stationsState.stations.find((station) => station.id === stationEditorId) ?? null)
			: null;

	try {
		const station = editingStation
			? await updateExistingStation({ id: editingStation.id, ...input })
			: await createStation(input);

		closeStationEditor();
		await navigateToStation(station.id);
		toast.success(editingStation ? 'Station updated.' : 'Station created.');
	} catch (error: unknown) {
		toast.error(error instanceof Error ? error.message : 'Unable to save station.');
	}
}
