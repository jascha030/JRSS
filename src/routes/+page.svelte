<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import FeedArticle from '$lib/components/FeedArticle.svelte';
	import FeedListView from '$lib/components/FeedListView.svelte';
	import ReaderArticle from '$lib/components/ReaderArticle.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import {
		app,
		createFeed,
		deleteFeed,
		ensureVisibleRangeLoaded,
		getActiveItemIdsByIndex,
		getActiveTotalCount,
		getCurrentAudioItem,
		getIsActiveInitialLoading,
		getSelectedFeed,
		getSelectedItem,
		getSelectedItemFeed,
		initializeApp,
		loadItemDetails,
		loadReaderView,
		markItemRead,
		persistPlaybackPosition,
		playAudioItem,
		refreshExistingFeed,
		selectFeed,
		selectItem,
		selectSection,
		setPlaybackPlaying,
		stopPlayback,
		updatePlaybackPosition
	} from '$lib/stores/app.svelte';
	import { formatDuration } from '$lib/utils/format';
	import { open } from '@tauri-apps/plugin-shell';
	import { onMount } from 'svelte';

	let newFeedUrl = $state('');
	let notice = $state('');
	let isSidebarCollapsed = $state(false);
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
									onPlay={playAudioItem}
									{totalCount}
								/>
							</div>

							<aside
								class="hidden min-h-0 min-w-0 flex-col justify-between overflow-y-auto bg-surface-glass p-8 xl:flex xl:flex-1 2xl:basis-2/3"
							>
								{#if selectedItem}
									<div class="space-y-9">
										<div class="flex flex-wrap items-center gap-4">
											<button
												class="btn-primary rounded-2xl px-4 py-3"
												type="button"
												onclick={() => {
													void open(selectedItem.url);
												}}
											>
												Open original
											</button>

											{#if selectedItem.mediaEnclosure}
												<button
													class="btn-secondary rounded-2xl px-4 py-3"
													type="button"
													onclick={() => playAudioItem(selectedItem)}
												>
													{selectedItem.playbackPositionSeconds > -1
														? 'Resume playback'
														: 'Start playback'}
												</button>
											{/if}

											{#if canUseReaderMode && hasSelectedItemReaderContent}
												<div
													class="inline-flex rounded-2xl border border-border-strong bg-surface p-1"
												>
													<button
														class={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${readerPaneMode === 'feed' ? 'bg-interactive text-interactive-text' : 'text-fg-muted hover:text-fg'}`}
														type="button"
														onclick={() => {
															readerPaneMode = 'feed';
														}}
													>
														Feed view
													</button>
													<button
														class={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${readerPaneMode === 'reader' ? 'bg-interactive text-interactive-text' : 'text-fg-muted hover:text-fg'}`}
														type="button"
														onclick={() => {
															readerPaneMode = 'reader';
														}}
													>
														Reader view
													</button>
												</div>
											{:else if canUseReaderMode}
												<button
													class="btn-secondary rounded-2xl px-4 py-3"
													disabled={isSelectedItemReaderLoading}
													type="button"
													onclick={() => handleLoadReaderView(selectedItem.id)}
												>
													{isSelectedItemReaderLoading
														? 'Loading reader view...'
														: selectedItem.readerStatus === 'failed'
															? 'Retry Reader View'
															: 'Load Reader View'}
												</button>
											{/if}
										</div>

										{#if readerNotice}
											<p class="text-sm leading-8 text-fg-muted">
												{readerNotice}
											</p>
										{/if}

										{#if isReaderPaneActive}
											<ReaderArticle
												feedTitle={selectedItemFeed?.title}
												title={selectedItem.readerTitle ?? selectedItem.title}
												byline={selectedItem.readerByline}
												excerpt={selectedItem.readerExcerpt}
												publishedAt={selectedItem.publishedAt}
												html={selectedItem.readerContentHtml}
												text={selectedItem.readerContentText}
											/>
										{:else}
											<FeedArticle
												feedTitle={selectedItemFeed?.title}
												title={selectedItem.title}
												publishedAt={selectedItem.publishedAt}
												contentHtml={selectedItem.contentHtml}
												contentText={selectedItem.contentText}
												summaryHtml={selectedItem.summaryHtml}
												summaryText={selectedItem.summaryText}
												summary={selectedItem.summary}
											/>

											{#if selectedItem.mediaEnclosure}
												<div class="rounded-3xl border border-dashed border-border-strong p-6">
													<p class="text-sm leading-8 text-fg-muted">
														Podcast controls remain available here and in the footer player.
														{#if selectedItem.mediaEnclosure.durationSeconds}
															Duration {formatDuration(
																selectedItem.mediaEnclosure.durationSeconds
															)}.
														{/if}
														{#if selectedItem.playbackPositionSeconds > -1}
															Resume point {formatDuration(selectedItem.playbackPositionSeconds)}.
														{/if}
													</p>
												</div>
											{/if}
										{/if}
									</div>
								{:else}
									<div class="flex h-full min-h-[21rem] flex-col justify-between">
										<div>
											<p class="text-sm font-medium tracking-[0.18em] text-fg-muted uppercase">
												Reader
											</p>
											<h1 class="mt-4 text-3xl font-semibold tracking-tight text-fg">
												No item selected
											</h1>
											<p class="mt-5 max-w-xl text-sm leading-7 text-fg-secondary">
												Pick an item from the list to read its details here. When a view has visible
												items, the first one is selected automatically.
											</p>
										</div>

										<div
											class="rounded-3xl border border-dashed border-border-strong bg-surface-inset p-6"
										>
											<p class="text-sm leading-8 text-fg-muted">
												This reader pane is still plain-text-only for now. Feed switching, refresh,
												playback, and the current list workflow remain unchanged.
											</p>
										</div>
									</div>
								{/if}
							</aside>
						</div>
					{/if}
				</main>

				<AudioPlayer
					item={currentAudioItem}
					playbackState={currentPlaybackState}
					onPlayingChange={setPlaybackPlaying}
					onPositionChange={updatePlaybackPosition}
					onPositionPersist={persistPlaybackPosition}
					onStop={stopPlayback}
				/>
			</div>
		</div>
	</div>
</div>
