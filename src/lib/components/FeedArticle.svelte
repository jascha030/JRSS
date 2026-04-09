<script lang="ts">
	import ArticleBase from './ArticleBase.svelte';

	type Props = {
		feedTitle?: string;
		title: string;
		publishedAt: string;
		contentHtml?: string;
		contentText?: string;
		summaryHtml?: string;
		summaryText?: string;
		summary: string;
	};

	let {
		feedTitle,
		title,
		publishedAt,
		contentHtml,
		contentText,
		summaryHtml,
		summaryText,
		summary
	}: Props = $props();

	// Prefer HTML content over text, with cascading fallback
	const hasHtmlContent = $derived(!!contentHtml);
	const hasTextContent = $derived(!!contentText && !hasHtmlContent);
	const hasHtmlSummary = $derived(!!summaryHtml && !hasHtmlContent && !hasTextContent);
	const hasTextSummary = $derived(
		!!summaryText && !hasHtmlContent && !hasTextContent && !hasHtmlSummary
	);
	const hasPlainSummary = $derived(
		!!summary && !hasHtmlContent && !hasTextContent && !hasHtmlSummary && !hasTextSummary
	);
</script>

<ArticleBase {feedTitle} {title} {publishedAt}>
	{#if hasHtmlContent}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="article-html mt-10">{@html contentHtml}</div>
	{:else if hasTextContent}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{contentText}
		</div>
	{:else if hasHtmlSummary}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="article-html mt-10">{@html summaryHtml}</div>
	{:else if hasTextSummary}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{summaryText}
		</div>
	{:else if hasPlainSummary}
		<div class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-fg-secondary">
			{summary}
		</div>
	{/if}
</ArticleBase>
