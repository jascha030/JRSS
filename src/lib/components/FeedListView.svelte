<script lang="ts">
	import { onMount } from 'svelte';

	import type { SidebarSection } from '$lib/stores/app.svelte';
	import type { Feed, FeedListItem } from '$lib/types/rss';
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
		totalCount: number;
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
		totalCount
	}: Props = $props();

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
	const DESKTOP_ROW_HEIGHT = 256;
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
</script>

<svelte:window bind:innerWidth={windowWidth} />

<section class="flex h-full flex-1 flex-col overflow-hidden bg-white dark:bg-zinc-950">
	<div class="shrink-0 border-b border-slate-200 px-6 py-8 lg:px-8 dark:border-slate-800">
		<div class="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
			<div>
				<h2 class="mt-2 text-2xl font-semibold tracking-tight text-slate-950 dark:text-white">
					{pageHeading}
				</h2>

				{#if selectedFeed}
					<p class="mt-2 text-sm text-slate-500 dark:text-slate-400">{selectedFeed.url}</p>

					{#if selectedFeed.lastFetchedAt}
						<p class="mt-1 text-xs text-slate-400 dark:text-slate-500">
							Last refreshed {formatDate(selectedFeed.lastFetchedAt)}
						</p>
					{/if}
				{/if}
			</div>

			<div class="flex items-center gap-3">
				<p class="text-sm text-slate-500 dark:text-slate-400">{totalCount} items in this view</p>

				{#if selectedFeed}
					<button
						title="Refresh feed"
						class="rounded-xl border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:text-slate-950 disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:text-white"
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
						<div
							class="rounded-3xl border border-slate-200/80 bg-white px-6 py-5 dark:border-slate-800 dark:bg-slate-900"
						>
							<div class="h-3 w-32 rounded-full bg-slate-200 dark:bg-slate-800"></div>
							<div class="mt-5 h-6 w-3/4 rounded-full bg-slate-200 dark:bg-slate-800"></div>
							<div class="mt-3 space-y-2">
								<div class="h-3 rounded-full bg-slate-200 dark:bg-slate-800"></div>
								<div class="h-3 w-11/12 rounded-full bg-slate-200 dark:bg-slate-800"></div>
								<div class="h-3 w-2/3 rounded-full bg-slate-200 dark:bg-slate-800"></div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{:else if totalCount === 0}
			<div class="px-6 py-8 lg:px-8">
				<div
					class="rounded-3xl border border-dashed border-slate-300 bg-white p-8 text-center shadow-sm dark:border-slate-800 dark:bg-slate-900"
				>
					<h3 class="text-xl font-semibold text-slate-950 dark:text-white">Nothing here yet</h3>
					<p class="mt-3 text-sm leading-6 text-slate-600 dark:text-slate-300">
						This view is wired up, but there are no matching items right now. Add more feeds or
						switch filters to keep exploring the shell.
					</p>
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
									index > 0 ? 'border-t border-slate-200 dark:border-slate-800 ' : ''
								}${
									selectedItemId === item.id
										? 'bg-slate-50 dark:bg-slate-900/60'
										: 'bg-transparent hover:bg-slate-50 dark:hover:bg-slate-900/60'
								}`}
								aria-labelledby={`feed-item-title-${item.id}`}
							>
								<button
									type="button"
									class="absolute inset-0 z-0 outline-none focus-visible:ring-2 focus-visible:ring-slate-300 focus-visible:ring-inset dark:focus-visible:ring-slate-700"
									aria-label={`Open article: ${item.title}`}
									aria-pressed={selectedItemId === item.id}
									onclick={() => onSelectItem(item.id)}
								></button>

								<div
									class="pointer-events-none relative z-10 flex min-h-0 flex-1 flex-col overflow-hidden"
								>
									<div
										class="flex flex-wrap items-center gap-2 text-xs font-medium tracking-[0.16em] text-slate-500 uppercase dark:text-slate-400"
									>
										<span>{feedTitle(item.feedId)}</span>
										<span>&bull;</span>
										<span>{formatDate(item.publishedAt)}</span>
									</div>

									<div class="">
										<h3
											id={`feed-item-title-${item.id}`}
											class="mt-3 text-lg font-semibold text-slate-950 dark:text-white"
										>
											{item.title}
										</h3>

										<p
											class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-300"
										>
											{getListPreview(item)}
										</p>
									</div>
								</div>

								<div
									class="pointer-events-none relative z-10 mt-auto flex flex-wrap items-center justify-between gap-2 pt-3"
								>
									<div class="flex flex-wrap items-center gap-2">
										<span
											class="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500 dark:bg-slate-800 dark:text-slate-400"
										>
											{item.read ? 'Read' : 'Unread'}
										</span>

										{#if item.mediaEnclosure}
											<span
												class="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500 dark:bg-slate-800 dark:text-slate-400"
											>
												Podcast
											</span>
										{/if}

										{#if item.playbackPositionSeconds > 0}
											<span
												class="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500 dark:bg-slate-800 dark:text-slate-400"
											>
												Resumes at {formatDuration(item.playbackPositionSeconds)}
											</span>
										{/if}
									</div>

									<div class="pointer-events-auto flex flex-wrap gap-2">
										{#if item.mediaEnclosure}
											<button
												class="rounded-lg bg-slate-950 px-3 py-2 text-sm font-medium text-white transition hover:bg-slate-800 dark:bg-slate-100 dark:text-slate-950 dark:hover:bg-white"
												type="button"
												onclick={() => onPlay(item)}
											>
												{item.playbackPositionSeconds > 0 ? 'Resume' : 'Listen'}
											</button>
										{/if}

										<button
											class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:text-slate-950 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:text-white"
											type="button"
											onclick={() => {
												void onMarkRead(item.id, !item.read);
											}}
										>
											{item.read ? 'Mark unread' : 'Mark read'}
										</button>
									</div>
								</div>
							</article>
						{:else}
							<div
								class={`feed-row flex h-full min-h-0 flex-col overflow-hidden px-6 py-5 lg:px-8 ${
									index > 0 ? 'border-t border-slate-200 dark:border-slate-800' : ''
								}`}
							>
								<div class="h-3 w-32 rounded-full bg-slate-200 dark:bg-slate-800"></div>
								<div class="mt-5 h-6 w-3/4 rounded-full bg-slate-200 dark:bg-slate-800"></div>
								<div class="mt-3 space-y-2">
									<div class="h-3 rounded-full bg-slate-200 dark:bg-slate-800"></div>
									<div class="h-3 w-11/12 rounded-full bg-slate-200 dark:bg-slate-800"></div>
									<div class="h-3 w-2/3 rounded-full bg-slate-200 dark:bg-slate-800"></div>
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
