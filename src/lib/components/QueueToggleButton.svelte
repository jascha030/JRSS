<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		isOpen: boolean;
		queueLength: number;
		onToggle: () => void;
	};

	let { isOpen, queueLength, onToggle }: Props = $props();
</script>

<button
	type="button"
	class="relative flex size-8 items-center justify-center rounded-lg text-fg-muted transition-colors hover:bg-surface-hover hover:text-fg"
	aria-label={isOpen ? 'Close queue' : `Open queue (${queueLength})`}
	title={isOpen ? 'Close queue' : `Queue (${queueLength})`}
	onclick={onToggle}
>
	<Icon icon="lucide:list" class="size-4" />

	{#if queueLength > 0}
		<span
			class="absolute -top-1 -right-1 flex size-4 items-center justify-center rounded-full bg-accent text-[9px] font-bold text-fg-inverse"
		>
			{queueLength}
		</span>
	{/if}
</button>
