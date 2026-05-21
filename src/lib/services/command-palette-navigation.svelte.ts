import { SvelteMap } from 'svelte/reactivity';

type Options = {
	getItemCount: () => number;
	onRequestClose?: () => void;
	scrollPadding?: number;
};

type KeydownArgs<T> = {
	event: KeyboardEvent;
	items: T[];
	onExecute: (item: T, index: number) => void;
};

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

export function createCommandPaletteNavigation(options: Options) {
	const scrollPadding = options.scrollPadding ?? 8;

	let selectedIndex = $state(-1);
	let hoveredIndex = $state(-1);
	let listRef = $state<HTMLDivElement | undefined>(undefined);
	const itemRefs = new SvelteMap<number, HTMLButtonElement>();

	const highlightedIndex = $derived(hoveredIndex >= 0 ? hoveredIndex : selectedIndex);

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
		selectedIndex = -1;
		hoveredIndex = -1;
	}

	function open() {
		selectedIndex = getInitialIndex(options.getItemCount());
		hoveredIndex = -1;
	}

	function setListRef(node: HTMLDivElement) {
		listRef = node;

		return {
			destroy() {
				if (listRef === node) {
					listRef = undefined;
				}
			}
		};
	}

	function setItemRef(node: HTMLButtonElement, index: number) {
		itemRefs.set(index, node);

		return {
			destroy() {
				itemRefs.delete(index);
			}
		};
	}

	function clearHover() {
		hoveredIndex = -1;
	}

	function hover(index: number) {
		hoveredIndex = index;
	}

	function leaveList() {
		hoveredIndex = -1;
	}

	function select(index: number) {
		selectedIndex = clampIndex(index, options.getItemCount());
	}

	function selectFirst() {
		selectedIndex = getInitialIndex(options.getItemCount());
	}

	function ensureSelectedVisible() {
		if (selectedIndex < 0) return;

		requestAnimationFrame(() => {
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
			clearHover();

			if (count === 0) return;

			if (event.shiftKey) {
				if (selectedIndex < 0) {
					selectedIndex = count - 1;
				} else {
					selectedIndex = (selectedIndex - 1 + count) % count;
				}
			} else {
				if (selectedIndex < 0) {
					selectedIndex = 0;
				} else {
					selectedIndex = (selectedIndex + 1) % count;
				}
			}

			return;
		}

		if (event.key === 'ArrowDown') {
			event.preventDefault();
			clearHover();

			if (count > 0) {
				if (selectedIndex < 0) {
					selectedIndex = 0;
				} else {
					selectedIndex = (selectedIndex + 1) % count;
				}
			}

			return;
		}

		if (event.key === 'ArrowUp') {
			event.preventDefault();
			clearHover();

			if (count > 0) {
				if (selectedIndex < 0) {
					selectedIndex = count - 1;
				} else {
					selectedIndex = (selectedIndex - 1 + count) % count;
				}
			}

			return;
		}

		if (event.key === 'Enter') {
			if (selectedIndex < 0 || selectedIndex >= count) return;

			event.preventDefault();
			onExecute(items[selectedIndex], selectedIndex);
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
