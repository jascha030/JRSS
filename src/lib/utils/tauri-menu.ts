import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

import {
	selection,
	deleteFeed,
	enqueueAudioItem,
	isAudioPlaying,
	isItemCurrentAudio,
	markItemRead,
	markItemsRead,
	playAudioItemNext,
	requestOpenInReader,
	requestSeekTo,
	requestTogglePlayback,
	selectFeed,
	startPlaybackFromContext,
	stopPlayback
} from '$lib/state';
import type { Feed } from '$lib/types/feed';
import type { ArticleListItem, FeedListItem, MediaListItem } from '$lib/types/item';

/**
 * Whether the user is browsing a section (all/unread/media) rather than
 * a specific feed. When true, "Open feed" is a meaningful navigation action.
 */
function isInSectionView(): boolean {
	return selection.selectedFeedId === null;
}

type MultiSelectOptions = {
	selectedIds: Set<string>;
	itemsById: Record<string, FeedListItem>;
};

/**
 * Native context menu for article items with reader, feed, and read-state actions.
 */
export async function openArticleContextMenu(
	event: MouseEvent,
	item: ArticleListItem,
	multiSelect?: MultiSelectOptions
): Promise<void> {
	event.preventDefault();

	const items: Array<MenuItem | PredefinedMenuItem> = [];

	if (multiSelect && multiSelect.selectedIds.size > 1 && multiSelect.selectedIds.has(item.id)) {
		const count = multiSelect.selectedIds.size;
		const allRead = [...multiSelect.selectedIds].every((id) => multiSelect.itemsById[id]?.read);

		items.push(
			await MenuItem.new({
				id: 'header',
				text: `${count} item${count > 1 ? 's' : ''} selected`,
				enabled: false
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await MenuItem.new({
				id: allRead ? 'mark-unread' : 'mark-read',
				text: allRead ? 'Mark unread' : 'Mark read',
				action: () => void markItemsRead([...multiSelect.selectedIds], !allRead)
			})
		);

		const menu = await Menu.new({ items });
		await menu.popup();
		return;
	}

	items.push(
		await MenuItem.new({
			id: 'open-reader',
			text: 'Open in reader',
			action: () => requestOpenInReader(item.id)
		})
	);

	if (isInSectionView()) {
		items.push(
			await MenuItem.new({
				id: 'open-feed',
				text: 'Open feed',
				action: () => selectFeed(item.feedId)
			})
		);
	}

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await MenuItem.new({
			id: item.read ? 'mark-unread' : 'mark-read',
			text: item.read ? 'Mark unread' : 'Mark read',
			action: () => void markItemRead(item.id, !item.read)
		})
	);

	if (item.url) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

		items.push(
			await MenuItem.new({
				id: 'copy-url',
				text: 'Copy URL',
				action: () => void writeText(item.url)
			})
		);
	}

	const menu = await Menu.new({ items });
	await menu.popup();
}

/**
 * Native context menu for audio items with playback, queue, and clipboard actions.
 */
export async function openAudioContextMenu(
	event: MouseEvent,
	item: MediaListItem,
	multiSelect?: MultiSelectOptions
): Promise<void> {
	event.preventDefault();

	const items: Array<MenuItem | PredefinedMenuItem> = [];

	if (multiSelect && multiSelect.selectedIds.size > 1 && multiSelect.selectedIds.has(item.id)) {
		const count = multiSelect.selectedIds.size;
		const allRead = [...multiSelect.selectedIds].every((id) => multiSelect.itemsById[id]?.read);

		items.push(
			await MenuItem.new({
				id: 'header',
				text: `${count} item${count > 1 ? 's' : ''} selected`,
				enabled: false
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await MenuItem.new({
				id: allRead ? 'mark-unplayed' : 'mark-played',
				text: allRead ? 'Mark unplayed' : 'Mark played',
				action: () => void markItemsRead([...multiSelect.selectedIds], !allRead)
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await MenuItem.new({
				id: 'add-to-queue',
				text: 'Add to queue',
				action: () => {
					for (const id of multiSelect.selectedIds) {
						const it = multiSelect.itemsById[id];
						if (it && it.itemType === 'media') {
							enqueueAudioItem(it);
						}
					}
				}
			})
		);
		items.push(
			await MenuItem.new({
				id: 'play-next',
				text: 'Play next',
				action: () => {
					for (const id of [...multiSelect.selectedIds].reverse()) {
						const it = multiSelect.itemsById[id];
						if (it && it.itemType === 'media') {
							playAudioItemNext(it);
						}
					}
				}
			})
		);

		const menu = await Menu.new({ items });
		await menu.popup();
		return;
	}

	const enclosureUrl = item.mediaEnclosure.url;
	const isCurrent = isItemCurrentAudio(item.id);
	const playing = isCurrent && isAudioPlaying();
	const hasProgress = item.playbackPositionSeconds > 0;

	if (playing) {
		items.push(
			await MenuItem.new({
				id: 'pause',
				text: 'Pause',
				action: () => requestTogglePlayback()
			})
		);
	} else if (isCurrent) {
		items.push(
			await MenuItem.new({
				id: 'play',
				text: 'Play',
				action: () => requestTogglePlayback()
			})
		);
	} else {
		items.push(
			await MenuItem.new({
				id: 'play-now',
				text: 'Play now',
				action: () => startPlaybackFromContext(item)
			})
		);
	}

	if (hasProgress) {
		items.push(
			await MenuItem.new({
				id: 'play-from-start',
				text: 'Play from start',
				action: () => {
					if (isCurrent) {
						requestSeekTo(0);
						if (!playing) {
							requestTogglePlayback();
						}
					} else {
						item.playbackPositionSeconds = 0;
						startPlaybackFromContext(item);
					}
				}
			})
		);
	}

	if (isInSectionView()) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await MenuItem.new({
				id: 'open-feed',
				text: 'Open feed',
				action: () => selectFeed(item.feedId)
			})
		);
	}

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await MenuItem.new({
			id: item.read ? 'mark-unplayed' : 'mark-played',
			text: item.read ? 'Mark unplayed' : 'Mark played',
			action: () => void markItemRead(item.id, !item.read)
		})
	);

	if (isCurrent) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
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

/**
 * Native context menu for feeds with clipboard and deletion actions.
 */
export async function openFeedContextMenu(event: MouseEvent, feed: Feed): Promise<void> {
	event.preventDefault();

	const items: Array<MenuItem | PredefinedMenuItem> = [];

	if (isInSectionView()) {
		items.push(
			await MenuItem.new({
				id: 'open-feed',
				text: 'Open feed',
				action: () => selectFeed(feed.id)
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
	}

	items.push(
		await MenuItem.new({
			id: 'copy-url',
			text: 'Copy URL',
			action: () => void writeText(feed.url)
		})
	);

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await MenuItem.new({
			id: 'remove-feed',
			text: 'Remove feed',
			action: () => void deleteFeed(feed.id)
		})
	);

	const menu = await Menu.new({ items });
	await menu.popup();
}
