<script lang="ts">
	import ArticleBase from './ArticleBase.svelte';
	import type { FeedItem } from '$lib/types/item';

	type Props = {
		item: FeedItem;
		feedTitle?: string;
		feedOrEpisodeImageUrl?: string;
		feedLink?: string | null;
		maximized?: boolean;
	};

	let {
		item,
		feedTitle,
		feedOrEpisodeImageUrl: imageUrl,
		feedLink = null,
		maximized = false
	}: Props = $props();

	const hasHtmlContent = $derived(!!item.contentHtml);
	const hasTextContent = $derived(!!item.contentText && !hasHtmlContent);
	const hasHtmlSummary = $derived(!!item.summaryHtml && !hasHtmlContent && !hasTextContent);
	const hasTextSummary = $derived(
		!!item.summaryText && !hasHtmlContent && !hasTextContent && !hasHtmlSummary
	);
	const hasPlainSummary = $derived(
		!!item.summary && !hasHtmlContent && !hasTextContent && !hasHtmlSummary && !hasTextSummary
	);
</script>

<ArticleBase {feedTitle} {feedLink} {imageUrl} {item} {maximized}>
	{#if hasHtmlContent}
		<div class="article-html mt-10">
			<!-- eslint-disable-next-line svelte/no-at-html-tags -->
			{@html item.contentHtml}
		</div>
	{:else if hasTextContent}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{item.contentText}
		</div>
	{:else if hasHtmlSummary}
		<div class="article-html mt-10">
			<!-- eslint-disable-next-line svelte/no-at-html-tags -->
			{@html item.summaryHtml}
		</div>
	{:else if hasTextSummary}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{item.summaryText}
		</div>
	{:else if hasPlainSummary}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{item.summary}
		</div>
	{/if}
</ArticleBase>
