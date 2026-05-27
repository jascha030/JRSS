import { loadReaderView } from '$lib/state/reader.svelte';
import type { Station } from '$lib/types/station';
import { toast } from 'svelte-sonner';

export type PlayerMode = 'default' | 'cover';
export type ReaderPaneMode = 'feed' | 'reader';

type DialogState =
	| { kind: 'none' }
	| { kind: 'feed-editor' }
	| { kind: 'station-editor'; stationId: string | null };

type ScrollToItemRequest = {
	itemId: string;
	seq: number;
};

type AppUiState = {
	isSidebarCollapsed: boolean;
	isQueueDrawerOpen: boolean;
	playerMode: PlayerMode;
	isCommandPaletteOpen: boolean;
	dialog: DialogState;
	readerPaneMode: ReaderPaneMode;
	isReaderMaximized: boolean;
	scrollToItemRequest: ScrollToItemRequest | null;
	scrollRequestSeq: number;
};

function createAppUiState(): AppUiState {
	return {
		isSidebarCollapsed: true,
		isQueueDrawerOpen: false,
		playerMode: 'default',
		isCommandPaletteOpen: false,
		dialog: { kind: 'none' },
		readerPaneMode: 'feed',
		isReaderMaximized: false,
		scrollToItemRequest: null,
		scrollRequestSeq: 0
	};
}

export const appUi = $state(createAppUiState());

export function toggleSidebar() {
	appUi.isSidebarCollapsed = !appUi.isSidebarCollapsed;
}

export function toggleQueue() {
	appUi.isQueueDrawerOpen = !appUi.isQueueDrawerOpen;
}

export function togglePlayerMode() {
	appUi.playerMode = appUi.playerMode === 'cover' ? 'default' : 'cover';
}

export function openFeedEditor() {
	appUi.dialog = { kind: 'feed-editor' };
}

export function closeFeedEditor() {
	if (appUi.dialog.kind === 'feed-editor') {
		appUi.dialog = { kind: 'none' };
	}
}

export function openStationEditor(station: Pick<Station, 'id'> | null = null) {
	appUi.dialog = { kind: 'station-editor', stationId: station?.id ?? null };
}

export function closeStationEditor() {
	if (appUi.dialog.kind === 'station-editor') {
		appUi.dialog = { kind: 'none' };
	}
}

export function openCommandPalette() {
	appUi.isCommandPaletteOpen = true;
}

export function closeCommandPalette() {
	appUi.isCommandPaletteOpen = false;
}

export function requestScrollToItem(itemId: string) {
	appUi.scrollRequestSeq += 1;
	appUi.scrollToItemRequest = { itemId, seq: appUi.scrollRequestSeq };
}

export function clearScrollToItem() {
	appUi.scrollToItemRequest = null;
}

export function resetScrollState() {
	appUi.scrollRequestSeq = 0;
	appUi.scrollToItemRequest = null;
}

export function toggleReaderMaximized() {
	appUi.isReaderMaximized = !appUi.isReaderMaximized;
}

export async function switchToReaderView(itemId: string): Promise<void> {
	try {
		const updatedItem = await loadReaderView(itemId);
		appUi.readerPaneMode = updatedItem.readerStatus === 'ready' ? 'reader' : 'feed';
		if (updatedItem.readerStatus !== 'ready') {
			toast.warning('Reader view was unavailable for this item. Showing feed content instead.');
		}
	} catch (error: unknown) {
		appUi.readerPaneMode = 'feed';
		toast.error(
			error instanceof Error ? error.message : 'Unable to load reader view for this item.'
		);
	}
}
