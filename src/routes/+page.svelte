<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import FeedListView from '$lib/components/FeedListView.svelte';
	import QueueDrawer from '$lib/components/QueueDrawer.svelte';
	import ReaderPane from '$lib/components/ReaderPane.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import {
		app,
		clearQueue,
		createFeed,
		deleteFeed,
		enqueueAudioItem,
		ensureVisibleRangeLoaded,
		getActiveItemIdsByIndex,
		getActiveTotalCount,
		getCurrentAudioItem,
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
		playAudioItemNext,
		refreshExistingFeed,
		removeQueuedItem,
		selectFeed,
		selectItem,
		selectSection,
		setFeedSearchTerm,
		setItemSortOrder,
		setPlaybackPlaying,
		startPlaybackFromContext,
		stopPlayback,
		updatePlaybackPosition
	} from '$lib/stores/app.svelte';
	import { onMount } from 'svelte';

	let newFeedUrl = $state('');
	let notice = $state('');
	let isSidebarCollapsed = $state(true);
	let isQueueDrawerOpen = $state(false);
	let readerPaneMode = $state<'feed' | 'reader'>('feed');
	let readerNotice = $state('');
	let lastSelectedItemId = $state<string | null>(null);

	const feeds = $derived(app.feeds);
	const isCreatingFeed = $derived(app.isCreatingFeed);
	const syncingFeedIds = $derived(app.syncingFeedIds);
	const readerLoadingItemIds = $derived(app.readerLoadingItemIds);
	const selectedFeedId = $derived(app.selectedFeedId);
	const selectedItemId = $derived(app.selectedItemId);
	const selectedSection = $derived(app.selectedSection);
	const selectedFeed = $derived(getSelectedFeed());
	const selectedItem = $derived(getSelectedItem());
	const selectedItemFeed = $derived(getSelectedItemFeed());
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentPlaybackState = $derived(app.currentPlaybackState);
	const itemIdsByIndex = $derived(getActiveItemIdsByIndex());
	const totalCount = $derived(getActiveTotalCount());
	const isInitialLoading = $derived(getIsActiveInitialLoading());
	const itemSummariesById = $derived(app.itemSummariesById);
	const feedSearchTerm = $derived(app.feedSearchTerm);
	const itemSortOrder = $derived(app.itemSortOrder);
	const upcomingQueue = $derived(getUpcomingQueue());
	const queueLength = $derived(upcomingQueue.length);
	const manualQueueLength = $derived(getManualQueueLength());

	const isSelectedFeedRefreshing = $derived(
		selectedFeed ? syncingFeedIds.includes(selectedFeed.id) : false
	);

	const isSelectedItemReaderLoading = $derived(
		selectedItem ? readerLoadingItemIds.includes(selectedItem.id) : false
	);

	const hasSelectedItemReaderContent = $derived(selectedItem?.readerStatus === 'ready');
	const isReaderPaneActive = $derived(readerPaneMode === 'reader' && hasSelectedItemReaderContent);
	const canUseReaderMode = $derived(selectedItem ? !selectedItem.mediaEnclosure : false);

	$effect(() => {
		if (selectedItemId !== lastSelectedItemId) {
			readerPaneMode = 'feed';
			readerNotice = '';
			lastSelectedItemId = selectedItemId;
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

			notice = error instanceof Error ? error.message : 'Unable to load article details.';
		});
	});

	function toggleSidebar() {
		isSidebarCollapsed = !isSidebarCollapsed;
	}

	onMount(() => {
		void initializeApp();
	});

	async function handleAddFeed() {
		const candidateUrl = newFeedUrl.trim();

		if (!candidateUrl) {
			notice = 'Enter a feed URL to add a source.';
			return;
		}

		try {
			await createFeed(candidateUrl);
			newFeedUrl = '';
			notice = 'Feed loaded and saved locally.';
		} catch (error: unknown) {
			notice = error instanceof Error ? error.message : 'Unable to add that feed.';
		}
	}

	async function handleRefreshFeed(feedId: string) {
		notice = '';

		try {
			await refreshExistingFeed(feedId);
			notice = 'Feed refreshed.';
		} catch (error: unknown) {
			notice = error instanceof Error ? error.message : 'Unable to refresh that feed.';
		}
	}

	async function handleLoadReaderView(itemId: string) {
		readerNotice = '';

		try {
			const updatedItem = await loadReaderView(itemId);

			if (updatedItem.readerStatus === 'ready') {
				readerPaneMode = 'reader';
				return;
			}

			readerPaneMode = 'feed';
			readerNotice = 'Reader view was unavailable for this item. Showing feed content instead.';
		} catch (error: unknown) {
			readerPaneMode = 'feed';
			readerNotice =
				error instanceof Error ? error.message : 'Unable to load reader view for this item.';
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
				onRemoveFeed={deleteFeed}
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
				<main class="flex min-h-0 flex-1 flex-col bg-surface-main">
					<header
						class="flex h-16 items-center border-b border-border bg-surface-glass px-6 py-10 backdrop-blur lg:px-8"
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
								<button
									class="btn-primary rounded-2xl px-4 py-3"
									disabled={isCreatingFeed}
									type="submit"
								>
									{isCreatingFeed ? 'Adding...' : 'Add feed'}
								</button>
							</form>
						</div>

						{#if notice}
							<p class="mt-5 text-sm text-fg-secondary">{notice}</p>
						{/if}
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
						<section class="flex-2 overflow-y-auto px-6 py-8 lg:px-8">
							<div class="max-w-4xl rounded-3xl border border-border bg-surface-card p-6 shadow-sm">
								<p class="text-sm font-medium tracking-[0.18em] text-fg-muted uppercase">
									Settings
								</p>
								<h1 class="mt-3 text-2xl font-semibold text-fg">Foundation-only for now</h1>
								<p class="mt-4 text-sm leading-6 text-fg-secondary">
									The UI still talks to the same frontend service layer, but feed ingestion and
									persistence now run through Tauri commands backed by local SQLite.
								</p>
							</div>
						</section>
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
									onPlay={startPlaybackFromContext}
									onPlayNext={playAudioItemNext}
									onEnqueue={enqueueAudioItem}
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
								{currentAudioItem}
								{currentPlaybackState}
								{readerPaneMode}
								{readerNotice}
								{isSelectedItemReaderLoading}
								{hasSelectedItemReaderContent}
								{isReaderPaneActive}
								{canUseReaderMode}
								onPlay={startPlaybackFromContext}
								onPlayNext={playAudioItemNext}
								onEnqueue={enqueueAudioItem}
								onStopPlayback={stopPlayback}
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
					playbackState={currentPlaybackState}
					onPlayingChange={setPlaybackPlaying}
					onPositionChange={updatePlaybackPosition}
					onPositionPersist={persistPlaybackPosition}
					onTransitionPersist={persistPlaybackForItem}
					onEnded={() => void handlePlaybackEnded()}
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
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="size-5"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M3.75 12h16.5m-16.5 3.75h16.5M3.75 19.5h16.5M5.625 4.5h12.75a1.875 1.875 0 0 1 0 3.75H5.625a1.875 1.875 0 0 1 0-3.75Z"
								/>
							</svg>
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
