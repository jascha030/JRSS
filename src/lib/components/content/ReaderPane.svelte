<script lang="ts">
	import FeedArticle from '$lib/components/article/FeedArticle.svelte';
	import ReaderArticle from '$lib/components/article/ReaderArticle.svelte';
	import { feedsState } from '$lib/state';
	import {
		getSelectedItemOrNull,
		getIsSelectedItemReaderLoading,
		getHasSelectedItemReaderContent,
		getCanUseReaderMode
	} from '$lib/state/selectors.svelte';
	import { appUi, toggleReaderMaximized } from '$lib/hooks/useAppUi.svelte';
	import { loadReaderView } from '$lib/state';
	import { isMediaItem } from '$lib/types/item';
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import Icon from '@iconify/svelte';

	let { class: className = '' }: { class?: string } = $props();

	type ReaderPaneMode = 'feed' | 'reader';

	const selectedItem = $derived(getSelectedItemOrNull());
	const isSelectedItemReaderLoading = $derived(getIsSelectedItemReaderLoading());
	const hasSelectedItemReaderContent = $derived(getHasSelectedItemReaderContent());
	const canUseReaderMode = $derived(getCanUseReaderMode());

	const selectedItemFeed = $derived.by(() => {
		const item = selectedItem;
		return item !== null ? (feedsState.feeds.find((f) => f.id === item.feedId) ?? null) : null;
	});

	const isReaderPaneActive = $derived(
		appUi.readerPaneMode === 'reader' && hasSelectedItemReaderContent
	);

	const podcastImageUrl = $derived(
		selectedItem !== null && isMediaItem(selectedItem)
			? selectedItem.imageUrl || selectedItemFeed?.imageUrl
			: undefined
	);
</script>

<aside
	class="flex-col justify-between overflow-y-auto bg-surface-glass p-8 backdrop-blur-md {className}"
>
	{#if selectedItem}
		<div class="space-y-9">
			<div
				class="mx-auto w-full max-w-xl min-w-lg 3xl:max-w-3xl 3xl:min-w-3xl 4xl:max-w-4xl 4xl:min-w-4xl"
			>
				<div class="flex flex-wrap items-center gap-4">
					{#if canUseReaderMode && hasSelectedItemReaderContent}
						<SegmentedControl
							value={appUi.readerPaneMode}
							onValueChange={(details) => (appUi.readerPaneMode = details.value as ReaderPaneMode)}
						>
							<SegmentedControl.Label class="sr-only">Article view mode</SegmentedControl.Label>

							<SegmentedControl.Control class="rounded-2xl">
								<SegmentedControl.Indicator />

								<SegmentedControl.Item value="feed">
									<SegmentedControl.ItemText>
										<Icon icon="lucide:file-text" class="size-4" />
									</SegmentedControl.ItemText>
									<SegmentedControl.ItemHiddenInput />
								</SegmentedControl.Item>

								<SegmentedControl.Item value="reader">
									<SegmentedControl.ItemText>
										<Icon icon="lucide:book-open-text" class="size-4" />
									</SegmentedControl.ItemText>
									<SegmentedControl.ItemHiddenInput />
								</SegmentedControl.Item>
							</SegmentedControl.Control>
						</SegmentedControl>
					{:else if canUseReaderMode}
						<button
							class="preset-outlined-subtle btn rounded-xl"
							disabled={isSelectedItemReaderLoading}
							type="button"
							onclick={() => selectedItem && void loadReaderView(selectedItem.id)}
						>
							{isSelectedItemReaderLoading
								? 'Loading reader view...'
								: selectedItem?.readerStatus === 'failed'
									? 'Retry Reader View'
									: 'Load Reader View'}
						</button>
					{/if}

					<button
						class="preset-outlined-subtle ml-auto flex aspect-square items-center justify-center rounded-xl"
						style="width: 2.875rem; height: 2.875rem;"
						type="button"
						aria-label={appUi.isReaderMaximized ? 'Minimize reader' : 'Maximize reader'}
						onclick={toggleReaderMaximized}
					>
						<Icon
							icon={appUi.isReaderMaximized ? 'lucide:minimize' : 'lucide:maximize'}
							class="size-4"
						/>
					</button>
				</div>
			</div>

			{#if isReaderPaneActive}
				<ReaderArticle item={selectedItem} feedTitle={selectedItemFeed?.title} />
			{:else}
				<FeedArticle
					item={selectedItem}
					feedTitle={selectedItemFeed?.title}
					feedOrEpisodeImageUrl={podcastImageUrl}
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
