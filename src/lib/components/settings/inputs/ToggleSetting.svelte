<script lang="ts">
	import type { ToggleDef } from '$lib/types/settings';

	interface Props {
		def: ToggleDef;
		value: boolean;
		disabled: boolean;
		onchange: (value: boolean) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();
</script>

<label
	class="flex cursor-pointer items-center gap-3 rounded-xl border border-border bg-surface-hover px-4 py-3 text-sm text-fg"
>
	<input
		type="checkbox"
		checked={value}
		{disabled}
		onchange={(e) => onchange(e.currentTarget.checked)}
		class="size-4 rounded border-border bg-surface disabled:cursor-not-allowed"
	/>
	<span class="font-medium">{def.checkboxLabel ?? def.label}</span>
</label>
