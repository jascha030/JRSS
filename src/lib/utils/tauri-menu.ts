import { Menu } from '@tauri-apps/api/menu';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

import { enqueueAudioItem, playAudioItemNext } from '$lib/stores/app.svelte';
import type { FeedListItem } from '$lib/types/rss';

/**
 * Native context menu for audio items with queue and clipboard actions.
 */
export async function openAudioContextMenu(event: MouseEvent, item: FeedListItem): Promise<void> {
	event.preventDefault();

	const enclosureUrl = item.mediaEnclosure?.url;

	const menu = await Menu.new({
		items: [
			{ id: 'play-next', text: 'Play next', action: () => playAudioItemNext(item) },
			{ id: 'add-to-queue', text: 'Add to queue', action: () => enqueueAudioItem(item) },
			...(enclosureUrl
				? [
						{
							id: 'copy-url',
							text: 'Copy URL',
							action: () => void writeText(enclosureUrl)
						}
					]
				: [])
		]
	});

	await menu.popup();
}
