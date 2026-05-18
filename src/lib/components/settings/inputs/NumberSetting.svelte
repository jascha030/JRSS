<script lang="ts">
	import type { NumberDef } from '$lib/types/settings';

	interface Props {
		def: NumberDef;
		value: number;
		disabled: boolean;
		onchange: (value: number) => void;
	}

	let { def, value, disabled, onchange }: Props = $props();

	/** Display value derived from the stored prop — used when not focused. */
	const parentDisplay = $derived(String(def.toDisplay ? def.toDisplay(value) : value));

	let editString = $state<string | null>(null);
	let validationError = $state<string | null>(null);

	/** What the input shows: the in-progress edit string, or the synced parent value. */
	const displayString = $derived(editString ?? parentDisplay);

	function handleInput(e: Event & { currentTarget: HTMLInputElement }) {
		editString = e.currentTarget.value;
		const raw = parseFloat(e.currentTarget.value);
		if (!isFinite(raw)) {
			validationError = 'Please enter a valid number.';
			return;
		}
		const err = def.validate ? def.validate(raw) : null;
		validationError = err;
		if (err === null) {
			const stored = def.fromDisplay ? def.fromDisplay(raw) : raw;
			onchange(stored);
		}
	}

	function handleBlur() {
		editString = null;
		validationError = null;
	}
</script>

<div class="flex flex-col gap-1">
	<div class="flex items-center gap-2">
		<input
			type="number"
			{disabled}
			value={displayString}
			min={def.min}
			max={def.max}
			step={def.step}
			oninput={handleInput}
			onblur={handleBlur}
			class="w-full rounded-xl border border-border bg-surface px-4 py-2.5 text-sm text-fg transition outline-none placeholder:text-fg-muted focus:border-border-hover focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-60"
		/>
		{#if def.unit}
			<span class="text-xs text-fg-muted">{def.unit}</span>
		{/if}
	</div>
	{#if validationError}
		<p class="mt-1 text-xs text-error">{validationError}</p>
	{/if}
</div>
