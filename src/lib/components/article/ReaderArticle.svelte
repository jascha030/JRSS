<script lang="ts">
	import ArticleBase from './ArticleBase.svelte';
	import type { FeedItem } from '$lib/types/item';

	type Props = {
		item: FeedItem;
		feedTitle?: string;
		feedImageUrl?: string;
		maximized?: boolean;
	};

	let { item, feedTitle, feedImageUrl, maximized = false }: Props = $props();
</script>

<ArticleBase {feedTitle} imageUrl={feedImageUrl} {item} {maximized}>
	{#if item.readerContentHtml}
		<div class="article-html mt-10">
			<!-- eslint-disable-next-line svelte/no-at-html-tags -->
			{@html item.readerContentHtml}
		</div>
	{:else if item.readerContentText}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{item.readerContentText}
		</div>
	{/if}
</ArticleBase>
