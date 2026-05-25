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
	import StationEditor from '$lib/components/station/StationEditor.svelte';
	import CommandPalette from '$lib/components/command/CommandPalette.svelte';
	import {
		feedsState,
		stationsState,
		playbackState,
		selection,
		getActiveQueryKey,
		clearQueue,
		createFeed,
		createStation,
		getCurrentAudioItem,
		getCurrentAudioItemFeed,
		getIsActiveInitialLoading,
		getPlaybackContext,
		getReaderRequestItemId,
		getReaderRequestSeq,
		inspectorState,
		getUpcomingQueue,
		loadInitialItemsPage,
		loadItemDetails,
		loadReaderView,
		moveQueuedItemDown,
		moveQueuedItemUp,
		removeQueuedItem,
		requestTogglePlayback,
		selectFeed,
		selectItem,
		selectSection,
		selectStation,
		closeInspector,
		updateExistingStation
	} from '$lib/state';
	import { appUi, requestScrollToItem } from '$lib/hooks/useAppUi.svelte';
	import { popOutMiniPlayer } from '$lib/utils/mini-player';

	import { onMount } from 'svelte';
	import type FeedInspectorComponent from '$lib/components/feed/FeedInspector.svelte';
	import { toast } from 'svelte-sonner';

	let isSidebarCollapsed = $state(true);
	let isQueueDrawerOpen = $state(false);
	let FeedInspector = $state<typeof FeedInspectorComponent | null>(null);
	let playerMode = $state<'default' | 'cover'>('default');
	let isFeedEditorOpen = $state(false);
	let isStationEditorOpen = $state(false);
	let editingStation = $state<import('$lib/types/station').Station | null>(null);
	let lastQueryKey = $state<string | null>(null);

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

	const isInspectorActive = $derived(inspectorState.activeFeedId !== null);

	$effect(() => {
		if (isInspectorActive && FeedInspector === null) {
			void import('$lib/components/feed/FeedInspector.svelte').then((m) => {
				FeedInspector = m.default;
			});
		}
	});

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
			appUi.readerPaneMode = 'feed';
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

	async function handleLoadReaderView(itemId: string) {
		try {
			const updatedItem = await loadReaderView(itemId);
			appUi.readerPaneMode = updatedItem.readerStatus === 'ready' ? 'reader' : 'feed';
			if (updatedItem.readerStatus !== 'ready') {
				toast.warning('Reader view was unavailable for this item. Showing feed content instead.');
			}
		} catch (error: unknown) {
			appUi.readerPaneMode = 'feed';
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
		requestScrollToItem(item.id);
	}

	function handleSelectFeedSearchResult(feed: import('$lib/types/feed').Feed): void {
		if (playerMode === 'cover') {
			playerMode = 'default';
		}
		closeInspector();
		selectFeed(feed.id);
	}

	function handleSelectStationSearchResult(station: import('$lib/types/station').Station): void {
		if (playerMode === 'cover') {
			playerMode = 'default';
		}
		closeInspector();
		selectStation(station.id);
	}

	function handleNavigateToItem() {
		if (!currentAudioItem) return;

		if (playerMode === 'cover') {
			playerMode = 'default';
		}

		const context = getPlaybackContext();
		const station =
			context?.contextType === 'station'
				? stationsState.stations.find((s) => s.id === context.id)
				: null;

		closeInspector();

		if (station && station.feedIds.includes(currentAudioItem.feedId)) {
			selectStation(station.id);
			selectItem(currentAudioItem.id);
		} else {
			selectFeed(currentAudioItem.feedId);
			selectItem(currentAudioItem.id);
		}

		requestScrollToItem(currentAudioItem.id);
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
				await popOutMiniPlayer();
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

		const handleCommandPaletteKeyDown = (e: KeyboardEvent) => {
			const isMod = e.metaKey || e.ctrlKey;
			if (!isMod || e.key !== 'k') return;

			const target = e.target;
			if (
				target instanceof HTMLInputElement ||
				target instanceof HTMLTextAreaElement ||
				(target instanceof HTMLElement && target.isContentEditable)
			) {
				return;
			}

			e.preventDefault();
			appUi.isCommandPaletteOpen = true;
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

<CommandPalette />

<div class="h-screen overflow-hidden bg-surface-shell">
	{#if playerMode === 'cover'}
		<CoverView
			item={currentAudioItem}
			imageUrl={currentAudioItem?.imageUrl ?? currentAudioItemFeed?.imageUrl}
			onNavigateToItem={handleNavigateToItem}
			onClose={() => (playerMode = 'default')}
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
						onSelectResult={handleSelectSearchResult}
						onSelectFeedResult={handleSelectFeedSearchResult}
						onSelectStationResult={handleSelectStationSearchResult}
						onAddFeed={handleAddFeed}
					/>
				</div>
			</AppBar.Toolbar>
		</AppBar>

		<QueueDrawer
			open={isQueueDrawerOpen}
			onRemoveItem={removeQueuedItem}
			onMoveItemUp={moveQueuedItemUp}
			onMoveItemDown={moveQueuedItemDown}
			onClearQueue={clearQueue}
			onClose={() => (isQueueDrawerOpen = false)}
		/>

		<div class="flex h-[calc(100%-54px)] overflow-hidden">
			<div
				class={`hidden shrink-0 overflow-hidden motion-reduce:transition-none md:block md:transition-[width] md:duration-300 md:ease-[cubic-bezier(0.22,1,0.36,1)] ${
					isSidebarCollapsed ? 'md:w-16' : 'md:w-60'
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
					onToggleCollapse={() => (isSidebarCollapsed = !isSidebarCollapsed)}
					onCreateStation={handleCreateStation}
					onAddFeed={() => (isFeedEditorOpen = true)}
					refreshingFeedIds={syncingFeedIds}
					isCollapsed={isSidebarCollapsed}
				/>
			</div>

			<div class="relative z-30 min-w-0 flex-1">
				<div class="flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden">
					<main class="flex min-h-0 flex-1 flex-col">
						{#if feeds.length === 0 && !isInitialLoading}
							<EmptyFeedView />
						{:else if selectedSection === 'settings'}
							<SettingsView />
						{:else if selectedSection === 'home'}
							<HomeView {feeds} {stations} />
						{:else if isInspectorActive && FeedInspector !== null}
							<FeedInspector />
						{:else}
							<div class="flex min-h-0 flex-1 overflow-hidden">
								<div
									class="min-h-0 min-w-0 grow lg:shrink-0 lg:grow-0 lg:basis-1/3 lg:border-r lg:border-border 3xl:basis-4/10"
								>
									<ItemListView />
								</div>

								<ReaderPane />
							</div>
						{/if}
					</main>

					<AudioPlayer
						item={currentAudioItem}
						imageUrl={currentAudioItem?.imageUrl ?? currentAudioItemFeed?.imageUrl}
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
