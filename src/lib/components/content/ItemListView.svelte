<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { fade } from 'svelte/transition';
	import { markItemFavorite, markItemRead } from '$lib/state/items.svelte';
	import { isMediaItem, type FeedListItem } from '$lib/types/item';
	import type { useItemSelection } from '$lib/hooks/useItemSelection.svelte';
	import type { appUi } from '$lib/hooks/useAppUi.svelte';
	import { formatDateOnly } from '$lib/utils/format';
	import SkeletonRow from '$lib/components/ui/SkeletonRow.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import PlayButton from '../playback/PlayButton.svelte';
	import Icon from '@iconify/svelte';

	type VisibleRow = {
		index: number;
		item: FeedListItem | null;
		top: number;
	};

	type VisibleRange = {
		startIndex: number;
		endIndex: number;
	};

	type Props = {
		itemsById: Record<string, FeedListItem>;
		displayedItemIdsByIndex: Record<number, string>;
		displayedTotalCount: number;
		displayedQueryKey: string | null;
		selectedItemId: string | null;
		hasActiveSearch: boolean;
		showFeedTitle: boolean;
		feedTitleById: Map<string, string>;
		localStatusById: Record<string, { isCached: boolean; isExported: boolean }>;
		isInitialLoading: boolean;
		isQueryTransitioning: boolean;
		scrollToItemRequest: typeof appUi.scrollToItemRequest;
		itemSelection: ReturnType<typeof useItemSelection>;
		hasAppliedInitialScroll: boolean;
		onEnsureItemLoaded: (itemId: string) => Promise<void>;
		onVisibleRangeLoad: (startIndex: number, endIndex: number) => Promise<void>;
		onScrollApplied: () => void;
		class?: string;
	};

	let {
		itemsById,
		displayedItemIdsByIndex,
		displayedTotalCount,
		displayedQueryKey,
		selectedItemId,
		hasActiveSearch,
		showFeedTitle,
		feedTitleById,
		localStatusById,
		isInitialLoading,
		isQueryTransitioning,
		scrollToItemRequest,
		itemSelection,
		hasAppliedInitialScroll,
		onEnsureItemLoaded,
		onVisibleRangeLoad,
		onScrollApplied,
		class: className = ''
	}: Props = $props();

	const DESKTOP_ROW_HEIGHT = 200;
	const MOBILE_ROW_HEIGHT = 304;
	const OVERSCAN_ROWS = 1;

	let scrollViewport = $state<HTMLDivElement | null>(null);
	let viewportHeight = $state(0);
	let windowWidth = $state(0);
	let scrollTop = $state(0);
	let pendingScrollTop = 0;
	let scrollFrame = 0;

	const rowHeight = $derived(windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT);
	const totalHeight = $derived(displayedTotalCount * rowHeight);
	const showInitialSkeleton = $derived(isInitialLoading && displayedQueryKey === null);

	function feedTitle(feedId: string): string {
		return feedTitleById.get(feedId) ?? 'Unknown feed';
	}

	const visibleRange = $derived.by((): VisibleRange | null => {
		if (displayedTotalCount === 0) {
			return null;
		}

		const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - OVERSCAN_ROWS);
		const visibleCount = Math.ceil(viewportHeight / rowHeight) + OVERSCAN_ROWS * 2;
		const endIndex = Math.min(displayedTotalCount, startIndex + visibleCount);

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
			const itemId = displayedItemIdsByIndex[index];
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

	$effect(() => {
		if (!scrollViewport) {
			return;
		}

		scrollTop = scrollViewport.scrollTop;
	});

	$effect(() => {
		const request = scrollToItemRequest;
		if (hasAppliedInitialScroll || !scrollViewport || !request || displayedTotalCount <= 0) {
			return;
		}

		const index = itemSelection.getItemIndexById(request.itemId);

		if (index !== null) {
			setInitialScrollPosition(request.itemId);
			onScrollApplied();
			return;
		}

		void onEnsureItemLoaded(request.itemId).then(() => {
			void tick().then(() => {
				setInitialScrollPosition(request.itemId);
				onScrollApplied();
			});
		});
	});

	$effect(() => {
		if (!visibleRange) {
			return;
		}

		void onVisibleRangeLoad(visibleRange.startIndex, visibleRange.endIndex - 1);
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

<section
	class="flex h-full w-full flex-1 flex-col overflow-hidden bg-surface backdrop-blur-md {className}"
>
	<div
		bind:this={scrollViewport}
		bind:clientHeight={viewportHeight}
		class="min-h-0 flex-1 overflow-y-auto"
		onscroll={handleScroll}
	>
		{#if showInitialSkeleton}
			<div class="px-6 py-4 lg:px-8">
				<div class="space-y-4">
					{#each Array.from({ length: 4 }), index (index)}
						<SkeletonRow />
					{/each}
				</div>
			</div>
		{:else if displayedTotalCount === 0}
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
			<div
				class="relative min-h-full transition-[opacity,filter] duration-220 ease-out motion-reduce:transition-none"
			>
				<div
					class={`transition-[opacity,filter,transform] duration-220 ease-out motion-reduce:transition-none ${
						isQueryTransitioning
							? 'scale-[0.995] opacity-55 blur-[1px]'
							: 'blur-0 scale-100 opacity-100'
					}`}
				>
					{#key displayedQueryKey}
						<div in:fade={{ duration: 180 }} out:fade={{ duration: 120 }}>
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
													<div
														class="absolute top-6 left-3 z-10 size-2 rounded-full bg-accent-dot"
													></div>
												{/if}

												<div class="relative z-10 flex min-h-0 flex-1 flex-col overflow-hidden">
													<div
														class="flex flex-wrap items-center gap-2 text-xs font-medium tracking-wide text-fg-muted uppercase"
													>
														{#if showFeedTitle}
															<span>{feedTitle(item.feedId)}</span>
															<span>&bull;</span>
														{/if}
														<span>{formatDateOnly(item.publishedAt)}</span>
													</div>

													<div class="mt-2 line-clamp-2 4xl:line-clamp-3">
														<h3
															id={`feed-item-title-${item.id}`}
															class="text-md font-semibold text-fg 2xl:text-lg"
														>
															{item.title}
														</h3>

														<p class="text-xs leading-6 text-fg-secondary">
															{item.previewText}
														</p>
													</div>
												</div>

												<div
													class="relative z-10 mt-auto flex flex-wrap items-center justify-between gap-2"
												>
													<div class="flex flex-wrap items-center gap-2">
														{#if isMediaItem(item)}
															<span class="badge preset-filled-primary-500 text-xs">
																<Icon icon="heroicons:microphone" /> Podcast
															</span>
															{@const status = localStatusById[item.id]}
															{#if status?.isExported}
																<span
																	class="badge preset-filled-success-500 text-xs"
																	title="Exported"
																>
																	<Icon icon="heroicons:arrow-down-on-square-solid" />
																</span>
															{:else if status?.isCached}
																<span
																	class="badge preset-filled-warning-500 text-xs"
																	title="Cached"
																>
																	<Icon icon="heroicons:cloud-arrow-down-solid" />
																</span>
															{/if}
														{:else}
															<span class="badge preset-filled-primary-500 text-xs">
																<Icon icon="heroicons:book-open" /> Article
															</span>
														{/if}
													</div>

													<!-- svelte-ignore a11y_no_static_element_interactions -->
													<div
														class="flex flex-wrap gap-2"
														onclick={(event) => event.stopPropagation()}
													>
														{#if isMediaItem(item)}
															<PlayButton {item} compact={true} size="sm" />
														{/if}
														<IconButton
															icon={item.favorite ? 'heroicons:heart-solid' : 'heroicons:heart'}
															label={item.favorite ? 'Remove favorite' : 'Add favorite'}
															iconClass="size-5"
															onclick={() => void markItemFavorite(item.id, !item.favorite)}
														/>
														{#if !isMediaItem(item)}
															<IconButton
																icon={item.read
																	? 'heroicons:envelope-open-solid'
																	: 'heroicons:envelope-solid'}
																label={item.read ? 'Mark as unread' : 'Mark as read'}
																iconClass="size-6"
																onclick={() => void markItemRead(item.id, !item.read)}
															/>
														{/if}
													</div>
												</div>
											</article>
										{:else}
											<div
												class={`feed-row flex h-full min-h-0 flex-col overflow-hidden px-6 py-5 lg:px-8 ${index > 0 ? 'border-t border-border' : ''}`}
											>
												<SkeletonRow />
											</div>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/key}
				</div>

				{#if isQueryTransitioning}
					<div
						class="pointer-events-none absolute inset-0 bg-linear-to-b from-surface/35 via-surface/12 to-surface/35 opacity-100 backdrop-blur-[1px] transition-opacity duration-220 ease-out motion-reduce:transition-none"
					></div>
				{/if}
			</div>
		{/if}
	</div>
</section>

<style>
	.feed-row {
		contain: layout paint;
	}
</style>
