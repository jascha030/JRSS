<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import FeedListView from '$lib/components/FeedListView.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { formatDate, formatDuration } from '$lib/utils/format';
	import {
		createFeed,
		currentAudioItem,
		currentPlaybackState,
		deleteFeed,
		feeds,
		initializeApp,
		isCreatingFeed,
		markItemRead,
		playAudioItem,
		refreshExistingFeed,
		selectedItem,
		selectedItemFeed,
		selectedItemId,
		syncingFeedIds,
		selectedFeed,
		selectedFeedId,
		selectedSection,
		selectFeed,
		selectItem,
		selectSection,
		setPlaybackPlaying,
		stopPlayback,
		updatePlaybackPosition,
		visibleItems
	} from '$lib/stores/app';
	import { onMount } from 'svelte';

	let newFeedUrl = $state('');
	let notice = $state('');
	let isSidebarCollapsed = $state(false);
	const isSelectedFeedRefreshing = $derived(
		$selectedFeed ? $syncingFeedIds.includes($selectedFeed.id) : false
	);

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
</script>

<svelte:head>
	<title>JRSS</title>
	<meta
		name="description"
		content="Local-first RSS reader and podcast MVP foundation built with SvelteKit."
	/>
</svelte:head>

<div class="flex h-screen flex-col overflow-hidden bg-slate-100/80 dark:bg-slate-950">
	<div class="flex min-h-0 flex-1 overflow-hidden">
		<Sidebar
			collapsed={isSidebarCollapsed}
			feeds={$feeds}
			refreshingFeedIds={$syncingFeedIds}
			selectedFeedId={$selectedFeedId}
			selectedSection={$selectedSection}
			onRemoveFeed={deleteFeed}
			onSelectFeed={selectFeed}
			onSelectSection={selectSection}
			onToggleCollapse={() => {
				isSidebarCollapsed = !isSidebarCollapsed;
			}}
		/>

		<main class="flex min-h-0 min-w-0 flex-1 flex-col bg-slate-100/70 dark:bg-slate-950/70">
			<header
				class="border-b border-slate-200/70 bg-white/80 px-6 py-5 backdrop-blur lg:px-8 dark:border-slate-800 dark:bg-slate-950/80"
			>
				<div class="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
					<div class="space-y-2">
						<p
							class="text-sm font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
						>
							Local-first RSS + podcasts
						</p>
						<div>
							<h1 class="text-3xl font-semibold tracking-tight text-slate-950 dark:text-white">
								JRSS MVP
							</h1>
							<p class="mt-1 max-w-2xl text-sm text-slate-600 dark:text-slate-300">
								Client-side foundation with a small service layer, mock data, and static output
								ready for a later Tauri move.
							</p>
						</div>
					</div>

					<form
						class="flex flex-col gap-3 sm:flex-row xl:min-w-[28rem]"
						onsubmit={(event) => {
							event.preventDefault();
							void handleAddFeed();
						}}
					>
						<input
							bind:value={newFeedUrl}
							class="min-w-0 flex-1 rounded-2xl border border-slate-300 bg-white px-4 py-3 text-sm text-slate-900 transition outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-300 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100 dark:focus:border-slate-500 dark:focus:ring-slate-700"
							disabled={$isCreatingFeed}
							placeholder="https://example.com/feed.xml"
							type="url"
						/>
						<button
							class="rounded-2xl bg-slate-950 px-4 py-3 text-sm font-medium text-white transition hover:bg-slate-800 dark:bg-slate-100 dark:text-slate-950 dark:hover:bg-white"
							disabled={$isCreatingFeed}
							type="submit"
						>
							{$isCreatingFeed ? 'Adding...' : 'Add feed'}
						</button>
					</form>
				</div>

				{#if notice}
					<p class="mt-4 text-sm text-slate-600 dark:text-slate-300">{notice}</p>
				{/if}
			</header>

			{#if $feeds.length === 0}
				<section class="flex flex-1 items-center justify-center overflow-y-auto px-6 py-12 lg:px-8">
					<div
						class="max-w-lg rounded-3xl border border-dashed border-slate-300 bg-white p-8 text-center shadow-sm dark:border-slate-800 dark:bg-slate-900"
					>
						<p
							class="text-sm font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
						>
							No feeds yet
						</p>
						<h2 class="mt-3 text-2xl font-semibold text-slate-950 dark:text-white">
							Add your first RSS feed or podcast
						</h2>
						<p class="mt-3 text-sm leading-6 text-slate-600 dark:text-slate-300">
							Add an RSS or Atom URL above and the desktop app will fetch, parse, and persist real
							feed entries in local SQLite.
						</p>
					</div>
				</section>
			{:else if $selectedSection === 'settings'}
				<section class="flex-1 overflow-y-auto px-6 py-8 lg:px-8">
					<div
						class="max-w-3xl rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900"
					>
						<p
							class="text-sm font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
						>
							Settings
						</p>
						<h2 class="mt-3 text-2xl font-semibold text-slate-950 dark:text-white">
							Foundation-only for now
						</h2>
						<p class="mt-3 text-sm leading-6 text-slate-600 dark:text-slate-300">
							The UI still talks to the same frontend service layer, but feed ingestion and
							persistence now run through Tauri commands backed by local SQLite.
						</p>
					</div>
				</section>
			{:else}
				<div class="flex min-h-0 flex-1 overflow-hidden">
					<div
						class="min-h-0 min-w-0 flex-1 xl:border-r xl:border-slate-200 xl:dark:border-slate-800"
					>
						<FeedListView
							feeds={$feeds}
							isRefreshing={isSelectedFeedRefreshing}
							items={$visibleItems}
							onRefresh={handleRefreshFeed}
							onSelectItem={selectItem}
							selectedFeed={$selectedFeed}
							selectedItemId={$selectedItemId}
							selectedSection={$selectedSection}
							onMarkRead={markItemRead}
							onPlay={playAudioItem}
						/>
					</div>

					<aside
						class="hidden min-h-0 min-w-0 flex-1 flex-col justify-between overflow-y-auto bg-white/80 p-8 xl:flex dark:bg-slate-950/80"
					>
						{#if $selectedItem}
							<div class="space-y-8">
								<div>
									<p
										class="text-sm font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
									>
										{$selectedItemFeed?.title ?? 'Unknown feed'}
									</p>
									<h2
										class="mt-3 text-3xl font-semibold tracking-tight text-slate-950 dark:text-white"
									>
										{$selectedItem.title}
									</h2>
									<div
										class="mt-4 flex flex-wrap items-center gap-3 text-sm text-slate-500 dark:text-slate-400"
									>
										<span>{formatDate($selectedItem.publishedAt)}</span>
										{#if $selectedItem.mediaEnclosure}
											<span
												class="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300"
											>
												Podcast
											</span>
										{/if}
										<span
											class="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300"
										>
											{$selectedItem.read ? 'Read' : 'Unread'}
										</span>
									</div>
								</div>

								<div class="flex flex-wrap gap-3">
									<button
										class="rounded-2xl bg-slate-950 px-4 py-3 text-sm font-medium text-white transition hover:bg-slate-800 dark:bg-slate-100 dark:text-slate-950 dark:hover:bg-white"
										type="button"
										onclick={() => window.open($selectedItem.url, '_blank', 'noopener,noreferrer')}
									>
										Open original
									</button>

									{#if $selectedItem.mediaEnclosure}
										<button
											class="rounded-2xl border border-slate-300 px-4 py-3 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:text-slate-950 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-500 dark:hover:text-white"
											type="button"
											onclick={() => playAudioItem($selectedItem)}
										>
											{$selectedItem.playbackPositionSeconds > 0
												? 'Resume playback'
												: 'Start playback'}
										</button>
									{/if}
								</div>

								<div
									class="rounded-[2rem] border border-slate-200 bg-slate-50/80 p-6 dark:border-slate-800 dark:bg-slate-900/60"
								>
									<p
										class="text-sm leading-7 whitespace-pre-line text-slate-700 dark:text-slate-200"
									>
										{$selectedItem.contentText?.trim() ||
											$selectedItem.summaryText?.trim() ||
											$selectedItem.summary.trim() ||
											'No summary or content is available for this item yet.'}
									</p>
								</div>

								{#if $selectedItem.mediaEnclosure}
									<div
										class="rounded-[2rem] border border-dashed border-slate-300 p-6 dark:border-slate-800"
									>
										<p class="text-sm leading-7 text-slate-500 dark:text-slate-400">
											Podcast controls remain available here and in the footer player.
											{#if $selectedItem.mediaEnclosure.durationSeconds}
												Duration {formatDuration($selectedItem.mediaEnclosure.durationSeconds)}.
											{/if}
											{#if $selectedItem.playbackPositionSeconds > 0}
												Resume point {formatDuration($selectedItem.playbackPositionSeconds)}.
											{/if}
										</p>
									</div>
								{/if}
							</div>
						{:else}
							<div class="flex h-full min-h-[22rem] flex-col justify-between">
								<div>
									<p
										class="text-sm font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
									>
										Reader
									</p>
									<h2
										class="mt-3 text-3xl font-semibold tracking-tight text-slate-950 dark:text-white"
									>
										No item selected
									</h2>
									<p class="mt-4 max-w-xl text-sm leading-7 text-slate-600 dark:text-slate-300">
										Pick an item from the list to read its details here. When a view has visible
										items, the first one is selected automatically.
									</p>
								</div>

								<div
									class="rounded-[2rem] border border-dashed border-slate-300 bg-slate-50/80 p-6 dark:border-slate-800 dark:bg-slate-900/60"
								>
									<p class="text-sm leading-7 text-slate-500 dark:text-slate-400">
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
	</div>

	<AudioPlayer
		item={$currentAudioItem}
		playbackState={$currentPlaybackState}
		onPlayingChange={setPlaybackPlaying}
		onStop={stopPlayback}
		onTimeChange={updatePlaybackPosition}
	/>
</div>
