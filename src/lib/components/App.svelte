<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import type { Snippet } from 'svelte';
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

	type RouteLoadingCoverState = 'idle' | 'covering' | 'loading' | 'revealing';

	type Props = {
		children: Snippet;
		routeLoadingCoverState?: RouteLoadingCoverState;
	};

	let { children, routeLoadingCoverState = 'idle' }: Props = $props();

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
	const isRouteLoadingCoverVisible = $derived(routeLoadingCoverState !== 'idle');
	const isRouteContentHidden = $derived(
		routeLoadingCoverState === 'covering' || routeLoadingCoverState === 'loading'
	);
	const isRouteLoadingCoverOpaque = $derived(
		routeLoadingCoverState === 'covering' || routeLoadingCoverState === 'loading'
	);

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
					<div
						class={`flex min-h-0 flex-1 flex-col transition-[opacity,transform] duration-180 ease-out motion-reduce:transition-none ${
							isRouteContentHidden
								? 'translate-y-1 scale-[0.995] opacity-0'
								: 'translate-y-0 scale-100 opacity-100'
						}`}
						style="view-transition-name: page-content"
					>
						{#if shouldShowEmptyFeedView}
							<EmptyFeedView />
						{:else}
							{@render children?.()}
						{/if}
					</div>

					<div
						aria-hidden={!isRouteLoadingCoverVisible}
						class={`pointer-events-none absolute inset-0 z-20 transition-opacity duration-180 ease-out motion-reduce:transition-none ${
							isRouteLoadingCoverVisible ? 'opacity-100' : 'opacity-0'
						}`}
					>
						<div
							class={`absolute inset-0 bg-surface/80 transition-opacity duration-180 ease-out motion-reduce:transition-none ${
								isRouteLoadingCoverOpaque ? 'opacity-100' : 'opacity-0'
							}`}
						></div>

						<div class="absolute inset-0 flex items-center justify-center">
							<div
								class={`flex items-center gap-3 rounded-full border border-border/70 bg-surface-shell/88 px-4 py-2.5 shadow-lg transition-[opacity,transform] duration-180 ease-out motion-reduce:transition-none ${
									routeLoadingCoverState === 'revealing'
										? 'translate-y-1 scale-[0.985] opacity-0'
										: 'translate-y-0 scale-100 opacity-100'
								}`}
							>
								<div
									class="size-4 rounded-full border-2 border-accent/25 border-t-accent motion-safe:animate-spin motion-reduce:animate-none"
								></div>
								<span class="text-sm font-medium text-fg-secondary">Loading view</span>
							</div>
						</div>
					</div>
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

	<div
		class="fixed inset-0 z-50 transition-[opacity,transform] duration-500 ease-[cubic-bezier(0.22,1,0.36,1)] motion-reduce:transition-none"
		class:opacity-0={appUi.playerMode !== 'cover'}
		class:pointer-events-none={appUi.playerMode !== 'cover'}
		class:scale-[0.985]={appUi.playerMode !== 'cover'}
		aria-hidden={appUi.playerMode !== 'cover'}
	>
		<CoverView item={currentAudioItem} imageUrl={currentAudioItem?.imageUrl} />
	</div>
</div>
