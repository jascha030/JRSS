<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { openStationEditor } from '$lib/hooks/useAppUi.svelte';
	import ItemListHeader from '$lib/components/content/ItemListHeader.svelte';
	import ItemListView from '$lib/components/content/ItemListView.svelte';
	import { appUi } from '$lib/hooks/useAppUi.svelte';
	import { useItemSelection } from '$lib/hooks/useItemSelection.svelte';
	import { navigateToFeedInspector, replaceCurrentSearchTerm } from '$lib/navigation/app-router';
	import { toast } from 'svelte-sonner';
	import type { SidebarSection } from '$lib/state';
	import {
		deleteExistingStation,
		ensureItemLoaded,
		ensureVisibleRangeLoaded,
		feedsState,
		getActiveQueryKey,
		getIsActiveInitialLoading,
		itemsState,
		playStation,
		refreshExistingFeed,
		setFeedSortOrder,
		selection,
		stationsState
	} from '$lib/state';

	const sectionHeadings: Record<Exclude<SidebarSection, null>, string> = {
		home: 'Home',
		all: 'All feeds',
		unread: 'Unread',
		media: 'Media',
		settings: 'Settings'
	};

	const itemSelection = useItemSelection();
	const activeQueryKey = $derived(getActiveQueryKey());
	const itemsById = $derived(itemsState.itemSummariesById);
	const selectedFeed = $derived(
		feedsState.feeds.find((feed) => feed.id === selection.selectedFeedId) ?? null
	);

	const selectedStation = $derived(
		stationsState.stations.find((station) => station.id === selection.selectedStationId) ?? null
	);

	const selectedItemId = $derived(selection.selectedItemId);
	const scrollToItemRequest = $derived(appUi.scrollToItemRequest);
	const isRefreshing = $derived(
		selectedFeed ? feedsState.syncingFeedIds.includes(selectedFeed.id) : false
	);

	const itemSortOrder = $derived(
		selectedStation?.sortOrder ?? selectedFeed?.sortOrder ?? 'newest_first'
	);

	const totalCount = $derived(
		activeQueryKey ? (itemsState.totalCountByQueryKey[activeQueryKey] ?? 0) : 0
	);

	const pageHeading = $derived(
		selectedStation?.name ??
			selectedFeed?.title ??
			(selection.selectedSection ? sectionHeadings[selection.selectedSection] : 'All feeds')
	);

	const searchContext = $derived.by(() => {
		if (selectedFeed) {
			return {
				label: 'Search this feed',
				placeholder: 'Search this feed',
				value: selection.feedSearchTerm
			};
		}

		if (selectedStation) {
			return {
				label: 'Search this station',
				placeholder: 'Search this station',
				value: selection.stationSearchTerm
			};
		}

		if (selection.selectedSection === 'unread' || selection.selectedSection === 'media') {
			return {
				label: `Search ${selection.selectedSection}`,
				placeholder: `Search ${selection.selectedSection}`,
				value: selection.sectionSearchTerm
			};
		}

		return null;
	});

	const hasActiveSearch = $derived(
		selection.feedSearchTerm.trim().length > 0 ||
			selection.stationSearchTerm.trim().length > 0 ||
			selection.sectionSearchTerm.trim().length > 0
	);

	const feedTitleById = $derived(new Map(feedsState.feeds.map((feed) => [feed.id, feed.title])));

	const showFeedTitle = $derived(
		selection.selectedFeedId === null &&
			(selection.selectedStationId === null ||
				stationsState.stations.find((station) => station.id === selection.selectedStationId)
					?.feedIds.length !== 1)
	);

	const resolvedItemIdsByIndex = $derived(
		activeQueryKey ? (itemsState.itemIdsByIndexByQueryKey[activeQueryKey] ?? {}) : {}
	);

	const resolvedTotalCount = $derived(
		activeQueryKey ? (itemsState.totalCountByQueryKey[activeQueryKey] ?? 0) : 0
	);

	const isInitialLoading = $derived(getIsActiveInitialLoading());

	let displayedQueryKey = $state<string | null>(null);
	let displayedItemIdsByIndex = $state<Record<number, string>>({});
	let displayedTotalCount = $state(0);
	let isQueryTransitioning = $state(false);
	let hasAppliedInitialScroll = $state(false);

	$effect(() => {
		if (!activeQueryKey) {
			displayedQueryKey = null;
			displayedItemIdsByIndex = {};
			displayedTotalCount = 0;
			isQueryTransitioning = false;
			return;
		}

		const isNewQuery = displayedQueryKey !== null && displayedQueryKey !== activeQueryKey;

		if (isNewQuery && isInitialLoading) {
			isQueryTransitioning = true;
			return;
		}

		displayedQueryKey = activeQueryKey;
		displayedItemIdsByIndex = resolvedItemIdsByIndex;
		displayedTotalCount = resolvedTotalCount;
		isQueryTransitioning = false;
	});

	$effect(() => {
		if (scrollToItemRequest) {
			void scrollToItemRequest.seq;
			hasAppliedInitialScroll = false;
		}
	});

	function handleScrollApplied(): void {
		hasAppliedInitialScroll = true;
	}

	function handleSearchChange(value: string): void {
		void replaceCurrentSearchTerm(value);
	}

	async function withToast<T>(promise: Promise<T>, fallback: string): Promise<void> {
		try {
			await promise;
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : fallback);
		}
	}

	async function handlePlayStation(): Promise<void> {
		if (!selectedStation) return;
		await withToast(playStation(selectedStation.id), 'Unable to play station.');
	}

	function handleEditStation(): void {
		if (!selectedStation) return;
		openStationEditor({ id: selectedStation.id });
	}

	async function handleDeleteStation(): Promise<void> {
		if (!selectedStation) return;
		await withToast(deleteExistingStation(selectedStation.id), 'Unable to delete station.');
	}

	function handleRefreshFeed(): void {
		if (selectedFeed) {
			void refreshExistingFeed(selectedFeed.id);
		}
	}

	function handleInspectFeed(): void {
		if (selectedFeed) {
			void navigateToFeedInspector(selectedFeed.id);
		}
	}

	function handleSetSortOrder(order: 'newest_first' | 'oldest_first'): void {
		void setFeedSortOrder(order);
	}

	let searchInputRef = $state<HTMLInputElement | null>(null);

	onMount(() => {
		let unlistenSearchFeed: UnlistenFn | undefined;
		let unlistenRefreshFeed: UnlistenFn | undefined;

		const setupListeners = async () => {
			unlistenSearchFeed = await listen('menu-search-feed', () => {
				searchInputRef?.focus();
			});

			unlistenRefreshFeed = await listen('menu-refresh-feed', () => {
				if (selectedFeed && !isRefreshing) {
					void refreshExistingFeed(selectedFeed.id);
				}
			});
		};

		void setupListeners();

		return () => {
			if (unlistenSearchFeed) unlistenSearchFeed();
			if (unlistenRefreshFeed) unlistenRefreshFeed();
			hasAppliedInitialScroll = false;
		};
	});
</script>

<div
	class="flex min-h-0 min-w-0 grow flex-col border-r border-border xl:basis-1/2 2xl:shrink-0 2xl:grow-0 4xl:basis-4/10 {appUi.isReaderMaximized
		? 'hidden'
		: ''}"
>
	<ItemListHeader
		{pageHeading}
		{totalCount}
		{selectedFeed}
		{selectedStation}
		bind:searchInputRef
		{isRefreshing}
		{itemSortOrder}
		searchLabel={searchContext?.label ?? null}
		searchPlaceholder={searchContext?.placeholder ?? null}
		searchValue={searchContext?.value ?? ''}
		onSearchChange={handleSearchChange}
		onPlayStation={handlePlayStation}
		onEditStation={handleEditStation}
		onDeleteStation={handleDeleteStation}
		onRefreshFeed={handleRefreshFeed}
		onInspectFeed={handleInspectFeed}
		onSetSortOrder={handleSetSortOrder}
	/>
	<ItemListView
		{itemsById}
		{displayedItemIdsByIndex}
		{displayedTotalCount}
		{displayedQueryKey}
		{selectedItemId}
		{hasActiveSearch}
		{showFeedTitle}
		{feedTitleById}
		{isInitialLoading}
		{isQueryTransitioning}
		{scrollToItemRequest}
		{itemSelection}
		{hasAppliedInitialScroll}
		onEnsureItemLoaded={ensureItemLoaded}
		onVisibleRangeLoad={ensureVisibleRangeLoaded}
		onScrollApplied={handleScrollApplied}
	/>
</div>
