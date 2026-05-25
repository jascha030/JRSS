<script lang="ts">
	import { Switch } from '@skeletonlabs/skeleton-svelte';
	import type { ColorDef } from '$lib/types/settings';

	interface Props {
		def: ColorDef;
		value: string | null;
		disabled: boolean;
		onchange: (value: string | null) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();

	const pickerHex = $derived(value ?? def.fallbackHex);

	function handleToggle(checked: boolean) {
		if (checked) {
			onchange(pickerHex);
		} else {
			onchange(null);
		}
	}
</script>

<div class="flex flex-col gap-2">
	<Switch
		class="flex cursor-pointer items-center justify-between gap-3 rounded-xl border border-border bg-surface-hover px-4 py-3 text-sm text-fg"
		checked={value !== null}
		{disabled}
		onCheckedChange={(e) => handleToggle(e.checked)}
	>
		<Switch.Label class="font-medium">Use custom {def.label.toLowerCase()}</Switch.Label>
		<Switch.HiddenInput />
		<Switch.Control>
			<Switch.Thumb />
		</Switch.Control>
	</Switch>

	{#if value !== null}
		<div class="flex items-center gap-3">
			<input
				type="color"
				value={pickerHex}
				oninput={(e) => onchange(e.currentTarget.value)}
				{disabled}
				class="size-10 cursor-pointer rounded-lg p-0.5"
			/>
			<span class="font-mono text-sm text-fg-secondary">{pickerHex}</span>
			<button
				type="button"
				{disabled}
				onclick={() => onchange(null)}
				class="preset-outlined-subtle btn btn-sm"
			>
				Reset
			</button>
		</div>
	{/if}
</div>
