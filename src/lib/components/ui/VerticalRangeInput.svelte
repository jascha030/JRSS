<script lang="ts">
	import { Slider } from '@skeletonlabs/skeleton-svelte';

	type Props = {
		value: number;
		min?: number;
		max?: number;
		step?: number;
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
		ariaLabel = 'Vertical range',
		disabled = false,
		oninput,
		onchange
	}: Props = $props();

	let progressPercent = $derived(max === min ? 0 : ((value - min) / (max - min)) * 100);

	function createFakeEvent(val: number): Event & { currentTarget: HTMLInputElement } {
		return { currentTarget: { value: String(val) } } as Event & { currentTarget: HTMLInputElement };
	}

	function handleValueChange(details: { value: number[] }) {
		const val = details.value[0] ?? min;
		oninput?.(createFakeEvent(val));
	}

	function handleValueChangeEnd(details: { value: number[] }) {
		const val = details.value[0] ?? min;
		onchange?.(createFakeEvent(val));
	}
</script>

<Slider
	value={[value]}
	{min}
	{max}
	{step}
	{disabled}
	aria-label={[ariaLabel]}
	orientation="vertical"
	onValueChange={handleValueChange}
	onValueChangeEnd={handleValueChangeEnd}
	class="player-range items-center"
>
	<Slider.Control class="relative flex h-24 w-5 flex-col items-center justify-center">
		<Slider.Track
			class="h-full w-1 rounded-full"
			style={`background: linear-gradient(to top, var(--color-accent) 0%, var(--color-accent) ${progressPercent}%, var(--color-border) ${progressPercent}%, var(--color-border) 100%)`}
		>
			<Slider.Range class="opacity-0" />
		</Slider.Track>
		<Slider.Thumb
			index={0}
			class="left-1/2 size-3.5 -translate-x-1/2 rounded-full transition-transform hover:scale-110 focus:outline-2 focus:outline-offset-2 focus:outline-accent"
			style="background-color: var(--color-accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 20%, transparent);"
		/>
	</Slider.Control>
</Slider>
