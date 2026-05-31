<script lang="ts">
	import { requestSetVolume } from '$lib/state';
	import { useVolumeControl } from '$lib/hooks/useVolumeControl.svelte';
	import Icon from '@iconify/svelte';
	import RangeInput from '../ui/RangeInput.svelte';

	type Props = {
		class?: string;
		volume: number;
		showRange?: boolean;
	};

	let { volume, class: className = '', showRange: showRange = true }: Props = $props();

	const volumeControl = useVolumeControl({
		getVolume: () => volume,
		setVolume: requestSetVolume
	});
</script>

<div class={`group flex ${showRange ? '' : 'not-hover:flex-0'} items-center gap-2 ${className}`}>
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

	<RangeInput
		class={showRange
			? 'w-24'
			: 'w-24 origin-left scale-x-0 transition-transform duration-150 group-hover:scale-x-100'}
		value={volumeControl.effectiveVolume}
		max={1}
		step={0.01}
		ariaLabel="Volume"
		oninput={volumeControl.handleVolumeInput}
	/>
</div>
