<script lang="ts">
	import { isAudioLoading } from "$lib/stores/app.svelte";
	import Icon from "@iconify/svelte";

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
		class="flex size-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
		type="button"
		aria-label="Back {skipSeconds} seconds"
		onclick={() => onSkip(-skipSeconds)}
	>
		<Icon icon="lucide:rewind" class="size-5" />
	</button>

	<button
		class="btn-primary flex size-9 items-center justify-center rounded-xl text-sm"
		type="button"
		onclick={onTogglePlayback}
		disabled={isAudioLoading()}
	>
		{#if isAudioLoading()}
			{#key isAudioLoading()}
				<Icon icon="lucide:loader-2" class="size-5 animate-spin" />
			{/key}
		{:else if isPlaying}
			<Icon icon="lucide:pause" class="size-5" />
		{:else}
			<Icon icon="lucide:play" class="size-5" />
		{/if}
	</button>

	<button
		class="flex size-9 items-center justify-center rounded-xl text-fg-muted transition-colors hover:text-fg"
		type="button"
		aria-label="Forward {skipSeconds} seconds"
		onclick={() => onSkip(skipSeconds)}
	>
		<Icon icon="lucide:fast-forward" class="size-5" />
	</button>
</div>
