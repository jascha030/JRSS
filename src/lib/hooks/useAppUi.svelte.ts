import { loadReaderView } from '$lib/state/reader.svelte';
import { toast } from 'svelte-sonner';

export const appUi = $state({
	isSidebarCollapsed: true,
	isQueueDrawerOpen: false,
	playerMode: 'default' as 'default' | 'cover',
	isFeedEditorOpen: false,
	isStationEditorOpen: false,
	isCommandPaletteOpen: false,
	editingStation: null as { id: string; name: string; feedIds: string[] } | null,
	readerPaneMode: 'feed' as 'feed' | 'reader',
	scrollToItemRequest: null as { itemId: string; seq: number } | null,
	scrollRequestSeq: 0
});

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
	appUi.isFeedEditorOpen = true;
}

export function closeFeedEditor() {
	appUi.isFeedEditorOpen = false;
}

export function openStationEditor(
	station: { id: string; name: string; feedIds: string[] } | null = null
) {
	appUi.editingStation = station;
	appUi.isStationEditorOpen = true;
}

export function closeStationEditor() {
	appUi.isStationEditorOpen = false;
	appUi.editingStation = null;
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
