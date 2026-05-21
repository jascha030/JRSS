<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';
	import type { CommandPaletteItem } from '$lib/types/command';
	import { getCommandPaletteItems, shouldShowCommandCategory } from '$lib/services/command';
	import { createKeyboardListNavigation } from '$lib/services/keyboard-list-navigation.svelte';
	import CommandRow from './CommandRow.svelte';

	type Props = {
		open: boolean;
		feeds: Feed[];
		stations: Station[];
		isPlaying: boolean;
		onClose: () => void;
		onToggleCover: () => void;
		onToggleMiniPlayer: () => void;
		onToggleSidebar: () => void;
		onAddFeed: () => void;
		onAddStation: () => void;
	};

	let {
		open,
		feeds,
		stations,
		isPlaying,
		onClose,
		onToggleCover,
		onToggleMiniPlayer,
		onToggleSidebar,
		onAddFeed,
		onAddStation
	}: Props = $props();

	let inputValue = $state('');
	let inputRef = $state<HTMLInputElement | undefined>(undefined);
	let wasOpen = false;

	const items = $derived(
		getCommandPaletteItems({
			feeds,
			stations,
			isPlaying,
			term: inputValue,
			onClose,
			onToggleCover,
			onToggleMiniPlayer,
			onToggleSidebar,
			onAddFeed,
			onAddStation
		})
	);

	const isEmpty = $derived(items.length === 0 && inputValue.trim() !== '');

	const navigation = createKeyboardListNavigation<HTMLDivElement, HTMLButtonElement>({
		getItemCount: () => items.length,
		onRequestClose: () => onClose(),
		scrollPadding: 8
	});

	function resetPalette() {
		inputValue = '';
		navigation.reset();
	}

	function focusInput() {
		requestAnimationFrame(() => {
			inputRef?.focus();
			inputRef?.select();
		});
	}

	$effect(() => {
		if (open && !wasOpen) {
			navigation.open();
			focusInput();
		}

		if (!open && wasOpen) {
			resetPalette();
		}

		wasOpen = open;
	});

	$effect(() => {
		if (!open) return;
		navigation.sync();
	});

	$effect(() => {
		if (!open) return;
		navigation.ensureSelectedVisible();
	});

	function handleInputKeydown(event: KeyboardEvent) {
		navigation.handleKeydown({
			event,
			items,
			onExecute: (item) => {
				item.action();
			}
		});
	}

	function handleClear() {
		inputValue = '';
		navigation.clearHover();

		requestAnimationFrame(() => {
			navigation.selectFirst();
			inputRef?.focus();
			inputRef?.select();
		});
	}

	function handleBackdropClick() {
		onClose();
	}

	function handlePanelClick(event: MouseEvent) {
		event.stopPropagation();
	}

	function handleRootKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onClose();
		}
	}

	function handleItemHover(index: number) {
		navigation.hover(index);
	}

	function handleListPointerLeave() {
		navigation.leaveList();
	}

	function handleItemClick(item: CommandPaletteItem, index: number) {
		navigation.select(index);
		navigation.clearHover();
		item.action();
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-start justify-center bg-black/50 pt-[15vh] backdrop-blur-sm"
		tabindex="-1"
		onclick={handleBackdropClick}
		onkeydown={handleRootKeydown}
		role="dialog"
		aria-label="Command palette"
	>
		<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
		<div
			class="w-full max-w-lg rounded-xl border border-border bg-surface-shell-opaque shadow-2xl"
			onclick={handlePanelClick}
		>
			<div class="flex items-center gap-3 border-b border-border px-4 py-3">
				<Icon icon="lucide:search" class="size-4 shrink-0 text-fg-muted" />
				<input
					bind:this={inputRef}
					bind:value={inputValue}
					type="text"
					class="w-full bg-transparent text-sm text-fg placeholder:text-fg-muted focus:outline-none"
					placeholder="Type a command..."
					onkeydown={handleInputKeydown}
				/>
				{#if inputValue}
					<button
						type="button"
						class="cursor-pointer text-xs text-fg-muted hover:text-fg"
						onclick={handleClear}
						aria-label="Clear search"
					>
						<Icon icon="lucide:x" class="size-3" />
					</button>
				{/if}
			</div>

			<div
				use:navigation.setListRef
				class="scrollbar-none max-h-80 overflow-y-auto p-2"
				onpointerleave={handleListPointerLeave}
			>
				{#if isEmpty}
					<div class="px-4 py-8 text-center text-sm text-fg-muted">No results found</div>
				{:else if items.length > 0}
					{#each items as item, index (item.id)}
						{#if shouldShowCommandCategory(items, index)}
							<div
								class="px-3 pt-2 pb-1 text-[10px] font-semibold tracking-wider text-fg-muted uppercase"
							>
								{item.category}
							</div>
						{/if}

						<CommandRow
							{item}
							isHighlighted={index === navigation.highlightedIndex}
							{index}
							setRef={navigation.setItemRef}
							onClick={(item) => handleItemClick(item, index)}
							onHover={handleItemHover}
						/>
					{/each}
				{/if}
			</div>

			<div class="border-t border-border px-4 py-2">
				<div class="flex items-center gap-4 text-[10px] text-fg-muted">
					<span class="flex items-center gap-1">
						<kbd class="kbd text-xs">↑↓</kbd> <kbd class="kbd text-xs">⇥</kbd> Navigate
					</span>
					<span class="flex items-center gap-1">
						<kbd class="kbd text-xs">↵</kbd> Execute
					</span>
					<span class="flex items-center gap-1">
						<kbd class="kbd text-xs">Esc</kbd> Close
					</span>
				</div>
			</div>
		</div>
	</div>
{/if}
