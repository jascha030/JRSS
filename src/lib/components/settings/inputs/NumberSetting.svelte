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
	<div class="input-group h-9 grid-cols-[1fr_auto] rounded-xl border border-border">
		<input
			type="number"
			{disabled}
			value={displayString}
			min={def.min}
			max={def.max}
			step={def.step}
			oninput={handleInput}
			onblur={handleBlur}
			class="ig-input"
		/>
		{#if def.unit}
			<div class="ig-cell preset-tonal text-xs text-fg-muted">{def.unit}</div>
		{/if}
	</div>
	{#if validationError}
		<p class="mt-1 text-xs text-error">{validationError}</p>
	{/if}
</div>
