<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { useItemSelection } from '$lib/hooks/useItemSelection.svelte';

	import {
		feedsState,
		itemsState,
		selection,
		getActiveItemIdsByIndex,
		getActiveTotalCount,
		getIsActiveInitialLoading,
		ensureVisibleRangeLoaded,
		ensureItemLoaded,
		markItemRead
	} from '$lib/state';
	import { appUi } from '$lib/hooks/useAppUi.svelte';
	import { isMediaItem } from '$lib/types/item';
	import { formatDate } from '$lib/utils/format';

	import ItemListHeader from '$lib/components/content/ItemListHeader.svelte';
	import SkeletonRow from '$lib/components/ui/SkeletonRow.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import PlayButton from '../playback/PlayButton.svelte';

	const DESKTOP_ROW_HEIGHT = 200;
	const MOBILE_ROW_HEIGHT = 304;
	const OVERSCAN_ROWS = 1;

	const itemIdsByIndex = $derived(getActiveItemIdsByIndex());
	const itemsById = $derived(itemsState.itemSummariesById);
	const totalCount = $derived(getActiveTotalCount());
	const isInitialLoading = $derived(getIsActiveInitialLoading());
	const selectedItemId = $derived(selection.selectedItemId);
	const searchTerm = $derived(selection.feedSearchTerm);
	const stationSearchTerm = $derived(selection.stationSearchTerm);
	const sectionSearchTerm = $derived(selection.sectionSearchTerm);
	const scrollToItemRequest = $derived(appUi.scrollToItemRequest);

	const { hasActiveSearch, feedTitleById, rowHeight, totalHeight } = $derived.by(() => ({
		hasActiveSearch:
			searchTerm.trim().length > 0 ||
			stationSearchTerm.trim().length > 0 ||
			sectionSearchTerm.trim().length > 0,
		feedTitleById: new Map(feedsState.feeds.map((feed) => [feed.id, feed.title])),
		rowHeight: windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT,
		totalHeight: totalCount * (windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT)
	}));

	const itemSelection = useItemSelection();

	type VisibleRow = {
		index: number;
		item: import('$lib/types/item').FeedListItem | null;
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

	function getListPreview(item: import('$lib/types/item').FeedListItem) {
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
			const index = itemSelection.getItemIndexById(request.itemId);

			if (index !== null) {
				setInitialScrollPosition(request.itemId);
				hasAppliedInitialScroll = true;
			} else {
				void ensureItemLoaded(request.itemId).then(() => {
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

		void ensureVisibleRangeLoaded(visibleRange.startIndex, visibleRange.endIndex - 1);
	});

	function setInitialScrollPosition(itemId: string): void {
		if (!scrollViewport) return;

		const index = itemSelection.getItemIndexById(itemId);
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
</script>

<svelte:window bind:innerWidth={windowWidth} />

<section class="flex h-full w-full flex-1 flex-col overflow-hidden bg-surface backdrop-blur-md">
	<ItemListHeader />

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
									selectedItemId === item.id || itemSelection.selectedIds.has(item.id)
										? 'bg-surface-active text-fg'
										: 'bg-surface text-fg hover:bg-surface-hover'
								}`}
								aria-labelledby={`feed-item-title-${item.id}`}
								oncontextmenu={(event) => itemSelection.handleItemContextMenu(event, item)}
								onclick={(event) => itemSelection.handleItemClick(event, item.id)}
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
												onclick={() => void markItemRead(item.id, !item.read)}
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
