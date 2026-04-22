<script lang="ts">
	type Props = {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		fillColor?: string;
		trackColor?: string;
		ariaLabel?: string;
		disabled?: boolean;
		oninput?: (event: Event & { currentTarget: HTMLInputElement }) => void;
		onchange?: (event: Event & { currentTarget: HTMLInputElement }) => void;
		class?: string;
	};

	let {
		value,
		min = 0,
		max = 100,
		step = 1,
		fillColor = 'var(--color-accent)',
		trackColor = 'var(--color-border)',
		ariaLabel = 'Range input',
		disabled = false,
		oninput,
		onchange,
		class: className = ''
	}: Props = $props();

	let progressPercent = $derived(max === min ? 0 : ((value - min) / (max - min)) * 100);
</script>

<input
	class={`player-range w-full ${className}`}
	type="range"
	{min}
	{max}
	{step}
	{value}
	{disabled}
	aria-label={ariaLabel}
	{oninput}
	{onchange}
	style={`--progress: ${progressPercent}%; --fill: ${fillColor}; --track: ${trackColor}`}
/>

<style>
	.player-range {
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		cursor: pointer;
		height: 1.25rem;
		min-width: 0;
	}

	.player-range:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.player-range:focus {
		outline: none;
	}

	.player-range:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
		border-radius: 9999px;
	}

	.player-range::-webkit-slider-runnable-track {
		height: 0.25rem;
		border-radius: 9999px;
		background: linear-gradient(
			to right,
			var(--fill) 0%,
			var(--fill) var(--progress),
			var(--track) var(--progress),
			var(--track) 100%
		);
	}

	.player-range::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: 0.875rem;
		height: 0.875rem;
		border-radius: 9999px;
		background: var(--fill);
		margin-top: -0.3125rem;
		border: none;
		transition:
			transform 0.12s ease,
			box-shadow 0.12s ease;
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--fill) 20%, transparent);
	}

	.player-range:hover::-webkit-slider-thumb {
		transform: scale(1.12);
	}

	.player-range::-moz-range-track {
		height: 0.25rem;
		border-radius: 9999px;
		background: var(--track);
		border: none;
	}

	.player-range::-moz-range-progress {
		height: 0.25rem;
		border-radius: 9999px;
		background: var(--fill);
	}

	.player-range::-moz-range-thumb {
		width: 0.875rem;
		height: 0.875rem;
		border-radius: 9999px;
		background: var(--fill);
		border: none;
		transition:
			transform 0.12s ease,
			box-shadow 0.12s ease;
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--fill) 20%, transparent);
	}

	.player-range:hover::-moz-range-thumb {
		transform: scale(1.12);
	}
</style>
