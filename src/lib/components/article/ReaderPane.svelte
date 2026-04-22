<script lang="ts">
	import FeedArticle from '$lib/components/article/FeedArticle.svelte';
	import ReaderArticle from '$lib/components/article/ReaderArticle.svelte';
	import type { Feed, FeedItem } from '$lib/types/rss';
	import { isMediaItem } from '$lib/types/rss';
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';

	type ReaderPaneMode = 'feed' | 'reader';

	type Props = {
		selectedItem: FeedItem | null;
		selectedItemFeed: Feed | null;
		readerPaneMode: ReaderPaneMode;
		isSelectedItemReaderLoading: boolean;
		hasSelectedItemReaderContent: boolean;
		isReaderPaneActive: boolean;
		canUseReaderMode: boolean;
		onLoadReaderView: (itemId: string) => Promise<void>;
		onReaderPaneModeChange: (mode: ReaderPaneMode) => void;
	};

	let {
		selectedItem,
		selectedItemFeed,
		readerPaneMode,
		isSelectedItemReaderLoading,
		hasSelectedItemReaderContent,
		isReaderPaneActive,
		canUseReaderMode,
		onLoadReaderView,
		onReaderPaneModeChange
	}: Props = $props();

	const podcastImageUrl = $derived(
		selectedItem && isMediaItem(selectedItem) ? selectedItemFeed?.imageUrl : undefined
	);
</script>

<aside
	class="hidden min-h-0 min-w-0 flex-col justify-between overflow-y-auto bg-surface-glass p-8 xl:flex xl:flex-1 2xl:basis-2/3"
>
	{#if selectedItem}
		<div class="space-y-9">
			<div
				class="mx-auto w-full max-w-xl min-w-lg 2xl:max-w-3xl 2xl:min-w-3xl 3xl:max-w-4xl 3xl:min-w-4xl"
			>
				<div class="flex flex-wrap items-center gap-4">
					{#if canUseReaderMode && hasSelectedItemReaderContent}
						<SegmentedControl
							value={readerPaneMode}
							onValueChange={(details) => onReaderPaneModeChange(details.value as ReaderPaneMode)}
						>
							<SegmentedControl.Label class="sr-only">Article view mode</SegmentedControl.Label>

							<SegmentedControl.Control class="rounded-2xl">
								<SegmentedControl.Indicator />

								<SegmentedControl.Item value="feed">
									<SegmentedControl.ItemText>Feed view</SegmentedControl.ItemText>
									<SegmentedControl.ItemHiddenInput />
								</SegmentedControl.Item>

								<SegmentedControl.Item value="reader">
									<SegmentedControl.ItemText>Reader view</SegmentedControl.ItemText>
									<SegmentedControl.ItemHiddenInput />
								</SegmentedControl.Item>
							</SegmentedControl.Control>
						</SegmentedControl>
					{:else if canUseReaderMode}
						<button
							class="preset-outlined-subtle btn rounded-xl"
							disabled={isSelectedItemReaderLoading}
							type="button"
							onclick={() => onLoadReaderView(selectedItem.id)}
						>
							{isSelectedItemReaderLoading
								? 'Loading reader view...'
								: selectedItem.readerStatus === 'failed'
									? 'Retry Reader View'
									: 'Load Reader View'}
						</button>
					{/if}
				</div>
			</div>

			{#if isReaderPaneActive}
				<ReaderArticle item={selectedItem} feedTitle={selectedItemFeed?.title} />
			{:else}
				<FeedArticle
					item={selectedItem}
					feedTitle={selectedItemFeed?.title}
					feedImageUrl={podcastImageUrl}
				/>
			{/if}
		</div>
	{:else}
		<div class="flex h-full min-h-84 flex-col justify-between">
			<div>
				<p class="text-sm font-medium tracking-[0.18em] text-fg-muted uppercase">Reader</p>

				<h1 class="mt-4 text-3xl font-semibold tracking-tight text-fg">No item selected</h1>

				<p class="mt-5 max-w-xl text-sm leading-7 text-fg-secondary">
					Pick an item from the list to read its details here. When a view has visible items, the
					first one is selected automatically.
				</p>
			</div>
		</div>
	{/if}
</aside>
