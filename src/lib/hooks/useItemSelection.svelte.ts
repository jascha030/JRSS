import { SvelteMap, SvelteSet } from 'svelte/reactivity';

import type { FeedListItem } from '$lib/types/item';
import { isMediaItem } from '$lib/types/item';
import { openArticleContextMenu, openAudioContextMenu } from '$lib/utils/tauri-menu';

type Options = {
	getItemIdsByIndex: () => Record<number, string>;
	getItemsById: () => Record<string, FeedListItem>;
	onSelectItem: (itemId: string) => void;
};

export function useItemSelection({ getItemIdsByIndex, getItemsById, onSelectItem }: Options) {
	const selectedIds = new SvelteSet<string>();
	let anchorIndex = $state<number | null>(null);

	const itemIndexById = $derived.by(() => {
		const indexById = new SvelteMap<string, number>();

		for (const [index, itemId] of Object.entries(getItemIdsByIndex())) {
			indexById.set(itemId, Number(index));
		}

		return indexById;
	});

	const isMultiSelecting = $derived(selectedIds.size > 0);

	function getItemIndexById(itemId: string): number | null {
		return itemIndexById.get(itemId) ?? null;
	}

	function findItemIndex(element: HTMLElement): number | null {
		const article = element.closest('[data-item-index]');

		if (!(article instanceof HTMLElement)) {
			return null;
		}

		const index = article.dataset.itemIndex;
		return index !== undefined ? Number(index) : null;
	}

	function clearSelection() {
		selectedIds.clear();
	}

	function handleItemClick(event: MouseEvent, itemId: string): void {
		const currentTarget = event.currentTarget;

		if (!(currentTarget instanceof HTMLElement)) {
			return;
		}

		const index = findItemIndex(currentTarget);

		if (index === null) {
			return;
		}

		if (event.metaKey || event.ctrlKey) {
			event.preventDefault();

			if (selectedIds.has(itemId)) {
				selectedIds.delete(itemId);
			} else {
				selectedIds.add(itemId);
			}

			anchorIndex = index;
			return;
		}

		if (event.shiftKey && anchorIndex !== null) {
			event.preventDefault();
			const start = Math.min(anchorIndex, index);
			const end = Math.max(anchorIndex, index);
			clearSelection();

			for (let currentIndex = start; currentIndex <= end; currentIndex += 1) {
				const nextItemId = getItemIdsByIndex()[currentIndex];

				if (nextItemId) {
					selectedIds.add(nextItemId);
				}
			}

			return;
		}

		clearSelection();
		anchorIndex = index;
		onSelectItem(itemId);
	}

	function handleItemContextMenu(event: MouseEvent, item: FeedListItem): void {
		if (isMultiSelecting && selectedIds.has(item.id)) {
			if (isMediaItem(item)) {
				void openAudioContextMenu(event, item, { selectedIds, itemsById: getItemsById() });
			} else {
				void openArticleContextMenu(event, item, { selectedIds, itemsById: getItemsById() });
			}

			return;
		}

		clearSelection();

		if (isMediaItem(item)) {
			void openAudioContextMenu(event, item);
		} else {
			void openArticleContextMenu(event, item);
		}
	}

	return {
		get selectedIds() {
			return selectedIds;
		},
		get isMultiSelecting() {
			return isMultiSelecting;
		},
		getItemIndexById,
		handleItemClick,
		handleItemContextMenu
	};
}
