<script lang="ts">
	import FeedArticle from '$lib/components/FeedArticle.svelte';
	import ReaderArticle from '$lib/components/ReaderArticle.svelte';
	import {
		enqueueAudioItem,
		playAudioItemNext,
		startPlaybackFromContext
	} from '$lib/stores/app.svelte';
	import type { Feed, FeedItem, PlaybackState } from '$lib/types/rss';

	type ReaderPaneMode = 'feed' | 'reader';

	type Props = {
		selectedItem: FeedItem | null;
		selectedItemFeed: Feed | null;
		currentAudioItem: FeedItem | null;
		currentPlaybackState: PlaybackState | null;
		readerPaneMode: ReaderPaneMode;
		readerNotice: string;
		isSelectedItemReaderLoading: boolean;
		hasSelectedItemReaderContent: boolean;
		isReaderPaneActive: boolean;
		canUseReaderMode: boolean;
		onStopPlayback: () => void;
		onLoadReaderView: (itemId: string) => Promise<void>;
		onReaderPaneModeChange: (mode: ReaderPaneMode) => void;
	};

	let {
		selectedItem,
		selectedItemFeed,
		currentAudioItem,
		currentPlaybackState,
		readerPaneMode,
		readerNotice,
		isSelectedItemReaderLoading,
		hasSelectedItemReaderContent,
		isReaderPaneActive,
		canUseReaderMode,
		onStopPlayback,
		onLoadReaderView,
		onReaderPaneModeChange
	}: Props = $props();

	const podcastImageUrl = $derived(
		selectedItem?.mediaEnclosure ? selectedItemFeed?.imageUrl : undefined
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
					{#if selectedItem.mediaEnclosure}
						<button
							class="btn-secondary rounded-2xl px-4 py-3"
							type="button"
							onclick={() => startPlaybackFromContext(selectedItem)}
						>
							{selectedItem.playbackPositionSeconds > 0 ? 'Resume playback' : 'Start playback'}
						</button>

						<button
							class="btn-secondary rounded-2xl px-4 py-3 text-xs"
							type="button"
							onclick={() => playAudioItemNext(selectedItem)}
						>
							Play next
						</button>

						<button
							class="btn-secondary rounded-2xl px-4 py-3 text-xs"
							type="button"
							onclick={() => enqueueAudioItem(selectedItem)}
						>
							Add to queue
						</button>

						{#if currentPlaybackState && currentAudioItem?.id === selectedItem.id}
							<button
								class="btn-secondary rounded-2xl px-4 py-3"
								type="button"
								onclick={onStopPlayback}
							>
								Stop playback
							</button>
						{/if}
					{/if}

					{#if canUseReaderMode && hasSelectedItemReaderContent}
						<div class="inline-flex rounded-2xl border border-border-strong bg-surface p-1">
							<button
								class={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${
									readerPaneMode === 'feed'
										? 'bg-interactive text-interactive-text'
										: 'text-fg-muted hover:text-fg'
								}`}
								type="button"
								onclick={() => onReaderPaneModeChange('feed')}
							>
								Feed view
							</button>

							<button
								class={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${
									readerPaneMode === 'reader'
										? 'bg-interactive text-interactive-text'
										: 'text-fg-muted hover:text-fg'
								}`}
								type="button"
								onclick={() => onReaderPaneModeChange('reader')}
							>
								Reader view
							</button>
						</div>
					{:else if canUseReaderMode}
						<button
							class="btn-secondary rounded-2xl px-4 py-3"
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

			{#if readerNotice}
				<p class="text-sm leading-8 text-fg-muted">
					{readerNotice}
				</p>
			{/if}

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
