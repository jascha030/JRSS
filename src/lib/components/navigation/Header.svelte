<script lang="ts">
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	import type { Feed } from '$lib/types/feed';
	import type { FeedListItem } from '$lib/types/item';
	import type { Station } from '$lib/types/station';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import GlobalSearch from '../search/GlobalSearch.svelte';

	type Props = {
		onOpenDialog: () => void;
		onSelectResult: (item: FeedListItem) => void;
		onSelectFeedResult: (feed: Feed) => void;
		onSelectStationResult: (station: Station) => void;
		onAddFeed: (url: string) => void;
	};

	let {
		onOpenDialog,
		onSelectResult,
		onSelectFeedResult,
		onSelectStationResult,
		onAddFeed
	}: Props = $props();

	$effect(() => {
		let unlistenAddFeed: UnlistenFn | undefined;

		void listen('menu-add-feed', () => {
			onOpenDialog();
		}).then((unlisten) => {
			unlistenAddFeed = unlisten;
		});

		return () => {
			unlistenAddFeed?.();
		};
	});
</script>

<div class="pointer-events-none flex w-full items-center gap-4 px-2">
	<GlobalSearch {onSelectResult} {onSelectFeedResult} {onSelectStationResult} {onAddFeed} />
	<IconButton icon="lucide:plus" title="Add feed" variant="accent" onclick={onOpenDialog} />
</div>
