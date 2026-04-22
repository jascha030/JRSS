<script lang="ts">
	import { requestSetVolume } from '$lib/stores/app.svelte';
	import Icon from '@iconify/svelte';
	import RangeInput from '../ui/RangeInput.svelte';

	type Props = {
		volume: number;
		class?: string;
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

<div class="flex shrink-0 items-center gap-2 {className}">
	<button
		class="flex h-7 w-7 items-center justify-center rounded-lg text-fg-muted transition-colors hover:text-fg"
		type="button"
		onclick={toggleMute}
		aria-label={isMuted ? 'Unmute' : 'Mute'}
	>
		{#if isMuted || effectiveVolume === 0}
			<Icon icon="lucide:volume-x" class="size-4" />
		{:else if effectiveVolume < 0.5}
			<Icon icon="lucide:volume-1" class="size-4" />
		{:else}
			<Icon icon="lucide:volume-2" class="size-4" />
		{/if}
	</button>

	<RangeInput
		class="w-24"
		value={effectiveVolume}
		max={1}
		step={0.01}
		fillColor="var(--color-fg-muted)"
		oninput={handleVolumeInput}
	/>
</div>
