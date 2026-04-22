<script lang="ts">
	/**
	 * Reusable styled range input component with progress fill.
	 * Used for audio seek and volume controls.
	 */
	type Props = {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		fillColor?: string;
		trackColor?: string;
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
		oninput,
		onchange,
		class: className = ''
	}: Props = $props();

	let progressPercent = $derived(((value - min) / (max - min)) * 100);
</script>

<input
	class="player-range {className}"
	type="range"
	{min}
	{max}
	{step}
	{value}
	{oninput}
	{onchange}
	style="--progress: {progressPercent}%; --fill: {fillColor}; --track: {trackColor}"
/>

<style>
	.player-range {
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		cursor: pointer;
		height: 20px;
	}

	.player-range:focus {
		outline: none;
	}

	.player-range:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
		border-radius: 2px;
	}

	.player-range::-webkit-slider-runnable-track {
		height: 4px;
		border-radius: 2px;
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
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--fill);
		margin-top: -5px;
		transition: transform 0.1s ease;
	}

	.player-range:hover::-webkit-slider-thumb {
		transform: scale(1.2);
	}

	.player-range::-moz-range-track {
		height: 4px;
		border-radius: 2px;
		background: var(--track);
		border: none;
	}

	.player-range::-moz-range-progress {
		height: 4px;
		border-radius: 2px;
		background: var(--fill);
	}

	.player-range::-moz-range-thumb {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--fill);
		border: none;
		transition: transform 0.1s ease;
	}

	.player-range:hover::-moz-range-thumb {
		transform: scale(1.2);
	}
</style>
