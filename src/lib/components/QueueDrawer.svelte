<script lang="ts">
	import type { Feed, FeedListItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';

	type Props = {
		open: boolean;
		queueItems: FeedListItem[];
		feeds: Feed[];
		onRemoveItem: (itemId: string) => void;
		onClearQueue: () => void;
		onClose: () => void;
	};

	let { open, queueItems, feeds, onRemoveItem, onClearQueue, onClose }: Props = $props();

	let feedTitleById = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const feed of feeds) {
			map[feed.id] = feed.title;
		}
		return map;
	});

	function feedTitleForItem(item: FeedListItem): string | null {
		return feedTitleById[item.feedId] ?? null;
	}

	function durationLabel(item: FeedListItem): string | null {
		const duration = item.mediaEnclosure?.durationSeconds;
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

<!-- Backdrop -->
{#if open}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/30 backdrop-blur-[2px] transition-opacity"
		aria-label="Close queue"
		onclick={onClose}
		tabindex="-1"
	></button>
{/if}

<!-- Drawer panel -->
<div
	class={`fixed inset-y-0 right-0 z-50 flex w-80 transform-gpu flex-col border-l border-border bg-surface shadow-xl transition-transform duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform motion-reduce:transition-none ${
		open ? 'translate-x-0' : 'translate-x-full'
	}`}
	role="dialog"
	aria-label="Playback queue"
>
	<!-- Header -->
	<div class="flex h-16 shrink-0 items-center justify-between border-b border-border px-5">
		<div class="min-w-0">
			<h2 class="text-sm font-semibold text-fg">Playing next</h2>
			<p class="text-xs text-fg-muted">
				{queueItems.length}
				{queueItems.length === 1 ? 'episode' : 'episodes'}
			</p>
		</div>
		<button
			type="button"
			class="flex size-8 items-center justify-center rounded-lg text-fg-muted transition-colors hover:bg-surface-hover hover:text-fg"
			aria-label="Close queue"
			onclick={onClose}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="size-5"
			>
				<path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
			</svg>
		</button>
	</div>

	<!-- Queue list -->
	<div class="flex-1 overflow-y-auto">
		{#if queueItems.length === 0}
			<div class="flex flex-col items-center justify-center px-6 py-16 text-center">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="mb-3 size-10 text-fg-subtle"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M3.75 12h16.5m-16.5 3.75h16.5M3.75 19.5h16.5M5.625 4.5h12.75a1.875 1.875 0 0 1 0 3.75H5.625a1.875 1.875 0 0 1 0-3.75Z"
					/>
				</svg>
				<p class="text-sm font-medium text-fg-muted">Queue is empty</p>
				<p class="mt-1 text-xs text-fg-subtle">Use "Play next" to add episodes</p>
			</div>
		{:else}
			<ul class="py-2">
				{#each queueItems as item, index (item.id)}
					<li
						class="group relative flex items-start gap-3 px-5 py-3 transition-colors hover:bg-surface-hover"
					>
						<!-- Index badge -->
						<span
							class="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-fg-subtle tabular-nums"
						>
							{index + 1}
						</span>

						<!-- Item info -->
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

						<!-- Remove button -->
						<button
							type="button"
							title="Remove from queue"
							aria-label={`Remove ${item.title} from queue`}
							class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-lg text-fg-subtle opacity-0 transition-[opacity,background-color,color] duration-150 group-hover:opacity-100 hover:bg-surface-active hover:text-fg-secondary"
							onclick={() => onRemoveItem(item.id)}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="size-4"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
							</svg>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>

	<!-- Footer -->
	{#if queueItems.length > 0}
		<div class="shrink-0 border-t border-border px-5 py-3">
			<button
				type="button"
				class="btn-secondary w-full rounded-xl px-3 py-2"
				onclick={onClearQueue}
			>
				Clear queue
			</button>
		</div>
	{/if}
</div>
