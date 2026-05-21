<script lang="ts">
	import { requestSetVolume } from '$lib/state';
	import { useVolumeControl } from '$lib/hooks/useVolumeControl.svelte';
	import Icon from '@iconify/svelte';
	import VerticalRangeInput from '../ui/VerticalRangeInput.svelte';

	type Props = {
		class?: string;
		volume: number;
	};

	let { volume, class: className = '' }: Props = $props();

	const volumeControl = useVolumeControl({
		getVolume: () => volume,
		setVolume: requestSetVolume
	});
</script>

<div class={`group relative flex items-center justify-center ${className}`}>
	<div
		class="volume-range-container absolute bottom-full left-1/2 mb-2 flex w-8 -translate-x-1/2 justify-center rounded-xl py-2 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
	>
		<VerticalRangeInput
			value={volumeControl.effectiveVolume}
			max={1}
			step={0.01}
			ariaLabel="Volume"
			oninput={volumeControl.handleVolumeInput}
		/>
	</div>

	<button
		class="preset-icon-subtle btn-icon size-5 rounded-xl"
		type="button"
		onclick={volumeControl.toggleMute}
		aria-label={volumeControl.isMuted ? 'Unmute' : 'Mute'}
		aria-pressed={volumeControl.isMuted}
	>
		{#if volumeControl.isMuted || volumeControl.effectiveVolume === 0}
			<Icon icon="lucide:volume-x" class="size-5" />
		{:else if volumeControl.effectiveVolume < 0.5}
			<Icon icon="lucide:volume-1" class="size-5" />
		{:else}
			<Icon icon="lucide:volume-2" class="size-5" />
		{/if}
	</button>
</div>
