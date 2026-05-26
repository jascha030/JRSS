<script lang="ts">
	import { Switch } from '@skeletonlabs/skeleton-svelte';
	import type { ToggleDef } from '$lib/types/settings';

	interface Props {
		def: ToggleDef;
		value: boolean;
		disabled: boolean;
		onchange: (value: boolean) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();
</script>

<Switch
	class="flex cursor-pointer items-center justify-between gap-3 rounded-xl border border-border bg-surface-hover px-4 py-3 text-sm text-fg"
	checked={value}
	{disabled}
	onCheckedChange={(e) => onchange(e.checked)}
>
	<Switch.Label class="font-medium">{def.checkboxLabel ?? def.label}</Switch.Label>
	<Switch.HiddenInput />
	<Switch.Control>
		<Switch.Thumb />
	</Switch.Control>
</Switch>
