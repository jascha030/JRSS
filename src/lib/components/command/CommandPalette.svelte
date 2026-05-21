<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';
	import type { CommandPaletteItem } from '$lib/types/command';
	import {
		getCommandPaletteItems,
		clampHighlightedIndex,
		shouldShowCommandCategory,
		handleCommandPaletteKeydown
	} from '$lib/services/command';

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
	let highlightedIndex = $state(-1);
	let inputRef: HTMLInputElement | undefined = $state();
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

	$effect(() => {
		if (open && !wasOpen) {
			requestAnimationFrame(() => {
				inputRef?.focus();
				inputRef?.select();
			});

			highlightedIndex = items.length > 0 ? 0 : -1;
		}

		if (!open && wasOpen) {
			inputValue = '';
			highlightedIndex = -1;
		}

		wasOpen = open;
	});

	$effect(() => {
		if (!open) return;
		highlightedIndex = clampHighlightedIndex(highlightedIndex, items.length);
	});

	function handleKeydown(event: KeyboardEvent) {
		handleCommandPaletteKeydown(event, {
			items,
			highlightedIndex,
			onClose,
			setHighlightedIndex: (index: number) => {
				highlightedIndex = index;
			}
		});
	}

	function handleClear() {
		inputValue = '';
		highlightedIndex = clampHighlightedIndex(0, items.length);
		inputRef?.focus();
	}

	function handleBackdropClick() {
		onClose();
	}

	function handlePanelClick(event: MouseEvent) {
		event.stopPropagation();
	}

	function handleItemHover(index: number) {
		highlightedIndex = index;
	}

	function handleRootKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onClose();
		}
	}

	function handleItemClick(item: CommandPaletteItem) {
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
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
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
					onkeydown={handleKeydown}
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

			<div class="max-h-80 overflow-y-auto p-2">
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

						<button
							type="button"
							class="flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors"
							class:bg-surface-raised={index === highlightedIndex}
							class:text-fg={index === highlightedIndex}
							class:text-fg-muted={index !== highlightedIndex}
							onclick={() => handleItemClick(item)}
							onmouseenter={() => handleItemHover(index)}
						>
							<Icon icon={item.icon} class="size-4 shrink-0" />
							<span class="truncate">{item.title}</span>

							{#if item.badge}
								<span
									class="ml-auto shrink-0 text-[10px] font-medium tracking-wider text-fg-muted uppercase"
								>
									{item.badge}
								</span>
							{/if}
						</button>
					{/each}
				{/if}
			</div>

			<div class="border-t border-border px-4 py-2">
				<div class="flex items-center gap-4 text-[10px] text-fg-muted">
					<span class="flex items-center gap-1">
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]"> ↑↓ </span>
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]"> ⇥ </span>
						Navigate
					</span>
					<span class="flex items-center gap-1">
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]"> ↵ </span>
						Execute
					</span>
					<span class="flex items-center gap-1">
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]"> Esc </span>
						Close
					</span>
				</div>
			</div>
		</div>
	</div>
{/if}
