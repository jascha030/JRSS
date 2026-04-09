<script lang="ts">
	import ArticleBase from './ArticleBase.svelte';

	type Props = {
		feedTitle?: string;
		title: string;
		byline?: string;
		excerpt?: string;
		publishedAt: string;
		html?: string;
		text?: string;
	};

	let { feedTitle, title, byline, excerpt, publishedAt, html, text }: Props = $props();
</script>

<ArticleBase {feedTitle} {title} {publishedAt} {byline} {excerpt}>
	{#if html}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="article-html mt-10">{@html html}</div>
	{:else if text}
		<div
			class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-slate-700 dark:text-slate-200"
		>
			{text}
		</div>
	{/if}
</ArticleBase>
