import { isTauriRuntime } from '$lib/services/tauri';
import { shareText as shareKitShareText } from '@choochmeque/tauri-plugin-sharekit-api';
import type { FeedListItem } from '$lib/types/item';

export async function shareItem(
	item: FeedListItem,
	position?: { x: number; y: number }
): Promise<void> {
	if (!isTauriRuntime()) {
		throw new Error('Tauri backend unavailable. Use `bun run tauri:dev` to run the desktop app.');
	}

	if (!item.url) {
		throw new Error('Item has no URL to share.');
	}

	await shareKitShareText(
		item.url,
		position ? { position: { ...position, preferredEdge: 'bottom' } } : undefined
	);
}
