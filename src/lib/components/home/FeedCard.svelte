<script lang="ts">
	import type { Feed } from '$lib/types/feed';
	import FeedCardImage from './FeedCardImage.svelte';

	let {
		feed,
		unreadCount = 0,
		onClick
	}: {
		feed: Feed;
		unreadCount?: number;
		onClick: () => void;
	} = $props();
</script>

<button
	type="button"
	class="group relative flex w-full flex-col items-center gap-2 rounded-xl p-2 transition-colors contain-[paint] hover:bg-surface-hover"
	onclick={onClick}
	title={feed.title}
	aria-label={`Open feed ${feed.title}`}
>
	<FeedCardImage
		imageUrl={feed.imageUrl}
		alt={feed.title}
		letter={(feed.title?.trim()?.[0] ?? '?').toUpperCase()}
	/>

	{#if unreadCount > 0}
		<div class="absolute top-4 right-4 size-3 rounded-full bg-accent shadow-sm"></div>
	{/if}

	<span class="line-clamp-2 w-full text-center text-xs font-medium text-fg">
		{feed.title}
	</span>
</button>
