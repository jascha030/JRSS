<script lang="ts">
	import './layout.css';
	import { page } from '$app/state';
	import App from '$lib/components/App.svelte';
	import MiniPlayer from '$lib/components/MiniPlayer.svelte';
	import { appUi, requestScrollToItem } from '$lib/hooks/useAppUi.svelte';
	import { useGlobalShortcuts } from '$lib/hooks/useGlobalShortcuts.svelte';
	import { parseAppUrl, toRouteSelection, navigateToSection } from '$lib/navigation/app-router';
	import {
		appState,
		applyRouteSelection,
		closeInspector,
		feedsState,
		getCurrentAudioItem,
		playbackState,
		stationsState
	} from '$lib/state';

	let { children } = $props();

	const isMiniWindow = $derived(page.url.searchParams.get('window') === 'mini');
	const currentPlaybackState = $derived(playbackState.currentPlaybackState);
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentRoute = $derived(parseAppUrl(page.url));
	const isInitialized = $derived(appState.initialized);
	let lastRouteItemId = $state<string | null>(null);

	useGlobalShortcuts();

	$effect(() => {
		applyRouteSelection(toRouteSelection(currentRoute));
		appUi.readerPaneMode =
			currentRoute.kind === 'section' ||
			currentRoute.kind === 'feed' ||
			currentRoute.kind === 'station'
				? currentRoute.readerPaneMode
				: 'feed';

		if (currentRoute.kind !== 'inspect') {
			closeInspector();
		}
	});

	$effect(() => {
		const itemId =
			currentRoute.kind === 'section' ||
			currentRoute.kind === 'feed' ||
			currentRoute.kind === 'station'
				? currentRoute.itemId
				: null;

		if (!itemId || itemId === lastRouteItemId) {
			lastRouteItemId = itemId;
			return;
		}

		lastRouteItemId = itemId;
		requestScrollToItem(itemId);
	});

	$effect(() => {
		if (!isInitialized) {
			return;
		}

		if (
			(currentRoute.kind === 'feed' || currentRoute.kind === 'inspect') &&
			!feedsState.feeds.some((feed) => feed.id === currentRoute.feedId)
		) {
			void navigateToSection('all', { replaceState: true });
			return;
		}

		if (
			currentRoute.kind === 'station' &&
			!stationsState.stations.some((station) => station.id === currentRoute.stationId)
		) {
			void navigateToSection('all', { replaceState: true });
		}
	});
</script>

{#if isMiniWindow}
	<MiniPlayer
		item={currentAudioItem}
		imageUrl={currentAudioItem?.imageUrl}
		playbackState={currentPlaybackState}
	/>
{:else}
	<App>
		{@render children?.()}
	</App>
{/if}
