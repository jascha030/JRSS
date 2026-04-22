<script lang="ts">
	import type { Feed, MediaListItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';
	import Icon from '@iconify/svelte';

	type Props = {
		open: boolean;
		queueItems: MediaListItem[];
		manualQueueLength: number;
		feeds: Feed[];
		onRemoveItem: (itemId: string) => void;
		onMoveItemUp: (itemId: string) => void;
		onMoveItemDown: (itemId: string) => void;
		onClearQueue: () => void;
		onClose: () => void;
	};

	let {
		open,
		queueItems,
		manualQueueLength,
		feeds,
		onRemoveItem,
		onMoveItemUp,
		onMoveItemDown,
		onClearQueue,
		onClose
	}: Props = $props();

	const hasAutoItems = $derived(queueItems.length > manualQueueLength);

	let feedTitleById = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const feed of feeds) {
			map[feed.id] = feed.title;
		}
		return map;
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

{#if open}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/30 backdrop-blur-[2px] transition-opacity"
		aria-label="Close queue"
		tabindex="-1"
		onclick={onClose}
	></button>
{/if}

<div
	class={`fixed inset-y-0 right-0 z-50 flex w-80 transform-gpu flex-col border-l border-border bg-surface shadow-xl transition-transform duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform motion-reduce:transition-none ${
		open ? 'translate-x-0' : 'translate-x-full'
	}`}
	role="dialog"
	aria-modal="true"
	aria-labelledby="queue-title"
	aria-hidden={!open}
>
	<div class="flex h-16 shrink-0 items-center justify-between border-b border-border px-5">
		<div class="min-w-0">
			<h2 id="queue-title" class="text-sm font-semibold text-fg">Playing next</h2>
			<p class="text-xs text-fg-muted">
				{queueItems.length}
				{queueItems.length === 1 ? 'episode' : 'episodes'}
			</p>
		</div>

		<button
			type="button"
			class="preset-icon-subtle btn-icon rounded-xl"
			aria-label="Close queue"
			title="Close queue"
			onclick={onClose}
		>
			<Icon icon="lucide:x" class="size-4" />
		</button>
	</div>

	<div class="flex-1 overflow-y-auto">
		{#if queueItems.length === 0}
			<div class="flex flex-col items-center justify-center px-6 py-16 text-center">
				<Icon icon="lucide:list-music" class="mb-3 size-10 text-fg-subtle" />
				<p class="text-sm font-medium text-fg-muted">Queue is empty</p>
				<p class="mt-1 text-xs text-fg-subtle">Press play on an episode to auto-populate</p>
			</div>
		{:else}
			<ul class="py-2">
				{#each queueItems as item, index (item.id)}
					{#if index === manualQueueLength && hasAutoItems}
						<li class="flex items-center gap-3 px-5 py-2" aria-hidden="true">
							<div class="h-px flex-1 bg-border"></div>
							<span class="text-[10px] font-medium tracking-widest text-fg-subtle uppercase">
								From this feed
							</span>
							<div class="h-px flex-1 bg-border"></div>
						</li>
					{/if}

					<li
						class="group relative flex items-start gap-3 px-5 py-3 transition-colors hover:bg-surface-hover"
					>
						<span
							class="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-fg-subtle tabular-nums"
						>
							{index + 1}
						</span>

						<div class="min-w-0 flex-1">
							<p class="truncate text-sm font-medium text-fg">
								{item.title}
							</p>

							{#if feedTitleForItem(item)}
								<p class="mt-0.5 truncate text-xs text-fg-muted">
									{feedTitleForItem(item)}
								</p>
							{/if}

							{#if durationLabel(item)}
								<p class="mt-0.5 text-xs text-fg-subtle tabular-nums">
									{durationLabel(item)}
								</p>
							{/if}
						</div>

						<div
							class="mt-0.5 flex shrink-0 flex-col items-center gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
						>
							{#if index > 0}
								<button
									type="button"
									title="Move up"
									aria-label={`Move ${item.title} up in queue`}
									class="preset-icon-subtle btn-icon rounded-lg"
									onclick={() => onMoveItemUp(item.id)}
								>
									<Icon icon="lucide:chevron-up" class="size-3.5" />
								</button>
							{/if}

							<button
								type="button"
								title="Remove from queue"
								aria-label={`Remove ${item.title} from queue`}
								class="preset-icon-subtle btn-icon rounded-lg"
								onclick={() => onRemoveItem(item.id)}
							>
								<Icon icon="lucide:x" class="size-3.5" />
							</button>

							{#if index < queueItems.length - 1}
								<button
									type="button"
									title="Move down"
									aria-label={`Move ${item.title} down in queue`}
									class="preset-icon-subtle btn-icon rounded-lg"
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
	</div>

	{#if queueItems.length > 0}
		<div class="shrink-0 border-t border-border px-5 py-3">
			<button
				type="button"
				class="preset-outlined-subtle btn w-full justify-center rounded-xl"
				onclick={onClearQueue}
			>
				Clear queue
			</button>
		</div>
	{/if}
</div>
