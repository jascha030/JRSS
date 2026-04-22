<script lang="ts">
	import type { Feed, MediaListItem } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';

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
		if (!duration || duration <= 0) return null;

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
			class="btn-icon hover:preset-tonal text-fg-muted hover:text-fg"
			aria-label="Close queue"
			onclick={onClose}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1,5"
				stroke="currentColor"
				class="size-5"
			>
				<path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
			</svg>
		</button>
	</div>

	<div class="flex-1 overflow-y-auto">
		{#if queueItems.length === 0}
			<div class="flex flex-col items-center justify-center px-6 py-16 text-center">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1,5"
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
				<p class="mt-1 text-xs text-fg-subtle">Press play on an episode to auto-populate</p>
			</div>
		{:else}
			<ul class="space-y-1 p-2">
				{#each queueItems as item, index (item.id)}
					{#if index === manualQueueLength && hasAutoItems}
						<li class="flex items-center gap-3 px-3 py-2" aria-hidden="true">
							<div class="h-px flex-1 bg-border"></div>
							<span class="text-[10px] font-medium tracking-widest text-fg-subtle uppercase">
								From this feed
							</span>
							<div class="h-px flex-1 bg-border"></div>
						</li>
					{/if}

					<li
						class="group flex items-start gap-3 rounded-xl px-3 py-3 transition-colors hover:bg-surface-hover"
					>
						<span
							class="mt-0,5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-fg-subtle tabular-nums"
						>
							{index + 1}
						</span>

						<div class="min-w-0 flex-1">
							<p class="truncate text-sm font-medium text-fg">
								{item.title}
							</p>

							{#if feedTitleForItem(item)}
								<p class="mt-0,5 truncate text-xs text-fg-muted">
									{feedTitleForItem(item)}
								</p>
							{/if}

							{#if durationLabel(item)}
								<p class="mt-0,5 text-xs text-fg-subtle tabular-nums">
									{durationLabel(item)}
								</p>
							{/if}
						</div>

						<div
							class="mt-0,5 flex shrink-0 flex-col items-center gap-1 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
						>
							{#if index > 0}
								<button
									type="button"
									title="Move up"
									aria-label={`Move ${item.title} up in queue`}
									class="btn-icon hover:preset-tonal size-7 text-fg-subtle hover:text-fg"
									onclick={() => onMoveItemUp(item.id)}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										fill="none"
										viewBox="0 0 24 24"
										stroke-width="2"
										stroke="currentColor"
										class="size-3,5"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											d="m4.5 15.75 7.5-7.5 7.5 7.5"
										/>
									</svg>
								</button>
							{/if}

							<button
								type="button"
								title="Remove from queue"
								aria-label={`Remove ${item.title} from queue`}
								class="btn-icon hover:preset-tonal size-7 text-fg-subtle hover:text-fg"
								onclick={() => onRemoveItem(item.id)}
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="1,5"
									stroke="currentColor"
									class="size-3,5"
								>
									<path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
								</svg>
							</button>

							{#if index < queueItems.length - 1}
								<button
									type="button"
									title="Move down"
									aria-label={`Move ${item.title} down in queue`}
									class="btn-icon hover:preset-tonal size-7 text-fg-subtle hover:text-fg"
									onclick={() => onMoveItemDown(item.id)}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										fill="none"
										viewBox="0 0 24 24"
										stroke-width="2"
										stroke="currentColor"
										class="size-3,5"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											d="m19.5 8.25-7.5 7.5-7.5-7.5"
										/>
									</svg>
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
			<button type="button" class="btn preset-tonal w-full justify-center" onclick={onClearQueue}>
				Clear queue
			</button>
		</div>
	{/if}
</div>
