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

	$effect(() => {
		const request = scrollToItemRequest;
		if (!hasAppliedInitialScroll && scrollViewport && request && totalCount > 0) {
			const index = getItemIndexById(request.itemId);

			if (index !== null) {
				setInitialScrollPosition(request.itemId);
				hasAppliedInitialScroll = true;
			} else if (onEnsureItemLoaded) {
				void onEnsureItemLoaded(request.itemId).then(() => {
					void tick().then(() => {
						setInitialScrollPosition(request.itemId);
					});
				});
				hasAppliedInitialScroll = true;
			}
		}
	});

	$effect(() => {
		if (scrollToItemRequest) {
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
		<div class="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
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
		</div>

		<p class="text-sm whitespace-nowrap text-fg-muted">{totalCount} episodes</p>

		{#if selectedFeed}
			<div class="mt-4 flex w-full flex-row flex-wrap items-center justify-between gap-4">
				<div class="flex-1">
					<label class="sr-only" for="feed-search">Search this feed</label>

					<div class="input-group grid-cols-[auto_1fr_auto]">
						<div class="ig-cell preset-tonal">
							<Icon icon="lucide:search" class="size-4" />
						</div>

						<input
							id="feed-search"
							bind:this={searchInputRef}
							class="ig-input"
							placeholder="Search this feed"
							type="search"
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

						<div class="ig-cell flex items-center gap-1 text-fg-muted">
							<kbd class="kbd">⌘</kbd>
							<kbd class="kbd">F</kbd>
						</div>
					</div>
				</div>
				<div class="flex flex-wrap items-center justify-end gap-3 align-top">
					{#if selectedStation}
						<button
							type="button"
							title="Play station"
							class="preset-filled-accent btn-icon rounded-xl"
							onclick={onPlayStation}
						>
							<Icon icon="lucide:play" class="size-4" />
						</button>

						<button
							type="button"
							title="Edit station"
							class="preset-outlined-subtle btn-icon rounded-xl"
							onclick={onEditStation}
							aria-label="Edit station"
						>
							<Icon icon="lucide:pencil" class="size-4" />
						</button>

						<button
							type="button"
							title="Delete station"
							class="preset-filled-error btn-icon rounded-xl"
							onclick={onDeleteStation}
							aria-label="Delete station"
						>
							<Icon icon="lucide:trash-2" class="size-4" />
						</button>
					{:else if selectedFeed}
						<div class="flex shrink-0 items-center gap-2">
							<label class="sr-only" for="feed-sort-order">Sort order</label>
							<select
								id="feed-sort-order"
								class="select min-w-36 rounded-xl text-sm"
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

							<button
								title="Refresh feed"
								class="preset-outlined-subtle btn-icon shrink-0 rounded-xl"
								disabled={isRefreshing}
								type="button"
								onclick={() => {
									void onRefresh(selectedFeed.id);
								}}
								aria-label="Refresh feed"
							>
								{#key isRefreshing}
									<Icon
										icon="lucide:refresh-cw"
										class={`size-4 ${isRefreshing ? 'animate-spin' : ''}`}
									/>
								{/key}
							</button>
						</div>
					{/if}
				</div>
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
					{#each Array.from({ length: 4 }) as _, index (index)}
						<div class="px-0 py-5">
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
			<div class="px-6 py-12 text-center lg:px-8">
				{#if hasActiveSearch}
					<h3 class="text-xl font-semibold text-fg">No matching items</h3>
					<p class="mt-3 text-sm leading-6 text-fg-secondary">
						Try a different search term or clear the filter.
					</p>
				{:else}
					<h3 class="text-xl font-semibold text-fg">Nothing here yet</h3>
					<p class="mx-auto mt-3 max-w-xl text-sm leading-6 text-fg-secondary">
						This view is wired up, but there are no matching items right now. Add more feeds or
						switch filters to keep exploring the shell.
					</p>
				{/if}
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
									<div class="absolute top-6 left-3 z-10 size-2 rounded-full bg-accent-dot"></div>
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
											<span class="badge preset-tonal-surface text-xs"> Podcast </span>
										{/if}
									</div>

									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div class="flex flex-wrap gap-2" onclick={(e) => e.stopPropagation()}>
										{#if isMediaItem(item)}
											<DynamicPlayButton {item} compact={true} size="sm" />
										{/if}

										{#if !isMediaItem(item)}
											<button
												class="preset-outlined-subtle btn-icon rounded-xl"
												type="button"
												aria-label={item.read ? 'Mark as unread' : 'Mark as read'}
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
