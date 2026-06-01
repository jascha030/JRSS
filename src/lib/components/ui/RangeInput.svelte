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
		class?: string;
	};

	let {
		value,
		min = 0,
		max = 100,
		step = 1,
		ariaLabel = 'Range input',
		disabled = false,
		oninput,
		onchange,
		class: className = ''
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
	onValueChange={handleValueChange}
	onValueChangeEnd={handleValueChangeEnd}
	class={`player-range ${className}`}
>
	<Slider.Control class="group/control relative flex h-5 w-full min-w-0 items-center">
		<Slider.Track
			class="h-1 w-full rounded-full"
			style={`background: linear-gradient(to right, var(--color-accent) 0%, var(--color-accent) ${progressPercent}%, var(--color-border) ${progressPercent}%, var(--color-border) 100%)`}
		>
			<Slider.Range class="opacity-0" />
		</Slider.Track>
		<Slider.Thumb
			index={0}
			class="size-1 rounded-full transition-[width,height] group-hover/control:size-3.5"
			style="background-color: var(--color-accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 20%, transparent);"
		/>
	</Slider.Control>
</Slider>
