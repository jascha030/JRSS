<script lang="ts">
	import ArticleBase from './ArticleBase.svelte';
	import type { FeedItem } from '$lib/types/rss';

	type Props = {
		item: FeedItem;
		feedTitle?: string;
		feedImageUrl?: string;
	};

	let { item, feedTitle, feedImageUrl }: Props = $props();

	// Prefer HTML content over text, with cascading fallback
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

<ArticleBase {feedTitle} {feedImageUrl} {item}>
	{#if hasHtmlContent}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="article-html mt-10">{@html item.contentHtml}</div>
	{:else if hasTextContent}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{item.contentText}
		</div>
	{:else if hasHtmlSummary}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="article-html mt-10">{@html item.summaryHtml}</div>
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
