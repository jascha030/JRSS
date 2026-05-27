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
	import { popOutMiniPlayer } from '$lib/utils/mini-player';

	import { onMount } from 'svelte';
	import type FeedInspectorComponent from '$lib/components/feed/FeedInspector.svelte';
	import { toast } from 'svelte-sonner';

	let FeedInspector = $state<typeof FeedInspectorComponent | null>(null);
	let lastQueryKey = $state<string | null>(null);
	let renderedShellViewKey = $state('library');
	let renderedTransitionKey = $state('library');
	let isShellOverlayVisible = $state(false);
	let isShellContentVisible = $state(true);
	let isLibraryViewReady = $state(false);
	let shellSwapFrame = 0;
	let shellSwapTimer = 0;
	let shellRevealTimer = 0;

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

	const isInspectorActive = $derived(inspectorState.activeFeedId !== null);
	const activeShellViewKey = $derived.by(() => {
		if (feeds.length === 0 && !isInitialLoading) {
			return 'empty';
		}

		if (selectedSection === 'settings') {
			return 'settings';
		}

		if (selectedSection === 'home') {
			return 'home';
		}

		if (isInspectorActive) {
			return inspectorState.activeFeedId ? `inspector:${inspectorState.activeFeedId}` : 'inspector';
		}

		return 'library';
	});
	const activeTransitionKey = $derived.by(() => {
		if (activeShellViewKey !== 'library') {
			return activeShellViewKey;
		}

		return `library:${activeQueryKey ?? 'none'}`;
	});
	const isRenderedShellReady = $derived.by(() => {
		if (renderedShellViewKey === 'library') {
			return isLibraryViewReady;
		}

		if (renderedShellViewKey.startsWith('inspector:')) {
			return FeedInspector !== null && !inspectorState.loading;
		}

		return true;
	});

	$effect(() => {
		if (activeTransitionKey === renderedTransitionKey) {
			return;
		}

		if (shellRevealTimer !== 0) {
			clearTimeout(shellRevealTimer);
			shellRevealTimer = 0;
		}

		if (shellSwapTimer !== 0) {
			clearTimeout(shellSwapTimer);
			shellSwapTimer = 0;
		}

		if (shellSwapFrame !== 0) {
			cancelAnimationFrame(shellSwapFrame);
			shellSwapFrame = 0;
		}

		isShellOverlayVisible = true;
		isShellContentVisible = false;

		if (activeShellViewKey === renderedShellViewKey) {
			return;
		}

		shellSwapTimer = window.setTimeout(() => {
			renderedShellViewKey = activeShellViewKey;
			shellSwapTimer = 0;
		}, 170);
	});

	$effect(() => {
		if (!isShellOverlayVisible) {
			return;
		}

		if (renderedShellViewKey !== activeShellViewKey) {
			return;
		}

		if (!isRenderedShellReady) {
			return;
		}

		if (shellRevealTimer !== 0) {
			clearTimeout(shellRevealTimer);
		}

		shellRevealTimer = window.setTimeout(() => {
			renderedTransitionKey = activeTransitionKey;
			isShellContentVisible = true;
			shellRevealTimer = 0;
			shellSwapFrame = requestAnimationFrame(() => {
				isShellOverlayVisible = false;
				shellSwapFrame = 0;
			});
		}, 120);
	});

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
			if (itemId) void switchToReaderView(itemId);
		}
	});

	async function handleAddFeed(url: string) {
		try {
			await createFeed(url);
			closeFeedEditor();
			toast.success('Feed loaded and saved locally.');
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to add that feed.');
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
			closeStationEditor();
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
		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}
		closeInspector();
		selectFeed(item.feedId);
		selectItem(item.id);
		requestScrollToItem(item.id);
	}

	function handleSelectFeedSearchResult(feed: import('$lib/types/feed').Feed): void {
		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}
		closeInspector();
		selectFeed(feed.id);
	}

	function handleSelectStationSearchResult(station: import('$lib/types/station').Station): void {
		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
		}
		closeInspector();
		selectStation(station.id);
	}

	function handleNavigateToItem() {
		if (!currentAudioItem) return;

		if (appUi.playerMode === 'cover') {
			appUi.playerMode = 'default';
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
				appUi.playerMode = 'default';
				selectSection('settings');
			}
		},
		{
			event: 'menu-toggle-sidebar',
			handler: toggleSidebar
		},
		{
			event: 'menu-next-source',
			handler: () => {
				if (appUi.playerMode === 'cover') return;
				handleCycleSource(1);
			}
		},
		{
			event: 'menu-prev-source',
			handler: () => {
				if (appUi.playerMode === 'cover') return;
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
			openCommandPalette();
		};

		document.addEventListener('keydown', handleKeyDown);
		document.addEventListener('keydown', handleCommandPaletteKeyDown);
		renderedShellViewKey = activeShellViewKey;
		renderedTransitionKey = activeTransitionKey;

		return () => {
			document.removeEventListener('keydown', handleKeyDown);
			document.removeEventListener('keydown', handleCommandPaletteKeyDown);

			if (shellRevealTimer !== 0) {
				clearTimeout(shellRevealTimer);
			}

			if (shellSwapTimer !== 0) {
				clearTimeout(shellSwapTimer);
			}

			if (shellSwapFrame !== 0) {
				cancelAnimationFrame(shellSwapFrame);
			}
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
						<div
							class="flex min-h-0 flex-1 flex-col transition-[opacity,transform,filter] duration-220 ease-out motion-reduce:transition-none"
							class:translate-y-1={!isShellContentVisible}
							class:opacity-0={!isShellContentVisible}
							class:blur-[3px]={!isShellContentVisible}
						>
							{#if renderedShellViewKey === 'empty'}
								<EmptyFeedView />
							{:else if renderedShellViewKey === 'settings'}
								<SettingsView />
							{:else if renderedShellViewKey === 'home'}
								<HomeView {feeds} {stations} />
							{:else if renderedShellViewKey.startsWith('inspector:') && FeedInspector !== null}
								<FeedInspector />
							{:else if renderedShellViewKey.startsWith('inspector:')}
								<div class="flex min-h-0 flex-1 bg-surface"></div>
							{:else}
								<div class="flex min-h-0 flex-1 overflow-hidden">
									<ItemListView
										onReadyStateChange={(ready) => (isLibraryViewReady = ready)}
										class="min-h-0 min-w-0 grow border-r border-border xl:basis-1/2 2xl:shrink-0 2xl:grow-0 4xl:basis-4/10 {appUi.isReaderMaximized
											? 'hidden'
											: ''}"
									/>
									<ReaderPane
										class="min-h-0 min-w-0 {appUi.isReaderMaximized
											? 'grow'
											: 'xl:basis-1/2 2xl:flex 2xl:flex-1 4xl:basis-4/10'}"
									/>
								</div>
							{/if}
						</div>

						<div
							aria-hidden={!isShellOverlayVisible}
							class={`pointer-events-none absolute inset-0 z-20 flex items-center justify-center transition-opacity duration-220 ease-out motion-reduce:transition-none ${
								isShellOverlayVisible ? 'opacity-100' : 'opacity-0'
							}`}
						>
							<div class="absolute inset-0 bg-surface/72 backdrop-blur-md"></div>
							<div
								class="relative flex items-center gap-3 rounded-full border border-border/70 bg-surface-shell/88 px-4 py-2.5 shadow-lg"
							>
								<div
									class="size-4 rounded-full border-2 border-accent/25 border-t-accent motion-safe:animate-spin motion-reduce:animate-none"
								></div>
								<span class="text-sm font-medium text-fg-secondary">Loading view</span>
							</div>
						</div>
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
