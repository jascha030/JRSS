<script lang="ts">
	import { onMount } from 'svelte';

	import { Menu } from '@tauri-apps/api/menu';

	import type { SidebarSection } from '$lib/stores/app.svelte';
	import type { Feed, FeedListItem, ItemSortOrder } from '$lib/types/rss';
	import { formatDate, formatDuration } from '$lib/utils/format';

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
		selectedItemId: string | null;
		selectedSection: SidebarSection;
		onMarkRead: (itemId: string, read: boolean) => Promise<void>;
		onPlay: (item: FeedListItem) => void;
		onPlayNext: (item: FeedListItem) => void;
		onEnqueue: (item: FeedListItem) => void;
		totalCount: number;
		searchTerm: string;
		onSearchChange: (term: string) => void;
		itemSortOrder: ItemSortOrder;
		onSortOrderChange: (order: ItemSortOrder) => void;
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
		selectedItemId,
		selectedSection,
		onMarkRead,
		onPlay,
		onPlayNext,
		onEnqueue,
		totalCount,
		searchTerm,
		onSearchChange,
		itemSortOrder,
		onSortOrderChange
	}: Props = $props();

	// -----------------------------------------------------------------------
	// Native context menu (Tauri Menu API)
	// -----------------------------------------------------------------------

	async function openContextMenu(event: MouseEvent, item: FeedListItem): Promise<void> {
		event.preventDefault();

		const menu = await Menu.new({
			items: [
				{ id: 'play-next', text: 'Play next', action: () => onPlayNext(item) },
				{ id: 'add-to-queue', text: 'Add to queue', action: () => onEnqueue(item) }
			]
		});

		await menu.popup();
	}

	let searchInputRef = $state<HTMLInputElement | null>(null);
	const hasActiveSearch = $derived(searchTerm.trim().length > 0);

	const sectionHeadings: Record<Exclude<SidebarSection, null>, string> = {
		all: 'All feeds',
		unread: 'Unread',
		podcasts: 'Podcasts',
		settings: 'Settings'
	};

	const pageHeading = $derived(
		selectedFeed?.title ?? (selectedSection ? sectionHeadings[selectedSection] : 'All feeds')
	);

	const feedTitleById = $derived(new Map(feeds.map((feed) => [feed.id, feed.title])));
	const DESKTOP_ROW_HEIGHT = 200;
	const MOBILE_ROW_HEIGHT = 304;
	const OVERSCAN_ROWS = 1;

	type VisibleRow = {
		index: number;
		item: FeedListItem | null;
		top: number;
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

	const rowHeight = $derived(windowWidth >= 768 ? DESKTOP_ROW_HEIGHT : MOBILE_ROW_HEIGHT);
	const totalHeight = $derived(totalCount * rowHeight);

	const visibleRows = $derived.by((): VisibleRow[] => {
		if (totalCount === 0) {
			return [];
		}

		const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - OVERSCAN_ROWS);
		const visibleCount = Math.ceil(viewportHeight / rowHeight) + OVERSCAN_ROWS * 2;
		const endIndex = Math.min(totalCount, startIndex + visibleCount);
		const rows: VisibleRow[] = [];

		for (let index = startIndex; index < endIndex; index += 1) {
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

	$effect(() => {
		if (!scrollViewport) {
			return;
		}

		scrollTop = scrollViewport.scrollTop;
	});

	$effect(() => {
		if (visibleRows.length === 0) {
			return;
		}

		const startIndex = visibleRows[0].index;
		const endIndex = visibleRows[visibleRows.length - 1].index;

		void onVisibleRangeChange(startIndex, endIndex);
	});

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
				<h2 class="mt-2 text-2xl font-semibold tracking-tight text-fg">
					{pageHeading}
				</h2>

				{#if selectedFeed}
					<p class="mt-2 text-sm text-fg-muted">{selectedFeed.url}</p>

					{#if selectedFeed.lastFetchedAt}
						<p class="mt-1 text-xs text-fg-subtle">
							Last refreshed {formatDate(selectedFeed.lastFetchedAt)}
						</p>
					{/if}
				{/if}
			</div>

			<div class="flex items-center gap-3">
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
							<article
								class={`feed-row relative flex h-full min-h-0 flex-col overflow-hidden px-6 py-5 transition-colors duration-150 lg:px-8 ${
									index > 0 ? 'border-t border-border' : ''
								} ${
									selectedItemId === item.id
										? 'bg-surface-active text-fg'
										: 'bg-surface text-fg hover:bg-surface-hover'
								}`}
								aria-labelledby={`feed-item-title-${item.id}`}
								oncontextmenu={item.mediaEnclosure
									? (event) => void openContextMenu(event, item)
									: undefined}
							>
								<button
									type="button"
									class="absolute inset-0 z-0 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
									aria-label={`Open article: ${item.title}`}
									aria-pressed={selectedItemId === item.id}
									onclick={() => onSelectItem(item.id)}
								></button>

								{#if !item.read}
									<div class="absolute inset-x-3 inset-y-6 size-2 rounded-full bg-accent-dot"></div>
								{/if}

								<div
									class="pointer-events-none relative z-10 flex min-h-0 flex-1 flex-col overflow-hidden"
								>
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
									class="pointer-events-none relative z-10 mt-auto flex flex-wrap items-center justify-between gap-2 pt-3"
								>
									<div class="flex flex-wrap items-center gap-2">
										{#if item.mediaEnclosure}
											<span
												class="rounded-full bg-surface-active px-2.5 py-1 text-xs font-medium text-fg-muted"
											>
												Podcast
											</span>
										{/if}

										{#if item.playbackPositionSeconds > 0}
											<span
												class="rounded-full bg-surface-active px-2.5 py-1 text-xs font-medium text-fg-muted"
											>
												Resumes at {formatDuration(item.playbackPositionSeconds)}
											</span>
										{/if}
									</div>

									<div class="pointer-events-auto flex flex-wrap gap-2">
										{#if item.mediaEnclosure}
											<button
												class="btn-primary rounded-xl px-3 py-2"
												type="button"
												onclick={() => onPlay(item)}
											>
												{item.playbackPositionSeconds > 0 ? 'Resume' : 'Listen'}
											</button>
											<button
												class="btn-secondary rounded-xl px-3 py-2 text-xs"
												type="button"
												onclick={() => onPlayNext(item)}
											>
												Play next
											</button>
										{/if}

										<button
											class="btn-secondary rounded-xl px-3 py-2"
											type="button"
											onclick={() => {
												void onMarkRead(item.id, !item.read);
											}}
										>
											{#if item.read}
												<svg
													xmlns="http://www.w3.org/2000/svg"
													viewBox="0 0 16 16"
													fill="currentColor"
													class="size-4"
												>
													<path
														fill-rule="evenodd"
														d="M1.756 4.568A1.5 1.5 0 0 0 1 5.871V12.5A1.5 1.5 0 0 0 2.5 14h11a1.5 1.5 0 0 0 1.5-1.5V5.87a1.5 1.5 0 0 0-.756-1.302l-5.5-3.143a1.5 1.5 0 0 0-1.488 0l-5.5 3.143Zm1.82 2.963a.75.75 0 0 0-.653 1.35l4.1 1.98a2.25 2.25 0 0 0 1.955 0l4.1-1.98a.75.75 0 1 0-.653-1.35L8.326 9.51a.75.75 0 0 1-.652 0L3.575 7.53Z"
														clip-rule="evenodd"
													/>
												</svg>
											{:else}
												<svg
													xmlns="http://www.w3.org/2000/svg"
													viewBox="0 0 16 16"
													fill="currentColor"
													class="size-4"
												>
													<path
														d="M2.5 3A1.5 1.5 0 0 0 1 4.5v.793c.026.009.051.02.076.032L7.674 8.51c.206.1.446.1.652 0l6.598-3.185A.755.755 0 0 1 15 5.293V4.5A1.5 1.5 0 0 0 13.5 3h-11Z"
													/>
													<path
														d="M15 6.954 8.978 9.86a2.25 2.25 0 0 1-1.956 0L1 6.954V11.5A1.5 1.5 0 0 0 2.5 13h11a1.5 1.5 0 0 0 1.5-1.5V6.954Z"
													/>
												</svg>
											{/if}
										</button>
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
