<script lang="ts">
	import type { FeedItem } from '$lib/types/rss';
	import { isMediaItem } from '$lib/types/rss';
	import { formatDate } from '$lib/utils/format';
	import { openAudioContextMenu } from '$lib/utils/tauri-menu';
	import Icon from '@iconify/svelte';
	import DynamicPlayButton from '../player/DynamicPlayButton.svelte';

	type Props = {
		feedTitle?: string;
		feedImageUrl?: string;
		item: FeedItem;
	};

	let { feedTitle, feedImageUrl, item }: Props = $props();

	const { readerByline, publishedAt, readerExcerpt } = $derived.by(() => item);

	const title: string = $derived(item.readerTitle || item.title);
	const isMedia: boolean = $derived(isMediaItem(item));
</script>

<header class="border-b border-border pb-8">
	<div class="flex gap-6">
		{#if isMedia && feedImageUrl}
			<img
				src={feedImageUrl}
				alt="Podcast Cover"
				class="size-48 shrink-0 rounded-2xl object-cover shadow-sm"
			/>
		{/if}

		<div class="flex min-w-0 flex-1 flex-col gap-y-2">
			{#if feedTitle}
				<p class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
					{feedTitle}
				</p>
			{/if}

			<h1
				class="font-semibold tracking-tight text-fg"
				class:text-2xl={isMedia}
				class:text-4xl={!isMedia}
				class:select-none={isMedia}
				oncontextmenu={isMedia && isMediaItem(item)
					? (e) => void openAudioContextMenu(e, item)
					: undefined}
			>
				{title}
			</h1>

			<div class="flex flex-wrap items-center gap-2 text-sm text-fg-muted">
				{#if readerByline}
					<span>{readerByline}</span>
				{/if}

				<span>{formatDate(publishedAt)}</span>
			</div>

			{#if isMedia && isMediaItem(item)}
				<div class="flex flex-wrap items-center gap-2">
					<DynamicPlayButton {item} size="sm" />

					<button
						class="btn-secondary btn-round btn-sm"
						type="button"
						onclick={(e) => void openAudioContextMenu(e, item)}
					>
						<Icon icon="lucide:ellipsis" class="size-4" />
					</button>
				</div>
			{/if}
		</div>
	</div>

	{#if readerExcerpt}
		<p class="mt-4 max-w-2xl text-lg leading-8 text-fg-secondary">
			{readerExcerpt}
		</p>
	{/if}
</header>
