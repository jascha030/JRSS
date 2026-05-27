<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import AudioPlayer from '$lib/components/player/AudioPlayer.svelte';
	import CommandPalette from '$lib/components/command/CommandPalette.svelte';
	import CoverView from '$lib/components/player/CoverView.svelte';
	import EmptyFeedView from '$lib/components/home/EmptyFeedView.svelte';
	import FeedEditor from '$lib/components/feed/FeedEditor.svelte';
	import Header from '$lib/components/navigation/Header.svelte';
	import QueueDrawer from '$lib/components/player/QueueDrawer.svelte';
	import QueueToggleButton from '$lib/components/player/QueueToggleButton.svelte';
	import Sidebar from '$lib/components/navigation/Sidebar.svelte';
	import StationEditor from '$lib/components/station/StationEditor.svelte';
	import { useMenuShortcuts } from '$lib/hooks/useMenuShortcuts.svelte';
	import {
		clearQueue,
		createFeed,
		createStation,
		feedsState,
		getActiveQueryKey,
		getCurrentAudioItem,
		getCurrentAudioItemFeed,
		getIsActiveInitialLoading,
		getPlaybackContext,
		getReaderRequestItemId,
		getReaderRequestSeq,
		getUpcomingQueue,
		loadInitialItemsPage,
		loadItemDetails,
		moveQueuedItemDown,
		moveQueuedItemUp,
		playbackState,
		removeQueuedItem,
		requestTogglePlayback,
		selection,
		stationsState,
		updateExistingStation
	} from '$lib/state';
	import {
		appUi,
		closeFeedEditor,
		closeStationEditor,
		openCommandPalette,
		openFeedEditor,
		openStationEditor,
		requestScrollToItem,
		switchToReaderView,
		togglePlayerMode,
		toggleQueue,
		toggleSidebar
	} from '$lib/hooks/useAppUi.svelte';
	import {
		isAppListSection,
		navigateToFeedItem,
		navigateToHome,
		navigateToFeed,
		navigateToSettings,
		navigateToSection,
		navigateToStationItem,
		navigateToStation
	} from '$lib/navigation/app-router';
	import { popOutMiniPlayer } from '$lib/utils/mini-player';

	let { children } = $props();

	let lastQueryKey = $state<string | null>(null);
	let lastConsumedReaderSeq = 0;

	const feeds = $derived(feedsState.feeds);
	const stations = $derived(stationsState.stations);
	const isCreatingFeed = $derived(feedsState.isCreatingFeed);
	const syncingFeedIds = $derived(feedsState.syncingFeedIds);
	const selectedFeedId = $derived(selection.selectedFeedId);
	const selectedItemId = $derived(selection.selectedItemId);
	const selectedSection = $derived(selection.selectedSection);
	const selectedStationId = $derived(selection.selectedStationId);
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentAudioItemFeed = $derived(getCurrentAudioItemFeed());
	const isInitialLoading = $derived(getIsActiveInitialLoading());
	const upcomingQueue = $derived(getUpcomingQueue());
	const queueLength = $derived(upcomingQueue.length);
	const readerRequestSeq = $derived(getReaderRequestSeq());
	const activeQueryKey = $derived(getActiveQueryKey());
	const isFeedEditorOpen = $derived(appUi.dialog.kind === 'feed-editor');
	const stationEditorId = $derived(
		appUi.dialog.kind === 'station-editor' ? appUi.dialog.stationId : null
	);
	const editingStation = $derived(
		stationEditorId !== null
			? (stations.find((station) => station.id === stationEditorId) ?? null)
			: null
	);
	const shouldShowEmptyFeedView = $derived(feeds.length === 0 && !isInitialLoading);

	$effect(() => {
		const queryKey = activeQueryKey;

		if (!queryKey) {
			lastQueryKey = null;
			return;
		}

		if (queryKey === lastQueryKey) {
			return;
		}

		lastQueryKey = queryKey;
		void loadInitialItemsPage().catch((error: unknown) => {
			console.error('Failed to load items:', error);
		});
	});

	$effect(() => {
		if (!selectedItemId) {
			return;
		}

		void loadItemDetails(selectedItemId).catch((error: unknown) => {
			toast.error(error instanceof Error ? error.message : 'Unable to load article details.');
		});
	});

	$effect(() => {
		if (readerRequestSeq <= lastConsumedReaderSeq) {
			return;
		}

		lastConsumedReaderSeq = readerRequestSeq;
		const itemId = getReaderRequestItemId();

		if (itemId) {
			void switchToReaderView(itemId);
		}
	});

	async function handleAddFeed(url: string) {
		try {
			const createdFeed = await createFeed(url);
			closeFeedEditor();
			await navigateToFeed(createdFeed.id);
			toast.success('Feed loaded and saved locally.');
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to add that feed.');
		}
	}

	async function handleStationSave(input: import('$lib/types/station').CreateStationInput) {
		try {
			const station = editingStation
				? await updateExistingStation({
						id: editingStation.id,
						name: input.name,
						feedIds: input.feedIds,
						episodeFilter: input.episodeFilter,
						sortOrder: input.sortOrder,
						gradient: input.gradient
					})
				: await createStation(input);

			closeStationEditor();
			await navigateToStation(station.id);
			toast.success(editingStation ? 'Station updated.' : 'Station created.');
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to save station.');
		}
	}

	function handleCreateStation() {
		openStationEditor();
	}

	function handleCloseQueue() {
		appUi.isQueueDrawerOpen = false;
	}

	function handleSelectFeed(feedId: string | null) {
		if (feedId === null) {
			void navigateToSection('all');
			return;
		}

		void navigateToFeed(feedId);
	}

	function handleSelectSection(section: import('$lib/state').SidebarSection) {
		if (section === 'home') {
			void navigateToHome();
			return;
		}

		if (section === 'settings') {
			void navigateToSettings();
			return;
		}

		if (section && isAppListSection(section)) {
			void navigateToSection(section);
		}
	}

	function handleSelectStation(stationId: string) {
		void navigateToStation(stationId);
	}

	function handleSelectSearchResult(item: import('$lib/types/item').FeedListItem): void {
		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}

		void navigateToFeedItem(item.feedId, item.id).then(() => {
			requestScrollToItem(item.id);
		});
	}

	function handleSelectFeedSearchResult(feed: import('$lib/types/feed').Feed): void {
		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}

		void navigateToFeed(feed.id);
	}

	function handleSelectStationSearchResult(station: import('$lib/types/station').Station): void {
		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}

		void navigateToStation(station.id);
	}

	function handleNavigateToItem() {
		if (!currentAudioItem) {
			return;
		}

		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}

		const context = getPlaybackContext();
		const station =
			context?.contextType === 'station'
				? stationsState.stations.find((candidate) => candidate.id === context.id)
				: null;

		if (station && station.feedIds.includes(currentAudioItem.feedId)) {
			void navigateToStationItem(station.id, currentAudioItem.id).then(() => {
				requestScrollToItem(currentAudioItem.id);
			});
			return;
		}

		void navigateToFeedItem(currentAudioItem.feedId, currentAudioItem.id).then(() => {
			requestScrollToItem(currentAudioItem.id);
		});
	}

	function handleCycleSource(direction: 1 | -1) {
		const sources = [
			...feeds.map((feed) => ({ type: 'feed' as const, id: feed.id })),
			...stations.map((station) => ({ type: 'station' as const, id: station.id }))
		];

		if (sources.length === 0) {
			return;
		}

		let currentIndex = -1;

		if (selectedFeedId) {
			currentIndex = sources.findIndex(
				(source) => source.type === 'feed' && source.id === selectedFeedId
			);
		} else if (selectedStationId) {
			currentIndex = sources.findIndex(
				(source) => source.type === 'station' && source.id === selectedStationId
			);
		}

		if (currentIndex === -1) {
			currentIndex = direction === 1 ? 0 : sources.length - 1;
		} else {
			currentIndex = (currentIndex + direction + sources.length) % sources.length;
		}

		const next = sources[currentIndex];

		if (next.type === 'feed') {
			void navigateToFeed(next.id);
			return;
		}

		void navigateToStation(next.id);
	}

	useMenuShortcuts([
		{
			event: 'menu-settings',
			handler: () => {
				appUi.playerMode = 'default';
				void navigateToSettings();
			}
		},
		{
			event: 'menu-toggle-sidebar',
			handler: toggleSidebar
		},
		{
			event: 'menu-next-source',
			handler: () => {
				if (appUi.playerMode === 'cover') {
					return;
				}

				handleCycleSource(1);
			}
		},
		{
			event: 'menu-prev-source',
			handler: () => {
				if (appUi.playerMode === 'cover') {
					return;
				}

				handleCycleSource(-1);
			}
		},
		{
			event: 'menu-toggle-cover',
			handler: togglePlayerMode
		},
		{
			event: 'menu-toggle-mini-player',
			handler: async () => {
				await popOutMiniPlayer();
			}
		}
	]);

	onMount(() => {
		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key !== ' ') {
				return;
			}

			const target = event.target;
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

			event.preventDefault();
			requestTogglePlayback();
		};

		const handleCommandPaletteKeyDown = (event: KeyboardEvent) => {
			const isMod = event.metaKey || event.ctrlKey;
			if (!isMod || event.key !== 'k') {
				return;
			}

			const target = event.target;
			if (
				target instanceof HTMLInputElement ||
				target instanceof HTMLTextAreaElement ||
				(target instanceof HTMLElement && target.isContentEditable)
			) {
				return;
			}

			event.preventDefault();
			openCommandPalette();
		};

		document.addEventListener('keydown', handleKeyDown);
		document.addEventListener('keydown', handleCommandPaletteKeyDown);

		return () => {
			document.removeEventListener('keydown', handleKeyDown);
			document.removeEventListener('keydown', handleCommandPaletteKeyDown);
		};
	});
</script>

<FeedEditor
	open={isFeedEditorOpen}
	isLoading={isCreatingFeed}
	onSave={handleAddFeed}
	onClose={closeFeedEditor}
/>

<StationEditor
	open={appUi.dialog.kind === 'station-editor'}
	station={editingStation}
	{feeds}
	onSave={handleStationSave}
	onClose={closeStationEditor}
/>

<CommandPalette />

<div class="h-screen overflow-hidden bg-surface-shell">
	{#if appUi.playerMode === 'cover'}
		<CoverView
			item={currentAudioItem}
			imageUrl={currentAudioItem?.imageUrl}
			onNavigateToItem={handleNavigateToItem}
			onClose={() => (appUi.playerMode = 'default')}
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
						onOpenDialog={openFeedEditor}
						onSelectResult={handleSelectSearchResult}
						onSelectFeedResult={handleSelectFeedSearchResult}
						onSelectStationResult={handleSelectStationSearchResult}
						onAddFeed={handleAddFeed}
					/>
				</div>
			</AppBar.Toolbar>
		</AppBar>

		<QueueDrawer
			open={appUi.isQueueDrawerOpen}
			onRemoveItem={removeQueuedItem}
			onMoveItemUp={moveQueuedItemUp}
			onMoveItemDown={moveQueuedItemDown}
			onClearQueue={clearQueue}
			onClose={handleCloseQueue}
		/>

		<div class="flex h-[calc(100%-54px)] overflow-hidden">
			<div
				class={`hidden shrink-0 overflow-hidden motion-reduce:transition-none md:block md:transition-[width] md:duration-300 md:ease-[cubic-bezier(0.22,1,0.36,1)] ${
					appUi.isSidebarCollapsed ? 'md:w-16' : 'md:w-60'
				}`}
			>
				<Sidebar
					{feeds}
					{stations}
					{selectedFeedId}
					{selectedStationId}
					{selectedSection}
					onSelectFeed={handleSelectFeed}
					onSelectSection={handleSelectSection}
					onSelectStation={handleSelectStation}
					onToggleCollapse={toggleSidebar}
					onCreateStation={handleCreateStation}
					onAddFeed={openFeedEditor}
					refreshingFeedIds={syncingFeedIds}
					isCollapsed={appUi.isSidebarCollapsed}
				/>
			</div>

			<div class="relative z-30 min-w-0 flex-1">
				<div class="flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden">
					<main class="relative flex min-h-0 flex-1 flex-col">
						{#if shouldShowEmptyFeedView}
							<EmptyFeedView />
						{:else}
							{@render children?.()}
						{/if}
					</main>

					<AudioPlayer
						item={currentAudioItem}
						imageUrl={currentAudioItem?.imageUrl ?? currentAudioItemFeed?.imageUrl}
						onNavigateToItem={handleNavigateToItem}
						onShowCover={() => (appUi.playerMode = 'cover')}
					>
						{#snippet controls()}
							<QueueToggleButton
								isOpen={appUi.isQueueDrawerOpen}
								{queueLength}
								onToggle={toggleQueue}
							/>
						{/snippet}
					</AudioPlayer>
				</div>
			</div>
		</div>
	{/if}
</div>
