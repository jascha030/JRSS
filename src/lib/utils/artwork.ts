import { getCachedImageDimensions } from '$lib/services/imageCache';

type ImageDimensions = {
	width: number;
	height: number;
};

function getArtworkScore(dimensions: ImageDimensions): number {
	return Math.min(dimensions.width, dimensions.height);
}

export async function pickBestArtworkUrl(
	episodeUrl?: string,
	feedUrl?: string
): Promise<string | undefined> {
	if (!episodeUrl) {
		return feedUrl;
	}

	if (!feedUrl || feedUrl === episodeUrl) {
		return episodeUrl;
	}

	const [episodeDimensions, feedDimensions] = await Promise.all([
		getCachedImageDimensions(episodeUrl),
		getCachedImageDimensions(feedUrl)
	]);

	if (!episodeDimensions) {
		return feedDimensions ? feedUrl : undefined;
	}

	if (!feedDimensions) {
		return episodeUrl;
	}

	return getArtworkScore(episodeDimensions) >= getArtworkScore(feedDimensions)
		? episodeUrl
		: feedUrl;
}
