<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
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
		app,
		clearQueue,
		createFeed,
		createStation,
		deleteExistingStation,
		ensureVisibleRangeLoaded,
		getActiveItemIdsByIndex,
		getActiveTotalCount,
		getCurrentAudioItem,
		getCurrentAudioItemFeed,
		getEffectiveSortOrder,
		getIsActiveInitialLoading,
		getManualQueueLength,
		getReaderRequestItemId,
		getSelectedFeed,
		getSelectedItem,
		getSelectedItemFeed,
		getSelectedStation,
		getUpcomingQueue,
		initializeApp,
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
		setItemSortOrder,
		updateExistingStation,
		getReaderRequestSeq
	} from '$lib/stores/app.svelte';
	import { isMediaItem } from '$lib/types/rss';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	let isSidebarCollapsed = $state(true);
	let isQueueDrawerOpen = $state(false);
	let readerPaneMode = $state<'feed' | 'reader'>('feed');
	let isStationEditorOpen = $state(false);
	let editingStation = $state<import('$lib/types/rss').Station | null>(null);

	const {
		feeds,
		stations,
		isCreatingFeed,
		syncingFeedIds,
		readerLoadingItemIds,
		selectedFeedId,
		selectedItemId,
		selectedSection,
		selectedStationId,
		currentPlaybackState,
		itemSummariesById,
		feedSearchTerm
	} = $derived.by(() => app);

	const selectedFeed = $derived(getSelectedFeed());
	const selectedStation = $derived(getSelectedStation());
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

	onMount(() => void initializeApp());

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

	function handleStationDelete() {
		if (!selectedStationId) return;
		return async () => {
			try {
				await deleteExistingStation(selectedStationId);
				toast.success('Station deleted.');
			} catch (error: unknown) {
				toast.error(error instanceof Error ? error.message : 'Unable to delete station.');
			}
		};
	}

	function handlePlayStation() {
		if (!selectedStationId) return;
		return async () => {
			try {
				await playStation(selectedStationId);
			} catch (error: unknown) {
				toast.error(error instanceof Error ? error.message : 'Unable to play station.');
			}
		};
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
		if (currentAudioItem) {
			selectFeed(currentAudioItem.feedId);
			selectItem(currentAudioItem.id);
		}
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
									onDeleteStation={handleStationDelete()}
									onEditStation={handleEditStation}
									onMarkRead={markItemRead}
									onPlayStation={handlePlayStation()}
									onRefresh={handleRefreshFeed}
									onSearchChange={setFeedSearchTerm}
									onSelectItem={selectItem}
									onSortOrderChange={setItemSortOrder}
									onVisibleRangeChange={ensureVisibleRangeLoaded}
									searchTerm={feedSearchTerm}
									{isInitialLoading}
									{itemSortOrder}
									{selectedFeed}
									{selectedItemId}
									{selectedSection}
									{selectedStation}
									{totalCount}
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
</div>
