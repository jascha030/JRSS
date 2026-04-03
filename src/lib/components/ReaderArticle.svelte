<script lang="ts">
	import { formatDate } from '$lib/utils/format';

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

<article class="reader-surface mx-auto w-full max-w-3xl pb-12">
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

	{#if html}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="reader-html mt-10">{@html html}</div>
	{:else if text}
		<div
			class="mt-10 text-[1.05rem] leading-8 whitespace-pre-line text-slate-700 dark:text-slate-200"
		>
			{text}
		</div>
	{/if}
</article>

<style>
	.reader-surface :global(.reader-html) {
		color: rgb(51 65 85);
		font-size: 1.05rem;
		line-height: 1.9;
	}

	.reader-surface :global(.reader-html > *:first-child) {
		margin-top: 0;
	}

	.reader-surface :global(.reader-html > *:last-child) {
		margin-bottom: 0;
	}

	.reader-surface :global(.reader-html p),
	.reader-surface :global(.reader-html ul),
	.reader-surface :global(.reader-html ol),
	.reader-surface :global(.reader-html blockquote),
	.reader-surface :global(.reader-html pre),
	.reader-surface :global(.reader-html figure),
	.reader-surface :global(.reader-html table) {
		margin: 1.4rem 0;
	}

	.reader-surface :global(.reader-html h2),
	.reader-surface :global(.reader-html h3),
	.reader-surface :global(.reader-html h4) {
		margin: 2.4rem 0 0.9rem;
		color: rgb(15 23 42);
		font-weight: 650;
		line-height: 1.25;
	}

	.reader-surface :global(.reader-html h2) {
		font-size: 1.85rem;
	}

	.reader-surface :global(.reader-html h3) {
		font-size: 1.45rem;
	}

	.reader-surface :global(.reader-html h4) {
		font-size: 1.2rem;
	}

	.reader-surface :global(.reader-html ul),
	.reader-surface :global(.reader-html ol) {
		padding-left: 1.5rem;
	}

	.reader-surface :global(.reader-html li + li) {
		margin-top: 0.45rem;
	}

	.reader-surface :global(.reader-html a) {
		color: rgb(37 99 235);
		text-decoration: underline;
		text-underline-offset: 0.2em;
		overflow-wrap: anywhere;
	}

	.reader-surface :global(.reader-html blockquote) {
		border-left: 3px solid rgb(203 213 225);
		padding-left: 1.25rem;
		color: rgb(71 85 105);
	}

	.reader-surface :global(.reader-html img) {
		display: block;
		max-width: 100%;
		height: auto;
		border-radius: 1rem;
	}

	.reader-surface :global(.reader-html figcaption) {
		margin-top: 0.75rem;
		font-size: 0.92rem;
		color: rgb(100 116 139);
	}

	.reader-surface :global(.reader-html pre),
	.reader-surface :global(.reader-html code) {
		font-family: 'JetBrains Mono', 'SFMono-Regular', 'SF Mono', 'Fira Code', 'Menlo', monospace;
	}

	.reader-surface :global(.reader-html pre) {
		overflow-x: auto;
		border-radius: 1rem;
		background: rgb(241 245 249);
		padding: 1rem 1.1rem;
	}

	.reader-surface :global(.reader-html table) {
		display: block;
		overflow-x: auto;
		border-collapse: collapse;
		max-width: 100%;
	}

	.reader-surface :global(.reader-html th),
	.reader-surface :global(.reader-html td) {
		border: 1px solid rgb(226 232 240);
		padding: 0.65rem 0.8rem;
		text-align: left;
	}

	@media (prefers-color-scheme: dark) {
		.reader-surface :global(.reader-html) {
			color: rgb(226 232 240);
		}

		.reader-surface :global(.reader-html h2),
		.reader-surface :global(.reader-html h3),
		.reader-surface :global(.reader-html h4) {
			color: rgb(248 250 252);
		}

		.reader-surface :global(.reader-html a) {
			color: rgb(96 165 250);
		}

		.reader-surface :global(.reader-html blockquote) {
			border-left-color: rgb(71 85 105);
			color: rgb(203 213 225);
		}

		.reader-surface :global(.reader-html figcaption) {
			color: rgb(148 163 184);
		}

		.reader-surface :global(.reader-html pre) {
			background: rgb(15 23 42);
		}

		.reader-surface :global(.reader-html th),
		.reader-surface :global(.reader-html td) {
			border-color: rgb(51 65 85);
		}
	}
</style>
