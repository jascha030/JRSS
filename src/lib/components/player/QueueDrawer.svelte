<script lang="ts">
	import type { Feed, MediaListItem } from '$lib/types/rss';
	import Icon from '@iconify/svelte';
	import QueueList from './QueueList.svelte';

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
		<QueueList
			{queueItems}
			{manualQueueLength}
			{feeds}
			{onRemoveItem}
			{onMoveItemUp}
			{onMoveItemDown}
		/>
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
