<script lang="ts">
	import { formatDate } from '$lib/utils/format';

	type Props = {
		feedTitle?: string;
		feedImageUrl?: string;
		title: string;
		publishedAt: string;
		byline?: string;
		excerpt?: string;
		surfaceClass?: string;
		children?: import('svelte').Snippet;
	};

	let {
		feedTitle,
		feedImageUrl,
		title,
		publishedAt,
		byline,
		excerpt,
		surfaceClass = 'article-surface',
		children
	}: Props = $props();
</script>

<article class="{surfaceClass} mx-auto w-full max-w-xl min-w-lg pb-12 2xl:max-w-3xl 2xl:min-w-3xl 3xl:min-w-4xl 3xl:max-w-4xl">
	<header class="border-b border-border pb-8">
		<div class="flex gap-6">
			{#if feedImageUrl}
				<img
					src={feedImageUrl}
					alt=""
					class="size-48 shrink-0 rounded-2xl object-cover shadow-sm"
				/>
			{/if}

			<div class="min-w-0 flex-1">
				{#if feedTitle}
					<p class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
						{feedTitle}
					</p>
				{/if}

				<h1 class="mt-4 text-4xl font-semibold tracking-tight text-fg">
					{title}
				</h1>

				<div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-fg-muted">
					{#if byline}
						<span>{byline}</span>
					{/if}
					<span>{formatDate(publishedAt)}</span>
				</div>
			</div>
		</div>

		{#if excerpt}
			<p class="mt-4 max-w-2xl text-lg leading-8 text-fg-secondary">
				{excerpt}
			</p>
		{/if}
	</header>

	{@render children?.()}
</article>

<style>
	.article-surface :global(.article-html) {
		color: var(--color-article-body);
		font-size: 1.05rem;
		line-height: 1.9;
	}

	.article-surface :global(.article-html > *:first-child) {
		margin-top: 0;
	}

	.article-surface :global(.article-html > *:last-child) {
		margin-bottom: 0;
	}

	.article-surface :global(.article-html p),
	.article-surface :global(.article-html ul),
	.article-surface :global(.article-html ol),
	.article-surface :global(.article-html blockquote),
	.article-surface :global(.article-html pre),
	.article-surface :global(.article-html figure),
	.article-surface :global(.article-html table) {
		margin: 1.4rem 0;
	}

	.article-surface :global(.article-html h2),
	.article-surface :global(.article-html h3),
	.article-surface :global(.article-html h4) {
		margin: 2.4rem 0 0.9rem;
		color: var(--color-article-heading);
		font-weight: 650;
		line-height: 1.25;
	}

	.article-surface :global(.article-html h2) {
		font-size: 1.85rem;
	}

	.article-surface :global(.article-html h3) {
		font-size: 1.45rem;
	}

	.article-surface :global(.article-html h4) {
		font-size: 1.2rem;
	}

	.article-surface :global(.article-html ul),
	.article-surface :global(.article-html ol) {
		padding-left: 1.5rem;
	}

	.article-surface :global(.article-html li + li) {
		margin-top: 0.45rem;
	}

	.article-surface :global(.article-html a) {
		color: var(--color-article-link);
		text-decoration: underline;
		text-underline-offset: 0.2em;
		overflow-wrap: anywhere;
	}

	.article-surface :global(.article-html blockquote) {
		border-left: 3px solid var(--color-article-blockquote-border);
		padding-left: 1.25rem;
		color: var(--color-article-blockquote-text);
	}

	.article-surface :global(.article-html img) {
		display: block;
		max-width: 100%;
		height: auto;
		border-radius: 1rem;
	}

	.article-surface :global(.article-html figcaption) {
		margin-top: 0.75rem;
		font-size: 0.92rem;
		color: var(--color-article-caption);
	}

	.article-surface :global(.article-html pre),
	.article-surface :global(.article-html code) {
		font-family: 'JetBrains Mono', 'SFMono-Regular', 'SF Mono', 'Fira Code', 'Menlo', monospace;
	}

	.article-surface :global(.article-html pre) {
		overflow-x: auto;
		border-radius: 1rem;
		background: var(--color-article-code-bg);
		padding: 1rem 1.1rem;
	}

	.article-surface :global(.article-html table) {
		display: block;
		overflow-x: auto;
		border-collapse: collapse;
		max-width: 100%;
	}

	.article-surface :global(.article-html th),
	.article-surface :global(.article-html td) {
		border: 1px solid var(--color-article-table-border);
		padding: 0.65rem 0.8rem;
		text-align: left;
	}
</style>
