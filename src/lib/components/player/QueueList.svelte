<script lang="ts">
	import type { Feed, MediaListItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';
	import Icon from '@iconify/svelte';

	type QueueListAppearance = 'default' | 'inverse';

	type Props = {
		queueItems: MediaListItem[];
		manualQueueLength?: number;
		feeds?: Feed[];
		appearance?: QueueListAppearance;
		rowPaddingClass?: string;
		separatorPaddingClass?: string;
		onRemoveItem?: (itemId: string) => void;
		onMoveItemUp?: (itemId: string) => void;
		onMoveItemDown?: (itemId: string) => void;
	};

	let {
		queueItems,
		manualQueueLength = 0,
		feeds = [],
		appearance = 'default',
		rowPaddingClass = 'px-5',
		separatorPaddingClass = 'px-5',
		onRemoveItem,
		onMoveItemUp,
		onMoveItemDown
	}: Props = $props();

	const hasAutoItems = $derived(queueItems.length > manualQueueLength);

	let feedTitleById = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const feed of feeds) {
			map[feed.id] = feed.title;
		}
		return map;
	});

	let classes = $derived.by(() => {
		if (appearance === 'inverse') {
			return {
				divider: 'bg-white/20',
				separatorLabel: 'text-[10px] font-medium tracking-widest text-white/50 uppercase',
				itemHover: 'hover:bg-white/10',
				index:
					'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-white/60 tabular-nums',
				title: 'truncate text-sm font-medium text-white',
				feedTitle: 'mt-0.5 truncate text-xs text-white/60',
				duration: 'mt-0.5 text-xs text-white/40 tabular-nums',
				emptyIcon: 'mb-3 size-10 text-white/40',
				emptyTitle: 'text-sm font-medium text-white/60',
				emptyText: 'mt-1 text-xs text-white/40',
				actionStack:
					'mt-0.5 flex shrink-0 flex-col items-center gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100',
				actionButton:
					'flex size-6 items-center justify-center rounded text-white/50 hover:bg-white/20 hover:text-white'
			};
		}

		return {
			divider: 'bg-border',
			separatorLabel: 'text-[10px] font-medium tracking-widest text-fg-subtle uppercase',
			itemHover: 'hover:bg-surface-hover',
			index:
				'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-fg-subtle tabular-nums',
			title: 'truncate text-sm font-medium text-fg',
			feedTitle: 'mt-0.5 truncate text-xs text-fg-muted',
			duration: 'mt-0.5 text-xs text-fg-subtle tabular-nums',
			emptyIcon: 'mb-3 size-10 text-fg-subtle',
			emptyTitle: 'text-sm font-medium text-fg-muted',
			emptyText: 'mt-1 text-xs text-fg-subtle',
			actionStack:
				'mt-0.5 flex shrink-0 flex-col items-center gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100',
			actionButton: 'preset-icon-subtle btn-icon rounded-lg'
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
</script>

{#if queueItems.length === 0}
	<div class="flex flex-col items-center justify-center px-6 py-16 text-center">
		<Icon icon="lucide:list-music" class={classes.emptyIcon} />
		<p class={classes.emptyTitle}>Queue is empty</p>
		<p class={classes.emptyText}>Press play on an episode to auto-populate</p>
	</div>
{:else}
	<ul class="py-2">
		{#each queueItems as item, index (item.id)}
			{#if index === manualQueueLength && hasAutoItems}
				<li class={`flex items-center gap-3 py-2 ${separatorPaddingClass}`} aria-hidden="true">
					<div class={`h-px flex-1 ${classes.divider}`}></div>
					<span class={classes.separatorLabel}>From this feed</span>
					<div class={`h-px flex-1 ${classes.divider}`}></div>
				</li>
			{/if}

			<li
				class={`group relative flex items-start gap-3 py-3 transition-colors ${rowPaddingClass} ${classes.itemHover}`}
			>
				<span class={classes.index}>
					{index + 1}
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
					{#if index > 0 && onMoveItemUp}
						<button
							type="button"
							title="Move up"
							aria-label={`Move ${item.title} up in queue`}
							class={classes.actionButton}
							onclick={() => onMoveItemUp(item.id)}
						>
							<Icon icon="lucide:chevron-up" class="size-3.5" />
						</button>
					{/if}

					{#if onRemoveItem}
						<button
							type="button"
							title="Remove from queue"
							aria-label={`Remove ${item.title} from queue`}
							class={classes.actionButton}
							onclick={() => onRemoveItem(item.id)}
						>
							<Icon icon="lucide:x" class="size-3.5" />
						</button>
					{/if}

					{#if index < queueItems.length - 1 && onMoveItemDown}
						<button
							type="button"
							title="Move down"
							aria-label={`Move ${item.title} down in queue`}
							class={classes.actionButton}
							onclick={() => onMoveItemDown(item.id)}
						>
							<Icon icon="lucide:chevron-down" class="size-3.5" />
						</button>
					{/if}
				</div>
			</li>
		{/each}
	</ul>
{/if}
