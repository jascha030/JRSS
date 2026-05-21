<script lang="ts">
	import { requestSetVolume } from '$lib/state';
	import Icon from '@iconify/svelte';
	import VerticalRangeInput from '../ui/VerticalRangeInput.svelte';

	type Props = {
		class?: string;
		volume: number;
	};

	let { volume, class: className = '' }: Props = $props();

	let volumeOverride = $state<number | null>(null);
	let previousNonZeroVolume = $state(1);

	let effectiveVolume = $derived(volumeOverride ?? volume);
	let isMuted = $derived(effectiveVolume === 0);

	function handleVolumeInput(event: Event & { currentTarget: HTMLInputElement }) {
		const nextVolume = Number(event.currentTarget.value);
		volumeOverride = nextVolume;

		if (nextVolume > 0) {
			previousNonZeroVolume = nextVolume;
		}
	}

	function toggleMute() {
		if (effectiveVolume === 0) {
			volumeOverride = previousNonZeroVolume > 0 ? previousNonZeroVolume : 1;
			return;
		}

		previousNonZeroVolume = effectiveVolume;
		volumeOverride = 0;
	}

	$effect(() => {
		if (volume > 0) {
			previousNonZeroVolume = volume;
		}
	});

	$effect(() => {
		if (volumeOverride !== null && Math.abs(volumeOverride - volume) < 0.001) {
			volumeOverride = null;
		}
	});

	$effect(() => {
		if (Math.abs(effectiveVolume - volume) < 0.001) {
			return;
		}

		requestSetVolume(effectiveVolume);
	});
</script>

<div class={`group relative flex items-center justify-center ${className}`}>
	<div
		class="volume-range-container absolute bottom-full left-1/2 mb-2 flex w-8 -translate-x-1/2 justify-center rounded-xl py-2 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
	>
		<VerticalRangeInput
			value={effectiveVolume}
			max={1}
			step={0.01}
			ariaLabel="Volume"
			oninput={handleVolumeInput}
		/>
	</div>

	<button
		class="preset-icon-subtle btn-icon size-5 rounded-xl"
		type="button"
		onclick={toggleMute}
		aria-label={isMuted ? 'Unmute' : 'Mute'}
		aria-pressed={isMuted}
	>
		{#if isMuted || effectiveVolume === 0}
			<Icon icon="lucide:volume-x" class="size-5" />
		{:else if effectiveVolume < 0.5}
			<Icon icon="lucide:volume-1" class="size-5" />
		{:else}
			<Icon icon="lucide:volume-2" class="size-5" />
		{/if}
	</button>
</div>
