<script lang="ts" generics="V extends string | number">
	import type { SelectDef } from '$lib/types/settings';
	import {
		Combobox,
		Portal,
		type ComboboxRootProps,
		useListCollection
	} from '@skeletonlabs/skeleton-svelte';

	interface Props {
		def: SelectDef<V>;
		value: V;
		disabled: boolean;
		onchange: (value: V) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();

	const options = $derived(def.options.map((o) => ({ label: o.label, value: String(o.value) })));

	const collection = $derived(
		useListCollection({
			items: options,
			itemToString: (item) => item.label,
			itemToValue: (item) => item.value
		})
	);

	const selectedValues = $derived([String(value)]);

	const onValueChange: ComboboxRootProps['onValueChange'] = (event) => {
		const selected = event.value[0] ?? '';
		const found = def.options.find((o) => String(o.value) === selected);
		if (found !== undefined) onchange(found.value);
	};
</script>

<Combobox class="w-full" {collection} value={selectedValues} {onValueChange} {disabled} openOnClick>
	<Combobox.Label class="sr-only">{def.label}</Combobox.Label>
	<Combobox.Control class="flex h-9 w-full items-center gap-2">
		<Combobox.Input class="h-full min-w-0 flex-1" />
		<Combobox.Trigger class="h-full" />
	</Combobox.Control>
	<Portal>
		<Combobox.Positioner>
			<Combobox.Content class="z-50 max-h-64 overflow-y-auto">
				{#each options as item (item.value)}
					<Combobox.Item {item}>
						<Combobox.ItemText>{item.label}</Combobox.ItemText>
						<Combobox.ItemIndicator />
					</Combobox.Item>
				{/each}
			</Combobox.Content>
		</Combobox.Positioner>
	</Portal>
</Combobox>
