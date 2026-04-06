<script lang="ts">
	import { formatDate } from '$lib/utils/format';

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

<article class="feed-surface mx-auto w-full min-w-3xl max-w-3xl pb-12">
	<header class="border-b border-slate-200/80 pb-8 dark:border-slate-800/80">
		{#if feedTitle}
			<p
				class="text-xs font-semibold tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
			>
				{feedTitle}
			</p>
		{/if}

		<h1 class="mt-4 text-4xl font-semibold tracking-tight text-slate-950 dark:text-white">
			{title}
		</h1>

		<div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-slate-500 dark:text-slate-400">
			<span>{formatDate(publishedAt)}</span>
		</div>
	</header>

	{#if hasHtmlContent}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="feed-html mt-10">{@html contentHtml}</div>
	{:else if hasTextContent}
		<div
			class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-slate-700 dark:text-slate-200"
		>
			{contentText}
		</div>
	{:else if hasHtmlSummary}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="feed-html mt-10">{@html summaryHtml}</div>
	{:else if hasTextSummary}
		<div
			class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-slate-700 dark:text-slate-200"
		>
			{summaryText}
		</div>
	{:else if hasPlainSummary}
		<div
			class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-slate-700 dark:text-slate-200"
		>
			{summary}
		</div>
	{/if}
</article>

<style>
	.feed-surface :global(.feed-html) {
		color: rgb(51 65 85);
		font-size: 1.05rem;
		line-height: 1.9;
	}

	.feed-surface :global(.feed-html > *:first-child) {
		margin-top: 0;
	}

	.feed-surface :global(.feed-html > *:last-child) {
		margin-bottom: 0;
	}

	.feed-surface :global(.feed-html p),
	.feed-surface :global(.feed-html ul),
	.feed-surface :global(.feed-html ol),
	.feed-surface :global(.feed-html blockquote),
	.feed-surface :global(.feed-html pre),
	.feed-surface :global(.feed-html figure),
	.feed-surface :global(.feed-html table) {
		margin: 1.4rem 0;
	}

	.feed-surface :global(.feed-html h2),
	.feed-surface :global(.feed-html h3),
	.feed-surface :global(.feed-html h4) {
		margin: 2.4rem 0 0.9rem;
		color: rgb(15 23 42);
		font-weight: 650;
		line-height: 1.25;
	}

	.feed-surface :global(.feed-html h2) {
		font-size: 1.85rem;
	}

	.feed-surface :global(.feed-html h3) {
		font-size: 1.45rem;
	}

	.feed-surface :global(.feed-html h4) {
		font-size: 1.2rem;
	}

	.feed-surface :global(.feed-html ul),
	.feed-surface :global(.feed-html ol) {
		padding-left: 1.5rem;
	}

	.feed-surface :global(.feed-html li + li) {
		margin-top: 0.45rem;
	}

	.feed-surface :global(.feed-html a) {
		color: rgb(37 99 235);
		text-decoration: underline;
		text-underline-offset: 0.2em;
		overflow-wrap: anywhere;
	}

	.feed-surface :global(.feed-html blockquote) {
		border-left: 3px solid rgb(203 213 225);
		padding-left: 1.25rem;
		color: rgb(71 85 105);
	}

	.feed-surface :global(.feed-html img) {
		display: block;
		max-width: 100%;
		height: auto;
		border-radius: 1rem;
	}

	.feed-surface :global(.feed-html figcaption) {
		margin-top: 0.75rem;
		font-size: 0.92rem;
		color: rgb(100 116 139);
	}

	.feed-surface :global(.feed-html pre),
	.feed-surface :global(.feed-html code) {
		font-family: 'JetBrains Mono', 'SFMono-Regular', 'SF Mono', 'Fira Code', 'Menlo', monospace;
	}

	.feed-surface :global(.feed-html pre) {
		overflow-x: auto;
		border-radius: 1rem;
		background: rgb(241 245 249);
		padding: 1rem 1.1rem;
	}

	.feed-surface :global(.feed-html table) {
		display: block;
		overflow-x: auto;
		border-collapse: collapse;
		max-width: 100%;
	}

	.feed-surface :global(.feed-html th),
	.feed-surface :global(.feed-html td) {
		border: 1px solid rgb(226 232 240);
		padding: 0.65rem 0.8rem;
		text-align: left;
	}

	@media (prefers-color-scheme: dark) {
		.feed-surface :global(.feed-html) {
			color: rgb(226 232 240);
		}

		.feed-surface :global(.feed-html h2),
		.feed-surface :global(.feed-html h3),
		.feed-surface :global(.feed-html h4) {
			color: rgb(248 250 252);
		}

		.feed-surface :global(.feed-html a) {
			color: rgb(96 165 250);
		}

		.feed-surface :global(.feed-html blockquote) {
			border-left-color: rgb(71 85 105);
			color: rgb(203 213 225);
		}

		.feed-surface :global(.feed-html figcaption) {
			color: rgb(148 163 184);
		}

		.feed-surface :global(.feed-html pre) {
			background: rgb(15 23 42);
		}

		.feed-surface :global(.feed-html th),
		.feed-surface :global(.feed-html td) {
			border-color: rgb(51 65 85);
		}
	}
</style>
