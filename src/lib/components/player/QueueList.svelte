<script lang="ts">
	import type { MediaListItem } from '$lib/types/item';
	import { feedsState } from '$lib/state/feeds.svelte';
	import { getPlaybackHistory, getUpcomingQueue } from '$lib/state/playback.svelte';
	import { moveQueuedItemUp, moveQueuedItemDown, removeQueuedItem } from '$lib/state';
	import { formatDuration } from '$lib/utils/format';
	import { openAudioContextMenu } from '$lib/utils/tauri-menu';
	import Icon from '@iconify/svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';

	type QueueListAppearance = 'default' | 'inverse';

	type Props = {
		appearance?: QueueListAppearance;
		rowPaddingClass?: string;
		separatorPaddingClass?: string;
	};

	let {
		appearance = 'default',
		rowPaddingClass = 'px-5',
		separatorPaddingClass = 'px-5'
	}: Props = $props();

	const historyItems = $derived(getPlaybackHistory());
	const queueItems = $derived(getUpcomingQueue());
	const hasHistory = $derived(historyItems.length > 0);
	const hasQueue = $derived(queueItems.length > 0);
	const hasAnyItems = $derived(hasHistory || hasQueue);

	const feedTitleById = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const feed of feedsState.feeds) {
			map[feed.id] = feed.title;
		}
		return map;
	});

	const classes = $derived.by(() => {
		if (appearance === 'inverse') {
			return {
				divider: 'bg-white/20',
				separatorLabel: 'text-[10px] font-medium tracking-widest text-white/50 uppercase',
				itemHover: 'hover:bg-white/10',
				historyItem: 'opacity-60',
				index:
					'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-white/60 tabular-nums',
				title: 'truncate text-sm font-medium text-white',
				feedTitle: 'mt-0.5 truncate text-xs text-white/60',
				duration: 'mt-0.5 text-base text-white/40 tabular-nums',
				emptyIcon: 'mb-3 size-10 text-white/40',
				emptyTitle: 'text-sm font-medium text-white/60',
				emptyText: 'mt-1 text-xs text-white/40',
				actionStack:
					'mt-0.5 flex shrink-0 flex-col items-center gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100',
				actionButtonClass: 'size-6 rounded text-white/50 hover:bg-white/20 hover:text-white',
				actionButtonIconClass: 'size-3.5'
			};
		}

		return {
			divider: 'bg-border',
			separatorLabel: 'text-[10px] font-medium tracking-widest text-fg-subtle uppercase',
			itemHover: 'hover:bg-surface-hover',
			historyItem: 'opacity-60',
			index:
				'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-fg-subtle tabular-nums',
			title: 'truncate text-base font-medium text-fg',
			feedTitle: 'mt-0.5 truncate text-xs text-fg-muted',
			duration: 'mt-0.5 text-xs text-fg-subtle tabular-nums',
			emptyIcon: 'mb-3 size-10 text-fg-subtle',
			emptyTitle: 'text-sm font-medium text-fg-muted',
			emptyText: 'mt-1 text-xs text-fg-subtle',
			actionStack:
				'mt-0.5 flex shrink-0 flex-col items-center gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100',
			actionButtonClass: 'rounded-lg p-1',
			actionButtonIconClass: 'size-3.5'
		};
	});

	function feedTitleForItem(item: MediaListItem): string | null {
		return feedTitleById[item.feedId] ?? null;
	}

	function durationLabel(item: MediaListItem): string | null {
		const duration = item.mediaEnclosure.durationSeconds;
		if (!duration || duration <= 0) {
			return null;
		}

		const position = item.playbackPositionSeconds;
		if (position > 0) {
			const remaining = Math.max(0, duration - position);
			return `${formatDuration(remaining)} left`;
		}

		return formatDuration(duration);
	}

	function getQueueIndex(queueIndex: number): number {
		return queueIndex + 1;
	}

	function scrollIntoView(node: HTMLElement) {
		queueMicrotask(() => {
			node.scrollIntoView({ block: 'start', behavior: 'instant' });
		});
	}
</script>

{#if !hasAnyItems}
	<EmptyState
		icon="lucide:list-music"
		title="Queue is empty"
		description="Press play on an episode to auto-populate"
		iconClass={classes.emptyIcon}
		titleClass={classes.emptyTitle}
		descriptionClass={classes.emptyText}
		class="py-16"
	/>
{:else}
	<ul class="px-0 py-2">
		{#if hasHistory}
			{#each [...historyItems].reverse() as item (item.id)}
				<li
					oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
					class={`group relative flex items-start gap-3 py-3 transition-colors ${rowPaddingClass} ${classes.itemHover} ${classes.historyItem}`}
				>
					<span class={classes.index}>
						<Icon icon="lucide:history" class="size-3" />
					</span>

					<div class="min-w-0 flex-1">
						<p class={classes.title}>
							{item.title}
						</p>

						{#if feedTitleForItem(item)}
							<p class={classes.feedTitle}>
								{feedTitleForItem(item)}
							</p>
						{/if}

						{#if durationLabel(item)}
							<p class={classes.duration}>
								{durationLabel(item)}
							</p>
						{/if}
					</div>
				</li>
			{/each}

			<li
				class={`flex items-center gap-3 py-2 ${separatorPaddingClass}`}
				aria-hidden="true"
				use:scrollIntoView
			>
				<div class={`h-px flex-1 ${classes.divider}`}></div>
				<span class={classes.separatorLabel}>Playing next</span>
				<div class={`h-px flex-1 ${classes.divider}`}></div>
			</li>
		{/if}

		{#each queueItems as item, index (item.id)}
			<li
				oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
				class={`group relative flex items-start gap-3 py-3 transition-colors ${rowPaddingClass} ${classes.itemHover}`}
			>
				<span class={classes.index}>
					{getQueueIndex(index)}
				</span>

				<div class="min-w-0 flex-1">
					<p class={classes.title}>
						{item.title}
					</p>

					{#if feedTitleForItem(item)}
						<p class={classes.feedTitle}>
							{feedTitleForItem(item)}
						</p>
					{/if}

					{#if durationLabel(item)}
						<p class={classes.duration}>
							{durationLabel(item)}
						</p>
					{/if}
				</div>

				<div class={classes.actionStack}>
					{#if index > 0}
						<IconButton
							icon="lucide:chevron-up"
							variant={appearance === 'inverse' ? 'ghost' : 'icon-subtle'}
							class={classes.actionButtonClass}
							iconClass={classes.actionButtonIconClass}
							title="Move up"
							label={`Move ${item.title} up in queue`}
							onclick={() => moveQueuedItemUp(item.id)}
						/>
					{/if}

					<IconButton
						icon="lucide:x"
						variant={appearance === 'inverse' ? 'ghost' : 'icon-subtle'}
						class={classes.actionButtonClass}
						iconClass={classes.actionButtonIconClass}
						title="Remove from queue"
						label={`Remove ${item.title} from queue`}
						onclick={() => removeQueuedItem(item.id)}
					/>

					{#if index < queueItems.length - 1}
						<IconButton
							icon="lucide:chevron-down"
							variant={appearance === 'inverse' ? 'ghost' : 'icon-subtle'}
							class={classes.actionButtonClass}
							iconClass={classes.actionButtonIconClass}
							title="Move down"
							label={`Move ${item.title} down in queue`}
							onclick={() => moveQueuedItemDown(item.id)}
						/>
					{/if}
				</div>
			</li>
		{/each}
	</ul>
{/if}
