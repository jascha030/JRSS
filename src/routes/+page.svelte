<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import FeedListView from '$lib/components/FeedListView.svelte';
	import QueueDrawer from '$lib/components/QueueDrawer.svelte';
	import ReaderPane from '$lib/components/ReaderPane.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import {
		app,
		clearQueue,
		createFeed,
		ensureVisibleRangeLoaded,
		getActiveItemIdsByIndex,
		getActiveTotalCount,
		getCurrentAudioItem,
		getCurrentAudioItemFeed,
		getEffectiveSortOrder,
		getIsActiveInitialLoading,
		getManualQueueLength,
		getSelectedFeed,
		getSelectedItem,
		getSelectedItemFeed,
		getUpcomingQueue,
		handlePlaybackEnded,
		initializeApp,
		loadItemDetails,
		loadReaderView,
		markItemRead,
		moveQueuedItemDown,
		moveQueuedItemUp,
		persistPlaybackForItem,
		persistPlaybackPosition,
		refreshExistingFeed,
		removeQueuedItem,
		selectFeed,
		selectItem,
		selectSection,
		setFeedSearchTerm,
		setItemSortOrder,
		setPlaybackPlaying,
		updatePlaybackPosition,
		getPlaybackToggleSeq,
		getReaderRequestSeq,
		getReaderRequestItemId,
		getSeekRequestSeq,
		getSeekRequestPositionSeconds
	} from '$lib/stores/app.svelte';
	import { isMediaItem } from '$lib/types/rss';
	import { List } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	let newFeedUrl = $state('');
	let isSidebarCollapsed = $state(true);
	let isQueueDrawerOpen = $state(false);
	let readerPaneMode = $state<'feed' | 'reader'>('feed');
	let lastSelectedItemId = $state<string | null>(null);

	const {
		feeds,
		isCreatingFeed,
		syncingFeedIds,
		readerLoadingItemIds,
		selectedFeedId,
		selectedItemId,
		selectedSection,
		currentPlaybackState,
		itemSummariesById,
		feedSearchTerm
	} = $derived.by(() => app);

	const selectedFeed = $derived(getSelectedFeed());
	const selectedItem = $derived(getSelectedItem());
	const selectedItemFeed = $derived(getSelectedItemFeed());
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentAudioItemFeed = $derived(getCurrentAudioItemFeed());
	const itemIdsByIndex = $derived(getActiveItemIdsByIndex());
	const totalCount = $derived(getActiveTotalCount());
	const isInitialLoading = $derived(getIsActiveInitialLoading());
	const itemSortOrder = $derived(getEffectiveSortOrder());
	const upcomingQueue = $derived(getUpcomingQueue());
	const queueLength = $derived(upcomingQueue.length);
	const manualQueueLength = $derived(getManualQueueLength());
	const playbackToggleSeq = $derived(getPlaybackToggleSeq());
	const readerRequestSeq = $derived(getReaderRequestSeq());
	const seekRequestSeq = $derived(getSeekRequestSeq());
	const seekRequestPositionSeconds = $derived(getSeekRequestPositionSeconds());

	const isSelectedFeedRefreshing = $derived(
		selectedFeed ? syncingFeedIds.includes(selectedFeed.id) : false
	);

	const isSelectedItemReaderLoading = $derived(
		selectedItem ? readerLoadingItemIds.includes(selectedItem.id) : false
	);

	const hasSelectedItemReaderContent = $derived(selectedItem?.readerStatus === 'ready');
	const isReaderPaneActive = $derived(readerPaneMode === 'reader' && hasSelectedItemReaderContent);
	const canUseReaderMode = $derived(selectedItem ? !isMediaItem(selectedItem) : false);

	$effect(() => {
		if (selectedItemId !== lastSelectedItemId) {
			readerPaneMode = 'feed';
		}
	});

	$effect(() => {
		const currentSelectedItemId = selectedItemId;

		if (!currentSelectedItemId) {
			return;
		}

		void loadItemDetails(currentSelectedItemId).catch((error: unknown) => {
			if (selectedItemId !== currentSelectedItemId) {
				return;
			}

			toast.error(error instanceof Error ? error.message : 'Unable to load article details.');
		});
	});

	function toggleSidebar() {
		isSidebarCollapsed = !isSidebarCollapsed;
	}

	onMount(() => {
		void initializeApp();
	});

	// React to reader-open requests from context menus
	let lastConsumedReaderSeq = 0;
	$effect(() => {
		if (readerRequestSeq > lastConsumedReaderSeq) {
			lastConsumedReaderSeq = readerRequestSeq;
			const itemId = getReaderRequestItemId();
			if (itemId) {
				void handleLoadReaderView(itemId);
			}
		}
	});

	async function handleAddFeed() {
		const candidateUrl = newFeedUrl.trim();

		if (!candidateUrl) {
			toast.warning('Enter a feed URL to add a source.');
			return;
		}

		try {
			await createFeed(candidateUrl);
			newFeedUrl = '';
			toast.success('Feed loaded and saved locally.');
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to add that feed.');
		}
	}

	async function handleRefreshFeed(feedId: string) {
		try {
			await refreshExistingFeed(feedId);
			toast.success('Feed refreshed.');
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to refresh that feed.');
		}
	}

	async function handleLoadReaderView(itemId: string) {
		try {
			const updatedItem = await loadReaderView(itemId);

			if (updatedItem.readerStatus === 'ready') {
				readerPaneMode = 'reader';
				return;
			}

			readerPaneMode = 'feed';
			toast.warning('Reader view was unavailable for this item. Showing feed content instead.');
		} catch (error: unknown) {
			readerPaneMode = 'feed';
			toast.error(
				error instanceof Error ? error.message : 'Unable to load reader view for this item.'
			);
		}
	}
</script>

<svelte:head>
	<title>JRSS</title>
	<meta
		name="description"
		content="Local-first RSS reader and podcast MVP foundation built with SvelteKit."
	/>
</svelte:head>

<div class="h-screen overflow-hidden bg-surface-shell">
	<QueueDrawer
		open={isQueueDrawerOpen}
		queueItems={upcomingQueue}
		{manualQueueLength}
		{feeds}
		onRemoveItem={removeQueuedItem}
		onMoveItemUp={moveQueuedItemUp}
		onMoveItemDown={moveQueuedItemDown}
		onClearQueue={clearQueue}
		onClose={() => {
			isQueueDrawerOpen = false;
		}}
	/>

	<div class="relative h-full overflow-hidden">
		<div class="absolute inset-y-0 left-0 z-20 hidden md:block">
			<Sidebar
				collapsed={isSidebarCollapsed}
				{feeds}
				refreshingFeedIds={syncingFeedIds}
				{selectedFeedId}
				{selectedSection}
				onSelectFeed={selectFeed}
				onSelectSection={selectSection}
				onToggleCollapse={toggleSidebar}
			/>
		</div>

		<div
			class={`relative z-30 h-full transform-gpu transition-transform duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform motion-reduce:transition-none md:absolute md:inset-y-0 md:left-0 ${
				isSidebarCollapsed
					? 'md:w-[calc(100%+6rem)] md:translate-x-24'
					: 'md:w-[calc(100%+18rem)] md:translate-x-72'
			}`}
		>
			<div
				class={`flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden ${
					isSidebarCollapsed ? 'md:w-[calc(100%-12rem)]' : 'md:w-[calc(100%-36rem)]'
				}`}
			>
				<main class="flex min-h-0 flex-1 flex-col bg-surface-shell">
					<header
						class="flex h-20 shrink-0 items-center justify-end border-b border-border bg-surface-glass px-6 backdrop-blur lg:px-8"
					>
						<div class="flex flex-col gap-5 xl:flex-row xl:items-center xl:justify-between">
							<form
								class="flex min-w-full flex-col gap-4 sm:flex-row"
								onsubmit={(event) => {
									event.preventDefault();
									void handleAddFeed();
								}}
							>
								<input
									bind:value={newFeedUrl}
									class="min-w-1 flex-1 rounded-2xl border border-border-strong bg-surface px-4 py-3 text-sm text-fg transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
									disabled={isCreatingFeed}
									placeholder="RSS URL, Apple Podcasts URL, or Apple ID"
									type="text"
								/>
								<button class="btn-primary btn" disabled={isCreatingFeed} type="submit">
									{isCreatingFeed ? 'Adding...' : 'Add feed'}
								</button>
							</form>
						</div>
					</header>

					{#if feeds.length === 0 && !isInitialLoading}
						<section
							class="flex flex-2 items-center justify-center overflow-y-auto px-6 py-12 lg:px-8"
						>
							<div
								class="max-w-lg rounded-3xl border border-dashed border-border-strong bg-surface-card p-8 text-center shadow-sm"
							>
								<p class="text-sm font-medium tracking-[0.18em] text-fg-muted uppercase">
									No feeds yet
								</p>
								<h1 class="mt-3 text-2xl font-semibold text-fg">
									Add your first RSS feed or podcast
								</h1>
								<p class="mt-4 text-sm leading-6 text-fg-secondary">
									Add an RSS or Atom URL above, or paste an Apple Podcasts show link or ID. The
									desktop app resolves the feed and persists it in local SQLite.
								</p>
							</div>
						</section>
					{:else if selectedSection === 'settings'}
						<SettingsView />
					{:else}
						<div class="flex min-h-0 flex-1 overflow-hidden">
							<div
								class="min-h-0 min-w-0 grow xl:flex-1 xl:border-r xl:border-border 2xl:basis-1/3"
							>
								<FeedListView
									{feeds}
									{itemIdsByIndex}
									itemsById={itemSummariesById}
									isRefreshing={isSelectedFeedRefreshing}
									{isInitialLoading}
									onRefresh={handleRefreshFeed}
									onVisibleRangeChange={ensureVisibleRangeLoaded}
									onSelectItem={selectItem}
									{selectedFeed}
									{selectedItemId}
									{selectedSection}
									onMarkRead={markItemRead}
									{totalCount}
									searchTerm={feedSearchTerm}
									onSearchChange={setFeedSearchTerm}
									{itemSortOrder}
									onSortOrderChange={setItemSortOrder}
								/>
							</div>

							<ReaderPane
								{selectedItem}
								{selectedItemFeed}
								{readerPaneMode}
								{isSelectedItemReaderLoading}
								{hasSelectedItemReaderContent}
								{isReaderPaneActive}
								{canUseReaderMode}
								onLoadReaderView={handleLoadReaderView}
								onReaderPaneModeChange={(mode) => {
									readerPaneMode = mode;
								}}
							/>
						</div>
					{/if}
				</main>

				<AudioPlayer
					item={currentAudioItem}
					imageUrl={currentAudioItemFeed?.imageUrl}
					playbackState={currentPlaybackState}
					onPlayingChange={setPlaybackPlaying}
					onPositionChange={updatePlaybackPosition}
					onPositionPersist={persistPlaybackPosition}
					onTransitionPersist={persistPlaybackForItem}
					onEnded={() => void handlePlaybackEnded()}
					toggleSeq={playbackToggleSeq}
					seekSeq={seekRequestSeq}
					seekToSeconds={seekRequestPositionSeconds}
				>
					{#snippet controls()}
						<button
							type="button"
							class="flex size-8 items-center justify-center rounded-lg text-fg-muted transition-colors hover:bg-surface-hover hover:text-fg"
							aria-label={isQueueDrawerOpen ? 'Close queue' : `Open queue (${queueLength})`}
							title={isQueueDrawerOpen ? 'Close queue' : `Queue (${queueLength})`}
							onclick={() => {
								isQueueDrawerOpen = !isQueueDrawerOpen;
							}}
						>
							<List class="size-4" />

							{#if queueLength > 0}
								<span
									class="absolute -top-1 -right-1 flex size-4 items-center justify-center rounded-full bg-accent text-[9px] font-bold text-fg-inverse"
								>
									{queueLength}
								</span>
							{/if}
						</button>
					{/snippet}
				</AudioPlayer>
			</div>
		</div>
	</div>
</div>
