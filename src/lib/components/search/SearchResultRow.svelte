<script lang="ts">
	import type { IndexedElementAction } from '$lib/services/keyboard-list-navigation.svelte';
	import type { SearchResultEntry } from '$lib/services/global-search.svelte.js';
	import { formatDate } from '$lib/utils/format';
	import { isMediaItem } from '$lib/types/item';

	type Props = {
		entry: SearchResultEntry;
		index: number;
		isHighlighted: boolean;
		feedTitleById: Map<string, string>;
		setRef: IndexedElementAction<HTMLDivElement>;
		onSelect: (entry: SearchResultEntry, index: number) => void;
		onHover: (index: number) => void;
	};

	let { entry, index, isHighlighted, feedTitleById, setRef, onSelect, onHover }: Props = $props();

	function handleMouseDown(event: MouseEvent) {
		event.preventDefault();
		onSelect(entry, index);
	}
</script>

<div
	use:setRef={index}
	class={`flex cursor-pointer flex-col gap-0.5 px-4 py-3 transition-colors ${
		index > 0 ? 'border-t border-border' : ''
	} ${isHighlighted ? 'bg-surface-sidebar-active-opaque' : 'hover:bg-surface-sidebar-hover-opaque'}`}
	role="option"
	tabindex="-1"
	aria-selected={isHighlighted}
	onmousedown={handleMouseDown}
	onmouseenter={() => onHover(index)}
>
	{#if entry.kind === 'action'}
		<div class="flex items-center gap-2">
			<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase">
				{entry.data.title}
			</span>
		</div>
		<p class="truncate text-sm font-medium text-fg">{entry.data.description}</p>
	{:else if entry.kind === 'feed'}
		<div class="flex items-center gap-2">
			<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase">
				Feed
			</span>
			{#if entry.data.kind === 'media'}
				<span class="badge shrink-0 preset-tonal-surface text-xs">Podcast</span>
			{/if}
		</div>
		<p class="truncate text-sm font-medium text-fg">{entry.data.title}</p>
		<p class="truncate text-xs text-fg-secondary">{entry.data.url}</p>
	{:else if entry.kind === 'station'}
		<div class="flex items-center gap-2">
			<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase">
				Station
			</span>
		</div>
		<p class="truncate text-sm font-medium text-fg">{entry.data.name}</p>
		<p class="truncate text-xs text-fg-secondary">
			{entry.data.feedIds.length}
			{entry.data.feedIds.length === 1 ? 'podcast' : 'podcasts'}
		</p>
	{:else if entry.kind === 'podcast'}
		<div class="flex items-center gap-2">
			<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase"
				>iTunes</span
			>
		</div>
		<p class="truncate text-sm font-medium text-fg">{entry.data.name}</p>
		<p class="truncate text-xs text-fg-secondary">{entry.data.artist}</p>
	{:else if entry.kind === 'item'}
		<div class="flex items-center gap-2">
			<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase">
				{feedTitleById.get(entry.data.feedId) ?? 'Unknown feed'}
			</span>

			{#if isMediaItem(entry.data)}
				<span class="badge shrink-0 preset-tonal-surface text-xs">Podcast</span>
			{/if}

			<span class="ml-auto shrink-0 text-xs text-fg-subtle">
				{formatDate(entry.data.publishedAt)}
			</span>
		</div>

		<p class="truncate text-sm font-medium text-fg">{entry.data.title}</p>

		{#if entry.data.previewText}
			<p class="truncate text-xs text-fg-secondary">{entry.data.previewText}</p>
		{/if}
	{/if}
</div>
