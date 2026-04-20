<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import CoverView from '$lib/components/CoverView.svelte';
	import EmptyFeedView from '$lib/components/feed/EmptyFeedView.svelte';
	import FeedListView from '$lib/components/FeedListView.svelte';
	import Header from '$lib/components/Header.svelte';
	import QueueDrawer from '$lib/components/QueueDrawer.svelte';
	import QueueToggleButton from '$lib/components/QueueToggleButton.svelte';
	import ReaderPane from '$lib/components/ReaderPane.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import SidebarContainer from '$lib/components/SidebarContainer.svelte';
	import StationEditor from '$lib/components/StationEditor.svelte';
	import {
		feedsState,
		stationsState,
		itemsState,
		playbackState,
		readerState,
		selection,
		getActiveQueryKey,
		clearQueue,
		createFeed,
		createStation,
		deleteExistingStation,
		ensureItemLoaded,
		ensureVisibleRangeLoaded,
		getActiveItemIdsByIndex,
		getActiveTotalCount,
		getCurrentAudioItem,
		getCurrentAudioItemFeed,
		getEffectiveSortOrder,
		getIsActiveInitialLoading,
		getManualQueueLength,
		getPlaybackContext,
		getReaderRequestItemId,
		getReaderRequestSeq,
		getSelectedFeed,
		getSelectedItem,
		getSelectedStation,
		getUpcomingQueue,
		initializeApp,
		loadInitialItemsPage,
		loadItemDetails,
		loadReaderView,
		markItemRead,
		moveQueuedItemDown,
		moveQueuedItemUp,
		playStation,
		refreshExistingFeed,
		removeQueuedItem,
		selectFeed,
		selectItem,
		selectSection,
		selectStation,
		setFeedSearchTerm,
		setFeedSortOrder,
		updateExistingStation
	} from '$lib/stores/app.svelte';
	import { isMediaItem } from '$lib/types/rss';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	// ---------------------------------------------------------------------------
	// Local UI state (not in shared stores)
	// ---------------------------------------------------------------------------
	let isSidebarCollapsed = $state(true);
	let isQueueDrawerOpen = $state(false);
	let readerPaneMode = $state<'feed' | 'reader'>('feed');
	let playerMode = $state<'default' | 'cover' | 'mini'>('default');
	let isStationEditorOpen = $state(false);
	let editingStation = $state<import('$lib/types/rss').Station | null>(null);
	let scrollToItemRequest = $state<{ itemId: string; seq: number } | null>(null);
	let scrollRequestSeq = 0;
	let lastQueryKey = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived state from stores
	// ---------------------------------------------------------------------------
	const feeds = $derived(feedsState.feeds);
	const stations = $derived(stationsState.stations);
	const isCreatingFeed = $derived(feedsState.isCreatingFeed);
	const syncingFeedIds = $derived(feedsState.syncingFeedIds);
	const readerLoadingItemIds = $derived(readerState.readerLoadingItemIds);
	const selectedFeedId = $derived(selection.selectedFeedId);
	const selectedItemId = $derived(selection.selectedItemId);
	const selectedSection = $derived(selection.selectedSection);
	const selectedStationId = $derived(selection.selectedStationId);
	const currentPlaybackState = $derived(playbackState.currentPlaybackState);
	const itemSummariesById = $derived(itemsState.itemSummariesById);
	const feedSearchTerm = $derived(selection.feedSearchTerm);

	// Computed selectors
	const selectedFeed = $derived(getSelectedFeed(feeds));
	const selectedStation = $derived(getSelectedStation(stations));
	const selectedItem = $derived(getSelectedItem());
	const selectedItemFeed = $derived(
		selectedItem ? (feeds.find((f) => f.id === selectedItem.feedId) ?? null) : null
	);
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentAudioItemFeed = $derived(getCurrentAudioItemFeed());
	const itemIdsByIndex = $derived(getActiveItemIdsByIndex());
	const totalCount = $derived(getActiveTotalCount());
	const isInitialLoading = $derived(getIsActiveInitialLoading());
	const itemSortOrder = $derived(getEffectiveSortOrder());
	const upcomingQueue = $derived(getUpcomingQueue());
	const queueLength = $derived(upcomingQueue.length);
	const manualQueueLength = $derived(getManualQueueLength());
	const readerRequestSeq = $derived(getReaderRequestSeq());

	const isSelectedFeedRefreshing = $derived(
		selectedFeed ? syncingFeedIds.includes(selectedFeed.id) : false
	);

	const isSelectedItemReaderLoading = $derived(
		selectedItem ? readerLoadingItemIds.includes(selectedItem.id) : false
	);

	const hasSelectedItemReaderContent = $derived(selectedItem?.readerStatus === 'ready');
	const isReaderPaneActive = $derived(readerPaneMode === 'reader' && hasSelectedItemReaderContent);
	const canUseReaderMode = $derived(selectedItem ? !isMediaItem(selectedItem) : false);

	// ---------------------------------------------------------------------------
	// Effects
	// ---------------------------------------------------------------------------

	// Load items when the active query changes (feed/station/section/search selection)
	$effect(() => {
		const queryKey = getActiveQueryKey();
		if (queryKey && queryKey !== lastQueryKey) {
			lastQueryKey = queryKey;
			void loadInitialItemsPage().catch((error: unknown) => {
				console.error('Failed to load items:', error);
			});
		}
	});

	// Reset reader mode when item changes
	$effect(() => {
		if (selectedItemId) {
			readerPaneMode = 'feed';
		}
	});

	// Load item details when selection changes
	$effect(() => {
		if (!selectedItemId) return;

		void loadItemDetails(selectedItemId).catch((error: unknown) => {
			toast.error(error instanceof Error ? error.message : 'Unable to load article details.');
		});
	});

	// Initialize app on mount
	onMount(() => void initializeApp());

	// Handle reader view requests from context menu
	let lastConsumedReaderSeq = 0;
	$effect(() => {
		if (readerRequestSeq > lastConsumedReaderSeq) {
			lastConsumedReaderSeq = readerRequestSeq;
			const itemId = getReaderRequestItemId();
			if (itemId) void handleLoadReaderView(itemId);
		}
	});

	// ---------------------------------------------------------------------------
	// Event handlers
	// ---------------------------------------------------------------------------

	async function handleAddFeed(url: string) {
		try {
			await createFeed(url);
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
			readerPaneMode = updatedItem.readerStatus === 'ready' ? 'reader' : 'feed';
			if (updatedItem.readerStatus !== 'ready') {
				toast.warning('Reader view was unavailable for this item. Showing feed content instead.');
			}
		} catch (error: unknown) {
			readerPaneMode = 'feed';
			toast.error(
				error instanceof Error ? error.message : 'Unable to load reader view for this item.'
			);
		}
	}

	async function handleStationSave(input: import('$lib/types/rss').CreateStationInput) {
		try {
			if (editingStation) {
				await updateExistingStation({
					id: editingStation.id,
					name: input.name,
					feedIds: input.feedIds,
					episodeFilter: input.episodeFilter,
					sortOrder: input.sortOrder
				});
				toast.success('Station updated.');
			} else {
				await createStation(input);
				toast.success('Station created.');
			}
			isStationEditorOpen = false;
			editingStation = null;
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to save station.');
		}
	}

	async function handleStationDelete() {
		if (!selectedStationId) return;

		try {
			await deleteExistingStation(selectedStationId);
			toast.success('Station deleted.');
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to delete station.');
		}
	}

	async function handlePlayStation() {
		if (!selectedStationId) return;

		try {
			await playStation(selectedStationId);
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to play station.');
		}
	}

	function handleEditStation() {
		editingStation = selectedStation;
		isStationEditorOpen = true;
	}

	function handleCreateStation() {
		editingStation = null;
		isStationEditorOpen = true;
	}

	function handleNavigateToItem() {
		if (!currentAudioItem) return;

        if (playerMode === 'cover') {
            playerMode = 'default';
        }

		const context = getPlaybackContext();

		if (context?.contextType === 'station') {
			selectStation(context.id);
			selectItem(currentAudioItem.id);
		} else {
			// Default to feed context (either from context or fallback to item's feed)
			const feedId = context?.contextType === 'feed' ? context.id : currentAudioItem.feedId;
			selectFeed(feedId);
			selectItem(currentAudioItem.id);
		}

		scrollRequestSeq += 1;
		scrollToItemRequest = { itemId: currentAudioItem.id, seq: scrollRequestSeq };
	}
</script>

<svelte:head>
	<title>JRSS</title>
	<meta name="description" content="RSS reader and podcast player." />
</svelte:head>

<StationEditor
	open={isStationEditorOpen}
	station={editingStation}
	{feeds}
	onSave={handleStationSave}
	onClose={() => {
		isStationEditorOpen = false;
		editingStation = null;
	}}
/>

<div class="h-screen overflow-hidden bg-surface-shell">
	{#if playerMode === 'cover'}
		<CoverView
			item={currentAudioItem}
			imageUrl={currentAudioItemFeed?.imageUrl}
			playbackState={currentPlaybackState}
			onNavigateToItem={handleNavigateToItem}
		/>
	{:else}
		<QueueDrawer
			open={isQueueDrawerOpen}
			queueItems={upcomingQueue}
			{manualQueueLength}
			{feeds}
			onRemoveItem={removeQueuedItem}
			onMoveItemUp={moveQueuedItemUp}
			onMoveItemDown={moveQueuedItemDown}
			onClearQueue={clearQueue}
			onClose={() => (isQueueDrawerOpen = false)}
		/>

		<div class="relative h-full overflow-hidden">
			<SidebarContainer
				{feeds}
				{stations}
				{selectedFeedId}
				{selectedStationId}
				{selectedSection}
				onSelectFeed={selectFeed}
				onSelectSection={selectSection}
				onSelectStation={selectStation}
				onToggleCollapse={() => (isSidebarCollapsed = !isSidebarCollapsed)}
				onCreateStation={handleCreateStation}
				refreshingFeedIds={syncingFeedIds}
				isCollapsed={isSidebarCollapsed}
			/>

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
								<Header isLoading={isCreatingFeed} onSubmit={handleAddFeed} />
							</div>
						</header>

						{#if feeds.length === 0 && !isInitialLoading}
							<EmptyFeedView />
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
										onDeleteStation={handleStationDelete}
										onEditStation={handleEditStation}
										onEnsureItemLoaded={ensureItemLoaded}
										onMarkRead={markItemRead}
										onPlayStation={handlePlayStation}
										onRefresh={handleRefreshFeed}
										onSearchChange={setFeedSearchTerm}
										onSelectItem={selectItem}
										onSortOrderChange={setFeedSortOrder}
										onVisibleRangeChange={ensureVisibleRangeLoaded}
										searchTerm={feedSearchTerm}
										{isInitialLoading}
										{itemSortOrder}
										{selectedFeed}
										{selectedItemId}
										{selectedSection}
										{selectedStation}
										{totalCount}
										{scrollToItemRequest}
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
									onReaderPaneModeChange={(mode) => (readerPaneMode = mode)}
								/>
							</div>
						{/if}
					</main>

					<AudioPlayer
						item={currentAudioItem}
						imageUrl={currentAudioItemFeed?.imageUrl}
						playbackState={currentPlaybackState}
						onNavigateToItem={handleNavigateToItem}
						onShowCover={() => (playerMode = 'cover')}
					>
						{#snippet controls()}
							<QueueToggleButton
								isOpen={isQueueDrawerOpen}
								{queueLength}
								onToggle={() => (isQueueDrawerOpen = !isQueueDrawerOpen)}
							/>
						{/snippet}
					</AudioPlayer>
				</div>
			</div>
		</div>
	{/if}
</div>
