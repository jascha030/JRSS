<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import Icon from '@iconify/svelte';
	import { queryItems } from '$lib/services/feedService';
	import type { Feed, FeedListItem } from '$lib/types/rss';
	import { isMediaItem } from '$lib/types/rss';
	import { formatDate } from '$lib/utils/format';

	type Props = {
		onOpenDialog: () => void;
		feeds: Feed[];
		onSelectResult: (item: FeedListItem) => void;
	};

	let { onOpenDialog, feeds, onSelectResult }: Props = $props();

	let searchInputRef = $state<HTMLInputElement | null>(null);
	let containerRef = $state<HTMLDivElement | null>(null);
	let inputValue = $state('');
	let results = $state<FeedListItem[]>([]);
	let isLoading = $state(false);
	let isOpen = $state(false);
	let highlightedIndex = $state(-1);

	const feedTitleById = $derived(new Map(feeds.map((f) => [f.id, f.title])));

	$effect(() => {
		const term = inputValue.trim();

		if (!term) {
			results = [];
			isOpen = false;
			isLoading = false;
			return;
		}

		isLoading = true;

		const timer = setTimeout(() => {
			void queryItems({
				section: 'all',
				offset: 0,
				limit: 8,
				search: term,
				sortOrder: 'newest_first'
			})
				.then((page) => {
					results = page.items;
					isOpen = true;
					highlightedIndex = -1;
				})
				.catch(() => {
					results = [];
				})
				.finally(() => {
					isLoading = false;
				});
		}, 220);

		return () => clearTimeout(timer);
	});

	function handleSelect(item: FeedListItem): void {
		onSelectResult(item);
		inputValue = '';
		isOpen = false;
		searchInputRef?.blur();
	}

	function handleInputKeydown(event: KeyboardEvent): void {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			highlightedIndex = Math.min(highlightedIndex + 1, results.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			highlightedIndex = Math.max(highlightedIndex - 1, -1);
		} else if (event.key === 'Enter') {
			event.preventDefault();
			const item = highlightedIndex >= 0 ? results[highlightedIndex] : results[0];
			if (item) handleSelect(item);
		} else if (event.key === 'Escape') {
			inputValue = '';
			isOpen = false;
			searchInputRef?.blur();
		}
	}

	function handleGlobalKeydown(event: KeyboardEvent): void {
		const target = event.target as HTMLElement;
		if (
			target instanceof HTMLInputElement ||
			target instanceof HTMLTextAreaElement ||
			target.getAttribute('contenteditable') === 'true'
		) {
			return;
		}
		if (event.key === '/') {
			event.preventDefault();
			searchInputRef?.focus();
		}
	}

	function handleWindowClick(event: MouseEvent): void {
		if (containerRef && !containerRef.contains(event.target as Node)) {
			isOpen = false;
		}
	}

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		const setupListener = async () => {
			unlisten = await listen('menu-add-feed', () => {
				onOpenDialog();
			});
		};

		void setupListener();

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});
</script>

<svelte:window onkeydown={handleGlobalKeydown} onclick={handleWindowClick} />

<div class="flex w-full items-center gap-4 px-2">
	<div bind:this={containerRef} class="relative mr-12 flex-1">
		<label class="sr-only" for="global-search">Search all feeds</label>
		<div
			class="flex h-9 items-center gap-2 rounded-xl border border-border bg-surface-shell px-3 transition-colors focus-within:border-border-hover focus-within:ring-2 focus-within:ring-ring"
		>
			{#if isLoading}
				<Icon icon="lucide:loader-circle" class="size-4 shrink-0 animate-spin text-fg-muted" />
			{:else}
				<Icon icon="lucide:search" class="size-4 shrink-0 text-fg-muted" />
			{/if}
			<input
				id="global-search"
				bind:this={searchInputRef}
				class="min-w-0 flex-1 bg-transparent text-sm text-fg outline-none placeholder:text-fg-muted [&::-webkit-search-cancel-button]:hidden"
				placeholder="Search all feeds"
				type="search"
				autocomplete="off"
				value={inputValue}
				oninput={(event) => {
					const target = event.currentTarget;
					if (target instanceof HTMLInputElement) {
						inputValue = target.value;
					}
				}}
				onkeydown={handleInputKeydown}
				onfocus={() => {
					if (results.length > 0) isOpen = true;
				}}
			/>
			<kbd
				class="rounded border border-border bg-surface-sidebar-hover px-1.5 py-0.5 text-xs font-medium text-fg-muted select-none"
				>/</kbd
			>
		</div>

		{#if isOpen && results.length > 0}
			<div
				class="absolute top-full right-0 left-0 z-50 mt-1 overflow-hidden rounded-xl border border-border bg-surface-glass shadow-xl backdrop-blur-xl"
				role="listbox"
				aria-label="Search results"
			>
				{#each results as item, i (item.id)}
					<div
						class={`flex cursor-pointer flex-col gap-0.5 px-4 py-3 transition-colors ${
							i > 0 ? 'border-t border-border' : ''
						} ${i === highlightedIndex ? 'bg-surface-sidebar-active' : 'hover:bg-surface-sidebar-hover'}`}
						role="option"
						tabindex="-1"
						aria-selected={i === highlightedIndex}
						onmousedown={(e) => {
							e.preventDefault();
							handleSelect(item);
						}}
						onmouseenter={() => (highlightedIndex = i)}
					>
						<div class="flex items-center gap-2">
							<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase">
								{feedTitleById.get(item.feedId) ?? 'Unknown feed'}
							</span>
							{#if isMediaItem(item)}
								<span class="badge shrink-0 preset-tonal-surface text-xs">Podcast</span>
							{/if}
							<span class="ml-auto shrink-0 text-xs text-fg-subtle">
								{formatDate(item.publishedAt)}
							</span>
						</div>
						<p class="truncate text-sm font-medium text-fg">{item.title}</p>
						{#if item.previewText}
							<p class="truncate text-xs text-fg-secondary">{item.previewText}</p>
						{/if}
					</div>
				{/each}
			</div>
		{:else if isOpen && !isLoading && inputValue.trim()}
			<div
				class="absolute top-full right-0 left-0 z-50 mt-1 rounded-xl border border-border bg-surface-glass px-4 py-6 text-center shadow-xl backdrop-blur-xl"
			>
				<p class="text-sm text-fg-muted">No results for "{inputValue.trim()}"</p>
			</div>
		{/if}
	</div>

	<button
		type="button"
		class="preset-filled-accent btn-icon shrink-0 rounded-xl"
		onclick={onOpenDialog}
	>
		<Icon icon="lucide:plus" class="size-4" />
	</button>
</div>
