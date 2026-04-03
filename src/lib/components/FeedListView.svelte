<script lang="ts">
	import type { SidebarSection } from '$lib/stores/app';
	import type { Feed, FeedItem } from '$lib/types/rss';
	import { formatDate, formatDuration } from '$lib/utils/format';

	type Props = {
		feeds: Feed[];
		isRefreshing: boolean;
		items: FeedItem[];
		onRefresh: (feedId: string) => Promise<void>;
		selectedFeed: Feed | null;
		selectedSection: SidebarSection;
		onMarkRead: (itemId: string, read: boolean) => Promise<void>;
		onPlay: (item: FeedItem) => void;
	};

	let {
		feeds,
		isRefreshing,
		items,
		onRefresh,
		selectedFeed,
		selectedSection,
		onMarkRead,
		onPlay
	}: Props = $props();

	function headingForSection(section: SidebarSection): string {
		if (selectedFeed) {
			return selectedFeed.title;
		}

		if (section === 'unread') {
			return 'Unread';
		}

		if (section === 'podcasts') {
			return 'Podcasts';
		}

		if (section === 'saved') {
			return 'Saved';
		}

		return 'All feeds';
	}

	function feedTitle(feedId: string): string {
		return feeds.find((feed) => feed.id === feedId)?.title ?? 'Unknown feed';
	}
</script>

<section class="flex-1 px-6 py-8 lg:px-8">
	<div class="mb-6 flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
		<div>
			<p class="text-sm font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400">
				Current view
			</p>
			<h2 class="mt-2 text-2xl font-semibold tracking-tight text-slate-950 dark:text-white">
				{headingForSection(selectedSection)}
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
			<p class="text-sm text-slate-500 dark:text-slate-400">{items.length} items in this view</p>
			{#if selectedFeed}
				<button
					class="rounded-xl border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:text-slate-950 disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:text-white"
					disabled={isRefreshing}
					type="button"
					onclick={() => {
						void onRefresh(selectedFeed.id);
					}}
				>
					{isRefreshing ? 'Refreshing...' : 'Refresh'}
				</button>
			{/if}
		</div>
	</div>

	{#if items.length === 0}
		<div
			class="rounded-3xl border border-dashed border-slate-300 bg-white p-8 text-center shadow-sm dark:border-slate-800 dark:bg-slate-900"
		>
			<h3 class="text-xl font-semibold text-slate-950 dark:text-white">Nothing here yet</h3>
			<p class="mt-3 text-sm leading-6 text-slate-600 dark:text-slate-300">
				This view is wired up, but there are no matching items right now. Add more feeds or switch
				filters to keep exploring the shell.
			</p>
		</div>
	{:else}
		<div class="grid gap-4">
			{#each items as item (item.id)}
				<article
					class="rounded-3xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-slate-300 dark:border-slate-800 dark:bg-slate-900 dark:hover:border-slate-700"
				>
					<div class="flex flex-wrap items-start justify-between gap-3">
						<div class="min-w-0 flex-1">
							<div
								class="flex flex-wrap items-center gap-2 text-xs font-medium tracking-[0.16em] text-slate-500 uppercase dark:text-slate-400"
							>
								<span>{feedTitle(item.feedId)}</span>
								<span>&bull;</span>
								<span>{formatDate(item.publishedAt)}</span>
							</div>
							<h3 class="mt-3 text-lg font-semibold text-slate-950 dark:text-white">
								{item.title}
							</h3>
						</div>

						<div class="flex flex-wrap gap-2">
							{#if item.mediaEnclosure}
								<button
									class="rounded-xl bg-slate-950 px-3 py-2 text-sm font-medium text-white transition hover:bg-slate-800 dark:bg-slate-100 dark:text-slate-950 dark:hover:bg-white"
									type="button"
									onclick={() => onPlay(item)}
								>
									{item.playbackPositionSeconds > 0 ? 'Resume' : 'Listen'}
								</button>
							{/if}
							<button
								class="rounded-xl border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:text-slate-950 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:text-white"
								type="button"
								onclick={() => {
									void onMarkRead(item.id, !item.read);
								}}
							>
								{item.read ? 'Mark unread' : 'Mark read'}
							</button>
						</div>
					</div>

					<p class="mt-4 text-sm leading-6 whitespace-pre-line text-slate-600 dark:text-slate-300">
						{item.summary}
					</p>

					<div
						class="mt-4 flex flex-wrap items-center gap-2 text-xs font-medium text-slate-500 dark:text-slate-400"
					>
						<span class="rounded-full bg-slate-100 px-2.5 py-1 dark:bg-slate-800">
							{item.read ? 'Read' : 'Unread'}
						</span>
						{#if item.saved}
							<span class="rounded-full bg-slate-100 px-2.5 py-1 dark:bg-slate-800">Saved</span>
						{/if}
						{#if item.mediaEnclosure}
							<span class="rounded-full bg-slate-100 px-2.5 py-1 dark:bg-slate-800">Podcast</span>
						{/if}
						{#if item.playbackPositionSeconds > 0}
							<span class="rounded-full bg-slate-100 px-2.5 py-1 dark:bg-slate-800">
								Resumes at {formatDuration(item.playbackPositionSeconds)}
							</span>
						{/if}
					</div>

					<button
						class="mt-5 inline-flex text-sm font-medium text-slate-700 transition hover:text-slate-950 dark:text-slate-300 dark:hover:text-white"
						type="button"
						onclick={() => window.open(item.url, '_blank', 'noopener,noreferrer')}
					>
						Open source
					</button>
				</article>
			{/each}
		</div>
	{/if}
</section>
