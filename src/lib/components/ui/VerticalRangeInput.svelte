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
	};

	let {
		value,
		min = 0,
		max = 100,
		step = 1,
		fillColor = 'var(--color-accent)',
		trackColor = 'var(--color-border)',
		ariaLabel = 'Vertical range',
		disabled = false,
		oninput,
		onchange
	}: Props = $props();

	let progressPercent = $derived(max === min ? 0 : ((value - min) / (max - min)) * 100);
</script>

<div
	class="vertical-range-wrapper"
	style="--progress: {progressPercent}%; --fill: {fillColor}; --track: {trackColor};"
>
	<input
		type="range"
		{min}
		{max}
		{step}
		{value}
		{disabled}
		aria-label={ariaLabel}
		{oninput}
		{onchange}
		class="player-range vertical-range"
	/>
</div>

<style>
	.vertical-range-wrapper {
		position: relative;
		width: 1.25rem;
		height: 6rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.vertical-range {
		position: absolute;
		width: 6rem;
		height: 1.25rem;
		transform: rotate(-90deg);
		transform-origin: center center;
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		cursor: pointer;
		left: 50%;
		top: 50%;
		margin-left: -3rem;
		margin-top: -0.625rem;
	}

	.vertical-range:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.vertical-range:focus {
		outline: none;
	}

	.vertical-range:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
		border-radius: 9999px;
	}

	.vertical-range::-webkit-slider-runnable-track {
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

	.vertical-range::-webkit-slider-thumb {
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

	.vertical-range:hover::-webkit-slider-thumb {
		transform: scale(1.12);
	}

	.vertical-range::-moz-range-track {
		height: 0.25rem;
		border-radius: 9999px;
		background: var(--track);
		border: none;
	}

	.vertical-range::-moz-range-progress {
		height: 0.25rem;
		border-radius: 9999px;
		background: var(--fill);
	}

	.vertical-range::-moz-range-thumb {
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

	.vertical-range:hover::-moz-range-thumb {
		transform: scale(1.12);
	}
</style>
