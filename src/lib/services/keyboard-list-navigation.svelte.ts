import { SvelteMap } from 'svelte/reactivity';

type Options = {
	getItemCount: () => number;
	onRequestClose?: () => void;
	scrollPadding?: number;
};

type KeydownArgs<T> = {
	event: KeyboardEvent;
	items: readonly T[];
	onExecute: (item: T, index: number) => void;
};

type IndexedActionReturn = {
	update: (index: number) => void;
	destroy: () => void;
};

export type IndexedElementAction<TElement extends HTMLElement = HTMLElement> = (
	node: TElement,
	index: number
) => IndexedActionReturn;

function clampIndex(index: number, length: number) {
	if (length <= 0) return -1;
	if (index < 0) return 0;
	if (index >= length) return length - 1;
	return index;
}

function getInitialIndex(length: number) {
	return length > 0 ? 0 : -1;
}

function ensureVisible(container: HTMLElement, element: HTMLElement, padding: number) {
	const containerRect = container.getBoundingClientRect();
	const elementRect = element.getBoundingClientRect();

	const topBoundary = containerRect.top + padding;
	const bottomBoundary = containerRect.bottom - padding;

	if (elementRect.top < topBoundary) {
		container.scrollTop -= topBoundary - elementRect.top;
	} else if (elementRect.bottom > bottomBoundary) {
		container.scrollTop += elementRect.bottom - bottomBoundary;
	}
}

export function createKeyboardListNavigation<
	TContainer extends HTMLElement = HTMLElement,
	TItem extends HTMLElement = HTMLElement
>(options: Options) {
	const scrollPadding = options.scrollPadding ?? 8;

	let selectedIndex = $state(-1);
	let hoveredIndex = $state(-1);
	let listRef = $state<TContainer | undefined>(undefined);
	let scrollFrame: number | undefined;

	const itemRefs = new SvelteMap<number, TItem>();
	const highlightedIndex = $derived(hoveredIndex >= 0 ? hoveredIndex : selectedIndex);

	function cancelPendingScroll() {
		if (scrollFrame === undefined) return;

		cancelAnimationFrame(scrollFrame);
		scrollFrame = undefined;
	}

	function clearHover() {
		hoveredIndex = -1;
	}

	function move(delta: 1 | -1) {
		const count = options.getItemCount();

		clearHover();

		if (count <= 0) {
			selectedIndex = -1;
			return;
		}

		if (selectedIndex < 0) {
			selectedIndex = delta > 0 ? 0 : count - 1;
			return;
		}

		selectedIndex = (selectedIndex + delta + count) % count;
	}

	function sync() {
		const count = options.getItemCount();

		selectedIndex = clampIndex(selectedIndex, count);

		if (count === 0) {
			selectedIndex = -1;
			hoveredIndex = -1;
			return;
		}

		if (hoveredIndex >= count) {
			hoveredIndex = -1;
		}
	}

	function reset() {
		cancelPendingScroll();
		selectedIndex = -1;
		hoveredIndex = -1;
	}

	function open() {
		selectedIndex = getInitialIndex(options.getItemCount());
		hoveredIndex = -1;
	}

	function select(index: number) {
		selectedIndex = clampIndex(index, options.getItemCount());
	}

	function selectFirst() {
		selectedIndex = getInitialIndex(options.getItemCount());
	}

	function hover(index: number) {
		const count = options.getItemCount();

		if (index < 0 || index >= count) {
			hoveredIndex = -1;
			return;
		}

		hoveredIndex = index;
	}

	function leaveList() {
		hoveredIndex = -1;
	}

	function setListRef(node: TContainer) {
		listRef = node;

		return {
			destroy() {
				if (listRef === node) {
					listRef = undefined;
				}
			}
		};
	}

	function setItemRef(node: TItem, index: number) {
		let currentIndex = index;

		itemRefs.set(currentIndex, node);

		return {
			update(nextIndex: number) {
				if (nextIndex === currentIndex) return;

				itemRefs.delete(currentIndex);
				currentIndex = nextIndex;
				itemRefs.set(currentIndex, node);
			},
			destroy() {
				itemRefs.delete(currentIndex);
			}
		};
	}

	function ensureSelectedVisible() {
		if (selectedIndex < 0) return;

		cancelPendingScroll();

		scrollFrame = requestAnimationFrame(() => {
			scrollFrame = undefined;

			const container = listRef;
			const element = itemRefs.get(selectedIndex);

			if (!container || !element) return;

			ensureVisible(container, element, scrollPadding);
		});
	}

	function handleKeydown<T>({ event, items, onExecute }: KeydownArgs<T>) {
		const count = items.length;

		if (event.key === 'Tab') {
			event.preventDefault();

			if (event.shiftKey) {
				move(-1);
			} else {
				move(1);
			}

			return;
		}

		if (event.key === 'ArrowDown') {
			event.preventDefault();
			move(1);
			return;
		}

		if (event.key === 'ArrowUp') {
			event.preventDefault();
			move(-1);
			return;
		}

		if (event.key === 'Enter') {
			const index = selectedIndex >= 0 ? selectedIndex : getInitialIndex(count);

			if (index < 0 || index >= count) return;

			event.preventDefault();
			clearHover();
			selectedIndex = index;
			onExecute(items[index], index);
			return;
		}

		if (event.key === 'Escape') {
			event.preventDefault();
			options.onRequestClose?.();
		}
	}

	return {
		get selectedIndex() {
			return selectedIndex;
		},
		get hoveredIndex() {
			return hoveredIndex;
		},
		get highlightedIndex() {
			return highlightedIndex;
		},
		sync,
		reset,
		open,
		select,
		selectFirst,
		hover,
		clearHover,
		leaveList,
		handleKeydown,
		ensureSelectedVisible,
		setListRef,
		setItemRef
	};
}
