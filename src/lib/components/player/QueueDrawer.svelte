<script lang="ts">
	import { getPlaybackHistory, getUpcomingQueue } from '$lib/state/playback.svelte';
	import { clearQueue } from '$lib/state';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import QueueList from './QueueList.svelte';

	type Props = {
		open: boolean;
		onClose: () => void;
	};

	let { open, onClose }: Props = $props();

	const historyItems = $derived(getPlaybackHistory());
	const queueItems = $derived(getUpcomingQueue());
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
	class={`fixed inset-y-0 right-0 z-50 flex w-80 transform-gpu flex-col border-l border-border bg-surface-shell-opaque shadow-xl transition-transform duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform motion-reduce:transition-none ${
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
				{historyItems.length + queueItems.length}
				{historyItems.length + queueItems.length === 1 ? 'episode' : 'episodes'}
			</p>
		</div>

		<IconButton
			icon="lucide:x"
			label="Close queue"
			title="Close queue"
			variant="icon-subtle"
			onclick={onClose}
		/>
	</div>

	<div class="flex-1 overflow-y-auto">
		<QueueList />
	</div>

	{#if queueItems.length > 0}
		<div class="h-16 shrink-0 border-t border-border px-5 py-3">
			<button
				type="button"
				class="preset-outlined-subtle btn w-full justify-center rounded-xl"
				onclick={() => clearQueue()}
			>
				Clear queue
			</button>
		</div>
	{/if}
</div>
