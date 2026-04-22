<script lang="ts">
	import { onMount, tick } from 'svelte';

	import type { SidebarSection } from '$lib/stores/app.svelte';
	import type { Feed, FeedListItem, ItemSortOrder, Station } from '$lib/types/rss';
	import { isMediaItem } from '$lib/types/rss';
	import { formatDate } from '$lib/utils/format';
	import {
		openArticleContextMenu,
		openAudioContextMenu,
		openFeedContextMenu
	} from '$lib/utils/tauri-menu';
	import Icon from '@iconify/svelte';
	import DynamicPlayButton from '../player/DynamicPlayButton.svelte';

	type Props = {
		feeds: Feed[];
		itemIdsByIndex: Record<number, string>;
		itemsById: Record<string, FeedListItem>;
		isRefreshing: boolean;
		isInitialLoading: boolean;
		onRefresh: (feedId: string) => Promise<void>;
		onVisibleRangeChange: (startIndex: number, endIndex: number) => Promise<void> | void;
		onSelectItem: (itemId: string) => void;
		selectedFeed: Feed | null;
		selectedStation: Station | null;
		selectedItemId: string | null;
		selectedSection: SidebarSection;
		onMarkRead: (itemId: string, read: boolean) => Promise<void>;
		totalCount: number;
		searchTerm: string;
		onSearchChange: (term: string) => void;
		itemSortOrder: ItemSortOrder;
		onSortOrderChange: (order: ItemSortOrder) => void;
		onPlayStation?: () => void;
		onEditStation?: () => void;
		onDeleteStation?: () => void;
		scrollToItemRequest?: { itemId: string; seq: number } | null;
		onEnsureItemLoaded?: (itemId: string) => Promise<void>;
	};

	let {
		feeds,
		itemIdsByIndex,
		itemsById,
		isRefreshing,
		isInitialLoading,
		onRefresh,
		onVisibleRangeChange,
		onSelectItem,
		selectedFeed,
		selectedStation,
		selectedItemId,
		selectedSection,
		onMarkRead,
		totalCount,
		searchTerm,
		onSearchChange,
		itemSortOrder,
		onSortOrderChange,
		onPlayStation,
		onEditStation,
		onDeleteStation,
		scrollToItemRequest = null,
		onEnsureItemLoaded
	}: Props = $props();

	let searchInputRef = $state<HTMLInputElement | null>(null);

	const DESKTOP_ROW_HEIGHT = 200;
	const MOBILE_ROW_HEIGHT = 304;
	const OVERSCAN_ROWS = 1;

	const sectionHeadings: Record<Exclude<SidebarSection, null>, string> = {
		all: 'All feeds',
		unread: 'Unread',
		media: 'Media',
		settings: 'Settings'
	};

	const { hasActiveSearch, pageHeading, feedTitleById, rowHeight, totalHeight } = $derived.by(
		() => ({
			hasActiveSearch: searchTerm.trim().length > 0,
			pageHeading:
				selectedStation?.name ??
				selectedFeed?.title ??
				(selectedSection ? sectionHeadings[selectedSection] : 'All feeds'),
			feedTitleById: new Map(feeds.map((feed) => [feed.id, feed.title])),
			rowHeight: windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT,
			totalHeight: totalCount * (windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT)
		})
	);

	type VisibleRow = {
		index: number;
		item: FeedListItem | null;
		top: number;
	};

	type VisibleRange = {
		startIndex: number;
		endIndex: number;
	};

	let scrollViewport = $state<HTMLDivElement | null>(null);
	let viewportHeight = $state(0);
	let windowWidth = $state(0);
	let scrollTop = $state(0);
	let pendingScrollTop = 0;
	let scrollFrame = 0;

	function feedTitle(feedId: string): string {
		return feedTitleById.get(feedId) ?? 'Unknown feed';
	}

	function getListPreview(item: FeedListItem) {
		return item.previewText;
	}

	const visibleRange = $derived.by((): VisibleRange | null => {
		if (totalCount === 0) {
			return null;
		}

		const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - OVERSCAN_ROWS);
		const visibleCount = Math.ceil(viewportHeight / rowHeight) + OVERSCAN_ROWS * 2;
		const endIndex = Math.min(totalCount, startIndex + visibleCount);

		return {
			startIndex,
			endIndex
		};
	});

	const visibleRows = $derived.by((): VisibleRow[] => {
		if (!visibleRange) {
			return [];
		}

		const rows: VisibleRow[] = [];

		for (let index = visibleRange.startIndex; index < visibleRange.endIndex; index += 1) {
			const itemId = itemIdsByIndex[index];
			const item = itemId ? (itemsById[itemId] ?? null) : null;

			rows.push({
				index,
				item,
				top: index * rowHeight
			});
		}

		return rows;
	});

	function scheduleScrollTop(nextScrollTop: number): void {
		pendingScrollTop = nextScrollTop;

		if (scrollFrame !== 0) {
			return;
		}

		scrollFrame = requestAnimationFrame(() => {
			scrollTop = pendingScrollTop;
			scrollFrame = 0;
		});
	}

	function handleScroll(event: Event): void {
		const currentTarget = event.currentTarget;

		if (!(currentTarget instanceof HTMLDivElement)) {
			return;
		}

		scheduleScrollTop(currentTarget.scrollTop);
	}

	let hasAppliedInitialScroll = $state(false);

	$effect(() => {
		if (!scrollViewport) {
			return;
		}

		scrollTop = scrollViewport.scrollTop;
	});

	// Handle initial scroll position for scrollToItemRequest
	$effect(() => {
		const request = scrollToItemRequest;
		if (!hasAppliedInitialScroll && scrollViewport && request && totalCount > 0) {
			const index = getItemIndexById(request.itemId);

			if (index !== null) {
				// Item already loaded, scroll immediately
				setInitialScrollPosition(request.itemId);
				hasAppliedInitialScroll = true;
			} else if (onEnsureItemLoaded) {
				// Item not loaded yet, load it first then scroll
				void onEnsureItemLoaded(request.itemId).then(() => {
					// Wait for next tick so itemIdsByIndex is updated
					void tick().then(() => {
						setInitialScrollPosition(request.itemId);
					});
				});
				hasAppliedInitialScroll = true;
			}
		}
	});

	// Reset hasAppliedInitialScroll when scrollToItemRequest changes (access seq to track changes)
	$effect(() => {
		if (scrollToItemRequest) {
			// Access seq to ensure effect re-runs when request changes
			void scrollToItemRequest.seq;
			hasAppliedInitialScroll = false;
		}
	});

	$effect(() => {
		if (!visibleRange) {
			return;
		}

		void onVisibleRangeChange(visibleRange.startIndex, visibleRange.endIndex - 1);
	});

	function getItemIndexById(itemId: string): number | null {
		for (const [index, id] of Object.entries(itemIdsByIndex)) {
			if (id === itemId) {
				return Number(index);
			}
		}
		return null;
	}

	function setInitialScrollPosition(itemId: string): void {
		if (!scrollViewport) return;

		const index = getItemIndexById(itemId);
		if (index === null) return;

		const targetScrollTop = index * rowHeight;
		// Set both reactive state and DOM element directly
		scrollTop = targetScrollTop;
		scrollViewport.scrollTop = targetScrollTop;
	}

	onMount(() => {
		return () => {
			if (scrollFrame !== 0) {
				cancelAnimationFrame(scrollFrame);
			}
		};
	});

	function handleWindowKeydown(event: KeyboardEvent): void {
		if (event.key === 'f' && event.metaKey && !event.shiftKey && selectedFeed) {
			event.preventDefault();
			searchInputRef?.focus();
		}
	}
</script>

<svelte:window bind:innerWidth={windowWidth} onkeydown={handleWindowKeydown} />

<section class="flex h-full w-full flex-1 flex-col overflow-hidden bg-surface">
	<div class="shrink-0 border-b border-border px-6 py-8 lg:px-8">
		<div class="flex flex-col flex-wrap gap-3 md:flex-row md:items-end md:justify-between">
			<div>
				<h2
					class="mt-2 text-2xl font-semibold tracking-tight text-fg"
					class:select-none={selectedFeed}
					oncontextmenu={selectedFeed
						? (event) => void openFeedContextMenu(event, selectedFeed)
						: undefined}
				>
					{pageHeading}
				</h2>

				{#if selectedFeed && selectedFeed.lastFetchedAt}
					<p class="mt-1 text-xs text-fg-subtle">
						Last refreshed {formatDate(selectedFeed.lastFetchedAt)}
					</p>
				{/if}
			</div>

			<div class="flex items-center gap-3">
				{#if selectedStation}
					<button
						type="button"
						title="Play station"
						class="btn-primary flex items-center gap-1.5 rounded-xl px-3 py-2 text-sm"
						onclick={onPlayStation}
					>
						<Icon icon="lucide:play" class="size-4" />
						Play
					</button>

					<button
						type="button"
						title="Edit station"
						class="btn-secondary rounded-xl px-3 py-2"
						onclick={onEditStation}
					>
						<Icon icon="lucide:pencil" class="size-4" />
					</button>

					<button
						type="button"
						title="Delete station"
						class="btn-danger btn-sm"
						onclick={onDeleteStation}
					>
						<Icon icon="lucide:trash-2" class="size-4" />
					</button>

					<p class="text-sm text-fg-muted">{totalCount} episodes</p>
				{:else if selectedFeed}
					<select
						class="rounded-xl border border-border bg-surface px-2 py-1.5 text-xs text-fg-muted transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
						aria-label="Sort order"
						value={itemSortOrder}
						onchange={(event) => {
							const target = event.currentTarget;
							if (target instanceof HTMLSelectElement) {
								const value = target.value;
								if (value === 'newest_first' || value === 'oldest_first') {
									onSortOrderChange(value);
								}
							}
						}}
					>
						<option value="newest_first">Newest first</option>
						<option value="oldest_first">Oldest first</option>
					</select>
				{/if}

				<p class="text-sm text-fg-muted">{totalCount} items</p>

				{#if selectedFeed}
					<button
						title="Refresh feed"
						class="btn-secondary rounded-xl px-3 py-2"
						disabled={isRefreshing}
						type="button"
						onclick={() => {
							void onRefresh(selectedFeed.id);
						}}
					>
						{#key isRefreshing}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="size-6"
								class:animate-spin={isRefreshing}
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
								/>
							</svg>
						{/key}
					</button>
				{/if}
			</div>
		</div>

		{#if selectedFeed}
			<div class="relative mt-4">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 16 16"
					fill="currentColor"
					class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-fg-muted"
				>
					<path
						fill-rule="evenodd"
						d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z"
						clip-rule="evenodd"
					/>
				</svg>
				<input
					bind:this={searchInputRef}
					class="w-full rounded-xl border border-border bg-surface py-2 pr-9 pl-9 text-sm text-fg transition outline-none placeholder:text-fg-muted focus:border-border-hover focus:ring-2 focus:ring-ring"
					placeholder="Search this feed"
					type="text"
					value={searchTerm}
					oninput={(event) => {
						const target = event.currentTarget;
						if (target instanceof HTMLInputElement) {
							onSearchChange(target.value);
						}
					}}
					onkeydown={(event) => {
						if (event.key === 'Escape') {
							onSearchChange('');
							searchInputRef?.blur();
						}
					}}
				/>
				{#if hasActiveSearch}
					<button
						type="button"
						class="absolute top-1/2 right-2.5 -translate-y-1/2 rounded-md p-0.5 text-fg-muted transition-colors hover:text-fg"
						title="Clear search"
						onclick={() => {
							onSearchChange('');
							searchInputRef?.focus();
						}}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 16 16"
							fill="currentColor"
							class="size-4"
						>
							<path
								d="M5.28 4.22a.75.75 0 0 0-1.06 1.06L6.94 8l-2.72 2.72a.75.75 0 1 0 1.06 1.06L8 9.06l2.72 2.72a.75.75 0 1 0 1.06-1.06L9.06 8l2.72-2.72a.75.75 0 0 0-1.06-1.06L8 6.94 5.28 4.22Z"
							/>
						</svg>
					</button>
				{/if}
			</div>
		{/if}
	</div>

	<div
		bind:this={scrollViewport}
		bind:clientHeight={viewportHeight}
		class="min-h-0 flex-1 overflow-y-auto"
		onscroll={handleScroll}
	>
		{#if isInitialLoading}
			<div class="px-6 py-4 lg:px-8">
				<div class="space-y-4">
					<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
					{#each Array.from({ length: 4 }) as _, index (index)}
						<div class="rounded-3xl border border-border bg-surface-card px-6 py-5">
							<div class="h-3 w-32 rounded-full bg-skeleton"></div>
							<div class="mt-5 h-6 w-3/4 rounded-full bg-skeleton"></div>
							<div class="mt-3 space-y-2">
								<div class="h-3 rounded-full bg-skeleton"></div>
								<div class="h-3 w-11/12 rounded-full bg-skeleton"></div>
								<div class="h-3 w-2/3 rounded-full bg-skeleton"></div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{:else if totalCount === 0}
			<div class="px-6 py-8 lg:px-8">
				<div
					class="rounded-3xl border border-dashed border-border-strong bg-surface-card p-8 text-center shadow-sm"
				>
					{#if hasActiveSearch}
						<h3 class="text-xl font-semibold text-fg">No matching items</h3>
						<p class="mt-3 text-sm leading-6 text-fg-secondary">
							Try a different search term or clear the filter.
						</p>
					{:else}
						<h3 class="text-xl font-semibold text-fg">Nothing here yet</h3>
						<p class="mt-3 text-sm leading-6 text-fg-secondary">
							This view is wired up, but there are no matching items right now. Add more feeds or
							switch filters to keep exploring the shell.
						</p>
					{/if}
				</div>
			</div>
		{:else}
			<div class="relative" style={`height: ${totalHeight}px;`}>
				{#each visibleRows as { item, index, top } (item?.id ?? index)}
					<div
						class="absolute inset-x-0 top-0"
						style={`height: ${rowHeight}px; transform: translateY(${top}px);`}
					>
						{#if item}
							<!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
							<article
								class={`feed-row relative flex h-full min-h-0 flex-col overflow-hidden px-6 py-5 transition-colors duration-150 lg:px-8 ${
									index > 0 ? 'border-t border-border' : ''
								} ${
									selectedItemId === item.id
										? 'bg-surface-active text-fg'
										: 'bg-surface text-fg hover:bg-surface-hover'
								}`}
								aria-labelledby={`feed-item-title-${item.id}`}
								oncontextmenu={isMediaItem(item)
									? (event) => void openAudioContextMenu(event, item)
									: (event) => void openArticleContextMenu(event, item)}
								onclick={() => onSelectItem(item.id)}
							>
								{#if !item.read}
									<div class="absolute inset-x-3 inset-y-6 size-2 rounded-full bg-accent-dot"></div>
								{/if}

								<div class="relative z-10 flex min-h-0 flex-1 flex-col overflow-hidden">
									<div
										class="flex flex-wrap items-center gap-2 text-xs font-medium tracking-[0.16em] text-fg-muted uppercase"
									>
										<span>{feedTitle(item.feedId)}</span>
										<span>&bull;</span>
										<span>{formatDate(item.publishedAt)}</span>
									</div>

									<div class="line-clamp-2">
										<h3
											id={`feed-item-title-${item.id}`}
											class="mt-3 text-lg font-semibold text-fg"
										>
											{item.title}
										</h3>

										<p class="mt-2 text-sm leading-6 text-fg-secondary">
											{getListPreview(item)}
										</p>
									</div>
								</div>

								<div
									class="relative z-10 mt-auto flex flex-wrap items-center justify-between gap-2 pt-3"
								>
									<div class="flex flex-wrap items-center gap-2">
										{#if isMediaItem(item)}
											<span
												class="rounded-full bg-surface-active px-2.5 py-1 text-xs font-medium text-fg-muted"
											>
												Podcast
											</span>
										{/if}
									</div>

									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div class="flex flex-wrap gap-2" onclick={(e) => e.stopPropagation()}>
										{#if isMediaItem(item)}
											<DynamicPlayButton {item} compact={true} size="sm" />
										{/if}

										{#if !isMediaItem(item)}
											<button
												class="btn-secondary rounded-xl px-3 py-2"
												type="button"
												onclick={() => {
													void onMarkRead(item.id, !item.read);
												}}
											>
												{#if item.read}
													<Icon icon="heroicons:envelope-open-solid" class="size-5" />
												{:else}
													<Icon icon="heroicons:envelope-solid" class="size-5" />
												{/if}
											</button>
										{/if}
									</div>
								</div>
							</article>
						{:else}
							<div
								class={`feed-row flex h-full min-h-0 flex-col overflow-hidden px-6 py-5 lg:px-8 ${
									index > 0 ? 'border-t border-border' : ''
								}`}
							>
								<div class="h-3 w-32 rounded-full bg-skeleton"></div>
								<div class="mt-5 h-6 w-3/4 rounded-full bg-skeleton"></div>
								<div class="mt-3 space-y-2">
									<div class="h-3 rounded-full bg-skeleton"></div>
									<div class="h-3 w-11/12 rounded-full bg-skeleton"></div>
									<div class="h-3 w-2/3 rounded-full bg-skeleton"></div>
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</section>

<style>
	.feed-row {
		contain: layout paint;
	}
</style>
