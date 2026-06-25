import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';
import { convertFileSrc } from '@tauri-apps/api/core';

const cacheUrlMap = new Map<string, string>();

export async function getCachedImageUrl(
	imageUrl: string,
	size?: number
): Promise<string | undefined> {
	if (!isTauriRuntime() || !imageUrl.trim()) {
		return imageUrl || undefined;
	}

	const cacheKey = size ? `${imageUrl}@${size}` : imageUrl;
	const cached = cacheUrlMap.get(cacheKey);
	if (cached) return cached;

	try {
		const filePath = await invokeCommand<string>('get_cached_image_path', {
			imageUrl,
			size: size ?? null
		});
		const assetUrl = convertFileSrc(filePath);
		cacheUrlMap.set(cacheKey, assetUrl);
		return assetUrl;
	} catch {
		return imageUrl || undefined;
	}
}

export async function getCachedImageDimensions(
	imageUrl: string
): Promise<{ width: number; height: number } | null> {
	if (!isTauriRuntime() || !imageUrl.trim()) {
		return null;
	}

	try {
		const result = await invokeCommand<[number, number] | null>('get_cached_image_dimensions', {
			imageUrl
		});
		if (!result) return null;
		return { width: result[0], height: result[1] };
	} catch {
		return null;
	}
}
