<script lang="ts">
	import type { ColorDef } from '$lib/types/settings';

	interface Props {
		def: ColorDef;
		value: string | null;
		disabled: boolean;
		onchange: (value: string | null) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();

	/**
	 * The hex shown in the color picker. When an override is active (`value !== null`),
	 * it tracks the prop; otherwise it holds the last-used color so the picker remembers
	 * its position if the user re-enables the override.
	 */
	const pickerHex = $derived(value ?? def.fallbackHex);

	function handleToggle(e: Event & { currentTarget: HTMLInputElement }) {
		if (e.currentTarget.checked) {
			onchange(pickerHex);
		} else {
			onchange(null);
		}
	}
</script>

<div class="flex flex-col gap-2">
	<label
		class="flex cursor-pointer items-center gap-3 rounded-xl border border-border bg-surface-hover px-4 py-3 text-sm text-fg"
	>
		<input
			type="checkbox"
			checked={value !== null}
			{disabled}
			onchange={handleToggle}
			class="size-4 rounded border-border bg-surface disabled:cursor-not-allowed"
		/>
		<span class="font-medium">Use custom {def.label.toLowerCase()}</span>
	</label>

	{#if value !== null}
		<div class="flex items-center gap-3">
			<input
				type="color"
				value={pickerHex}
				oninput={(e) => onchange(e.currentTarget.value)}
				{disabled}
				class="size-10 cursor-pointer rounded-lg border border-border bg-surface p-0.5 disabled:cursor-not-allowed disabled:opacity-60"
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
