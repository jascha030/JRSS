import type { MenuIcon } from '@tauri-apps/api/image';
import { IconMenuItem, Menu, MenuItem, NativeIcon, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { navigateToFeed, navigateToStation } from '$lib/navigation/app-router';

import {
	selection,
	deleteFeed,
	deleteExistingStation,
	enqueueAudioItem,
	isAudioPlaying,
	isItemCurrentAudio,
	markItemRead,
	markItemsFavorite,
	markItemsRead,
	playAudioItemNext,
	requestOpenInReader,
	requestSeekTo,
	requestTogglePlayback,
	startPlaybackFromContext,
	stopPlayback
} from '$lib/state';
import type { Feed } from '$lib/types/feed';
import type { ArticleListItem, FeedListItem, MediaListItem } from '$lib/types/item';
import type { Station } from '$lib/types/station';

type ContextMenuItem = MenuItem | IconMenuItem | PredefinedMenuItem;

type ActionMenuItemOptions = {
	id: string;
	text: string;
	action?: () => void;
	enabled?: boolean;
	icon?: MenuIcon;
};

function supportsNativeMenuIcons(): boolean {
	return navigator.userAgent.includes('Mac');
}

async function createActionMenuItem({
	id,
	text,
	action,
	enabled,
	icon
}: ActionMenuItemOptions): Promise<MenuItem | IconMenuItem> {
	if (icon && supportsNativeMenuIcons()) {
		return IconMenuItem.new({ id, text, action, enabled, icon });
	}

	return MenuItem.new({ id, text, action, enabled });
}

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

	const items: ContextMenuItem[] = [];

	if (multiSelect && multiSelect.selectedIds.size > 1 && multiSelect.selectedIds.has(item.id)) {
		const count = multiSelect.selectedIds.size;
		const allRead = [...multiSelect.selectedIds].every((id) => multiSelect.itemsById[id]?.read);
		const allFavorite = [...multiSelect.selectedIds].every(
			(id) => multiSelect.itemsById[id]?.favorite
		);

		items.push(
			await createActionMenuItem({
				id: 'header',
				text: `${count} item${count > 1 ? 's' : ''} selected`,
				enabled: false
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await createActionMenuItem({
				id: allRead ? 'mark-unread' : 'mark-read',
				text: allRead ? 'Mark unread' : 'Mark read',
				icon: NativeIcon.MenuOnState,
				action: () => void markItemsRead([...multiSelect.selectedIds], !allRead)
			})
		);
		items.push(
			await createActionMenuItem({
				id: allFavorite ? 'remove-favorite' : 'add-favorite',
				text: allFavorite ? 'Remove favorite' : 'Add favorite',
				icon: NativeIcon.Bookmarks,
				action: () => void markItemsFavorite([...multiSelect.selectedIds], !allFavorite)
			})
		);

		const menu = await Menu.new({ items });
		await menu.popup();
		return;
	}

	items.push(
		await createActionMenuItem({
			id: 'open-reader',
			text: 'Open in reader',
			icon: NativeIcon.QuickLook,
			action: () => requestOpenInReader(item.id)
		})
	);

	if (isInSectionView()) {
		items.push(
			await createActionMenuItem({
				id: 'open-feed',
				text: 'Open feed',
				icon: NativeIcon.FollowLinkFreestanding,
				action: () => void navigateToFeed(item.feedId)
			})
		);
	}

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await createActionMenuItem({
			id: item.read ? 'mark-unread' : 'mark-read',
			text: item.read ? 'Mark unread' : 'Mark read',
			icon: NativeIcon.MenuOnState,
			action: () => void markItemRead(item.id, !item.read)
		})
	);
	items.push(
		await createActionMenuItem({
			id: item.favorite ? 'remove-favorite' : 'add-favorite',
			text: item.favorite ? 'Remove favorite' : 'Add favorite',
			icon: NativeIcon.Bookmarks,
			action: () => void markItemsFavorite([item.id], !item.favorite)
		})
	);

	if (item.url) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

		items.push(
			await createActionMenuItem({
				id: 'copy-url',
				text: 'Copy URL',
				icon: NativeIcon.Share,
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

	const items: ContextMenuItem[] = [];

	if (multiSelect && multiSelect.selectedIds.size > 1 && multiSelect.selectedIds.has(item.id)) {
		const count = multiSelect.selectedIds.size;
		const allRead = [...multiSelect.selectedIds].every((id) => multiSelect.itemsById[id]?.read);
		const allFavorite = [...multiSelect.selectedIds].every(
			(id) => multiSelect.itemsById[id]?.favorite
		);

		items.push(
			await createActionMenuItem({
				id: 'header',
				text: `${count} item${count > 1 ? 's' : ''} selected`,
				enabled: false
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await createActionMenuItem({
				id: allRead ? 'mark-unplayed' : 'mark-played',
				text: allRead ? 'Mark unplayed' : 'Mark played',
				icon: NativeIcon.MenuOnState,
				action: () => void markItemsRead([...multiSelect.selectedIds], !allRead)
			})
		);
		items.push(
			await createActionMenuItem({
				id: allFavorite ? 'remove-favorite' : 'add-favorite',
				text: allFavorite ? 'Remove favorite' : 'Add favorite',
				icon: NativeIcon.Bookmarks,
				action: () => void markItemsFavorite([...multiSelect.selectedIds], !allFavorite)
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await createActionMenuItem({
				id: 'add-to-queue',
				text: 'Add to queue',
				icon: NativeIcon.Add,
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
			await createActionMenuItem({
				id: 'play-next',
				text: 'Play next',
				icon: NativeIcon.RightFacingTriangle,
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
			await createActionMenuItem({
				id: 'pause',
				text: 'Pause',
				icon: NativeIcon.StopProgress,
				action: () => requestTogglePlayback()
			})
		);
	} else if (isCurrent) {
		items.push(
			await createActionMenuItem({
				id: 'play',
				text: 'Play',
				icon: NativeIcon.RightFacingTriangle,
				action: () => requestTogglePlayback()
			})
		);
	} else {
		items.push(
			await createActionMenuItem({
				id: 'play-now',
				text: 'Play now',
				icon: NativeIcon.RightFacingTriangle,
				action: () => startPlaybackFromContext(item)
			})
		);
	}

	if (hasProgress) {
		items.push(
			await createActionMenuItem({
				id: 'play-from-start',
				text: 'Play from start',
				icon: NativeIcon.Refresh,
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
			await createActionMenuItem({
				id: 'open-feed',
				text: 'Open feed',
				icon: NativeIcon.FollowLinkFreestanding,
				action: () => void navigateToFeed(item.feedId)
			})
		);
	}

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await createActionMenuItem({
			id: item.read ? 'mark-unplayed' : 'mark-played',
			text: item.read ? 'Mark unplayed' : 'Mark played',
			icon: NativeIcon.MenuOnState,
			action: () => void markItemRead(item.id, !item.read)
		})
	);
	items.push(
		await createActionMenuItem({
			id: item.favorite ? 'remove-favorite' : 'add-favorite',
			text: item.favorite ? 'Remove favorite' : 'Add favorite',
			icon: NativeIcon.Bookmarks,
			action: () => void markItemsFavorite([item.id], !item.favorite)
		})
	);

	if (isCurrent) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		items.push(
			await createActionMenuItem({
				id: 'stop-playback',
				text: 'Stop playback',
				icon: NativeIcon.StopProgress,
				action: () => stopPlayback()
			})
		);
	}

	items.push(
		await createActionMenuItem({
			id: 'play-next',
			text: 'Play next',
			icon: NativeIcon.RightFacingTriangle,
			action: () => playAudioItemNext(item)
		})
	);

	items.push(
		await createActionMenuItem({
			id: 'add-to-queue',
			text: 'Add to queue',
			icon: NativeIcon.Add,
			action: () => enqueueAudioItem(item)
		})
	);

	if (enclosureUrl) {
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

		items.push(
			await createActionMenuItem({
				id: 'copy-url',
				text: 'Copy URL',
				icon: NativeIcon.Share,
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

	const items: ContextMenuItem[] = [];

	if (isInSectionView()) {
		items.push(
			await createActionMenuItem({
				id: 'open-feed',
				text: 'Open feed',
				icon: NativeIcon.FollowLinkFreestanding,
				action: () => void navigateToFeed(feed.id)
			})
		);
		items.push(await PredefinedMenuItem.new({ item: 'Separator' }));
	}

	items.push(
		await createActionMenuItem({
			id: 'copy-url',
			text: 'Copy URL',
			icon: NativeIcon.Share,
			action: () => void writeText(feed.url)
		})
	);

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await createActionMenuItem({
			id: 'remove-feed',
			text: 'Remove feed',
			icon: NativeIcon.Remove,
			action: () => void deleteFeed(feed.id)
		})
	);

	const menu = await Menu.new({ items });
	await menu.popup();
}

export async function openStationContextMenu(event: MouseEvent, station: Station): Promise<void> {
	event.preventDefault();

	const items: ContextMenuItem[] = [];

	items.push(
		await createActionMenuItem({
			id: 'open-station',
			text: 'Open station',
			icon: NativeIcon.FollowLinkFreestanding,
			action: () => void navigateToStation(station.id)
		})
	);

	items.push(await PredefinedMenuItem.new({ item: 'Separator' }));

	items.push(
		await createActionMenuItem({
			id: 'remove-station',
			text: 'Remove station',
			icon: NativeIcon.Remove,
			action: () => void deleteExistingStation(station.id)
		})
	);

	const menu = await Menu.new({ items });
	await menu.popup();
}
