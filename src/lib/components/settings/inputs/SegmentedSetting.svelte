<script lang="ts" generics="V extends string">
	import type { SegmentedDef } from '$lib/types/settings';

	interface Props {
		def: SegmentedDef<V>;
		value: V;
		disabled: boolean;
		onchange: (value: V) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();
</script>

<div
	class="flex overflow-hidden rounded-xl border border-border"
	role="group"
	aria-label={def.label}
>
	{#each def.options as option (option.value)}
		<button
			type="button"
			{disabled}
			onclick={() => onchange(option.value)}
			class={value === option.value
				? 'flex-1 bg-interactive px-3 py-2 text-sm font-medium text-interactive-text transition'
				: 'flex-1 bg-surface px-3 py-2 text-sm font-medium text-fg-muted transition hover:bg-surface-hover hover:text-fg'}
		>
			{option.label}
		</button>
	{/each}
</div>
