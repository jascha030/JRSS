<script lang="ts">
	import { isAudioLoading } from '$lib/stores/app.svelte';
	import Icon from '@iconify/svelte';

	type Props = {
		durationSeconds: number;
		isPlaying: boolean;
		onTogglePlayback: () => void;
		onSkip: (deltaSeconds: number) => void;
		skipSeconds?: number;
		class?: string;
	};

	let {
		isPlaying,
		onTogglePlayback,
		onSkip,
		skipSeconds = 15,
		class: className = ''
	}: Props = $props();
</script>

<div class={`flex items-center justify-center gap-2 ${className}`}>
	<button
		class="btn-icon rounded-xl text-fg-muted transition-colors hover:text-fg"
		type="button"
		aria-label={`Back ${skipSeconds} seconds`}
		onclick={() => onSkip(-skipSeconds)}
	>
		<Icon icon="heroicons:backward-solid" class="size-5" />
	</button>

	<button
		class="btn-icon preset-filled rounded-xl shadow-sm"
		type="button"
		onclick={onTogglePlayback}
		disabled={isAudioLoading()}
		aria-label={isPlaying ? 'Pause' : 'Play'}
		aria-pressed={isPlaying}
	>
		{#if isAudioLoading()}
			{#key isAudioLoading()}
				<Icon icon="lucide:loader-2" class="size-5 animate-spin" />
			{/key}
		{:else if isPlaying}
			<Icon icon="heroicons:pause-solid" class="size-5" />
		{:else}
			<Icon icon="heroicons:play-solid" class="size-5" />
		{/if}
	</button>

	<button
		class="btn-icon rounded-xl text-fg-muted transition-colors hover:text-fg"
		type="button"
		aria-label={`Forward ${skipSeconds} seconds`}
		onclick={() => onSkip(skipSeconds)}
	>
		<Icon icon="heroicons:forward-solid" class="size-5" />
	</button>
</div>
