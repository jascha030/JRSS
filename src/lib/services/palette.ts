import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';

export async function extractCoverPalette(imageUrl: string): Promise<string[]> {
	if (!isTauriRuntime()) {
		return [];
	}

	if (!imageUrl.trim()) {
		return [];
	}

	return invokeCommand<string[]>('extract_cover_palette', { imageUrl });
}
