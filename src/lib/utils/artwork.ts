export type ImageDimensions = {
	width: number;
	height: number;
};

const imageDimensionsCache = new Map<string, Promise<ImageDimensions | null>>();

export function loadImageDimensions(url: string): Promise<ImageDimensions | null> {
	const cached = imageDimensionsCache.get(url);
	if (cached) {
		return cached;
	}

	const dimensionsPromise = new Promise<ImageDimensions | null>((resolve) => {
		const image = new Image();

		image.onload = () => {
			resolve({
				width: image.naturalWidth,
				height: image.naturalHeight
			});
		};

		image.onerror = () => {
			resolve(null);
		};

		image.src = url;
	});

	imageDimensionsCache.set(url, dimensionsPromise);

	return dimensionsPromise;
}

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
		loadImageDimensions(episodeUrl),
		loadImageDimensions(feedUrl)
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
