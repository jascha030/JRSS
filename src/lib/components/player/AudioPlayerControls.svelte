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

<div class="flex items-center justify-center gap-3 {className}">
	<button
		class="btn-ghost flex size-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
		type="button"
		aria-label="Back {skipSeconds} seconds"
		onclick={() => onSkip(-skipSeconds)}
	>
		<Icon icon="heroicons:backward-solid" class="size-7" />
	</button>

	<button
		class="btn-primary flex size-9 items-center justify-center rounded-xl bg-fg-muted text-sm"
		type="button"
		onclick={onTogglePlayback}
		disabled={isAudioLoading()}
	>
		{#if isAudioLoading()}
			{#key isAudioLoading()}
				<Icon icon="lucide:loader-2" class="size-6 animate-spin" />
			{/key}
		{:else if isPlaying}
			<Icon icon="heroicons:pause-solid" class="size-6" />
		{:else}
			<Icon icon="heroicons:play-solid" class="size-6" />
		{/if}
	</button>

	<button
		class="flex size-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
		type="button"
		aria-label="Forward {skipSeconds} seconds"
		onclick={() => onSkip(skipSeconds)}
	>
		<Icon icon="heroicons:forward-solid" class="size-7" />
	</button>
</div>
