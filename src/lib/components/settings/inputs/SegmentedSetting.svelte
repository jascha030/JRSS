<script lang="ts" generics="V extends string">
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import type { SegmentedDef } from '$lib/types/settings';

	interface Props {
		def: SegmentedDef<V>;
		value: V;
		disabled: boolean;
		onchange: (value: V) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();
</script>

<SegmentedControl
	{value}
	{disabled}
	onValueChange={(details) => {
		const newValue = details.value;
		if (newValue && def.options.some((o) => o.value === newValue)) {
			onchange(newValue as V);
		}
	}}
>
	<SegmentedControl.Label class="sr-only">{def.label}</SegmentedControl.Label>
	<SegmentedControl.Control
		class="w-full overflow-hidden rounded-xl border border-border bg-surface"
	>
		<SegmentedControl.Indicator class="bg-interactive" />
		{#each def.options as option (option.value)}
			<SegmentedControl.Item
				value={option.value}
				class="flex-1 px-3 py-2 text-sm font-medium transition data-[state=checked]:text-interactive-text data-[state=unchecked]:text-fg-muted data-[state=unchecked]:hover:bg-surface-hover data-[state=unchecked]:hover:text-fg"
			>
				<SegmentedControl.ItemText>{option.label}</SegmentedControl.ItemText>
				<SegmentedControl.ItemHiddenInput />
			</SegmentedControl.Item>
		{/each}
	</SegmentedControl.Control>
</SegmentedControl>
