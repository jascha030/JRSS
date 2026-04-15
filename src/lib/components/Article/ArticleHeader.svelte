<script lang="ts">
	import type { FeedItem } from '$lib/types/rss';
	import { formatDate } from '$lib/utils/format';

	type Props = {
		feedTitle?: string;
		feedImageUrl?: string;
		item: FeedItem;
	};

	let { feedTitle, feedImageUrl, item }: Props = $props();

	const { readerByline, publishedAt, readerExcerpt, mediaEnclosure } = $derived.by(() => item);

	const title: string = $derived(item.readerTitle || item.title);
	const isPodcast: boolean = $derived(!!mediaEnclosure);
</script>

<header class="border-b border-border pb-8">
	<div class="flex gap-6">
		{#if isPodcast && feedImageUrl}
			<img
				src={feedImageUrl}
				alt="Podcast Cover"
				class="size-48 shrink-0 rounded-2xl object-cover shadow-sm"
			/>
		{/if}

		<div class="min-w-0 flex-1">
			{#if feedTitle}
				<p class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
					{feedTitle}
				</p>
			{/if}

			<h1
				class="mt-4 font-semibold tracking-tight text-fg"
				class:text-2xl={isPodcast}
				class:text-4xl={!isPodcast}
			>
				{title}
			</h1>

			<div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-fg-muted">
				{#if readerByline}
					<span>{readerByline}</span>
				{/if}

				<span>{formatDate(publishedAt)}</span>
			</div>
		</div>
	</div>

	{#if readerExcerpt}
		<p class="mt-4 max-w-2xl text-lg leading-8 text-fg-secondary">
			{readerExcerpt}
		</p>
	{/if}
</header>
