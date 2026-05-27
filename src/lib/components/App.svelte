<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
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
	import {
		feedsState,
		getCurrentAudioItem,
		getCurrentAudioItemFeed,
		getIsActiveInitialLoading,
		getUpcomingQueue,
		stationsState
	} from '$lib/state';
	import {
		appUi,
		closeFeedEditor,
		closeStationEditor,
		toggleQueue
	} from '$lib/hooks/useAppUi.svelte';
	import {
		useAppOrchestrator,
		addFeedFromUrl,
		saveStation
	} from '$lib/hooks/useAppOrchestrator.svelte';

	let { children } = $props();

	useAppOrchestrator();

	const feeds = $derived(feedsState.feeds);
	const stations = $derived(stationsState.stations);
	const isCreatingFeed = $derived(feedsState.isCreatingFeed);
	const isInitialLoading = $derived(getIsActiveInitialLoading());
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentAudioItemFeed = $derived(getCurrentAudioItemFeed());
	const upcomingQueue = $derived(getUpcomingQueue());
	const queueLength = $derived(upcomingQueue.length);
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

	function handleCloseQueue() {
		appUi.isQueueDrawerOpen = false;
	}
</script>

<FeedEditor
	open={isFeedEditorOpen}
	isLoading={isCreatingFeed}
	onSave={addFeedFromUrl}
	onClose={closeFeedEditor}
/>

<StationEditor
	open={appUi.dialog.kind === 'station-editor'}
	station={editingStation}
	{feeds}
	onSave={saveStation}
	onClose={closeStationEditor}
/>

<CommandPalette />

<div class="h-screen overflow-hidden bg-surface-shell">
	{#if appUi.playerMode === 'cover'}
		<CoverView item={currentAudioItem} imageUrl={currentAudioItem?.imageUrl} />
	{:else}
		<AppBar class="top-0 z-9999 h-12 bg-transparent! p-0">
			<AppBar.Toolbar
				class="flex h-12 w-full content-center border-b border-border bg-surface-sidebar p-0"
			>
				<div
					data-tauri-drag-region
					class="flex h-12 w-full items-center justify-end py-0 pr-4 pl-44 align-middle"
				>
					<Header />
				</div>
			</AppBar.Toolbar>
		</AppBar>

		<QueueDrawer open={appUi.isQueueDrawerOpen} onClose={handleCloseQueue} />

		<div class="flex h-[calc(100%-54px)] overflow-hidden">
			<div
				class={`hidden shrink-0 overflow-hidden motion-reduce:transition-none md:block md:transition-[width] md:duration-300 md:ease-[cubic-bezier(0.22,1,0.36,1)] ${
					appUi.isSidebarCollapsed ? 'md:w-16' : 'md:w-60'
				}`}
			>
				<Sidebar />
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
