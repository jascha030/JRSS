<script lang="ts">
	import { formatDate } from '$lib/utils/format';

	type Props = {
		feedTitle?: string;
		title: string;
		publishedAt: string;
		byline?: string;
		excerpt?: string;
		surfaceClass?: string;
		children?: import('svelte').Snippet;
	};

	let {
		feedTitle,
		title,
		publishedAt,
		byline,
		excerpt,
		surfaceClass = 'article-surface',
		children
	}: Props = $props();
</script>

<article class="{surfaceClass} mx-auto w-full max-w-xl min-w-lg pb-12 2xl:max-w-3xl 2xl:min-w-3xl">
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

		{#if excerpt}
			<p class="mt-4 max-w-2xl text-lg leading-8 text-slate-600 dark:text-slate-300">
				{excerpt}
			</p>
		{/if}

		<div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-slate-500 dark:text-slate-400">
			{#if byline}
				<span>{byline}</span>
			{/if}
			<span>{formatDate(publishedAt)}</span>
		</div>
	</header>

	{@render children?.()}
</article>

<style>
	.article-surface :global(.article-html) {
		color: rgb(51 65 85);
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
		color: rgb(15 23 42);
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
		color: rgb(37 99 235);
		text-decoration: underline;
		text-underline-offset: 0.2em;
		overflow-wrap: anywhere;
	}

	.article-surface :global(.article-html blockquote) {
		border-left: 3px solid rgb(203 213 225);
		padding-left: 1.25rem;
		color: rgb(71 85 105);
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
		color: rgb(100 116 139);
	}

	.article-surface :global(.article-html pre),
	.article-surface :global(.article-html code) {
		font-family: 'JetBrains Mono', 'SFMono-Regular', 'SF Mono', 'Fira Code', 'Menlo', monospace;
	}

	.article-surface :global(.article-html pre) {
		overflow-x: auto;
		border-radius: 1rem;
		background: rgb(241 245 249);
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
		border: 1px solid rgb(226 232 240);
		padding: 0.65rem 0.8rem;
		text-align: left;
	}

	@media (prefers-color-scheme: dark) {
		.article-surface :global(.article-html) {
			color: rgb(226 232 240);
		}

		.article-surface :global(.article-html h2),
		.article-surface :global(.article-html h3),
		.article-surface :global(.article-html h4) {
			color: rgb(248 250 252);
		}

		.article-surface :global(.article-html a) {
			color: rgb(96 165 250);
		}

		.article-surface :global(.article-html blockquote) {
			border-left-color: rgb(71 85 105);
			color: rgb(203 213 225);
		}

		.article-surface :global(.article-html figcaption) {
			color: rgb(148 163 184);
		}

		.article-surface :global(.article-html pre) {
			background: rgb(15 23 42);
		}

		.article-surface :global(.article-html th),
		.article-surface :global(.article-html td) {
			border-color: rgb(51 65 85);
		}
	}
</style>
