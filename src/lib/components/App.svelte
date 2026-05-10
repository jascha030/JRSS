<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { useMenuShortcuts } from '$lib/hooks/useMenuShortcuts.svelte';
	import AudioPlayer from '$lib/components/player/AudioPlayer.svelte';
	import CoverView from '$lib/components/player/CoverView.svelte';
	import EmptyFeedView from '$lib/components/home/EmptyFeedView.svelte';
	import ItemListView from '$lib/components/content/ItemListView.svelte';
	import HomeView from '$lib/components/home/HomeView.svelte';
	import Header from '$lib/components/navigation/Header.svelte';
	import QueueDrawer from '$lib/components/player/QueueDrawer.svelte';
	import QueueToggleButton from '$lib/components/player/QueueToggleButton.svelte';
	import ReaderPane from '$lib/components/content/ReaderPane.svelte';
	import SettingsView from '$lib/components/settings/SettingsView.svelte';
	import Sidebar from '$lib/components/navigation/Sidebar.svelte';
	import FeedEditor from '$lib/components/feed/FeedEditor.svelte';
	import FeedInspector from '$lib/components/feed/FeedInspector.svelte';
	import StationEditor from '$lib/components/station/StationEditor.svelte';
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
		getPlaybackContext,
		getPlaybackHistory,
		getReaderRequestItemId,
		getReaderRequestSeq,
		getSelectedFeed,
		getSelectedItem,
		getSelectedStation,
		inspectorState,
		openInspector,
		getUpcomingQueue,
		loadInitialItemsPage,
		loadItemDetails,
		loadReaderView,
		markItemRead,
		moveQueuedItemDown,
		moveQueuedItemUp,
		playStation,
		refreshExistingFeed,
		removeQueuedItem,
		requestTogglePlayback,
		selectFeed,
		selectItem,
		selectSection,
		selectStation,
		closeInspector,
		setFeedSearchTerm,
		setStationSearchTerm,
		setSectionSearchTerm,
		setFeedSortOrder,
		updateExistingStation
	} from '$lib/state';
	import { isMediaItem } from '$lib/types/item';
	import { openMiniPlayer, MINI_WINDOW_LABEL } from '$lib/utils/tauri-window';
	import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	let isSidebarCollapsed = $state(true);
	let isQueueDrawerOpen = $state(false);
	let readerPaneMode = $state<'feed' | 'reader'>('feed');
	let playerMode = $state<'default' | 'cover'>('default');
	let isFeedEditorOpen = $state(false);
	let isStationEditorOpen = $state(false);
	let editingStation = $state<import('$lib/types/station').Station | null>(null);
	let scrollToItemRequest = $state<{ itemId: string; seq: number } | null>(null);
	let scrollRequestSeq = 0;
	let lastQueryKey = $state<string | null>(null);

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
	const stationSearchTerm = $derived(selection.stationSearchTerm);
	const sectionSearchTerm = $derived(selection.sectionSearchTerm);

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
	const playbackHistory = $derived(getPlaybackHistory());
	const upcomingQueue = $derived(getUpcomingQueue());
	const queueLength = $derived(upcomingQueue.length);
	const readerRequestSeq = $derived(getReaderRequestSeq());

	const isSelectedFeedRefreshing = $derived(
		selectedFeed ? syncingFeedIds.includes(selectedFeed.id) : false
	);

	const isInspectorActive = $derived(inspectorState.activeFeedId !== null);

	const isSelectedItemReaderLoading = $derived(
		selectedItem ? readerLoadingItemIds.includes(selectedItem.id) : false
	);

	const hasSelectedItemReaderContent = $derived(selectedItem?.readerStatus === 'ready');
	const isReaderPaneActive = $derived(readerPaneMode === 'reader' && hasSelectedItemReaderContent);
	const canUseReaderMode = $derived(selectedItem ? !isMediaItem(selectedItem) : false);

	$effect(() => {
		const queryKey = getActiveQueryKey();
		if (queryKey && queryKey !== lastQueryKey) {
			lastQueryKey = queryKey;
			void loadInitialItemsPage().catch((error: unknown) => {
				console.error('Failed to load items:', error);
			});
		}
	});

	$effect(() => {
		if (selectedItemId) {
			readerPaneMode = 'feed';
		}
	});

	$effect(() => {
		if (!selectedItemId) return;

		void loadItemDetails(selectedItemId).catch((error: unknown) => {
			toast.error(error instanceof Error ? error.message : 'Unable to load article details.');
		});
	});

	let lastConsumedReaderSeq = 0;
	$effect(() => {
		if (readerRequestSeq > lastConsumedReaderSeq) {
			lastConsumedReaderSeq = readerRequestSeq;
			const itemId = getReaderRequestItemId();
			if (itemId) void handleLoadReaderView(itemId);
		}
	});

	async function handleAddFeed(url: string) {
		try {
			await createFeed(url);
			isFeedEditorOpen = false;
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

	async function handleStationSave(input: import('$lib/types/station').CreateStationInput) {
		try {
			if (editingStation) {
				await updateExistingStation({
					id: editingStation.id,
					name: input.name,
					feedIds: input.feedIds,
					episodeFilter: input.episodeFilter,
					sortOrder: input.sortOrder,
					gradient: input.gradient
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

	async function handleOpenInspector(feedId: string) {
		await openInspector(feedId);
	}

	function handleCreateStation() {
		editingStation = null;
		isStationEditorOpen = true;
	}

	function handleSelectFeed(feedId: string | null) {
		closeInspector();
		selectFeed(feedId);
	}

	function handleSelectSection(section: import('$lib/state').SidebarSection) {
		closeInspector();
		selectSection(section);
	}

	function handleSelectStation(stationId: string) {
		closeInspector();
		selectStation(stationId);
	}

	function handleSelectSearchResult(item: import('$lib/types/item').FeedListItem): void {
		if (playerMode === 'cover') {
			playerMode = 'default';
		}
		closeInspector();
		selectFeed(item.feedId);
		selectItem(item.id);
		scrollRequestSeq += 1;
		scrollToItemRequest = { itemId: item.id, seq: scrollRequestSeq };
	}

	function handleNavigateToItem() {
		if (!currentAudioItem) return;

		if (playerMode === 'cover') {
			playerMode = 'default';
		}

		const context = getPlaybackContext();

		closeInspector();

		if (context?.contextType === 'station') {
			selectStation(context.id);
			selectItem(currentAudioItem.id);
		} else {
			const feedId = context?.contextType === 'feed' ? context.id : currentAudioItem.feedId;
			selectFeed(feedId);
			selectItem(currentAudioItem.id);
		}

		scrollRequestSeq += 1;
		scrollToItemRequest = { itemId: currentAudioItem.id, seq: scrollRequestSeq };
	}

	async function handlePopOutMiniPlayer() {
		const miniWindow = await WebviewWindow.getByLabel(MINI_WINDOW_LABEL);
		if (miniWindow && (await miniWindow.isVisible())) {
			return;
		}

		try {
			await openMiniPlayer();
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to open mini player.');
		}
	}

	function handleCycleSource(direction: 1 | -1) {
		const sources = [
			...feeds.map((f) => ({ type: 'feed' as const, id: f.id })),
			...stations.map((s) => ({ type: 'station' as const, id: s.id }))
		];
		if (sources.length === 0) return;

		let currentIndex = -1;
		if (selectedFeedId) {
			currentIndex = sources.findIndex((s) => s.type === 'feed' && s.id === selectedFeedId);
		} else if (selectedStationId) {
			currentIndex = sources.findIndex((s) => s.type === 'station' && s.id === selectedStationId);
		}

		if (currentIndex === -1) {
			currentIndex = direction === 1 ? 0 : sources.length - 1;
		} else {
			currentIndex = (currentIndex + direction + sources.length) % sources.length;
		}

		const next = sources[currentIndex];
		closeInspector();
		if (next.type === 'feed') {
			selectFeed(next.id);
		} else {
			selectStation(next.id);
		}
	}

	useMenuShortcuts([
		{
			event: 'menu-settings',
			handler: () => {
				playerMode = 'default';
				selectSection('settings');
			}
		},
		{
			event: 'menu-toggle-sidebar',
			handler: () => {
				isSidebarCollapsed = !isSidebarCollapsed;
			}
		},
		{
			event: 'menu-next-source',
			handler: () => {
				if (playerMode === 'cover') return;
				handleCycleSource(1);
			}
		},
		{
			event: 'menu-prev-source',
			handler: () => {
				if (playerMode === 'cover') return;
				handleCycleSource(-1);
			}
		},
		{
			event: 'menu-toggle-cover',
			handler: () => {
				playerMode = playerMode === 'cover' ? 'default' : 'cover';
			}
		},
		{
			event: 'menu-toggle-mini-player',
			handler: async () => {
				await handlePopOutMiniPlayer();
			}
		}
	]);

	onMount(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key !== ' ') return;

			const target = e.target;
			if (
				target instanceof HTMLInputElement ||
				target instanceof HTMLTextAreaElement ||
				(target instanceof HTMLElement && target.isContentEditable)
			) {
				return;
			}

			if (!playbackState.currentPlaybackState) {
				return;
			}

			e.preventDefault();
			requestTogglePlayback();
		};

		document.addEventListener('keydown', handleKeyDown);

		return () => {
			document.removeEventListener('keydown', handleKeyDown);
		};
	});
</script>

<FeedEditor
	open={isFeedEditorOpen}
	isLoading={isCreatingFeed}
	onSave={handleAddFeed}
	onClose={() => (isFeedEditorOpen = false)}
/>

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
			onClose={() => (playerMode = 'default')}
			onPopOut={handlePopOutMiniPlayer}
			historyItems={playbackHistory}
			queueItems={upcomingQueue}
			{feeds}
			onRemoveQueueItem={removeQueuedItem}
			onMoveQueueItemUp={moveQueuedItemUp}
			onMoveQueueItemDown={moveQueuedItemDown}
			onClearQueue={clearQueue}
		/>
	{:else}
		<AppBar class="top-0 z-9999 h-12 bg-transparent! p-0">
			<AppBar.Toolbar
				class="flex h-12 w-full content-center border-b border-border bg-surface-sidebar p-0"
			>
				<div
					data-tauri-drag-region
					class="flex h-12 w-full items-center justify-end py-0 pr-4 pl-44 align-middle"
				>
					<Header
						onOpenDialog={() => (isFeedEditorOpen = true)}
						{feeds}
						onSelectResult={handleSelectSearchResult}
					/>
				</div>
			</AppBar.Toolbar>
		</AppBar>

		<QueueDrawer
			open={isQueueDrawerOpen}
			historyItems={playbackHistory}
			queueItems={upcomingQueue}
			{feeds}
			onRemoveItem={removeQueuedItem}
			onMoveItemUp={moveQueuedItemUp}
			onMoveItemDown={moveQueuedItemDown}
			onClearQueue={clearQueue}
			onClose={() => (isQueueDrawerOpen = false)}
		/>

		<div class="relative h-[calc(100%-54px)] overflow-hidden">
			<Sidebar
				{feeds}
				{stations}
				{selectedFeedId}
				{selectedStationId}
				{selectedSection}
				onSelectFeed={handleSelectFeed}
				onSelectSection={handleSelectSection}
				onSelectStation={handleSelectStation}
				onToggleCollapse={() => (isSidebarCollapsed = !isSidebarCollapsed)}
				onCreateStation={handleCreateStation}
				onAddFeed={() => (isFeedEditorOpen = true)}
				refreshingFeedIds={syncingFeedIds}
				isCollapsed={isSidebarCollapsed}
			/>

			<div
				class={`relative z-30 h-full transition-[left,width] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] motion-reduce:transition-none md:absolute md:inset-y-0 ${
					isSidebarCollapsed
						? 'md:left-16 md:w-[calc(100%-(var(--spacing)*16))]'
						: 'md:left-60 md:w-[calc(100%-(var(--spacing)*60))]'
				}`}
			>
				<div class="flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden">
					<main class="flex min-h-0 flex-1 flex-col">
						{#if feeds.length === 0 && !isInitialLoading}
							<EmptyFeedView />
						{:else if selectedSection === 'settings'}
							<SettingsView />
						{:else if selectedSection === 'home'}
							<HomeView {feeds} {stations} />
						{:else if isInspectorActive}
							<FeedInspector />
						{:else}
							<div class="flex min-h-0 flex-1 overflow-hidden">
								<div
									class="min-h-0 min-w-0 grow lg:shrink-0 lg:grow-0 lg:basis-1/3 lg:border-r lg:border-border"
								>
									<ItemListView
										{feeds}
										{itemIdsByIndex}
										itemsById={itemSummariesById}
										isRefreshing={isSelectedFeedRefreshing}
										onDeleteStation={handleStationDelete}
										onEditStation={handleEditStation}
										onEnsureItemLoaded={ensureItemLoaded}
										onInspect={handleOpenInspector}
										onMarkRead={markItemRead}
										onPlayStation={handlePlayStation}
										onRefresh={handleRefreshFeed}
										onSearchChange={setFeedSearchTerm}
										onStationSearchChange={setStationSearchTerm}
										onSectionSearchChange={setSectionSearchTerm}
										onSelectItem={selectItem}
										onSortOrderChange={setFeedSortOrder}
										onVisibleRangeChange={ensureVisibleRangeLoaded}
										searchTerm={feedSearchTerm}
										{stationSearchTerm}
										{sectionSearchTerm}
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
