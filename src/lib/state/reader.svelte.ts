import type { FeedItem } from '$lib/types/rss';
import { loadReaderContent } from '$lib/services/feedService';
import { mergeDetailedItem } from './items.svelte';
import { selection } from './selection.svelte';

export const readerState = $state({
	readerLoadingItemIds: [] as string[],
	/**
	 * When bumped, signals the page to open the item at `readerRequestItemId`
	 * in reader view. Consumed by the page-level $effect.
	 */
	readerRequestSeq: 0,
	readerRequestItemId: null as string | null
});

export function resetReaderState(): void {
	readerState.readerLoadingItemIds = [];
	readerState.readerRequestSeq = 0;
	readerState.readerRequestItemId = null;
}

function addUnique<T>(values: T[], value: T): void {
	if (!values.includes(value)) {
		values.push(value);
	}
}

function removeValue<T>(values: T[], value: T): void {
	const index = values.indexOf(value);
	if (index >= 0) {
		values.splice(index, 1);
	}
}

export async function loadReaderView(itemId: string): Promise<FeedItem> {
	addUnique(readerState.readerLoadingItemIds, itemId);

	try {
		const updatedItem = await loadReaderContent(itemId);
		mergeDetailedItem(updatedItem);
		return updatedItem;
	} finally {
		removeValue(readerState.readerLoadingItemIds, itemId);
	}
}

/**
 * Signal the page to select an item and open it in reader view.
 * The page-level $effect watches `readerRequestSeq` and handles the async flow.
 */
export function requestOpenInReader(itemId: string): void {
	selection.selectedItemId = itemId;
	readerState.readerRequestItemId = itemId;
	readerState.readerRequestSeq += 1;
}

/**
 * Read the current reader-request sequence number (for the page to watch).
 */
export function getReaderRequestSeq(): number {
	return readerState.readerRequestSeq;
}

/**
 * Read the item ID from the last reader-open request.
 */
export function getReaderRequestItemId(): string | null {
	return readerState.readerRequestItemId;
}
