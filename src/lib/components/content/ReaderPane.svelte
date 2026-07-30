<script lang="ts">
	import FeedArticle from '$lib/components/article/FeedArticle.svelte';
	import ReaderArticle from '$lib/components/article/ReaderArticle.svelte';
	import { feedsState, selection } from '$lib/state';
	import { getSelectedItem, readerState } from '$lib/state';
	import { markItemFavorite } from '$lib/state/items.svelte';
	import { shareItem } from '$lib/services/share';
	import { appUi, toggleReaderMaximized, type ReaderPaneMode } from '$lib/hooks/useAppUi.svelte';
	import { loadReaderView } from '$lib/state';
	import { isMediaItem } from '$lib/types/item';
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import Icon from '@iconify/svelte';
	import { toast } from 'svelte-sonner';
	import IconButton from '../ui/IconButton.svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';

	let {
		class: className = ''
	}: {
		class?: string;
	} = $props();

	function isReaderPaneMode(value: string | null): value is ReaderPaneMode {
		return value === 'feed' || value === 'reader';
	}

	const selectedItem = $derived(getSelectedItem());

	const isSelectedItemReaderLoading = $derived(
		selectedItem ? readerState.readerLoadingItemIds.includes(selectedItem.id) : false
	);

	const hasSelectedItemReaderContent = $derived(selectedItem?.readerStatus === 'ready');

	const canUseReaderMode = $derived(selectedItem ? !isMediaItem(selectedItem) : false);

	const readerPaneMode = $derived(appUi.readerPaneMode);
	const isReaderMaximized = $derived(appUi.isReaderMaximized);

	const selectedItemFeed = $derived(
		selectedItem ? (feedsState.feeds.find((f) => f.id === selectedItem.feedId) ?? null) : null
	);

	const isReaderPaneActive = $derived(readerPaneMode === 'reader' && hasSelectedItemReaderContent);

	const podcastImageUrl = $derived(
		selectedItem !== null && isMediaItem(selectedItem)
			? selectedItem.imageUrl || selectedItemFeed?.imageUrl
			: undefined
	);

	const feedId = $derived(selectedItemFeed?.id ?? null);
	const feedLink = $derived(
		feedId && selection.selectedFeedId !== feedId ? `/feeds/${feedId}` : null
	);

	const readerViewButtonLabel = $derived(
		isSelectedItemReaderLoading
			? 'Loading reader view...'
			: selectedItem?.readerStatus === 'failed'
				? 'Retry Reader View'
				: 'Load Reader View'
	);

	async function handleLoadReaderView(itemId: string): Promise<void> {
		const updatedItem = await loadReaderView(itemId);

		if (updatedItem.readerStatus === 'ready') {
			appUi.readerPaneMode = 'reader';
			return;
		}

		appUi.readerPaneMode = 'feed';
		toast.error('Reader view was unavailable for this item. Showing feed content instead.');
	}
</script>

<aside
	class="flex-col justify-between overflow-y-auto bg-surface-glass p-8 backdrop-blur-md {className}"
>
	{#if selectedItem}
		<div class="space-y-9">
			<div
				class="mx-auto w-full {isReaderMaximized
					? 'max-w-5xl 3xl:max-w-7xl 4xl:max-w-[100rem]'
					: 'max-w-xl min-w-lg 3xl:max-w-3xl 3xl:min-w-3xl 4xl:max-w-4xl 4xl:min-w-4xl'}"
			>
				<div class="flex flex-wrap items-center gap-4">
					{#if canUseReaderMode && hasSelectedItemReaderContent}
						<SegmentedControl
							value={readerPaneMode}
							onValueChange={(details) => {
								if (isReaderPaneMode(details.value)) {
									appUi.readerPaneMode = details.value;
								}
							}}
						>
							<SegmentedControl.Label class="sr-only">Article view mode</SegmentedControl.Label>

							<SegmentedControl.Control class="rounded-xl">
								<SegmentedControl.Indicator />

								<SegmentedControl.Item value="feed">
									<SegmentedControl.ItemText>
										<Icon icon="lucide:file-text" />
									</SegmentedControl.ItemText>
									<SegmentedControl.ItemHiddenInput />
								</SegmentedControl.Item>

								<SegmentedControl.Item value="reader">
									<SegmentedControl.ItemText>
										<Icon icon="lucide:book-open-text" />
									</SegmentedControl.ItemText>
									<SegmentedControl.ItemHiddenInput />
								</SegmentedControl.Item>
							</SegmentedControl.Control>
						</SegmentedControl>
					{:else if canUseReaderMode}
						<IconButton
							icon="lucide:file-text"
							disabled={isSelectedItemReaderLoading}
							label={readerViewButtonLabel}
							onclick={() =>
								selectedItem &&
								void handleLoadReaderView(selectedItem.id).catch((error: unknown) => {
									appUi.readerPaneMode = 'feed';
									toast.error(
										error instanceof Error
											? error.message
											: 'Unable to load reader view for this item.'
									);
								})}
						/>
					{/if}

					<div class="ml-auto flex items-center gap-2">
						<IconButton
							icon={selectedItem.favorite ? 'heroicons:heart-solid' : 'heroicons:heart'}
							label={selectedItem.favorite ? 'Remove favorite' : 'Add favorite'}
							onclick={() => void markItemFavorite(selectedItem.id, !selectedItem.favorite)}
						/>

						<IconButton
							icon="lucide:share"
							label="Share item"
							onclick={(event: MouseEvent) =>
								void shareItem(selectedItem, { x: event.clientX, y: event.clientY }).catch(
									(error: unknown) => {
										toast.error(
											error instanceof Error ? error.message : 'Unable to share this item.'
										);
									}
								)}
						/>

						<IconButton
							icon="iconoir:safari"
							label="Open in Browser"
							onclick={() => {
								void openUrl(selectedItem.url);
							}}
						/>

						<IconButton
							icon={isReaderMaximized ? 'lucide:minimize' : 'lucide:maximize'}
							label={isReaderMaximized ? 'Minimize reader' : 'Maximize reader'}
							onclick={toggleReaderMaximized}
						/>
					</div>
				</div>
			</div>

			{#if isReaderPaneActive}
				<ReaderArticle
					item={selectedItem}
					feedTitle={selectedItemFeed?.title}
					{feedLink}
					maximized={isReaderMaximized}
				/>
			{:else}
				<FeedArticle
					item={selectedItem}
					feedTitle={selectedItemFeed?.title}
					{feedLink}
					feedOrEpisodeImageUrl={podcastImageUrl}
					maximized={isReaderMaximized}
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
