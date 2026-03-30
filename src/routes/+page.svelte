<script lang="ts">
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import FeedListView from '$lib/components/FeedListView.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
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
		syncingFeedIds,
		selectedFeed,
		selectedFeedId,
		selectedSection,
		selectFeed,
		selectSection,
		setPlaybackPlaying,
		stopPlayback,
		updatePlaybackPosition,
		visibleItems
	} from '$lib/stores/app';
	import { onMount } from 'svelte';

	let newFeedUrl = $state('');
	let notice = $state('');
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

<div class="min-h-screen bg-slate-100/80 dark:bg-slate-950">
	<div class="flex min-h-screen flex-col lg:flex-row">
		<Sidebar
			feeds={$feeds}
			refreshingFeedIds={$syncingFeedIds}
			selectedFeedId={$selectedFeedId}
			selectedSection={$selectedSection}
			onRemoveFeed={deleteFeed}
			onSelectFeed={selectFeed}
			onSelectSection={selectSection}
		/>

		<main class="flex min-w-0 flex-1 flex-col">
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
				<section class="flex flex-1 items-center justify-center px-6 py-12 lg:px-8">
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
							Add an RSS or Atom URL above and the app will try to fetch, parse, and persist real
							feed entries locally.
						</p>
					</div>
				</section>
			{:else if $selectedSection === 'settings'}
				<section class="flex-1 px-6 py-8 lg:px-8">
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
							This MVP keeps everything in the browser with stores plus a feed service abstraction.
							That keeps the UI easy to migrate to Tauri commands or another local backend later.
						</p>
					</div>
				</section>
			{:else}
				<FeedListView
					feeds={$feeds}
					isRefreshing={isSelectedFeedRefreshing}
					items={$visibleItems}
					onRefresh={handleRefreshFeed}
					selectedFeed={$selectedFeed}
					selectedSection={$selectedSection}
					onMarkRead={markItemRead}
					onPlay={playAudioItem}
				/>
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
