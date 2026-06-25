import { pickBestArtworkUrl } from '$lib/utils/artwork';
import { getFeedById, feedImageUrls, feedImageUrlsLarge } from '$lib/state';

export function useArtwork(
	getEpisodeImageUrl: () => string | undefined,
	getFeedId: () => string | undefined,
	useLarge = false
) {
	const brokenImageUrls = $state<Record<string, true>>({});

	const activeEpisodeUrl = $derived.by(() => {
		const url = getEpisodeImageUrl();
		return url && !brokenImageUrls[url] ? url : undefined;
	});

	const activeFallbackUrl = $derived.by(() => {
		const feedId = getFeedId();
		if (!feedId) return undefined;
		const cache = useLarge ? feedImageUrlsLarge : feedImageUrls;
		const url = cache[feedId] || getFeedById(feedId)?.imageUrl;
		return url && !brokenImageUrls[url] ? url : undefined;
	});

	const artworkChoice = $derived.by(() => pickBestArtworkUrl(activeEpisodeUrl, activeFallbackUrl));

	function handleError(event: Event) {
		const target = event.currentTarget;
		if (!(target instanceof HTMLImageElement)) return;

		const failedUrl = target.currentSrc || target.src;
		if (!failedUrl) return;

		brokenImageUrls[failedUrl] = true;
	}

	return {
		get activeEpisodeUrl() {
			return activeEpisodeUrl;
		},
		get activeFallbackUrl() {
			return activeFallbackUrl;
		},
		get artworkChoice() {
			return artworkChoice;
		},
		handleError
	};
}
