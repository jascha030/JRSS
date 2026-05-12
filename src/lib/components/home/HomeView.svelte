<script lang="ts">
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';
	import { STATION_GRADIENTS } from '$lib/constants/station-gradients';
	import { itemsState } from '$lib/state/items.svelte';
	import { selectFeed, selectStation } from '$lib/state';
	import { SvelteSet } from 'svelte/reactivity';

	type Props = {
		feeds: Feed[];
		stations: Station[];
	};

	let { feeds, stations }: Props = $props();

	const feedsWithUnread = $derived.by(() => {
		const result = new SvelteSet<string>();

		for (const item of Object.values(itemsState.itemSummariesById)) {
			if (!item.read) {
				result.add(item.feedId);
			}
		}

		return result;
	});

	const podcasts = $derived(feeds.filter((f) => f.kind === 'media'));
	const articles = $derived(feeds.filter((f) => f.kind === 'article'));

	function handleFeedClick(feedId: string) {
		selectFeed(feedId);
	}

	function handleStationClick(stationId: string) {
		selectStation(stationId);
	}

	function getFeedCardImageUrl(feed: Feed) {
		return feed.imageUrl;
	}
</script>

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
						{@const imageUrl = getFeedCardImageUrl(feed)}

						<button
							type="button"
							class="group relative flex w-full flex-col items-center gap-2 rounded-xl p-2 transition-colors contain-[paint] hover:bg-surface-hover"
							onclick={() => handleFeedClick(feed.id)}
							title={feed.title}
							aria-label={`Open feed ${feed.title}`}
						>
							<div
								class="bg-surface-elevated relative aspect-square w-full overflow-hidden rounded-xl shadow-sm"
							>
								{#if imageUrl}
									<img
										src={imageUrl}
										alt={feed.title}
										loading="lazy"
										decoding="async"
										fetchpriority="low"
										draggable="false"
										class="size-full object-cover"
									/>
								{:else}
									<span
										class="flex size-full items-center justify-center bg-linear-to-br from-primary-400 to-primary-600 text-2xl font-bold text-fg-inverse"
									>
										{(feed.title?.trim()?.[0] ?? '?').toUpperCase()}
									</span>
								{/if}

								{#if feedsWithUnread.has(feed.id)}
									<div class="absolute top-2 right-2 size-3 rounded-full bg-accent shadow-sm"></div>
								{/if}
							</div>

							<span class="line-clamp-2 w-full text-center text-xs font-medium text-fg">
								{feed.title}
							</span>
						</button>
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
						{@const imageUrl = getFeedCardImageUrl(feed)}

						<button
							type="button"
							class="group relative flex w-full flex-col items-center gap-2 rounded-xl p-2 transition-colors contain-[paint] hover:bg-surface-hover"
							onclick={() => handleFeedClick(feed.id)}
							title={feed.title}
							aria-label={`Open feed ${feed.title}`}
						>
							<div
								class="bg-surface-elevated relative aspect-square w-full overflow-hidden rounded-xl shadow-sm"
							>
								{#if imageUrl}
									<img
										src={imageUrl}
										alt={feed.title}
										loading="lazy"
										decoding="async"
										fetchpriority="low"
										draggable="false"
										class="size-full object-cover"
									/>
								{:else}
									<span
										class="flex size-full items-center justify-center bg-linear-to-br from-primary-400 to-primary-600 text-2xl font-bold text-fg-inverse"
									>
										{(feed.title?.trim()?.[0] ?? '?').toUpperCase()}
									</span>
								{/if}

								{#if feedsWithUnread.has(feed.id)}
									<div class="absolute top-2 right-2 size-3 rounded-full bg-accent shadow-sm"></div>
								{/if}
							</div>

							<span class="line-clamp-2 w-full text-center text-xs font-medium text-fg">
								{feed.title}
							</span>
						</button>
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
						{@const gradient = STATION_GRADIENTS[station.gradient]}

						<button
							type="button"
							class="group relative flex w-full flex-col items-center gap-2 rounded-xl p-2 transition-colors contain-[paint] hover:bg-surface-hover"
							onclick={() => handleStationClick(station.id)}
							title={station.name}
							aria-label={`Open station ${station.name}`}
						>
							<div
								class={`relative aspect-square w-full overflow-hidden rounded-xl bg-linear-to-br ${gradient.from} ${gradient.to} shadow-sm`}
							>
								<span
									class="flex size-full items-center justify-center text-2xl font-bold text-fg-inverse"
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										fill="none"
										viewBox="0 0 24 24"
										stroke-width="1.5"
										stroke="currentColor"
										class="size-10"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											d="M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z"
										/>
									</svg>
								</span>
							</div>

							<span class="line-clamp-2 w-full text-center text-xs font-medium text-fg">
								{station.name}
							</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</section>
