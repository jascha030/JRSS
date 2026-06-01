<script lang="ts">
	import { onNavigate } from '$app/navigation';
	import './layout.css';
	import { page } from '$app/state';
	import App from '$lib/components/App.svelte';
	import MiniPlayer from '$lib/components/MiniPlayer.svelte';
	import { appUi, requestScrollToItem } from '$lib/hooks/useAppUi.svelte';
	import { useGlobalShortcuts } from '$lib/hooks/useGlobalShortcuts.svelte';
	import {
		parseAppUrl,
		toRouteSelection,
		navigateToSection,
		type AppRoute
	} from '$lib/utils/navigation/app-router';
	import { Toaster } from 'svelte-sonner';
	import {
		appState,
		applyRouteSelection,
		closeInspector,
		feedsState,
		getCurrentAudioItem,
		playbackState,
		stationsState
	} from '$lib/state';

	type RouteLoadingCoverState = 'idle' | 'covering' | 'loading' | 'revealing';

	const ROUTE_COVER_MIN_MS = 120;
	const ROUTE_COVER_OUT_MS = 180;

	let { children } = $props();

	const isMiniWindow = $derived(page.url.searchParams.get('window') === 'mini');
	const currentPlaybackState = $derived(playbackState.currentPlaybackState);
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentRoute = $derived(parseAppUrl(page.url));
	const isInitialized = $derived(appState.initialized);
	let lastRouteItemId = $state<string | null>(null);
	let routeLoadingCoverState = $state<RouteLoadingCoverState>('idle');
	let routeLoadingToken = 0;

	useGlobalShortcuts();

	function sleep(milliseconds: number): Promise<void> {
		return new Promise((resolve) => {
			window.setTimeout(resolve, milliseconds);
		});
	}

	onNavigate((navigation) => {
		if (navigation.from?.url?.pathname === navigation.to?.url?.pathname) {
			return;
		}

		const startViewTransition = document.startViewTransition;

		if (!navigation.to || !startViewTransition) {
			const token = ++routeLoadingToken;
			const startedAt = performance.now();
			routeLoadingCoverState = 'covering';

			return new Promise<void>((resolve) => {
				requestAnimationFrame(() => {
					routeLoadingCoverState = 'loading';
					resolve();

					void navigation.complete.then(async () => {
						const remaining = ROUTE_COVER_MIN_MS - (performance.now() - startedAt);
						if (remaining > 0) {
							await sleep(remaining);
						}

						if (token !== routeLoadingToken) {
							return;
						}

						routeLoadingCoverState = 'revealing';
						await sleep(ROUTE_COVER_OUT_MS);

						if (token === routeLoadingToken) {
							routeLoadingCoverState = 'idle';
						}
					});
				});
			});
		}

		const token = ++routeLoadingToken;
		const startedAt = performance.now();
		routeLoadingCoverState = 'covering';

		return new Promise<void>((resolve) => {
			requestAnimationFrame(() => {
				startViewTransition.call(document, async () => {
					routeLoadingCoverState = 'loading';
					resolve();
					await navigation.complete;

					const remaining = ROUTE_COVER_MIN_MS - (performance.now() - startedAt);
					if (remaining > 0) {
						await sleep(remaining);
					}

					if (token !== routeLoadingToken) {
						return;
					}

					routeLoadingCoverState = 'revealing';
					await sleep(ROUTE_COVER_OUT_MS);

					if (token === routeLoadingToken) {
						routeLoadingCoverState = 'idle';
					}
				});
			});
		});
	});

	function isContentRoute(
		route: AppRoute
	): route is Extract<AppRoute, { kind: 'section' | 'feed' | 'station' }> {
		return route.kind === 'section' || route.kind === 'feed' || route.kind === 'station';
	}

	$effect(() => {
		applyRouteSelection(toRouteSelection(currentRoute));

		appUi.readerPaneMode = isContentRoute(currentRoute) ? currentRoute.readerPaneMode : 'feed';

		if (currentRoute.kind !== 'inspect') {
			closeInspector();
		}

		const itemId = isContentRoute(currentRoute) ? currentRoute.itemId : null;
		const shouldScroll = itemId !== null && itemId !== lastRouteItemId;

		lastRouteItemId = itemId;

		if (shouldScroll) {
			requestScrollToItem(itemId);
		}
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
	<App {routeLoadingCoverState}>
		{@render children?.()}
	</App>
{/if}

<Toaster
	position="top-center"
	class="mt-14"
	toastOptions={{
		unstyled: true,
		classes: {
			toast: 'toast-base',
			success: 'toast-success',
			error: 'toast-error',
			warning: 'toast-warning',
			info: 'toast-info',
			title: 'text-sm font-medium text-fg',
			description: 'text-xs text-fg-secondary mt-0.5'
		}
	}}
/>
