<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { IndexedElementAction } from '$lib/services/keyboard-list-navigation.svelte.js';
	import type { CommandPaletteItem } from '$lib/types/command';

	interface Props {
		item: CommandPaletteItem;
		isHighlighted: boolean;
		index: number;
		setRef: IndexedElementAction<HTMLButtonElement>;
		onClick: (item: CommandPaletteItem) => void;
		onHover: (index: number) => void;
	}

	let { item, isHighlighted, index, setRef, onClick, onHover }: Props = $props();
</script>

<button
	use:setRef={index}
	type="button"
	class="flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-2 transition-colors"
	class:bg-accent={isHighlighted}
	class:text-white={isHighlighted}
	class:text-fg-muted={!isHighlighted}
	onclick={() => onClick(item)}
	onmouseenter={() => onHover(index)}
>
	<Icon icon={item.icon} class="size-4 shrink-0" />
	<span class="truncate text-sm">{item.title}</span>

	{#if item.badge}
		<span class="ml-auto shrink-0 text-xs font-medium tracking-wider text-fg-muted uppercase">
			{item.badge}
		</span>
	{/if}
</button>
