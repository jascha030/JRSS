<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { useItemSelection } from '$lib/hooks/useItemSelection.svelte';
	import { useMenuShortcuts } from '$lib/hooks/useMenuShortcuts.svelte';

	import type { SidebarSection } from '$lib/state';
	import type { Feed } from '$lib/types/feed';
	import type { FeedListItem, ItemSortOrder } from '$lib/types/item';
	import type { Station } from '$lib/types/station';
	import { isMediaItem } from '$lib/types/item';
	import { formatDate } from '$lib/utils/format';
	import { openFeedContextMenu } from '$lib/utils/tauri-menu';

	import SearchInput from '$lib/components/ui/SearchInput.svelte';
	import SkeletonRow from '$lib/components/ui/SkeletonRow.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import PlayButton from '../playback/PlayButton.svelte';

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
		onInspect?: (feedId: string) => void;
		totalCount: number;
		searchTerm: string;
		onSearchChange: (term: string) => void;
		stationSearchTerm: string;
		onStationSearchChange: (term: string) => void;
		sectionSearchTerm: string;
		onSectionSearchChange: (term: string) => void;
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
		onInspect,
		totalCount,
		searchTerm,
		onSearchChange,
		stationSearchTerm,
		onStationSearchChange,
		sectionSearchTerm,
		onSectionSearchChange,
		itemSortOrder,
		onSortOrderChange,
		onPlayStation,
		onEditStation,
		onDeleteStation,
		scrollToItemRequest = null,
		onEnsureItemLoaded
	}: Props = $props();

	let searchInputRef = $state<HTMLInputElement | null>(null);
	let stationSearchInputRef = $state<HTMLInputElement | null>(null);
	let sectionSearchInputRef = $state<HTMLInputElement | null>(null);

	const DESKTOP_ROW_HEIGHT = 200;
	const MOBILE_ROW_HEIGHT = 304;
	const OVERSCAN_ROWS = 1;

	const sectionHeadings: Record<Exclude<SidebarSection, null>, string> = {
		home: 'Home',
		all: 'All feeds',
		unread: 'Unread',
		media: 'Media',
		settings: 'Settings'
	};

	const { hasActiveSearch, pageHeading, feedTitleById, rowHeight, totalHeight } = $derived.by(
		() => ({
			hasActiveSearch:
				searchTerm.trim().length > 0 ||
				stationSearchTerm.trim().length > 0 ||
				sectionSearchTerm.trim().length > 0,
			pageHeading:
				selectedStation?.name ??
				selectedFeed?.title ??
				(selectedSection ? sectionHeadings[selectedSection] : 'All feeds'),
			feedTitleById: new Map(feeds.map((feed) => [feed.id, feed.title])),
			rowHeight: windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT,
			totalHeight: totalCount * (windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT)
		})
	);

	const selection = useItemSelection({
		getItemIdsByIndex: () => itemIdsByIndex,
		getItemsById: () => itemsById,
		onSelectItem: (itemId) => onSelectItem(itemId)
	});

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
			const index = selection.getItemIndexById(request.itemId);

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

	function setInitialScrollPosition(itemId: string): void {
		if (!scrollViewport) return;

		const index = selection.getItemIndexById(itemId);
		if (index === null) return;

		const targetScrollTop = index * rowHeight;
		scrollTop = targetScrollTop;
		scrollViewport.scrollTop = targetScrollTop;
	}

	useMenuShortcuts([
		{
			event: 'menu-search-feed',
			handler: () => {
				if (selectedFeed) {
					searchInputRef?.focus();
				} else if (selectedStation) {
					stationSearchInputRef?.focus();
				} else if (selectedSection === 'unread' || selectedSection === 'media') {
					sectionSearchInputRef?.focus();
				}
			}
		},
		{
			event: 'menu-refresh-feed',
			handler: () => {
				if (selectedFeed && !isRefreshing) {
					void onRefresh(selectedFeed.id);
				}
			}
		}
	]);

	onMount(() => {
		return () => {
			if (scrollFrame !== 0) {
				cancelAnimationFrame(scrollFrame);
			}
		};
	});
</script>

<svelte:window bind:innerWidth={windowWidth} />

<section class="flex h-full w-full flex-1 flex-col overflow-hidden bg-surface backdrop-blur-md">
	<div class="shrink-0 border-b border-border px-6 py-8 pb-7.75 lg:px-8">
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

		<div class="flex flex-wrap items-center justify-end gap-3 align-top">
			{#if selectedStation}
				<IconButton
					icon="lucide:play"
					title="Play station"
					label="Play station"
					variant="accent"
					onclick={onPlayStation}
				/>

				<IconButton
					icon="lucide:pencil"
					title="Edit station"
					label="Edit station"
					onclick={onEditStation}
				/>

				<IconButton
					icon="lucide:trash-2"
					title="Delete station"
					label="Delete station"
					variant="error"
					onclick={onDeleteStation}
				/>
			{:else if selectedFeed}
				<div class="flex shrink-0 items-center gap-2">
					<label class="sr-only" for="feed-sort-order">Sort order</label>
					<select
						id="feed-sort-order"
						class="min-w-36"
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

					{#key isRefreshing}
						<IconButton
							icon="lucide:refresh-cw"
							title="Refresh feed"
							label="Refresh feed"
							iconClass="size-4 {isRefreshing ? 'animate-spin' : ''}"
							disabled={isRefreshing}
							onclick={() => void onRefresh(selectedFeed.id)}
						/>
					{/key}

					{#if onInspect}
						<IconButton
							icon="lucide:scan-search"
							title="Inspect feed XML"
							label="Inspect feed XML"
							onclick={() => onInspect(selectedFeed.id)}
						/>
					{/if}
				</div>
			{/if}
		</div>

		{#if selectedFeed || selectedStation || selectedSection}
			<div class="mt-4 flex w-full flex-row flex-wrap items-center justify-between gap-4">
				<div class="flex-1">
					{#if selectedFeed}
						<SearchInput
							id="feed-search"
							label="Search this feed"
							placeholder="Search this feed"
							bind:value={searchTerm}
							bind:inputRef={searchInputRef}
							kbdShortcuts={['⌘', 'F']}
							oninput={(event) => onSearchChange(event.currentTarget.value)}
							onkeydown={(event) => {
								if (event.key === 'Escape') {
									onSearchChange('');
									searchInputRef?.blur();
								}
							}}
						/>
					{:else if selectedStation}
						<SearchInput
							id="station-search"
							label="Search this station"
							placeholder="Search this station"
							bind:value={stationSearchTerm}
							bind:inputRef={stationSearchInputRef}
							kbdShortcuts={['⌘', 'F']}
							oninput={(event) => onStationSearchChange(event.currentTarget.value)}
							onkeydown={(event) => {
								if (event.key === 'Escape') {
									onStationSearchChange('');
									stationSearchInputRef?.blur();
								}
							}}
						/>
					{:else if selectedSection === 'unread' || selectedSection === 'media'}
						<SearchInput
							id="section-search"
							label="Search {selectedSection === 'unread' ? 'unread' : 'media'}"
							placeholder="Search {selectedSection === 'unread' ? 'unread' : 'media'}"
							bind:value={sectionSearchTerm}
							bind:inputRef={sectionSearchInputRef}
							kbdShortcuts={['⌘', 'F']}
							oninput={(event) => onSectionSearchChange(event.currentTarget.value)}
							onkeydown={(event) => {
								if (event.key === 'Escape') {
									onSectionSearchChange('');
									sectionSearchInputRef?.blur();
								}
							}}
						/>
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
					{#each Array.from({ length: 4 }), index (index)}
						<SkeletonRow />
					{/each}
				</div>
			</div>
		{:else if totalCount === 0}
			{#if hasActiveSearch}
				<EmptyState
					title="No matching items"
					description="Try a different search term or clear the filter."
				/>
			{:else}
				<EmptyState
					title="Nothing here yet"
					description="This view is wired up, but there are no matching items right now. Add more feeds or switch filters to keep exploring the shell."
				/>
			{/if}
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
								data-item-index={index}
								class={`feed-row relative flex h-full min-h-0 flex-col overflow-hidden px-6 py-5 transition-colors duration-150 lg:px-8 ${
									index > 0 ? 'border-t border-border' : ''
								} ${
									selectedItemId === item.id || selection.selectedIds.has(item.id)
										? 'bg-surface-active text-fg'
										: 'bg-surface text-fg hover:bg-surface-hover'
								}`}
								aria-labelledby={`feed-item-title-${item.id}`}
								oncontextmenu={(event) => selection.handleItemContextMenu(event, item)}
								onclick={(event) => selection.handleItemClick(event, item.id)}
							>
								{#if !item.read}
									<div class="absolute top-6 left-3 z-10 size-2 rounded-full bg-accent-dot"></div>
								{/if}

								<div class="relative z-10 flex min-h-0 flex-1 flex-col overflow-hidden">
									<div
										class="flex flex-wrap items-center gap-2 text-xs font-medium tracking-wide text-fg-muted uppercase"
									>
										<span>{feedTitle(item.feedId)}</span>
										<span>&bull;</span>
										<span>{formatDate(item.publishedAt)}</span>
									</div>

									<div class="mt-2 line-clamp-2 4xl:line-clamp-3">
										<h3
											id={`feed-item-title-${item.id}`}
											class="text-md font-semibold text-fg 2xl:text-lg"
										>
											{item.title}
										</h3>

										<p class="text-sm leading-6 text-fg-secondary">
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
											<PlayButton {item} compact={true} size="sm" />
										{/if}

										{#if !isMediaItem(item)}
											<IconButton
												icon={item.read
													? 'heroicons:envelope-open-solid'
													: 'heroicons:envelope-solid'}
												label={item.read ? 'Mark as unread' : 'Mark as read'}
												iconClass="size-5"
												onclick={() => void onMarkRead(item.id, !item.read)}
											/>
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
								<SkeletonRow />
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
