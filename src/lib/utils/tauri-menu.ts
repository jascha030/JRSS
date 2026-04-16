import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

import {
	deleteFeed,
	enqueueAudioItem,
	isItemCurrentAudio,
	playAudioItemNext,
	stopPlayback
} from '$lib/stores/app.svelte';
import type { Feed, FeedListItem } from '$lib/types/rss';

/**
 * Native context menu for audio items with queue and clipboard actions.
 */
export async function openAudioContextMenu(event: MouseEvent, item: FeedListItem): Promise<void> {
	event.preventDefault();

	const enclosureUrl = item.mediaEnclosure?.url;

	const items: Array<MenuItem | PredefinedMenuItem> = [];

	if (isItemCurrentAudio(item.id)) {
		items.push(
			await MenuItem.new({
				id: 'stop-playback',
				text: 'Stop playback',
				action: () => stopPlayback()
			})
		);
	}

	items.push(
		await MenuItem.new({
			id: 'play-next',
			text: 'Play next',
			action: () => playAudioItemNext(item)
		})
	);

	items.push(
		await MenuItem.new({
			id: 'add-to-queue',
			text: 'Add to queue',
			action: () => enqueueAudioItem(item)
		})
	);

	if (enclosureUrl) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

		items.push(
			await MenuItem.new({
				id: 'copy-url',
				text: 'Copy URL',
				action: () => void writeText(enclosureUrl)
			})
		);
	}

	const menu = await Menu.new({ items });
	await menu.popup();
}

export async function openFeedContextMenu(event: MouseEvent, feed: Feed): Promise<void> {
	event.preventDefault();

	const items: Array<MenuItem | PredefinedMenuItem> = [
		await MenuItem.new({
			id: 'copy-url',
			text: 'Copy URL',
			action: () => void writeText(feed.url)
		}),
		await PredefinedMenuItem.new({ item: 'Separator' }),
		await MenuItem.new({
			id: 'remove-feed',
			text: 'Remove feed',
			action: () => void deleteFeed(feed.id)
		})
	];

	const menu = await Menu.new({ items });
	await menu.popup();
}
