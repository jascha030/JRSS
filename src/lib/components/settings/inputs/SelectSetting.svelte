<script lang="ts" generics="V extends string | number">
	import type { SelectDef } from '$lib/types/settings';

	interface Props {
		def: SelectDef<V>;
		value: V;
		disabled: boolean;
		onchange: (value: V) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();
</script>

<select
	{disabled}
	value={String(value)}
	onchange={(e) => {
		const found = def.options.find((o) => String(o.value) === e.currentTarget.value);
		if (found !== undefined) onchange(found.value);
	}}
	class="w-full rounded-xl px-4 py-2.5"
>
	{#each def.options as opt (opt.value)}
		<option value={String(opt.value)}>{opt.label}</option>
	{/each}
</select>
