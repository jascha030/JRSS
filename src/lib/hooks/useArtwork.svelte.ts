import { pickBestArtworkUrl } from '$lib/utils/artwork';

export function useArtwork(
	getEpisodeImageUrl: () => string | undefined,
	getFallbackImageUrl: () => string | undefined
) {
	const brokenImageUrls = $state<Record<string, true>>({});

	const activeEpisodeUrl = $derived.by(() => {
		const url = getEpisodeImageUrl();
		return url && !brokenImageUrls[url] ? url : undefined;
	});

	const activeFallbackUrl = $derived.by(() => {
		const url = getFallbackImageUrl();
		return url && !brokenImageUrls[url] ? url : undefined;
	});

	const artworkChoice = $derived(pickBestArtworkUrl(activeEpisodeUrl, activeFallbackUrl));

	function handleError(event: Event) {
		const target = event.currentTarget;
		if (!(target instanceof HTMLImageElement)) return;

		const failedUrl = target.currentSrc || target.src;
		if (!failedUrl) return;

		brokenImageUrls[failedUrl] = true;
	}

	return {
		activeEpisodeUrl,
		activeFallbackUrl,
		artworkChoice,
		handleError
	};
}
