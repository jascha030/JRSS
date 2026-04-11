<script lang="ts">
	import FeedArticle from '$lib/components/FeedArticle.svelte';
	import ReaderArticle from '$lib/components/ReaderArticle.svelte';
	import type { Feed, FeedItem, PlaybackState } from '$lib/types/rss';
	import { open } from '@tauri-apps/plugin-shell';

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
		onPlay: (item: FeedItem) => void;
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
		onPlay,
		onStopPlayback,
		onLoadReaderView,
		onReaderPaneModeChange
	}: Props = $props();
</script>

<aside
	class="hidden min-h-0 min-w-0 flex-col justify-between overflow-y-auto bg-surface-glass p-8 xl:flex xl:flex-1 2xl:basis-2/3"
>
	{#if selectedItem}
		<div class="space-y-9">
			<div class="flex flex-wrap items-center gap-4">
				<button
					class="btn-primary rounded-2xl px-4 py-3"
					type="button"
					onclick={() => {
						void open(selectedItem.url);
					}}
				>
					Open original
				</button>

				{#if selectedItem.mediaEnclosure}
					<button
						class="btn-secondary rounded-2xl px-4 py-3"
						type="button"
						onclick={() => onPlay(selectedItem)}
					>
						{selectedItem.playbackPositionSeconds > 0 ? 'Resume playback' : 'Start playback'}
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

			{#if readerNotice}
				<p class="text-sm leading-8 text-fg-muted">
					{readerNotice}
				</p>
			{/if}

			{#if isReaderPaneActive}
				<ReaderArticle
					feedTitle={selectedItemFeed?.title}
					title={selectedItem.readerTitle ?? selectedItem.title}
					byline={selectedItem.readerByline}
					excerpt={selectedItem.readerExcerpt}
					publishedAt={selectedItem.publishedAt}
					html={selectedItem.readerContentHtml}
					text={selectedItem.readerContentText}
				/>
			{:else}
				<FeedArticle
					feedTitle={selectedItemFeed?.title}
					title={selectedItem.title}
					publishedAt={selectedItem.publishedAt}
					contentHtml={selectedItem.contentHtml}
					contentText={selectedItem.contentText}
					summaryHtml={selectedItem.summaryHtml}
					summaryText={selectedItem.summaryText}
					summary={selectedItem.summary}
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
