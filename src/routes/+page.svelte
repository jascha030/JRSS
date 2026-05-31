<script lang="ts">
	import { navigateToFeed, navigateToStation } from '$lib/navigation/app-router';
	import { getFeedsUnreadCounts } from '$lib/services/feed';
	import { feedsState, stationsState } from '$lib/state';

	import FeedCard from '$lib/components/home/FeedCard.svelte';
	import StationCard from '$lib/components/home/StationCard.svelte';

	const feeds = $derived(feedsState.feeds);
	const stations = $derived(stationsState.stations);

	let unreadCountsByFeedId = $state<Record<string, number>>({});

	void getFeedsUnreadCounts().then((counts) => {
		unreadCountsByFeedId = counts;
	});

	const podcasts = $derived(feeds.filter((feed) => feed.kind === 'media'));
	const articles = $derived(feeds.filter((feed) => feed.kind === 'article'));

	function handleFeedClick(feedId: string) {
		void navigateToFeed(feedId);
	}

	function handleStationClick(stationId: string) {
		void navigateToStation(stationId);
	}
</script>

<svelte:head>
	<title>JRSS</title>
	<meta name="description" content="RSS reader and podcast player." />
</svelte:head>

<section class="flex h-full w-full flex-1 flex-col overflow-y-auto bg-surface-shell-opaque">
	<div class="px-6 py-8 lg:px-8">
		<h2 class="text-2xl font-semibold tracking-tight text-fg">Home</h2>

		{#if podcasts.length > 0}
			<div class="mt-8 [contain-intrinsic-size:960px] [content-visibility:auto]">
				<h3 class="text-sm font-semibold tracking-widest text-fg-muted uppercase">Podcasts</h3>

				<div
					class="mt-4 grid grid-cols-[repeat(auto-fill,10rem)] justify-center gap-4 sm:grid-cols-[repeat(auto-fill,11rem)]"
				>
					{#each podcasts as feed (feed.id)}
						<FeedCard
							{feed}
							unreadCount={unreadCountsByFeedId[feed.id] ?? 0}
							onClick={() => handleFeedClick(feed.id)}
						/>
					{/each}
				</div>
			</div>
		{/if}

		{#if articles.length > 0}
			<div class="mt-8 [contain-intrinsic-size:960px] [content-visibility:auto]">
				<h3 class="text-sm font-semibold tracking-widest text-fg-muted uppercase">Articles</h3>

				<div
					class="mt-4 grid grid-cols-[repeat(auto-fill,10rem)] justify-center gap-4 sm:grid-cols-[repeat(auto-fill,11rem)]"
				>
					{#each articles as feed (feed.id)}
						<FeedCard
							{feed}
							unreadCount={unreadCountsByFeedId[feed.id] ?? 0}
							onClick={() => handleFeedClick(feed.id)}
						/>
					{/each}
				</div>
			</div>
		{/if}

		{#if stations.length > 0}
			<div class="mt-8 [contain-intrinsic-size:720px] [content-visibility:auto]">
				<h3 class="text-sm font-semibold tracking-widest text-fg-muted uppercase">Stations</h3>

				<div
					class="mt-4 grid grid-cols-[repeat(auto-fill,10rem)] justify-center gap-4 sm:grid-cols-[repeat(auto-fill,11rem)]"
				>
					{#each stations as station (station.id)}
						<StationCard {station} onClick={() => handleStationClick(station.id)} />
					{/each}
				</div>
			</div>
		{/if}
	</div>
</section>
